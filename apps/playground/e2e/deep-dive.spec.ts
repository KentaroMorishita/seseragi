import { expect, test } from "@playwright/test"

// Deep Dive（設計の背景の記事）の v2 の画面。

test("opens an article from the list and keeps it in the URL", async ({
  page,
}) => {
  await page.goto("/deep-dive/")
  const title = page.locator("#deep-dive-title")
  await expect(title).not.toBeEmpty()
  const first = await title.textContent()
  const target = page
    .locator('#deep-dive-navigation-content button[aria-current="false"]')
    .first()
  const targetTitle = await target.textContent()
  await target.click()
  await expect(title).toHaveText(targetTitle ?? "")
  expect(targetTitle).not.toBe(first)
  await page.goBack()
  await expect(title).toHaveText(first ?? "")
})

test("opens the article list over the page on a phone", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto("/deep-dive/")
  const list = page.locator("#deep-dive-navigation")
  await expect(list).toBeHidden()
  await page.locator("#deep-dive-menu-button").click()
  await expect(list).toBeVisible()
  await page.keyboard.press("Escape")
  await expect(list).toBeHidden()
  await expect(page.locator("#deep-dive-menu-button")).toBeFocused()
})

test("shares the Search with the Playground and Tour", async ({ page }) => {
  await page.goto("/deep-dive/")
  await page.keyboard.press("ControlOrMeta+K")
  await expect(page.locator("#search-dialog")).toBeVisible()
  await page.locator('[data-filter="lessons"]').click()
  await expect(page.locator("[data-lesson-id]").first()).toBeVisible()
})
