import assert from "node:assert/strict"
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { chromium } from "../../playground/node_modules/@playwright/test"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { build } from "./build"

const temporary = mkdtempSync(join(tmpdir(), "docs-browser-"))
const output = join(temporary, "site")
const screenshots = process.env.DOCS_SCREENSHOTS
if (screenshots) mkdirSync(screenshots, { recursive: true })
try {
  const manifest = build({
    out: output,
    origin: "https://docs.example.com",
    base: "/docs/",
  })
  const server = Bun.serve({
    port: 0,
    hostname: "127.0.0.1",
    fetch(request) {
      const path = new URL(request.url).pathname
      if (!path.startsWith("/docs/"))
        return new Response("Not found", { status: 404 })
      const file = resolve(
        output,
        path.slice(6) + (path.endsWith("/") ? "index.html" : "")
      )
      if (!file.startsWith(`${output}/`))
        return new Response("Not found", { status: 404 })
      return new Response(Bun.file(file))
    },
  })
  try {
    const browser = await chromium.launch()
    try {
      for (const width of [1280, 390, 320]) {
        const context = await browser.newContext({
          viewport: { width, height: 900 },
          javaScriptEnabled: false,
        })
        const page = await context.newPage()
        const failures: string[] = []
        page.on("pageerror", (error) => failures.push(error.message))
        page.on("response", (response) => {
          if (response.status() >= 400) failures.push(response.url())
        })
        for (const item of manifest.pages) {
          await page.goto(`http://127.0.0.1:${server.port}${item.route}`)
          assert.equal(await page.locator("h1").textContent(), item.title)
          assert.equal(await page.locator("main").count(), 1)
          assert.equal(await page.locator('meta[name="viewport"]').count(), 1)
          assert.equal(await page.locator('meta[charset="utf-8"]').count(), 1)
          assert.equal(
            await page.locator('meta[name="description"]').count(),
            1
          )
          assert.equal(await page.locator("script").count(), 0)
          assert.equal(
            await page
              .locator('img[src="/docs/assets/seseragi-icon.svg"]')
              .count(),
            1
          )
          assert.ok(
            await page.evaluate(
              () => document.documentElement.scrollWidth <= innerWidth
            )
          )
        }
        await page
          .locator("nav")
          .getByRole("link", { name: "はじめてのプログラム", exact: true })
          .click()
        assert.ok(page.url().endsWith("/docs/getting-started/"))
        assert.equal(
          await page.locator("pre code").textContent(),
          readFileSync(
            resolve(
              import.meta.dir,
              "../../../examples/spec/lessons/02-values-and-functions.ssrg"
            ),
            "utf8"
          )
        )
        const source = readFileSync(
          resolve(
            import.meta.dir,
            "../../../examples/spec/lessons/02-values-and-functions.ssrg"
          ),
          "utf8"
        )
        const playground = await page
          .getByRole("link", { name: "Playgroundで試す", exact: false })
          .getAttribute("href")
        assert.ok(playground)
        assert.equal(sourceFromPlaygroundUrl(playground), source)
        assert.ok(
          (await page.locator(".seseragi-highlight .tok-keyword").count()) > 0
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `docs-${width}.png`),
            fullPage: true,
          })
        await page.goto(`http://127.0.0.1:${server.port}/docs/`)
        await page.keyboard.press("Tab")
        assert.equal(await page.locator(":focus").textContent(), "本文へ移動")
        await page.keyboard.press("Enter")
        assert.ok(page.url().endsWith("#content"))
        assert.deepEqual(failures, [])
        await context.close()
      }
      console.log(
        "Docs browser: 4 routes × 3 viewports; JS disabled; navigation, source, metadata, skip link and overflow passed"
      )
    } finally {
      await browser.close()
    }
  } finally {
    server.stop(true)
  }
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
