import { afterAll, beforeAll, expect, test } from "bun:test"
import { mkdtemp, readFile, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { type Browser, chromium } from "playwright"

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..")
let browser: Browser | undefined
let server: ReturnType<typeof Bun.serve> | undefined

beforeAll(async () => {
  const build = await Bun.build({
    entrypoints: [
      resolve(root, "runtime/ts/fixtures/dom-lifecycle-browser/main.ts"),
    ],
    target: "browser",
    format: "esm",
    minify: false,
  })
  expect(build.success).toBe(true)
  const output = build.outputs[0]
  if (output === undefined) throw new Error("missing lifecycle browser bundle")
  const javascript = await output.text()
  server = Bun.serve({
    hostname: "127.0.0.1",
    port: 0,
    fetch(request) {
      const pathname = new URL(request.url).pathname
      if (pathname === "/main.js") {
        return new Response(javascript, {
          headers: { "content-type": "text/javascript; charset=utf-8" },
        })
      }
      return new Response(
        '<!doctype html><html><body><script type="module" src="/main.js"></script></body></html>',
        { headers: { "content-type": "text/html; charset=utf-8" } }
      )
    },
  })
  browser = await chromium.launch()
})

afterAll(async () => {
  await browser?.close()
  server?.stop(true)
})

test("owns mount, hydration, coarse updates, cancellation, and cleanup in a browser", async () => {
  if (browser === undefined || server === undefined) {
    throw new Error("browser lifecycle harness did not start")
  }
  const page = await browser.newPage()
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text())
  })
  await page.goto(`http://127.0.0.1:${server.port}`)
  await page.waitForFunction(
    () => document.documentElement.dataset.domLifecycle === "complete"
  )
  expect(errors).toEqual([])
  expect(
    await page.evaluate(
      () =>
        (
          window as typeof window & {
            readonly domLifecycleResult?: unknown
          }
        ).domLifecycleResult
    )
  ).toEqual({
    strictMismatchPath: [0, 0],
    dispatched: 1,
    duplicateTargetRejected: true,
    coarseUpdateRendered: true,
    hydrationPreservedIdentity: true,
    replacementPreservedAncestor: true,
    cancellationReleasedTarget: true,
    targetRemoval: "DomTargetRemoved",
  })
  await page.close()
}, 30_000)

test("runs promoted DOM lifecycle fixtures through the CLI web product route", async () => {
  if (browser === undefined) throw new Error("browser harness did not start")
  const directory = await mkdtemp(resolve(tmpdir(), "seseragi-dom-fixtures-"))
  const cli = resolve(root, "target/debug/seseragi")
  const fixtureRoots = {
    hydration: resolve(
      root,
      "examples/spec/fixtures/projects/dom-hydration-mismatch"
    ),
    signal: resolve(
      root,
      "examples/spec/fixtures/projects/dom-signal-lifecycle"
    ),
  }
  const outputs = {
    hydration: resolve(directory, "hydration"),
    signal: resolve(directory, "signal"),
  }
  let fixtureServer: ReturnType<typeof Bun.serve> | undefined
  try {
    await runCommand(["cargo", "build", "-p", "seseragi-cli"])
    await runCommand([
      cli,
      "build",
      fixtureRoots.hydration,
      "--out-dir",
      outputs.hydration,
    ])
    await runCommand([
      cli,
      "build",
      fixtureRoots.signal,
      "--out-dir",
      outputs.signal,
    ])
    const hydrationIndex = (
      await readFile(resolve(outputs.hydration, "index.html"), "utf8")
    ).replace('<div id="app"></div>', '<div id="app"><p>server</p></div>')
    const signalIndex = await readFile(
      resolve(outputs.signal, "index.html"),
      "utf8"
    )
    fixtureServer = Bun.serve({
      hostname: "127.0.0.1",
      port: 0,
      async fetch(request) {
        const url = new URL(request.url)
        const match = /^\/(hydration|signal)(\/.*)?$/.exec(url.pathname)
        if (match === null) return new Response("Not found", { status: 404 })
        const fixture = match[1] as keyof typeof outputs
        const relative =
          match[2] === undefined || match[2] === "/"
            ? "index.html"
            : match[2].slice(1)
        if (relative === "index.html") {
          return new Response(
            fixture === "hydration" ? hydrationIndex : signalIndex,
            { headers: { "content-type": "text/html; charset=utf-8" } }
          )
        }
        const file = resolve(outputs[fixture], relative)
        if (!file.startsWith(`${outputs[fixture]}/`)) {
          return new Response("Forbidden", { status: 403 })
        }
        return new Response(await readFile(file), {
          headers: {
            "content-type": relative.endsWith(".js")
              ? "text/javascript; charset=utf-8"
              : relative.endsWith(".css")
                ? "text/css; charset=utf-8"
                : "application/octet-stream",
          },
        })
      },
    })

    const hydrationPage = await browser.newPage()
    const hydrationErrors: string[] = []
    hydrationPage.on("pageerror", (error) =>
      hydrationErrors.push(error.message)
    )
    await hydrationPage.goto(
      `http://127.0.0.1:${fixtureServer.port}/hydration/`
    )
    await hydrationPage.waitForFunction(
      () => document.documentElement.dataset.seseragiStatus === "completed"
    )
    expect(await hydrationPage.locator("#app").innerHTML()).toBe(
      "<p>server</p>"
    )
    expect(hydrationErrors).toEqual([])
    await hydrationPage.close()

    const signalPage = await browser.newPage()
    const signalErrors: string[] = []
    signalPage.on("pageerror", (error) => signalErrors.push(error.message))
    await signalPage.goto(`http://127.0.0.1:${fixtureServer.port}/signal/`)
    await signalPage.locator("#count").waitFor()
    await signalPage.locator("#increment").click()
    await signalPage.waitForFunction(
      () => document.querySelector("#count")?.textContent === "1"
    )
    await signalPage.locator("#stop").click()
    await signalPage.waitForFunction(
      () =>
        document.documentElement.dataset.seseragiStatus === "completed" &&
        document.querySelector("#app")?.childNodes.length === 0
    )
    expect(signalErrors).toEqual([])
    await signalPage.close()
  } finally {
    fixtureServer?.stop(true)
    await rm(directory, { recursive: true, force: true })
  }
}, 30_000)

async function runCommand(command: string[]): Promise<void> {
  const child = Bun.spawn(command, {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  })
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ])
  if (exitCode !== 0) {
    throw new Error(
      `${command.join(" ")} failed (${exitCode})\n${stdout}${stderr}`
    )
  }
}
