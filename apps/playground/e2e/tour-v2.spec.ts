import { expect, test } from "@playwright/test"

// Tour の v2 の画面（#1359）: Try it の手順、Walkthrough のピン、Search。

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    if (window === window.top) localStorage.clear()
  })
})

test("checks off Try it steps as the output matches", async ({ page }) => {
  await page.goto("/tour/?lesson=07-adts-and-patterns")
  const steps = page.locator(".tour-try-step")
  await expect(steps).toHaveCount(2)
  await expect(steps.nth(0)).toHaveAttribute("data-state", "current")
  await expect(page.locator("#tour-run-check")).toBeHidden()

  await page.locator("#tour-run-button").click()
  await expect(page.locator("#tour-status-text")).toHaveText("Completed")
  await expect(steps.nth(0)).toHaveAttribute("data-state", "done")
  await expect(steps.nth(1)).toHaveAttribute("data-state", "current")
  await expect(page.locator("#tour-run-check")).toBeVisible()

  await page.locator("#tour-exercise-button").click()
  await page.locator("#tour-run-button").click()
  await expect(page.locator("#tour-status-text")).toHaveText("Completed")
  await expect(steps.nth(1)).toHaveAttribute("data-state", "done")
})

test("pins each Walkthrough number to its line in the editor", async ({
  page,
}) => {
  await page.goto("/tour/?lesson=07-adts-and-patterns")
  const cards = page.locator(".tour-walkthrough-card")
  const pins = page.locator(".cm-line-pin")
  await expect(cards).not.toHaveCount(0)
  await expect(pins).toHaveCount(await cards.count())
  await expect(pins.first()).toHaveAttribute("data-label", "1")

  // 答えのコードは行がずれるので、ピンを出さない。Reset で戻る
  await page.locator("#tour-exercise-button").click()
  await expect(pins).toHaveCount(0)
  await page.locator("#tour-try-reset-button").click()
  await expect(pins).toHaveCount(await cards.count())
})

test("opens the course map over the lesson and closes with Escape", async ({
  page,
}) => {
  await page.goto("/tour/?lesson=07-adts-and-patterns")
  await page.locator("#tour-menu-button").click()
  const map = page.locator("#tour-navigation")
  await expect(map).toBeVisible()
  await expect(page.locator(".pg-top")).toBeVisible()
  await expect(
    map.locator('[data-lesson-id="07-adts-and-patterns"]')
  ).toHaveAttribute("aria-current", "step")
  await page.keyboard.press("Escape")
  await expect(map).toBeHidden()
  await expect(page.locator("#tour-menu-button")).toBeFocused()
})

test("opens an example in the Playground from the Tour's Search", async ({
  page,
}) => {
  await page.goto("/tour/?lesson=07-adts-and-patterns")
  await page.keyboard.press("ControlOrMeta+K")
  await expect(page.locator("#search-dialog")).toBeVisible()
  await page.locator('[data-filter="examples"]').click()
  await page.locator('[data-sample-id="fizzbuzz"]').click()
  await expect(page).toHaveURL(/\/$/u)
  await expect(page.locator("#current-sample-title")).toContainText("FizzBuzz")
})
