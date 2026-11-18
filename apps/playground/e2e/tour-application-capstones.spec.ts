import { expect, test } from "@playwright/test"

test("runs the console capstone and reaches the Web Preview sequence", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 })
  await page.goto("/tour/?lesson=applications-console-data")

  await expect(page.locator("#tour-lesson-title")).toContainText(
    "総合演習 1: 検証付き集計コンソール"
  )
  await page.locator("#tour-run-button").click()
  await expect(page.locator("#tour-output")).toContainText("Book: 2400")
  await expect(page.locator("#tour-output")).toContainText(
    "error: Cancelled: invalid amount"
  )

  await page.goto("/tour/?lesson=14-integrated-app")
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "総合演習 2: 型付き Action の対話型 Web UI"
  )
  await page.locator("#tour-run-button").click()
  await expect(page.locator("#tour-html-preview")).toBeVisible()
  const preview = page.locator("#tour-html-preview").contentFrame()
  const counterButton = preview.getByRole("button", {
    name: "Count: 0 — click to add 1",
  })
  await expect(counterButton).toBeVisible()
  await counterButton.click()
  await expect(
    preview.getByRole("button", { name: "Count: 1 — click to add 1" })
  ).toBeVisible()
})
