import { expect, type Locator, type Page, test } from "@playwright/test"

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear())
})

test("resizes and restores every desktop Tour pane", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 })
  await page.goto("/tour/?lesson=web-link-image")

  const navigation = page.locator("#tour-navigation")
  const lesson = page.locator(".tour-lesson")
  const lab = page.locator("#tour-lab")
  const codePane = page.locator("#tour-code-pane")
  const output = page.locator("#tour-output-section")
  const navigationResizer = page.locator("#tour-navigation-resizer")
  const lessonResizer = page.locator("#tour-lesson-resizer")
  const outputResizer = page.locator("#tour-output-resizer")

  await expect(navigationResizer).toBeVisible()
  await expect(lessonResizer).toBeVisible()
  await expect(outputResizer).toBeVisible()

  const initialNavigationWidth = await width(navigation)
  await dragBy(page, navigationResizer, 70, 0)
  expect(await width(navigation)).toBeGreaterThan(initialNavigationWidth + 40)

  const lessonWidthAfterNavigation = await width(lesson)
  await dragBy(page, lessonResizer, 70, 0)
  expect(await width(lesson)).toBeGreaterThan(lessonWidthAfterNavigation + 40)

  const initialOutputHeight = await height(output)
  await dragBy(page, outputResizer, 0, -70)
  expect(await height(output)).toBeGreaterThan(initialOutputHeight + 40)

  await dragTo(page, navigationResizer, -1000, 0)
  expect(await width(navigation)).toBeGreaterThanOrEqual(239)
  await dragTo(page, lessonResizer, 4000, 0)
  expect(await width(lab)).toBeGreaterThanOrEqual(459)

  await lessonResizer.press("Enter")
  const storedColumns = await page
    .locator(".tour-workspace")
    .evaluate((node) => ({
      lesson: (node as HTMLElement).style.getPropertyValue(
        "--tour-lesson-width"
      ),
      navigation: (node as HTMLElement).style.getPropertyValue(
        "--tour-navigation-width"
      ),
    }))
  await page.locator("#tour-next-button").click()
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "OnClickでtyped Actionを送る"
  )
  expect(
    await page.locator(".tour-workspace").evaluate((node) => ({
      lesson: (node as HTMLElement).style.getPropertyValue(
        "--tour-lesson-width"
      ),
      navigation: (node as HTMLElement).style.getPropertyValue(
        "--tour-navigation-width"
      ),
    }))
  ).toEqual(storedColumns)

  const navigationToggle = page.getByRole("button", {
    name: "lesson一覧を閉じる",
  })
  const lessonBeforeCollapse = await width(lesson)
  await navigationToggle.click()
  await expect(navigation).toBeHidden()
  expect(await width(lesson)).toBeGreaterThan(lessonBeforeCollapse)
  await page.getByRole("button", { name: "lesson一覧を開く" }).click()
  await expect(navigation).toBeVisible()
  expect(await width(navigation)).toBeGreaterThanOrEqual(239)

  await page.goto("/tour/?lesson=web-link-image")
  await page.locator("#tour-run-button").click()
  await expect(page.locator("#tour-html-preview")).toBeVisible()
  const previewSource = await page
    .locator("#tour-html-preview")
    .getAttribute("src")
  const sourceBeforeCollapse = await page.locator(".cm-content").textContent()
  const codeHeightBeforeCollapse = await height(codePane)
  await page.getByRole("button", { name: "Outputを閉じる" }).click()
  await expect(output).toBeHidden()
  expect(await height(codePane)).toBeGreaterThan(codeHeightBeforeCollapse)
  await page.getByRole("button", { name: "Outputを開く" }).click()
  await expect(output).toBeVisible()
  await expect(page.locator("#tour-html-preview")).toBeVisible()
  expect(await page.locator("#tour-html-preview").getAttribute("src")).toBe(
    previewSource
  )
  expect(await page.locator(".cm-content").textContent()).toBe(
    sourceBeforeCollapse
  )
  await expect(page.locator("#tour-fullscreen-button")).toBeVisible()
})

test("keeps desktop controls out of the narrow Tour layout", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto("/tour/?lesson=web-link-image")

  await expect(page.locator("#tour-navigation-resizer")).toBeHidden()
  await expect(page.locator("#tour-lesson-resizer")).toBeHidden()
  await expect(page.locator("#tour-output-resizer")).toBeHidden()
  await expect(page.locator("#tour-navigation-pane-toggle")).toBeHidden()
  await expect(page.locator("#tour-output-pane-toggle")).toBeHidden()
  await expect(page.locator("#tour-menu-button")).toBeVisible()
  await expect(page.locator("#tour-output-section")).toBeVisible()

  await page.locator("#tour-menu-button").click()
  await expect(page.locator("#tour-navigation")).toBeVisible()
  await page.locator("#tour-menu-close-button").click()
  await expect(page.locator("#tour-navigation")).toBeHidden()
})

async function dragBy(
  page: Page,
  separator: Locator,
  deltaX: number,
  deltaY: number
): Promise<void> {
  const box = await separator.boundingBox()
  if (box === null) throw new Error("separator is not visible")
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(
    box.x + box.width / 2 + deltaX,
    box.y + box.height / 2 + deltaY,
    { steps: 4 }
  )
  await page.mouse.up()
}

async function dragTo(
  page: Page,
  separator: Locator,
  clientX: number,
  clientY: number
): Promise<void> {
  const box = await separator.boundingBox()
  if (box === null) throw new Error("separator is not visible")
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(
    clientX === 0 ? box.x + box.width / 2 : clientX,
    clientY === 0 ? box.y + box.height / 2 : clientY,
    { steps: 4 }
  )
  await page.mouse.up()
}

async function width(locator: Locator): Promise<number> {
  return locator.evaluate((node) => node.getBoundingClientRect().width)
}

async function height(locator: Locator): Promise<number> {
  return locator.evaluate((node) => node.getBoundingClientRect().height)
}
