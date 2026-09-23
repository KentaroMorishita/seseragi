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
    let rejectPageError: ((error: Error) => void) | undefined
    const pageError = new Promise<never>((_resolve, reject) => {
      rejectPageError = reject
    })
    page.on("pageerror", (error) => {
      errors.push(error.message)
      rejectPageError?.(error)
    })
    await page.goto(`http://127.0.0.1:${server.port}`)
    await Promise.race([
      page.waitForFunction(
        () => document.documentElement.dataset.benchmark === "complete"
      ),
      pageError,
    ])
    expect(errors).toEqual([])
    const result = await page.evaluate(() => {
      const state = globalThis as typeof globalThis & {
        benchmarkReport: {
          cases: { status: string; samples: number[]; iterations: number }[]
        }
        benchmarkDomTrace: Array<{
          schema: number
          sequence: number
          transactionId: number | null
          activeSubscriptions: number
          activeListeners: number
          type: string
          outcome?: string
          mutations?: Record<string, number>
        }>
      }
      return {
        report: state.benchmarkReport,
        trace: state.benchmarkDomTrace,
      }
    })
    expect(result.report.cases).toHaveLength(6)
    for (const entry of result.report.cases) {
      expect(entry.status).toBe("passed")
      expect(entry.samples).toHaveLength(3)
      expect(
        // Compare in report units: dividing then multiplying a valid duration
        // can round below the minimum (for example, 1_000_000 / 29 * 29).
        entry.samples.every((sample) => sample >= 1_000_000 / entry.iterations)
      ).toBe(true)
    }
    expect(result.trace.length).toBeGreaterThan(0)
    expect(
      result.trace.every(
        (event, index) => event.schema === 1 && event.sequence === index
      )
    ).toBe(true)
    expect(
      result.trace.some(
        (event) =>
          event.type === "binding-update" &&
          event.transactionId !== null &&
          event.outcome === "equal-skip"
      )
    ).toBe(true)
    expect(
      result.trace.some(
        (event) =>
          event.type === "binding-update" && event.mutations?.replaced === 1
      )
    ).toBe(true)
    expect(
      result.trace.some(
        (event) =>
          event.type === "binding-update" && event.mutations?.property === 1
      )
    ).toBe(true)
    expect(
      result.trace.some(
        (event) =>
          event.type === "binding-update" && event.mutations?.style === 1
      )
    ).toBe(true)
    expect(result.trace.at(-1)).toMatchObject({
      type: "scope",
      operation: "cleanup",
      activeSubscriptions: 0,
      activeListeners: 0,
    })
    expect(await page.locator("main").count()).toBe(0)
  } finally {
    await browser?.close()
    server?.stop(true)
    await rm(temporary, { recursive: true, force: true })
  }
}, 30000)
