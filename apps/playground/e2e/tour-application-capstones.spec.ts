import { expect, test } from "@playwright/test"

test("runs the console capstone and reaches the Web Preview sequence", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 })
  await page.goto("/tour/?lesson=applications-console-data")

  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Step 1: input dataを定義する"
  )
  await page.locator("#tour-run-button").click()
  await expect(page.locator("#tour-output")).toContainText("Book: 2400")
  await expect(page.locator("#tour-output")).toContainText("Cancelled: 0")

  await page.goto("/tour/?lesson=applications-web-static")
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Web Step 1: static viewを作る"
  )
  await page.locator("#tour-run-button").click()
  await expect(page.locator("#tour-html-preview")).toBeVisible()
  await expect(
    page.locator("#tour-html-preview").contentFrame().getByRole("heading")
  ).toHaveText("Plan board")

  await page.goto("/tour/?lesson=applications-web-feature-ownership")
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Web Step 7: feature stateを階層合成する"
  )
  await page.locator("#tour-run-button").click()
  const preview = page.locator("#tour-html-preview").contentFrame()
  await expect(preview.getByRole("heading", { name: "Personal" })).toBeVisible()
  await expect(preview.getByRole("heading", { name: "Release" })).toBeVisible()
})
