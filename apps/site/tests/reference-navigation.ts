import assert from "node:assert/strict"
import { join } from "node:path"
import type { Browser } from "../../playground/node_modules/@playwright/test"

const principles = [
  ["expression-oriented", "Expression-oriented", "式指向"],
  ["immutable-by-default", "Immutable by default", "デフォルトで不変"],
  ["no-hidden-danger", "No hidden danger", "危険を暗黙に隠さない"],
  [
    "backend-independent-semantics",
    "Backend-independent semantics",
    "生成先に依存しない意味",
  ],
  ["diagnosable-behavior", "Diagnosable behavior", "説明可能な振る舞い"],
  ["visible-costs", "Visible costs", "コストを見えるようにする"],
  ["readable-density", "Readable density", "読みやすい密度"],
] as const

export async function verifyReferenceNavigation(
  browser: Browser,
  origin: string,
  screenshots?: string
) {
  for (const width of [390, 1280]) {
    const context = await browser.newContext({
      viewport: { width, height: 900 },
    })
    try {
      const page = await context.newPage()
      const errors: string[] = []
      page.on("pageerror", (error) => errors.push(error.message))
      page.on("console", (message) => {
        if (message.type() === "error") errors.push(message.text())
      })
      for (const locale of ["en", "ja"]) {
        const prefix = locale === "ja" ? "/ja" : ""
        const model = `${prefix}/docs/language/model/`
        for (const [slug, english, japanese] of principles) {
          await page.goto(`${origin}${model}design-principles/`)
          const title = locale === "ja" ? japanese : english
          const destination = `${model}${slug}/`
          const link = page.locator(`.article-content a[href="${destination}"]`)
          assert.equal(await link.innerText(), title)
          await link.click()
          await page.waitForURL(`${origin}${destination}`)
          assert.equal(await page.locator("h1").innerText(), title)
          assert.ok(
            (await page.locator(".breadcrumbs").innerText()).endsWith(title)
          )
          assert.equal(
            await page
              .locator(".docs-sidebar .sidebar-link.current")
              .textContent(),
            title
          )
          assert.equal(
            await page.evaluate(
              () => document.documentElement.scrollWidth > innerWidth
            ),
            false
          )
          if (screenshots && locale === "ja" && slug === "expression-oriented")
            await page.screenshot({
              path: join(screenshots, `reference-title-${width}.png`),
              fullPage: false,
            })
        }
        await page.goto(`${origin}${prefix}/docs/language/effects/task/`)
        const aliasPath = `${prefix}/docs/language/types/generic-aliases/`
        const aliasLink = page.locator(
          `.article-content a[href="${aliasPath}"]`
        )
        const aliasTitle =
          locale === "ja" ? "ジェネリックな型別名" : "Generic aliases"
        assert.equal(await aliasLink.innerText(), aliasTitle)
        await aliasLink.click()
        await page.waitForURL(`${origin}${aliasPath}`)
        assert.equal(await page.locator("h1").innerText(), aliasTitle)
      }
      assert.deepEqual(errors, [])
    } finally {
      await context.close()
    }
  }
}
