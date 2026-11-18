import { expect, test } from "@playwright/test"

// Search（⌘K）は、例・レッスン・API・設定を一つにまとめた入口（DESIGN.md「探す場所は一つ」）。
// キーボードだけで、探して、選んで、開けることを確かめる。

test.beforeEach(async ({ page }) => {
  await page.goto("/")
  await expect(page.locator("#status-text")).toHaveText("Analysis ready", {
    timeout: 60_000,
  })
})

test("opens with ⌘K, narrows by typing, and opens an example with Enter", async ({
  page,
}) => {
  const dialog = page.locator("#search-dialog")
  const input = page.locator("#search-input")
  await page.keyboard.press("ControlOrMeta+K")
  await expect(dialog).toBeVisible()
  await expect(input).toBeFocused()
  await expect(page.locator("#search-button")).toHaveAttribute(
    "aria-expanded",
    "true"
  )

  await input.fill("FizzBuzz")
  const option = page.locator('[data-sample-id="fizzbuzz"]')
  await expect(option).toHaveAttribute("aria-selected", "true")
  await expect(input).toHaveAttribute(
    "aria-activedescendant",
    (await option.getAttribute("id")) ?? ""
  )
  await page.keyboard.press("Enter")
  await expect(dialog).toBeHidden()
  await expect(page.locator("#current-sample-title")).toContainText("FizzBuzz")
  await expect(page.locator("#search-button")).toBeFocused()
})

test("moves through results with the arrow keys and closes with Escape", async ({
  page,
}) => {
  await page.locator("#search-button").click()
  const options = page.locator("#search-results [role=option]")
  await expect(options.first()).toHaveAttribute("aria-selected", "true")
  await page.keyboard.press("ArrowDown")
  await expect(options.nth(1)).toHaveAttribute("aria-selected", "true")
  await page.keyboard.press("ArrowUp")
  await page.keyboard.press("ArrowUp")
  await expect(options.last()).toHaveAttribute("aria-selected", "true")
  await page.keyboard.press("Escape")
  await expect(page.locator("#search-dialog")).toBeHidden()
})

test("opens on Examples from the example name and marks the open example", async ({
  page,
}) => {
  await page.locator("#sample-browser-button").click()
  await expect(page.locator('[data-filter="examples"]')).toHaveAttribute(
    "aria-pressed",
    "true"
  )
  await expect(page.locator('[data-scope="lessons"]')).toHaveCount(0)
  await page.locator('[data-sample-id="fizzbuzz"]').click()
  await expect(page.locator("#status-text")).toHaveText("Analysis ready")

  await page.locator("#sample-browser-button").click()
  await expect(
    page.locator('[data-sample-id="fizzbuzz"][aria-current="true"]')
  ).toHaveCount(1)
})

test("lists lessons and opens one in the Tour", async ({ page }) => {
  await page.locator("#search-button").click()
  await page.locator('[data-filter="lessons"]').click()
  const lesson = page.locator("[data-lesson-id]").first()
  await expect(lesson).toBeVisible()
  const id = await lesson.getAttribute("data-lesson-id")
  await lesson.click()
  await expect(page).toHaveURL(new RegExp(`/tour/\\?lesson=${id}`, "u"))
})

test("switches the theme and opens editor settings from Settings", async ({
  page,
}) => {
  await page.locator("#search-button").click()
  await page.locator('[data-filter="settings"]').click()
  await page.locator('[data-command="theme-dark"]').click()
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark")

  await page.locator("#search-button").click()
  await page.locator('[data-filter="settings"]').click()
  await expect(page.locator('[data-command="theme-dark"] .ui-tag')).toHaveText(
    "Current"
  )
  await page.locator('[data-command="editor-settings"]').click()
  await expect(page.locator("#playground-settings-dialog")).toBeVisible()
})

test("shows Search as a bottom sheet on a phone", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.locator("#search-button").click()
  const box = await page.locator("#search-dialog").boundingBox()
  expect(box).not.toBeNull()
  expect(Math.round((box?.y ?? 0) + (box?.height ?? 0))).toBe(844)
  expect(Math.round(box?.width ?? 0)).toBe(390)
  const fontSize = await page
    .locator("#search-input")
    .evaluate((element) => getComputedStyle(element).fontSize)
  expect(fontSize).toBe("16px")
})
