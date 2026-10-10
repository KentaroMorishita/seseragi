import assert from "node:assert/strict"
import { mkdirSync, readFileSync } from "node:fs"
import { resolve } from "node:path"
import type { Browser } from "../../playground/node_modules/@playwright/test"

const root = resolve(import.meta.dir, "../../..")
const chapter = [
  {
    route: "/docs/language/syntax/function-application/",
    sources: ["chapter-functions", "pilot-function-application"],
    invalid: "pilot-function-application",
    outputs: ["build", "3"],
    anchors: [
      "understand-this",
      "reading-the-example",
      "why",
      "rules",
      "mistakes",
      "related-rules",
    ],
    details: ["key", "String", "SES-T0101", "answer ()", "identity<String>"],
  },
  {
    route: "/docs/language/types/function-types-and-currying/",
    sources: [
      "chapter-function-values",
      "chapter-label-lengths",
      "pilot-currying",
    ],
    invalid: "pilot-currying",
    outputs: ["[ Build , bundle]\n[bundle]", "[5, 6, 5, 3]\n19", "3, 3"],
    anchors: [
      "understand-this",
      "reading-the-example",
      "why",
      "rules",
      "mistakes",
      "related-rules",
    ],
    details: [
      "String -> (String -> Bool)",
      "reduce 0 (+)",
      "SES-T0101",
      "Unit -> Int",
    ],
  },
  {
    route: "/docs/language/syntax/pipelines-and-low-precedence-application/",
    sources: ["syntax-reader-pipelines", "chapter-dollar-grouping"],
    invalid: "syntax-reader-pipelines",
    outputs: ["[> build, > bundle]\nTrue\nJust > build\nNothing", "READY"],
    anchors: [
      "understand-this",
      "purpose",
      "reading-the-example",
      "rules",
      "mistakes",
      "related-rules",
      "next-context",
    ],
    details: [
      "Maybe<String>",
      "badge <$> picked",
      "<*>",
      ">>=",
      "SES-T0101",
      "render (transform values)",
    ],
  },
] as const

function source(slug: string, invalid = false) {
  return readFileSync(
    resolve(
      root,
      `apps/site/examples/${invalid ? "invalid/" : ""}src/language/${slug}.ssrg`
    ),
    "utf8"
  )
}

export async function verifyFunctionChapter(browser: Browser, origin: string) {
  let pages = 0
  const screenshots = process.env.SITE_SCREENSHOTS
  if (screenshots) mkdirSync(screenshots, { recursive: true })
  for (const javaScriptEnabled of [false, true])
    for (const width of [320, 390, 1280]) {
      const context = await browser.newContext({
        viewport: { width, height: 900 },
        javaScriptEnabled,
      })
      try {
        const page = await context.newPage()
        const errors: string[] = []
        page.on("pageerror", (error) => errors.push(error.message))
        page.on("console", (message) => {
          if (message.type() === "error") errors.push(message.text())
        })
        page.on("response", (response) => {
          if (response.status() >= 400) errors.push(response.url())
        })
        for (const prefix of ["", "/ja"]) {
          await page.goto(origin + prefix + chapter[0].route)
          for (const [step, item] of chapter.entries()) {
            assert.equal(new URL(page.url()).pathname, prefix + item.route)
            assert.equal(await page.locator("h1").count(), 1)
            const article = page.locator(".article-content")
            for (const anchor of item.anchors)
              assert.equal(
                await article.locator(`h2#${anchor}`).count(),
                1,
                item.route
              )
            assert.deepEqual(
              await article.locator(".seseragi-highlight").allTextContents(),
              [
                ...item.sources.map((slug) => source(slug)),
                source(item.invalid, true),
              ],
              item.route
            )
            assert.deepEqual(
              await article
                .locator(".terminal-panel pre > code")
                .allTextContents(),
              [...item.outputs],
              item.route
            )
            const text = await article.innerText()
            for (const detail of item.details)
              assert.ok(
                text.includes(detail),
                `${prefix}${item.route}: ${detail}`
              )
            assert.ok(
              await article.locator("#rules").evaluate((heading) => {
                const output = document.querySelector(".terminal-panel")
                return (
                  !!output && !!(output.compareDocumentPosition(heading) & 4)
                )
              })
            )
            assert.ok(
              await page.evaluate(
                () => document.documentElement.scrollWidth <= innerWidth
              ),
              `${width}px: ${item.route}`
            )
            let outline = page.locator('.on-this-page a[href="#rules"]')
            if (!(await outline.isVisible())) {
              await page.locator(".mobile-on-this-page summary").click()
              outline = page.locator('.mobile-on-this-page a[href="#rules"]')
            }
            await outline.click()
            assert.ok(page.url().endsWith("#rules"))
            assert.ok(await article.locator("#rules").isVisible())
            if (screenshots)
              await page.screenshot({
                path: resolve(
                  screenshots,
                  `chapter-${step + 1}-${prefix ? "ja" : "en"}-${width}-${javaScriptEnabled ? "js" : "no-js"}.png`
                ),
                fullPage: true,
              })
            const next = chapter[step + 1]?.route ?? chapter[0].route
            const nextLink = article.locator(
              `#related-rules ~ p a[href="${prefix}${next}"]`
            )
            assert.equal(
              await nextLink.count(),
              1,
              `${item.route}: chapter continuation`
            )
            await nextLink.click()
            assert.equal(new URL(page.url()).pathname, prefix + next)
            pages++
          }
          const other = prefix ? "" : "/ja"
          const locale = page.locator(
            `.language-menu a[href="${other}${chapter[0].route}"]`
          )
          await page.locator(".language-menu summary").focus()
          await page.keyboard.press("Enter")
          await locale.focus()
          await Promise.all([
            page.waitForURL(origin + other + chapter[0].route),
            page.keyboard.press("Enter"),
          ])
          assert.equal(new URL(page.url()).pathname, other + chapter[0].route)
        }
        assert.deepEqual(errors, [])
      } finally {
        await context.close()
      }
    }
  console.info(
    `Function chapter: ${pages} page cases, 12 complete chapter walks passed`
  )
}
