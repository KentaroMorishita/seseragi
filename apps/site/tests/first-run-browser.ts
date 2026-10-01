import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import type { Browser } from "../../playground/node_modules/@playwright/test"

export async function verifyFirstRun(
  browser: Browser,
  origin: string,
  screenshots?: string
) {
  const source = readFileSync(
    resolve(import.meta.dir, "../../../examples/samples/hello-world/main.ssrg"),
    "utf8"
  )
  for (const width of [320, 390, 1280]) {
    for (const javaScriptEnabled of [false, true]) {
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
        for (const prefix of ["", "/ja"]) {
          const route = `${prefix}/docs/first-run/`
          const response = await page.goto(origin + route)
          assert.equal(response?.status(), 200)
          assert.equal(
            await page.locator("html").getAttribute("lang"),
            prefix ? "ja" : "en"
          )
          assert.equal(
            await page.locator("h1").textContent(),
            prefix ? "最初のプログラムを実行する" : "Run your first program"
          )
          assert.equal(await page.locator(".docs-sidebar").count(), 0)
          assert.equal(
            await page.locator(".seseragi-highlight").textContent(),
            source
          )
          const commands = await page
            .locator(".terminal-panel code")
            .allTextContents()
          assert.equal(commands.length, 15)
          assert.equal(commands[10], "seseragi lint main.ssrg")
          assert.equal(commands[11], "seseragi run main.ssrg")
          assert.equal(commands[12], "Hello, Seseragi!")
          for (const code of await page.locator(".code-panel pre").all()) {
            const layout = await code.evaluate((pre) => ({
              fontSize: parseFloat(getComputedStyle(pre).fontSize),
              overflow: getComputedStyle(pre).overflowX,
              width: pre.getBoundingClientRect().width,
            }))
            assert.ok(layout.fontSize >= 13, `${route}: legible code size`)
            assert.ok(layout.width <= width, `${route}: bounded code panel`)
            assert.equal(
              layout.overflow,
              "auto",
              `${route}: scrollable long commands`
            )
          }
          assert.equal(
            await page.evaluate(
              () => document.documentElement.scrollWidth > innerWidth
            ),
            false,
            `${route}: horizontal page overflow`
          )
          for (const destination of [
            "model/immutable-by-default",
            "types/annotations-and-inference",
            "syntax/function-application",
          ]) {
            assert.equal(
              await page
                .locator(
                  `.article-content a[href="${prefix}/docs/language/${destination}/"]`
                )
                .count(),
              1
            )
          }
          if (screenshots && !javaScriptEnabled)
            await page.screenshot({
              path: join(
                screenshots,
                `first-run-${prefix ? "ja" : "en"}-${width}.png`
              ),
              fullPage: true,
            })
          await page.goto(`${origin}${prefix}/docs/`)
          await page.locator(`.article-content a[href="${route}"]`).click()
          assert.equal(new URL(page.url()).pathname, route)
        }
        assert.deepEqual(errors, [])
      } finally {
        await context.close()
      }
    }
  }
}
