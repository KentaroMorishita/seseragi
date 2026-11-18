import { expect, type Locator, type Page, test } from "@playwright/test"

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    if (window === window.top) localStorage.clear()
  })
})

test("resizes and restores every desktop Tour pane", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 })
  await page.goto("/tour/?lesson=web-link-image")

  const lesson = page.locator(".tour-lesson")
  const lab = page.locator("#tour-lab")
  const codePane = page.locator("#tour-code-pane")
  const output = page.locator("#tour-output-section")
  const lessonResizer = page.locator("#tour-lesson-resizer")
  const outputResizer = page.locator("#tour-output-resizer")

  await expect(lessonResizer).toBeVisible()
  await expect(outputResizer).toBeVisible()
  // 目次はコースマップとして重ねて開くので、目次の列も、その幅の区切りもない
  await expect(page.locator("#tour-navigation")).toBeHidden()
  await expect(page.locator("#tour-navigation-resizer")).toHaveCount(0)

  const initialLessonWidth = await width(lesson)
  await dragBy(page, lessonResizer, 70, 0)
  expect(await width(lesson)).toBeGreaterThan(initialLessonWidth + 40)

  // 流れの線の真ん中には Run がある。線の端をつかんで高さを変える
  const initialOutputHeight = await height(output)
  await dragBy(page, outputResizer, 0, -70, 24)
  expect(await height(output)).toBeGreaterThan(initialOutputHeight + 40)

  await dragTo(page, lessonResizer, 4000, 0)
  expect(await width(lab)).toBeGreaterThanOrEqual(419)

  await lessonResizer.press("Enter")
  const storedColumns = await page
    .locator(".tour-workspace")
    .evaluate((node) =>
      (node as HTMLElement).style.getPropertyValue("--tour-lesson-width")
    )
  await page.locator("#tour-next-button").click()
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "OnClickでtyped Actionを送る"
  )
  expect(
    await page
      .locator(".tour-workspace")
      .evaluate((node) =>
        (node as HTMLElement).style.getPropertyValue("--tour-lesson-width")
      )
  ).toEqual(storedColumns)

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

  await expect(page.locator("#tour-lesson-resizer")).toBeHidden()
  await expect(page.locator("#tour-output-resizer")).toBeHidden()
  await expect(page.locator("#tour-menu-button")).toBeVisible()
  // 狭い画面では、レッスンとコードを下の Lesson / Code で切り替える
  await expect(page.locator("#tour-output-section")).toBeHidden()
  await page.locator("#tour-code-focus-button").click()
  await expect(page.locator("#tour-output-section")).toBeVisible()
  await expect(page.locator("#tour-output-pane-toggle")).toBeHidden()
  await expect(page.locator("#tour-run-button")).toBeVisible()
  await page.locator("#tour-lesson-focus-button").click()
  await expect(page.locator("#tour-lesson-title")).toBeVisible()

  await page.locator("#tour-menu-button").click()
  await expect(page.locator("#tour-navigation")).toBeVisible()
  await page.locator("#tour-menu-close-button").click()
  await expect(page.locator("#tour-navigation")).toBeHidden()
})

async function dragBy(
  page: Page,
  separator: Locator,
  deltaX: number,
  deltaY: number,
  startX?: number
): Promise<void> {
  const box = await separator.boundingBox()
  if (box === null) throw new Error("separator is not visible")
  const x = box.x + (startX ?? box.width / 2)
  await page.mouse.move(x, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(x + deltaX, box.y + box.height / 2 + deltaY, {
    steps: 4,
  })
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
