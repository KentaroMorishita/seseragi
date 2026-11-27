import { expect, test } from "@playwright/test"

test("opens the edited Tour lesson code in Playground free-writing mode", async ({
  page,
}) => {
  await page.goto("/tour/?lesson=01-hello-world")
  await expect(page.locator("#tour-status-text")).toHaveText("Analysis ready", {
    timeout: 60_000,
  })

  const tourEditor = page.locator("#tour-editor .cm-content")
  await tourEditor.click()
  await page.keyboard.press("End")
  await page.keyboard.type(" // from-tour")
  await expect(tourEditor).toContainText("// from-tour")

  await page.locator("#tour-open-playground-button").click()
  await expect(page).toHaveURL(/\/\?(source|workspace)=/u)
  await expect(page.locator("#status-text")).toHaveText("Analysis ready", {
    timeout: 60_000,
  })
  await expect(page.locator("#editor .cm-content")).toContainText(
    "// from-tour"
  )
  await expect(page.locator("#auto-run-note")).toBeHidden()
  await expect(page.locator("#try-next")).toBeHidden()
})

test("shows chapter examples on the final lesson of a chapter and opens them in Playground", async ({
  page,
}) => {
  // 章の途中のレッスンでは表示しない
  await page.goto("/tour/?lesson=01-hello-world")
  await expect(page.locator("#tour-chapter-examples")).toBeHidden()

  // 章の最後のレッスンでは Next カードの下に関連例を表示する
  await page.goto("/tour/?lesson=if-expression")
  const section = page.locator("#tour-chapter-examples")
  await expect(section).toBeVisible()
  await expect(section.locator("h2")).toHaveText("この章の考え方を使った例")
  const fizzbuzzLink = section.locator('[data-chapter-example="fizzbuzz"]')
  await expect(fizzbuzzLink).toBeVisible()

  await fizzbuzzLink.click()
  await expect(page).toHaveURL(/\/$/u)
  await expect(page.locator("#current-sample-title")).toContainText("FizzBuzz")
})
