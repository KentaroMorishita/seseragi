import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import type { Browser } from "../../playground/node_modules/@playwright/test"
import { traitReaders } from "../scripts/trait-readers"

const articles = [
  {
    route: "/docs/language/syntax/method-calls/",
    source: "syntax-reader-methods",
    output: "10\n15",
  },
  {
    route: "/docs/language/syntax/pipelines-and-low-precedence-application/",
    source: "syntax-reader-pipelines",
    output: "7\n7\n7",
  },
  {
    route: "/docs/language/data/records/",
    source: "reader-records",
    output: "10\n42\nAki",
  },
  {
    route: "/docs/language/data/structs/",
    source: "reader-structs",
    output: "Aki\nMio\n1",
  },
  {
    route: "/docs/language/data/tuples-arrays-and-lists/",
    source: "data-operations-collections",
    output: "answer\n42\nJust 20\nNothing\n20\n`[Ren, Aki, Mio]",
  },
  ...traitReaders.slice(0, 3).map(({ key, output }) => ({
    route: `/docs/language/traits/${key}/`,
    source: `trait-reader-${key}`,
    output: output.trimEnd(),
  })),
] as const

export async function verifyReaderArticles(
  browser: Browser,
  origin: string,
  screenshots?: string
) {
  const englishParagraphCounts = new Map<string, number>()
  for (const width of [320, 390, 1280]) {
    const context = await browser.newContext({
      viewport: { width, height: 900 },
      javaScriptEnabled: false,
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
      for (const locale of ["en", "ja"]) {
        const prefix = locale === "ja" ? "/ja" : ""
        for (const item of articles) {
          const route = prefix + item.route
          await page.goto(origin + route)
          const article = page.locator(".article-content")
          for (const id of [
            "understand-this",
            "rules",
            "mistakes",
            "related-rules",
            ...(item.route.includes("/traits/")
              ? item.route.endsWith("/model/")
                ? ["typescript"]
                : []
              : ["purpose", "reading-the-example"]),
          ])
            assert.equal(await article.locator(`h2#${id}`).count(), 1, route)
          // Compare paired locales; articles may have different explanation
          // lengths without losing or duplicating one locale's paragraphs.
          const paragraphs = await article.locator(":scope > p").count()
          if (locale === "en")
            englishParagraphCounts.set(item.route, paragraphs)
          else
            assert.equal(
              paragraphs,
              englishParagraphCounts.get(item.route),
              route
            )
          const code = article.locator(".code-panel pre > code")
          assert.equal(
            await code.nth(0).textContent(),
            readFileSync(
              resolve(
                import.meta.dir,
                `../examples/src/language/${item.source}.ssrg`
              ),
              "utf8"
            ),
            route
          )
          assert.equal(await code.nth(1).textContent(), item.output, route)
          const fontSize = await code
            .nth(0)
            .evaluate((code) => parseFloat(getComputedStyle(code).fontSize))
          assert.ok(fontSize >= 13 && fontSize <= 16, `${route}: code size`)
          const related = article.locator("h2#related-rules ~ p")
          assert.equal(await related.count(), 3, route)
          for (const paragraph of await related.all()) {
            assert.equal(await paragraph.locator("a").count(), 1)
            const reason = await paragraph.evaluate((p) => {
              const explanation = p.cloneNode(true) as HTMLElement
              for (const link of explanation.querySelectorAll("a"))
                link.remove()
              return explanation.textContent?.trim()
            })
            assert.ok(
              reason,
              `${route}: related link must have its own explanatory text`
            )
          }
          const topic = await page
            .locator(".reference-topic-link")
            .getAttribute("href")
          assert.ok(topic?.startsWith(`${prefix}/docs/language/`), route)
          assert.equal(
            await page.evaluate(
              () => document.documentElement.scrollWidth > innerWidth
            ),
            false,
            `${route}: horizontal overflow`
          )
          if (
            screenshots &&
            locale === "ja" &&
            ["records", "model"].some((name) =>
              item.route.endsWith(`/${name}/`)
            )
          )
            await page.screenshot({
              path: join(screenshots, `reader-${item.source}-${width}.png`),
              fullPage: true,
            })
        }
        for (const boundary of [
          "syntax/pipelines-and-low-precedence-application",
          "syntax/custom-operators",
          "model/non-features",
        ]) {
          const route = `${prefix}/docs/language/${boundary}/`
          await page.goto(origin + route)
          assert.equal(
            await page.locator(".reference-next").count(),
            0,
            `${route}: next must not cross a topic boundary`
          )
          assert.ok(
            await page.locator(".reference-topic-link").getAttribute("href")
          )
        }
      }
      assert.deepEqual(errors, [])
    } finally {
      await context.close()
    }
  }
}
