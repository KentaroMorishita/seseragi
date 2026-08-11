import { expect, test } from "@playwright/test"

const mockImage = `<svg xmlns="http://www.w3.org/2000/svg" width="960" height="480"><rect width="960" height="480" fill="#34d399"/></svg>`

test("runs Tour image, external link, form and Signal interactions", async ({
  page,
}) => {
  await page.route("https://images.unsplash.com/**", (route) =>
    route.fulfill({ contentType: "image/svg+xml", body: mockImage })
  )
  await page.setViewportSize({ width: 1440, height: 900 })
  await page.goto("/tour/?lesson=web-link-image")

  await expect(page.locator("#tour-lesson-title")).toContainText(
    "External linkとimageを表示する"
  )
  await page.locator("#tour-run-button").click()
  const staticPreview = page.locator("#tour-html-preview").contentFrame()
  const image = staticPreview.getByRole("img", {
    name: "明るい共同作業スペース",
  })
  await expect(image).toBeVisible()
  await expect(image).toHaveAttribute("width", "960")
  await expect(image).toHaveAttribute("height", "480")
  const link = staticPreview.getByRole("link", { name: "Repository" })
  await expect(link).toHaveAttribute(
    "href",
    "https://github.com/KentaroMorishita/seseragi"
  )
  await expect(link).toHaveAttribute("target", "_blank")
  await expect(link).toHaveAttribute("rel", "noopener")
  await expect(page.locator("#tour-progress-label")).toHaveText("1 completed")

  await page.goto("/tour/?lesson=web-feature-state")
  await page.locator("#tour-run-button").click()
  const interactivePreview = page.locator("#tour-html-preview").contentFrame()
  const status = interactivePreview.getByRole("status")
  await expect(status).toHaveText("Ready")
  await interactivePreview.getByLabel("Plan name").fill("Release 0.4.1")
  await expect(status).toHaveText("Editing")
  await interactivePreview.getByLabel("Pin").check()
  await interactivePreview.getByRole("button", { name: "Save" }).click()
  await expect(status).toHaveText("Saved: Release 0.4.1")
})

test("preserves direct routes across next, back, forward and reload", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto("/tour/?lesson=web-feature-state")
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Component分割とstate ownershipを保つ"
  )

  await page.locator("#tour-next-button").click()
  await expect(page).toHaveURL(/lesson=applications-console-data/u)
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Step 1: input dataを定義する"
  )

  await page.reload()
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Step 1: input dataを定義する"
  )
  await page.goBack()
  await expect(page).toHaveURL(/lesson=web-feature-state/u)
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Component分割とstate ownershipを保つ"
  )
  await page.goForward()
  await expect(page).toHaveURL(/lesson=applications-console-data/u)
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Step 1: input dataを定義する"
  )
})
