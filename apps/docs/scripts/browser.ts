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
          assert.equal(await page.locator('script[type="module"]').count(), 2)
          assert.equal(await page.locator("#docs-search").textContent(), "")
          assert.equal(
            await page.getByRole("button", { name: "コピー" }).count(),
            0
          )
          assert.equal(
            await page
              .locator('img[src="/docs/assets/seseragi-icon.svg"]')
              .count(),
            1
          )
          assert.ok(
            await page.evaluate(
              () => document.documentElement.scrollWidth <= innerWidth
            ),
            `Horizontal overflow at ${width}px: ${item.route}`
          )
          const examples = page.locator(".code-example:not(.terminal-example)")
          for (let index = 0; index < (await examples.count()); index++) {
            const example = examples.nth(index)
            const source = await example
              .locator("code.seseragi-highlight")
              .textContent()
            const playground = await example
              .getByRole("link", { name: "Playgroundで試す", exact: false })
              .getAttribute("href")
            assert.ok(playground)
            assert.equal(sourceFromPlaygroundUrl(playground), source)
          }
        }
        await page
          .locator("nav")
          .getByRole("link", { name: "Getting Started", exact: true })
          .click()
        assert.ok(page.url().endsWith("/docs/getting-started/"))
        assert.equal(
          await page
            .locator("pre code.seseragi-highlight")
            .first()
            .textContent(),
          readFileSync(
            resolve(
              import.meta.dir,
              "../../../examples/spec/lessons/01-hello-world.ssrg"
            ),
            "utf8"
          )
        )
        const source = readFileSync(
          resolve(
            import.meta.dir,
            "../../../examples/spec/lessons/01-hello-world.ssrg"
          ),
          "utf8"
        )
        const playground = await page
          .getByRole("link", { name: "Playgroundで試す", exact: false })
          .first()
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
        await page.goto(`http://127.0.0.1:${server.port}/docs/`)
        await page
          .locator("aside nav")
          .getByRole("link", { name: "API Reference", exact: true })
          .click()
        await page
          .getByRole("link", { name: "std/effect", exact: true })
          .click()
        assert.equal(await page.locator("h1").textContent(), "std/effect")
        assert.ok((await page.locator(".reference-item").count()) > 0)
        assert.ok(
          (await page.locator(".reference-item .tok-keyword").count()) > 0
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `reference-${width}.png`),
            fullPage: true,
          })
        assert.deepEqual(failures, [])
        await context.close()
      }
      for (const width of [1280, 390]) {
        const origin = `http://127.0.0.1:${server.port}`
        const context = await browser.newContext({
          viewport: { width, height: 900 },
        })
        await context.grantPermissions(["clipboard-read", "clipboard-write"], {
          origin,
        })
        const page = await context.newPage()
        const failures: string[] = []
        page.on("pageerror", (error) => failures.push(error.message))
        page.on("response", (response) => {
          if (response.status() >= 400) failures.push(response.url())
        })
        await page.goto(`${origin}/docs/getting-started/`)
        assert.equal(
          await page
            .getByRole("button", { name: "コピー" })
            .first()
            .isVisible(),
          true
        )
        const search = page.getByRole("searchbox", {
          name: "ドキュメントを検索",
        })
        await search.focus()
        await page.keyboard.type("std/effect::fail")
        const result = page.getByRole("link", {
          name: /std\/effect · fail/,
        })
        await result.waitFor()
        await page.waitForTimeout(100)
        assert.equal(
          await page.evaluate(() => document.activeElement?.id),
          "docs-search-input"
        )
        assert.ok((await result.textContent())?.includes("typed error"))
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `search-${width}.png`),
            fullPage: true,
          })
        assert.ok(
          await page.evaluate(
            () => document.documentElement.scrollWidth <= innerWidth
          ),
          `Enabled-JS search overflow at ${width}px`
        )
        await page.keyboard.press("Tab")
        const resultHref = await result.getAttribute("href")
        assert.ok(
          resultHref?.includes("/docs/reference/std/effect/#reference-")
        )
        assert.equal(
          await page.locator(":focus").getAttribute("href"),
          resultHref
        )
        await Promise.all([
          page.waitForURL(/\/docs\/reference\/std\/effect\/#reference-/),
          result.click(),
        ])
        assert.ok(page.url().includes("/docs/reference/std/effect/#reference-"))
        assert.equal(await page.locator("h1").textContent(), "std/effect")

        if (width === 1280) {
          await page.goto(`${origin}/docs/getting-started/`)
          const example = page
            .locator(".code-example:not(.terminal-example)")
            .first()
          const expected = await example.locator("code").textContent()
          await example.getByRole("button", { name: "コピー" }).click()
          assert.equal(
            await page.evaluate(() => navigator.clipboard.readText()),
            expected
          )
          assert.equal(
            await page.locator("#docs-copy-status").textContent(),
            "コードをクリップボードへコピーしました"
          )
        }
        assert.deepEqual(failures, [])
        await context.close()
      }
      console.log(
        `Docs browser: ${manifest.pages.length} routes × 3 static viewports plus enabled-JS desktop/mobile; navigation, search, copy, Reference, source, metadata, shared logo, highlighting, skip link and overflow passed`
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
