import assert from "node:assert/strict"
import { mkdirSync, readFileSync } from "node:fs"
import { resolve } from "node:path"
import {
  type Browser,
  chromium,
} from "../../playground/node_modules/@playwright/test"
import { staticSiteHandler } from "../scripts/static-site-handler"
import { verifyFunctionChapter } from "./function-chapter-browser"

const root = resolve(import.meta.dir, "../../..")
const route = "/docs/language/syntax/function-application/"
const source = readFileSync(
  resolve(
    root,
    "apps/site/examples/src/language/pilot-function-application.ssrg"
  ),
  "utf8"
)
const chapterSource = readFileSync(
  resolve(root, "apps/site/examples/src/language/chapter-functions.ssrg"),
  "utf8"
)
const invalid = readFileSync(
  resolve(
    root,
    "apps/site/examples/invalid/src/language/pilot-function-application.ssrg"
  ),
  "utf8"
)
const anchors = [
  "understand-this",
  "reading-the-example",
  "why",
  "rules",
  "mistakes",
  "related-rules",
]

export async function verifyArticleComposition(
  browser: Browser,
  origin: string
) {
  let cases = 0
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
          await page.goto(origin + prefix + route)
          const article = page.locator(".article-content")
          for (const anchor of anchors)
            assert.equal(await article.locator(`h2#${anchor}`).count(), 1)
          assert.deepEqual(
            await article.locator(".seseragi-highlight").allTextContents(),
            [chapterSource, source, invalid]
          )
          assert.deepEqual(
            await article
              .locator(".terminal-panel pre > code")
              .allTextContents(),
            ["build", "3"]
          )
          const text = await article.innerText()
          for (const detail of [
            "SES-T0101",
            "addOne 2",
            "Int -> (Int -> Int)",
            "identity<String>",
            "answer ()",
          ])
            assert.ok(text.includes(detail), `${prefix}${route}: ${detail}`)
          assert.ok(
            await article.locator("#rules").evaluate((heading) => {
              const output = document.querySelector(".terminal-panel")
              return !!output && !!(output.compareDocumentPosition(heading) & 4)
            }),
            "Output comes before exact rules"
          )
          if (screenshots)
            await page.screenshot({
              path: resolve(
                screenshots,
                `composition-${prefix ? "ja" : "en"}-${width}-${javaScriptEnabled ? "js" : "no-js"}.png`
              ),
              fullPage: true,
            })
          let outline = page.locator('.on-this-page a[href="#rules"]')
          if (!(await outline.isVisible())) {
            await page.locator(".mobile-on-this-page summary").click()
            outline = page.locator('.mobile-on-this-page a[href="#rules"]')
          }
          assert.equal(await outline.count(), 1)
          await outline.click()
          assert.ok(page.url().endsWith("#rules"))
          assert.ok(await page.locator("#rules").isVisible())
          assert.ok(
            await page.evaluate(
              () => document.documentElement.scrollWidth <= innerWidth
            ),
            `No horizontal overflow at ${width}px`
          )
          // Follow a real detail destination, then return to the same identity.
          const destination = `${prefix}/docs/language/types/function-types-and-currying/`
          await article
            .locator(`#related-rules ~ p a[href="${destination}"]`)
            .click()
          assert.equal(new URL(page.url()).pathname, destination)
          await page.goBack()
          assert.equal(new URL(page.url()).pathname, prefix + route)
          const other = prefix ? "" : "/ja"
          const localeLink = page.locator(
            `.language-menu a[href="${other}${route}"]`
          )
          assert.equal(await localeLink.count(), 1)
          await page.locator(".language-menu summary").focus()
          await page.keyboard.press("Enter")
          await localeLink.focus()
          await Promise.all([
            page.waitForURL(origin + other + route),
            page.keyboard.press("Enter"),
          ])
          assert.equal(new URL(page.url()).pathname, other + route)
          cases++
        }
        assert.deepEqual(errors, [])
      } finally {
        await context.close()
      }
    }
  console.info(`Article composition: ${cases} locale/viewport/JS cases passed`)
}

if (import.meta.main) {
  const output = resolve(process.argv[2] ?? "target/site")
  const configuration = JSON.parse(
    readFileSync(resolve(root, "apps/site/vercel.json"), "utf8")
  )
  const server = Bun.serve({
    port: 0,
    hostname: "127.0.0.1",
    fetch: staticSiteHandler(output, configuration),
  })
  try {
    const browser = await chromium.launch(
      process.env.SESERAGI_TEST_BROWSER_PATH
        ? { executablePath: process.env.SESERAGI_TEST_BROWSER_PATH }
        : {}
    )
    try {
      await verifyArticleComposition(browser, `http://127.0.0.1:${server.port}`)
      await verifyFunctionChapter(browser, `http://127.0.0.1:${server.port}`)
    } finally {
      await browser.close()
    }
  } finally {
    server.stop(true)
  }
}
