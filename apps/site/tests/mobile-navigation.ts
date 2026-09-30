import assert from "node:assert/strict"
import { join } from "node:path"
import type { Browser } from "../../playground/node_modules/@playwright/test"

export async function verifyMobileNavigation(
  browser: Browser,
  origin: string,
  screenshots?: string
) {
  for (const width of [320, 390, 760]) {
    const context = await browser.newContext({
      viewport: { width, height: 844 },
      isMobile: true,
      hasTouch: true,
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
        await page.goto(`${origin}${prefix}/docs/language/`)
        const menu = page.locator(".mobile-navigation-trigger")
        const drawer = page.locator(".mobile-navigation-drawer")
        const close = drawer.getByRole("button", {
          name: locale === "ja" ? "目次を閉じる" : "Close navigation",
        })
        await page.waitForSelector('[data-enhanced="true"]')
        assert.equal(
          await page.locator(".navigation-close-label").isVisible(),
          false
        )
        assert.equal(
          await menu.locator(".navigation-trigger-label").innerText(),
          locale === "ja" ? "リファレンス" : "Reference"
        )
        const articleTop = await page
          .locator("main")
          .evaluate((main) => main.getBoundingClientRect().top)
        await menu.tap()
        assert.equal(
          await drawer.evaluate((element) => element.matches(":modal")),
          true
        )
        assert.equal(await menu.getAttribute("aria-expanded"), "true")
        const geometry = await drawer.evaluate((element) => {
          const box = element.getBoundingClientRect()
          const content = element.querySelector(".mobile-navigation-content")
          return {
            left: box.left,
            width: box.width,
            height: box.height,
            contentOverflow: content && getComputedStyle(content).overflowY,
            bodyPosition: getComputedStyle(document.body).position,
            overflow: document.documentElement.scrollWidth - innerWidth,
          }
        })
        assert.equal(geometry.left, 0)
        assert.ok(geometry.width <= width * 0.88 + 1)
        assert.ok(Math.abs(geometry.height - 844) < 1)
        assert.equal(geometry.contentOverflow, "auto")
        assert.equal(geometry.bodyPosition, "fixed")
        assert.equal(geometry.overflow, 0)
        assert.equal(
          await page
            .locator("main")
            .evaluate((main) => main.getBoundingClientRect().top),
          articleTop,
          "Opening the menu must not push the article down"
        )
        assert.equal(
          await close.evaluate((button) => button === document.activeElement),
          true
        )
        assert.equal(
          await drawer.getAttribute("data-navigation-input"),
          "pointer"
        )
        assert.equal(
          await close.evaluate(
            (button) => getComputedStyle(button).outlineStyle
          ),
          "none",
          "Touch autofocus must not show Safari's default close-button outline"
        )
        const closeBox = await close.boundingBox()
        assert.ok(closeBox && closeBox.width >= 44 && closeBox.height >= 44)
        if (screenshots)
          await page.screenshot({
            caret: "initial",
            path: join(screenshots, `navigation-touch-${locale}-${width}.png`),
          })
        await close.tap()
        await page.waitForFunction(
          () => !document.querySelector("dialog[open]")
        )
        await menu.press("Enter")
        assert.equal(
          await drawer.getAttribute("data-navigation-input"),
          "keyboard"
        )
        assert.equal(
          await close.evaluate(
            (button) => getComputedStyle(button).outlineStyle
          ),
          "solid",
          "Keyboard users must retain a visible initial focus indicator"
        )
        // Tab may visit browser chrome (activeElement becomes body), but never
        // focuses links in the background article/header while modal.
        for (let index = 0; index < 18; index += 1) {
          await page.keyboard.press("Tab")
          assert.equal(
            await drawer.evaluate(
              (element) =>
                element.contains(document.activeElement) ||
                document.activeElement === document.body
            ),
            true
          )
        }
        if (screenshots) {
          await page.screenshot({
            caret: "initial",
            path: join(screenshots, `navigation-open-${locale}-${width}.png`),
          })
        }
        await page.keyboard.press("Escape")
        await page.waitForFunction(
          () => !document.querySelector("dialog[open]")
        )
        assert.equal(await menu.getAttribute("aria-expanded"), "false")
        assert.equal(
          await menu.evaluate((element) => element === document.activeElement),
          true
        )
        assert.equal(
          await page.evaluate(() => getComputedStyle(document.body).position),
          "static"
        )

        await menu.click()
        await page.mouse.click(width - 2, 400)
        await page.waitForFunction(
          () => !document.querySelector("dialog[open]")
        )
        await menu.click()
        await close.click()
        await page.waitForFunction(
          () => !document.querySelector("dialog[open]")
        )

        // A rapid close/reopen must survive the earlier queued close event.
        await page.evaluate(() => {
          const trigger = document.querySelector(
            ".mobile-navigation-trigger"
          ) as HTMLElement
          const close = document.querySelector(
            ".mobile-navigation-close"
          ) as HTMLElement
          trigger.click()
          close.click()
          trigger.click()
        })
        await page.waitForTimeout(50)
        assert.equal(
          await drawer.evaluate((element) => element.matches(":modal")),
          true
        )
        assert.equal(
          await page.evaluate(() =>
            document.documentElement.classList.contains("navigation-is-open")
          ),
          true
        )
        await close.click()

        // Reopen from a scrolled position via keyboard; restore it exactly.
        await page.evaluate(() =>
          window.scrollTo({ top: 300, behavior: "instant" })
        )
        const scroll = await page.evaluate(() => window.scrollY)
        await menu.evaluate((element) =>
          (element as HTMLElement).focus({ preventScroll: true })
        )
        await page.keyboard.press("Enter")
        await page.waitForSelector("dialog[open]")
        const paneScroll = await drawer
          .locator(".mobile-navigation-content")
          .evaluate((element) => {
            const previous = element.scrollTop
            element.scrollTop += 200
            return { previous, current: element.scrollTop }
          })
        assert.ok(paneScroll.current > paneScroll.previous)
        await page.keyboard.press("Escape")
        await page.waitForFunction(
          () =>
            !document.documentElement.classList.contains("navigation-is-open")
        )
        assert.equal(await page.evaluate(() => window.scrollY), scroll)
        await page.evaluate(() =>
          window.scrollTo({ top: 0, behavior: "instant" })
        )

        await menu.click()
        await page.setViewportSize({ width: 1280, height: 844 })
        await page.waitForFunction(
          () => !document.querySelector("dialog[open]")
        )
        assert.equal(
          await page.evaluate(() =>
            document.documentElement.classList.contains("navigation-is-open")
          ),
          false
        )
        assert.equal(await page.locator(".docs-sidebar").isVisible(), true)
        assert.equal(await menu.isVisible(), false)
        await page.setViewportSize({ width, height: 844 })

        // Actual hierarchical navigation, with locale preserved.
        await menu.click()
        await drawer.locator(".sidebar-group > summary").nth(2).click()
        await drawer.locator('a[href$="/syntax/function-application/"]').click()
        await page.waitForURL(
          `${origin}${prefix}/docs/language/syntax/function-application/`
        )
        assert.equal(await page.locator("dialog[open]").count(), 0)
        assert.equal(await page.locator("html").getAttribute("lang"), locale)

        await page.goto(`${origin}${prefix}/docs/library/array/function/get/`)
        await page.waitForSelector('[data-enhanced="true"]')
        await menu.click()
        assert.equal(await drawer.locator(".sidebar-link.current").count(), 1)
        assert.ok(
          (
            await drawer.locator(".sidebar-link.current").textContent()
          )?.includes("get")
        )
        await close.click()
      }
      assert.deepEqual(errors, [])
    } finally {
      await context.close()
    }
  }

  const fallback = await browser.newContext({
    viewport: { width: 390, height: 844 },
    javaScriptEnabled: false,
  })
  try {
    const page = await fallback.newPage()
    await page.goto(`${origin}/ja/docs/language/`)
    const trigger = page.locator(".mobile-navigation-trigger")
    const mainTop = await page
      .locator("main")
      .evaluate((main) => main.getBoundingClientRect().top)
    await trigger.click()
    assert.equal(
      await page.locator(".mobile-navigation-drawer").isVisible(),
      true
    )
    assert.equal(
      await page
        .locator("main")
        .evaluate((main) => main.getBoundingClientRect().top),
      mainTop
    )
    if (screenshots)
      await page.screenshot({
        caret: "initial",
        path: join(screenshots, "navigation-no-js-ja-390.png"),
      })
    await trigger.click()
    assert.equal(
      await page.locator(".mobile-navigation-drawer").isVisible(),
      false
    )
  } finally {
    await fallback.close()
  }
  console.log(
    "Verified bilingual off-canvas navigation, focus, dismissal, scroll restoration, responsive resize and no-JavaScript fallback"
  )
}
