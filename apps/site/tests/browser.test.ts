import assert from "node:assert/strict"
import { mkdirSync, mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { chromium } from "../../playground/node_modules/@playwright/test"
import { buildSite } from "../scripts/build"

const temporary = mkdtempSync(join(tmpdir(), "seseragi-site-browser-"))
const output = join(temporary, "site")
const screenshots = process.env.SITE_SCREENSHOTS
if (screenshots) mkdirSync(screenshots, { recursive: true })

try {
  buildSite({ output, origin: "https://seseragi.example" })
  const server = Bun.serve({
    port: 0,
    hostname: "127.0.0.1",
    fetch(request) {
      const url = new URL(request.url)
      const relative = url.pathname.endsWith("/")
        ? `${url.pathname.slice(1)}index.html`
        : url.pathname.slice(1)
      const path = resolve(output, relative || "index.html")
      if (!path.startsWith(`${output}/`))
        return new Response("Not found", { status: 404 })
      return new Response(Bun.file(path))
    },
  })
  try {
    const browser = await chromium.launch()
    try {
      for (const width of [1280, 390]) {
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

        await page.goto(`http://127.0.0.1:${server.port}/`)
        assert.equal(await page.locator("h1").textContent(), "Seseragi")
        assert.equal(await page.locator("html").getAttribute("lang"), "en")
        assert.equal(await page.locator(".seseragi-highlight").count(), 1)
        assert.ok(
          (await page.locator("body").innerText()).includes(
            "pub effect fn main"
          )
        )
        const homeWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          homeWidth <= width,
          `home ${width}px viewport is ${homeWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `home-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/syntax/function-application/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Function application"
        )
        assert.equal(await page.locator(".docs-sidebar").count(), 1)
        assert.equal(await page.locator(".on-this-page").count(), 1)
        assert.equal(await page.locator(".mobile-docs-navigation").count(), 1)
        assert.equal(await page.locator(".mobile-on-this-page").count(), 1)
        assert.ok(
          (await page.locator("body").innerText()).includes(
            "Application does not infer a different call grammar"
          )
        )
        const articleWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          articleWidth <= width,
          `article ${width}px viewport is ${articleWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `function-application-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/syntax/operator-precedence/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Operator precedence"
        )
        assert.ok(
          (await page.locator("body").innerText()).includes("a < b < c")
        )
        const operatorsWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          operatorsWidth <= width,
          `operator article ${width}px viewport is ${operatorsWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `operator-precedence-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/types/type-system/`
        )
        assert.equal(await page.locator("h1").textContent(), "Type system")
        assert.ok(
          (await page.locator("body").innerText()).includes(
            "unresolved meaning becomes a diagnostic"
          )
        )
        const typeSystemWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          typeSystemWidth <= width,
          `type system article ${width}px viewport is ${typeSystemWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `type-system-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/types/requirement-merge/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Requirement merge"
        )
        assert.ok(
          (await page.locator("body").innerText()).includes("SES-E0001")
        )
        const requirementWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          requirementWidth <= width,
          `requirement article ${width}px viewport is ${requirementWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `requirement-merge-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/model/non-features/`
        )
        assert.equal(
          await page.locator("h1").textContent(),
          "Features the language does not have"
        )
        assert.ok(
          (await page.locator("body").innerText()).includes(
            "return, break, and continue"
          )
        )
        const modelWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          modelWidth <= width,
          `language model ${width}px viewport is ${modelWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `language-model-${width}.png`),
            fullPage: true,
          })

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/language/syntax/literals/`
        )
        assert.equal(await page.locator("h1").textContent(), "Literals")
        assert.equal(await page.locator(".seseragi-highlight").count(), 2)
        assert.ok(
          (await page.locator("body").innerText()).includes("SES-P0203")
        )
        const literalsWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          literalsWidth <= width,
          `literals ${width}px viewport is ${literalsWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `literals-${width}.png`),
            fullPage: true,
          })

        await page.goto(`http://127.0.0.1:${server.port}/docs/`)
        assert.equal(await page.locator("h1").textContent(), "Documentation")
        assert.equal(await page.locator(".docs-sidebar").count(), 1)
        assert.equal(await page.locator(".mobile-docs-navigation").count(), 1)

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/get-started/hello-seseragi/`
        )
        assert.equal(await page.locator("h1").textContent(), "Hello, Seseragi")
        assert.equal(await page.locator(".seseragi-highlight").count(), 1)
        assert.equal(await page.locator(".guide-next").count(), 1)
        assert.ok(
          (await page.locator("body").innerText()).includes(
            'pub effect fn main = println "Hello, Seseragi!"'
          )
        )
        const guideWidth = await page.evaluate(
          () => document.documentElement.scrollWidth
        )
        assert.ok(
          guideWidth <= width,
          `getting started ${width}px viewport is ${guideWidth}px`
        )
        if (screenshots)
          await page.screenshot({
            path: join(screenshots, `getting-started-${width}.png`),
            fullPage: true,
          })

        await page.goto(`http://127.0.0.1:${server.port}/ja/docs/`)
        assert.equal(await page.locator("html").getAttribute("lang"), "ja")
        assert.equal(await page.locator("h1").textContent(), "ドキュメント")

        await page.goto(
          `http://127.0.0.1:${server.port}/docs/library/array/function/get/`
        )
        assert.equal(await page.locator("h1").textContent(), "get")
        assert.ok(
          (await page.locator(".api-reference").innerText()).includes(
            "std/array::get"
          )
        )
        assert.equal(
          await page
            .locator(".docs-sidebar .sidebar-link.current")
            .evaluate(
              (link) =>
                link.closest("details")?.querySelector(":scope > summary")
                  ?.textContent
            ),
          "std/array"
        )

        assert.deepEqual(failures, [])
        await context.close()
      }
    } finally {
      await browser.close()
    }
  } finally {
    server.stop(true)
  }
} finally {
  rmSync(temporary, { recursive: true, force: true })
}
