import { expect, test } from "bun:test"
import { mkdtemp, readFile, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { resolve } from "node:path"
import { chromium } from "playwright"

test("portable benchmark kernel measures reactive DOM and tears its host down", async () => {
  const temporary = await mkdtemp(
    resolve(tmpdir(), "seseragi-benchmark-browser-")
  )
  let browser: Awaited<ReturnType<typeof chromium.launch>> | undefined
  let server: ReturnType<typeof Bun.serve> | undefined
  try {
    const bundle = resolve(temporary, "main.js")
    const child = Bun.spawn(
      [
        process.execPath,
        "build",
        resolve(
          import.meta.dir,
          "../../../runtime/ts/fixtures/benchmark-quality/browser.ts"
        ),
        "--target=browser",
        `--outfile=${bundle}`,
      ],
      { stdout: "pipe", stderr: "pipe" }
    )
    const error = await new Response(child.stderr).text()
    expect(await child.exited, error).toBe(0)
    const javascript = await readFile(bundle, "utf8")
    server = Bun.serve({
      hostname: "127.0.0.1",
      port: 0,
      fetch(request) {
        return new URL(request.url).pathname === "/main.js"
          ? new Response(javascript, {
              headers: { "content-type": "text/javascript" },
            })
          : new Response(
              '<!doctype html><script type="module" src="/main.js"></script>',
              { headers: { "content-type": "text/html" } }
            )
      },
    })
    browser = await chromium.launch()
    const page = await browser.newPage()
    const errors: string[] = []
    page.on("pageerror", (error) => errors.push(error.message))
    await page.goto(`http://127.0.0.1:${server.port}`)
    await page.waitForFunction(
      () => document.documentElement.dataset.benchmark === "complete"
    )
    expect(errors).toEqual([])
    const report = await page.evaluate(
      () =>
        (
          globalThis as typeof globalThis & {
            benchmarkReport: {
              cases: { status: string; samples: number[]; iterations: number }[]
            }
          }
        ).benchmarkReport
    )
    expect(report.cases).toHaveLength(4)
    for (const entry of report.cases) {
      expect(entry.status).toBe("passed")
      expect(entry.samples).toHaveLength(3)
      expect(
        entry.samples.every((sample) => sample * entry.iterations >= 1_000_000)
      ).toBe(true)
    }
    expect(await page.locator("main").count()).toBe(0)
  } finally {
    await browser?.close()
    server?.stop(true)
    await rm(temporary, { recursive: true, force: true })
  }
}, 30000)
