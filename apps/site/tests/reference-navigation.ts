import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
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

const outputs: Record<string, string> = {
  "expression-oriented": "pass",
  "immutable-by-default": "10 -> 11",
  "no-hidden-danger": "not found",
  "backend-independent-semantics": "-3, 3.5",
  "diagnosable-behavior": "`[]",
  "visible-costs": "[2, 4, 6]",
  "readable-density": "7",
}

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
          const article = page.locator(".article-content")
          const code = article.locator(".code-panel pre > code")
          assert.equal(await code.count(), 2)
          assert.equal(
            await code.nth(0).textContent(),
            readFileSync(
              resolve(
                import.meta.dir,
                `../examples/src/language/principle-${slug}.ssrg`
              ),
              "utf8"
            )
          )
          assert.equal(await code.nth(1).textContent(), outputs[slug])
          for (const section of [
            "meaning",
            "reading-the-example",
            "why",
            "limits",
            "related-rules",
          ])
            assert.equal(await article.locator(`h2#${section}`).count(), 1)
          // Six explanation paragraphs plus the meaning and output introduction.
          // A missing locale paragraph must not disappear silently through zip.
          assert.equal(await article.locator(":scope > p").count(), 8)
          assert.equal(await article.locator(".callout").count(), 0)
          assert.equal(
            await page.locator(".breadcrumb-current").innerText(),
            title,
            `${destination}: current breadcrumb must remain readable at ${width}px`
          )
          const currentBreadcrumb = await page
            .locator(".breadcrumb-current")
            .boundingBox()
          assert.ok(currentBreadcrumb && currentBreadcrumb.width > 0)
          assert.equal(
            await page.locator(".breadcrumbs").evaluate((trail) => {
              const current = trail.querySelector(".breadcrumb-current")
              return (
                current !== null &&
                current.scrollWidth <= current.clientWidth + 1 &&
                trail.scrollWidth <= trail.clientWidth + 1
              )
            }),
            true,
            `${destination}: breadcrumb text must not be clipped at ${width}px`
          )
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
          if (screenshots && locale === "ja")
            await page.screenshot({
              path: join(screenshots, `principle-${slug}-${width}.png`),
              fullPage: true,
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
