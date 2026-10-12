import assert from "node:assert/strict"
import { join } from "node:path"
import {
  type Browser,
  expect,
} from "../../playground/node_modules/@playwright/test"
import type { compilerReferenceModules } from "../scripts/reference"

export async function verifySearch(
  browser: Browser,
  origin: string,
  modules: ReturnType<typeof compilerReferenceModules>,
  screenshots: string
) {
  const module = modules.find(({ specifier }) => specifier === "std/signal")
  const map = module?.items.find(({ name }) => name === "map")
  assert.ok(map)
  let cases = 0
  for (const javaScriptEnabled of [false, true])
    for (const width of [320, 390, 1280])
      for (const prefix of ["", "/ja"]) {
        const context = await browser.newContext({
          javaScriptEnabled,
          viewport: { width, height: 900 },
        })
        try {
          const page = await context.newPage()
          const errors: string[] = []
          page.on("pageerror", (error) => errors.push(error.message))
          page.on("console", (message) => {
            if (message.type() === "error") errors.push(message.text())
          })
          page.on("response", (response) => {
            if (response.status() >= 400)
              errors.push(`${response.status()} ${response.url()}`)
          })
          await page.goto(`${origin + prefix}/docs/`)
          assert.equal(await page.locator('script[src*="search"]').count(), 0)
          await Promise.all([
            page.waitForURL(`${origin + prefix}/docs/search/`),
            page.locator(`main a[href="${prefix}/docs/search/"]`).click(),
          ])
          const query = page.locator("#docs-search-query")
          await expect(page.locator('main a[href$="/docs/api/"]')).toBeVisible()
          if (!javaScriptEnabled) {
            await expect(query).toBeHidden()
            await Promise.all([
              page.waitForURL(`${origin + prefix}/docs/signals/`),
              page.locator(`main a[href="${prefix}/docs/signals/"]`).click(),
            ])
            assert.equal(
              new URL(page.url()).pathname,
              `${prefix}/docs/signals/`
            )
          } else {
            await expect(query).toBeVisible()
            await expect(page.locator("#docs-search-count")).toBeHidden()
            await query.fill("std/signal map")
            const result = page.locator("#docs-search-results a").first()
            await expect(result).toHaveAttribute(
              "href",
              `${prefix}/docs/api/signal/#${map.anchor}`
            )
            await query.evaluate((element) => {
              if (!(element instanceof HTMLInputElement))
                throw new Error("Missing search input")
              element.value = "transaction"
              element.dispatchEvent(
                new InputEvent("input", { bubbles: true, isComposing: true })
              )
            })
            await expect(result).toHaveText("map")
            await query.evaluate((element) =>
              element.dispatchEvent(
                new CompositionEvent("compositionend", { bubbles: true })
              )
            )
            await expect(result).toHaveText("transaction")
            const links = await page
              .locator("#docs-search-results a")
              .evaluateAll((elements) =>
                elements.map((element) => element.getAttribute("href"))
              )
            assert.ok(
              links.every((href) =>
                prefix ? href?.startsWith("/ja/") : !href?.startsWith("/ja/")
              )
            )
            await query.fill("std/signal map")
            await query.press("Enter")
            await expect(result).toBeFocused()
            await Promise.all([
              page.waitForURL(
                `${origin + prefix}/docs/api/signal/#${map.anchor}`
              ),
              page.keyboard.press("Enter"),
            ])
            await expect(page.locator(`#${map.anchor}`)).toBeVisible()
            assert.equal(
              await page.locator(`#${map.anchor} pre code`).textContent(),
              map.signature
            )
            await page.goto(
              `${origin + prefix}/docs/search/?q=${encodeURIComponent("<$>")}`
            )
            await expect(query).toHaveValue("<$>")
            await expect(
              page.locator("#docs-search-results a").first()
            ).toBeVisible()
            await expect(
              page.locator(
                '#docs-search-results a[href$="/docs/composition/transform/"]'
              )
            ).toBeVisible()
            if (prefix) {
              await query.fill("網羅性")
              await expect(
                page.locator(
                  '#docs-search-results a[href="/ja/docs/types/variants/"]'
                )
              ).toBeVisible()
              await page.screenshot({
                path: join(screenshots, `search-ja-${width}.png`),
                fullPage: true,
              })
            }
            await query.fill("map")
            assert.equal(
              await page.locator("#docs-search-results li").count(),
              40
            )
            await expect(page.locator("#docs-search-more")).toBeVisible()
            await query.fill("zzzz-unmatched")
            await expect(page.locator("#docs-search-empty")).toBeVisible()
            assert.equal(
              await page.locator("#docs-search-total").textContent(),
              "0"
            )
            await query.fill('<img src=x onerror="alert(1)">')
            assert.equal(
              await page.locator("#docs-search-results img").count(),
              0
            )
            await expect(page.locator("#docs-search-empty")).toBeVisible()
            await query.fill("")
            await expect(page.locator("#docs-search-count")).toBeHidden()
            await expect(page.locator("#docs-search-empty")).toBeHidden()
            assert.equal(new URL(page.url()).searchParams.has("q"), false)
          }
          assert.ok(
            await page.evaluate(
              () => document.documentElement.scrollWidth <= innerWidth
            ),
            `Search navigation layout: ${page.url()}, width=${width}, JS=${javaScriptEnabled}`
          )
          assert.deepEqual(errors, [])
          cases++
        } finally {
          await context.close()
        }
      }
  for (const prefix of ["", "/ja"]) {
    const context = await browser.newContext()
    try {
      await context.route("**/assets/search-index-*.js", (route) =>
        route.abort()
      )
      const page = await context.newPage()
      await page.goto(`${origin + prefix}/docs/search/`)
      await expect(page.locator("#docs-search-unavailable")).toBeVisible()
      await expect(page.locator("#docs-search-query")).toBeHidden()
      await Promise.all([
        page.waitForURL(`${origin + prefix}/docs/api/`),
        page.locator(`main a[href="${prefix}/docs/api/"]`).click(),
      ])
      assert.equal(new URL(page.url()).pathname, `${prefix}/docs/api/`)
      cases++
    } finally {
      await context.close()
    }
  }
  console.info(
    `Search: ${cases} locale/viewport/JS/failure cases; article and declaration discovery, keyboard, direct query, empty/limited results, CSP, and fallback verified`
  )
}
