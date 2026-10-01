import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import type { Browser } from "../../playground/node_modules/@playwright/test"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { entranceComparison } from "../scripts/comparisons"

export async function verifyExamplesAndReleases(
  browser: Browser,
  origin: string,
  screenshots?: string
) {
  const root = resolve(import.meta.dir, "../../..")
  const sources = [
    entranceComparison.seseragi,
    "apps/site/examples/src/language/reader-records.ssrg",
    "apps/site/examples/src/language/reader-pipelines.ssrg",
  ].map((path) => readFileSync(resolve(root, path), "utf8"))
  for (const width of [320, 390, 1280]) {
    const context = await browser.newContext({
      viewport: { width, height: 900 },
      javaScriptEnabled: false,
    })
    try {
      const page = await context.newPage()
      const errors: string[] = []
      page.on("pageerror", (error) => errors.push(error.message))
      for (const prefix of ["", "/ja"]) {
        for (const section of ["examples", "releases"]) {
          const route = `${prefix}/${section}/`
          const response = await page.goto(origin + route)
          assert.equal(response?.status(), 200)
          assert.equal(
            await page.locator("html").getAttribute("lang"),
            prefix ? "ja" : "en"
          )
          assert.equal(await page.locator(".site-build-error").count(), 0)
          assert.equal(
            await page.evaluate(
              () => document.documentElement.scrollWidth > innerWidth
            ),
            false,
            `${route}: page overflow`
          )
          const article = page.locator(".article-content")
          assert.ok(
            await article.locator(`a[href="${prefix}/docs/first-run/"]`).count()
          )
          if (section === "examples") {
            assert.deepEqual(
              await article.locator(".seseragi-highlight").allTextContents(),
              sources
            )
            const links = await article
              .locator("a.playground-link")
              .evaluateAll((anchors) =>
                anchors.map((anchor) => anchor.getAttribute("href") ?? "")
              )
            assert.deepEqual(links.map(sourceFromPlaygroundUrl), sources)
            for (const pre of await article.locator(".code-panel pre").all()) {
              const layout = await pre.evaluate((node) => ({
                size: parseFloat(getComputedStyle(node).fontSize),
                overflow: getComputedStyle(node).overflowX,
                width: node.getBoundingClientRect().width,
              }))
              assert.ok(layout.size >= 13)
              assert.equal(layout.overflow, "auto")
              assert.ok(layout.width <= width)
            }
          } else {
            assert.equal(
              await article
                .locator('a[href*="/releases/download/v0.61.19/"]')
                .count(),
              12
            )
            assert.ok((await article.innerText()).includes("development"))
          }
          if (screenshots)
            await page.screenshot({
              path: join(
                screenshots,
                `${section}-${prefix ? "ja" : "en"}-${width}.png`
              ),
              fullPage: true,
            })
          await article
            .locator(`a[href="${prefix}/docs/first-run/"]`)
            .first()
            .click()
          assert.equal(
            new URL(page.url()).pathname,
            `${prefix}/docs/first-run/`
          )
        }
      }
      assert.deepEqual(errors, [])
    } finally {
      await context.close()
    }
  }
}
