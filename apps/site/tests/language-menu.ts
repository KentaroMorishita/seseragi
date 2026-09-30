import assert from "node:assert/strict"
import { join } from "node:path"
import type { Browser } from "../../playground/node_modules/@playwright/test"

export async function verifyLanguageMenu(
  browser: Browser,
  origin: string,
  screenshots?: string
) {
  // macOS WebKit follows Safari's default link-skipping Tab preference.
  const tabKey = browser.browserType().name() === "webkit" ? "Alt+Tab" : "Tab"
  for (const width of [320, 390, 760, 1280, 1710]) {
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
      page.on("response", (response) => {
        if (response.status() >= 400) errors.push(response.url())
      })
      for (const route of [
        "/",
        "/docs/",
        "/docs/language/syntax/function-application/",
        "/docs/library/array/function/get/",
      ]) {
        for (const locale of ["en", "ja"]) {
          const path = locale === "ja" ? `/ja${route}` : route
          await page.goto(`${origin}${path}`)
          const menu = page.locator(".language-menu")
          const trigger = page.locator(".language-trigger")
          const options = page.locator(".language-options")
          const english = menu.getByRole("link", { name: "English" })
          const japanese = menu.getByRole("link", { name: "日本語" })
          await page.waitForFunction(() =>
            document
              .querySelector(".language-trigger")
              ?.hasAttribute("aria-expanded")
          )
          assert.equal(await trigger.innerText(), "")
          assert.equal(await options.isVisible(), false)
          assert.equal(
            await trigger.getAttribute("aria-label"),
            locale === "ja" ? "表示言語を選ぶ" : "Change language"
          )
          const articleTop = await page
            .locator("main")
            .evaluate((main) => main.getBoundingClientRect().top)
          await trigger.focus()
          await page.keyboard.press("Enter")
          assert.equal(await options.isVisible(), true)
          assert.equal(await english.getAttribute("href"), route)
          assert.equal(await japanese.getAttribute("href"), `/ja${route}`)
          assert.equal(
            await menu.locator('[aria-current="true"]').innerText(),
            locale === "ja" ? "日本語\n✓" : "English\n✓"
          )
          const box = await options.boundingBox()
          assert.ok(box && box.x >= 0 && box.x + box.width <= width)
          assert.equal(
            await page
              .locator("main")
              .evaluate((main) => main.getBoundingClientRect().top),
            articleTop
          )
          assert.equal(
            await page.evaluate(
              () => document.documentElement.scrollWidth - innerWidth
            ),
            0
          )
          await page.keyboard.press(tabKey)
          assert.equal(
            await english.evaluate((link) => link === document.activeElement),
            true
          )
          await page.keyboard.press("Escape")
          assert.equal(await options.isVisible(), false)
          assert.equal(
            await trigger.evaluate(
              (element) => element === document.activeElement
            ),
            true
          )
          await trigger.click()
          await page.locator("main").click({ position: { x: 5, y: 5 } })
          assert.equal(await options.isVisible(), false)
          await trigger.click()
          await trigger.focus()
          await page.keyboard.press(tabKey)
          await page.keyboard.press(tabKey)
          assert.equal(
            await japanese.evaluate((link) => link === document.activeElement),
            true
          )
          // Safari may send the next Tab into browser chrome instead of a DOM
          // element. Verify focus departure against an actual outside target.
          await page.locator(".site-brand").focus()
          await options.waitFor({ state: "hidden" })
          assert.equal(await options.isVisible(), false)
          if (screenshots && route === "/docs/") {
            await page.screenshot({
              caret: "initial",
              path: join(screenshots, `language-closed-${locale}-${width}.png`),
            })
            await trigger.click()
            await page.screenshot({
              caret: "initial",
              path: join(screenshots, `language-open-${locale}-${width}.png`),
            })
          } else {
            await trigger.click()
          }
          const destination = locale === "ja" ? route : `/ja${route}`
          await (locale === "ja" ? english : japanese).click()
          await page.waitForURL(`${origin}${destination}`)
          assert.equal(
            await page.locator("html").getAttribute("lang"),
            locale === "ja" ? "en" : "ja"
          )
          assert.equal(
            await page.locator(".language-options").isVisible(),
            false
          )
        }
      }
      assert.deepEqual(errors, [])
    } finally {
      await context.close()
    }
  }
  const fallback = await browser.newContext({
    javaScriptEnabled: false,
    viewport: { width: 390, height: 844 },
  })
  try {
    const page = await fallback.newPage()
    await page.goto(`${origin}/ja/docs/language/`)
    await page.locator(".language-trigger").click()
    await page.getByRole("link", { name: "English", exact: true }).click()
    await page.waitForURL(`${origin}/docs/language/`)
  } finally {
    await fallback.close()
  }
}
