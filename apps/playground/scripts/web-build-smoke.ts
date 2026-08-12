import { strict as assert } from "node:assert"
import { mkdtemp, readFile, rm, stat } from "node:fs/promises"
import { tmpdir } from "node:os"
import { dirname, extname, join, normalize, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { chromium } from "playwright"

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..")
const directory = await mkdtemp(join(tmpdir(), "seseragi-web-build-"))
const output = join(directory, "dist")
const cli = join(root, "target/debug/seseragi")
const source = join(root, "crates/seseragi-cli/tests/fixtures/web-project")

async function run(command: string[], cwd: string): Promise<void> {
  const process = Bun.spawn(command, { cwd, stdout: "pipe", stderr: "pipe" })
  const [exitCode, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ])
  if (exitCode !== 0) {
    throw new Error(
      `${command.join(" ")} failed (${exitCode})\n${stdout}${stderr}`
    )
  }
}

function contentType(path: string): string {
  switch (extname(path)) {
    case ".css":
      return "text/css; charset=utf-8"
    case ".html":
      return "text/html; charset=utf-8"
    case ".js":
      return "text/javascript; charset=utf-8"
    case ".map":
    case ".json":
      return "application/json; charset=utf-8"
    default:
      return "application/octet-stream"
  }
}

let server: ReturnType<typeof Bun.serve> | undefined
let browser: Awaited<ReturnType<typeof chromium.launch>> | undefined
try {
  await stat(cli).catch(async () =>
    run(["cargo", "build", "-p", "seseragi-cli"], root)
  )
  await run(
    [cli, "build", "--target", "web", source, "--out-dir", output],
    root
  )

  server = Bun.serve({
    hostname: "127.0.0.1",
    port: 0,
    async fetch(request) {
      const url = new URL(request.url)
      if (url.pathname === "/provider-probe") {
        return new Response("browser provider ready", {
          headers: { "content-type": "text/plain; charset=utf-8" },
        })
      }
      const relative =
        url.pathname === "/" ? "index.html" : url.pathname.slice(1)
      const path = normalize(join(output, relative))
      if (!path.startsWith(`${output}/`))
        return new Response("Forbidden", { status: 403 })
      try {
        return new Response(await readFile(path), {
          headers: { "content-type": contentType(path) },
        })
      } catch {
        return new Response("Not found", { status: 404 })
      }
    },
  })
  browser = await chromium.launch()
  const page = await browser.newPage()
  await page.addInitScript(() => {
    const probe = { listeners: 0 }
    Object.assign(globalThis, { __seseragiResourceProbe: probe })
    const add = EventTarget.prototype.addEventListener
    const remove = EventTarget.prototype.removeEventListener
    EventTarget.prototype.addEventListener = function (
      type,
      callback,
      options
    ) {
      if (this instanceof Element && this.id === "app") probe.listeners += 1
      return add.call(
        this,
        type,
        callback as EventListenerOrEventListenerObject,
        options
      )
    }
    EventTarget.prototype.removeEventListener = function (
      type,
      callback,
      options
    ) {
      if (this instanceof Element && this.id === "app") probe.listeners -= 1
      return remove.call(
        this,
        type,
        callback as EventListenerOrEventListenerObject,
        options
      )
    }
  })
  const errors: string[] = []
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text())
  })
  page.on("pageerror", (error) => errors.push(error.message))
  page.on("requestfailed", (request) => {
    errors.push(
      `${request.method()} ${request.url()}: ${request.failure()?.errorText ?? "failed"}`
    )
  })
  await page.goto(`http://127.0.0.1:${server.port}`)
  await page.locator("#count").waitFor().catch(async (cause) => {
    const status = await page.evaluate(
      () => document.documentElement.dataset.seseragiStatus
    )
    throw new Error(
      `web build did not mount (status=${status ?? "unset"}, errors=${errors.join(" | ")})`,
      { cause }
    )
  })
  assert.equal(await page.locator("#count").textContent(), "0")
  await page.locator("#increment").click()
  await page.waitForFunction(
    () => document.querySelector("#count")?.textContent === "1"
  )
  assert.equal(
    await page.evaluate(() => document.documentElement.dataset.seseragiStatus),
    "mounted"
  )
  const mountedResources = await page.evaluate(
    () =>
      (
        globalThis as typeof globalThis & {
          readonly __seseragiResourceProbe: { readonly listeners: number }
        }
      ).__seseragiResourceProbe.listeners
  )
  assert.ok(mountedResources > 0)
  await page.evaluate(() => globalThis.dispatchEvent(new Event("pagehide")))
  await page.waitForFunction(
    () =>
      document.querySelector("#app")?.childElementCount === 0 &&
      (
        globalThis as typeof globalThis & {
          readonly __seseragiResourceProbe: { readonly listeners: number }
        }
      ).__seseragiResourceProbe.listeners === 0
  )
  assert.deepEqual(errors, [])
  console.log(
    "Web build browser smoke passed: providers, mount, click, Signal update, dispose"
  )
} finally {
  await browser?.close()
  server?.stop(true)
  await rm(directory, { recursive: true, force: true })
}
