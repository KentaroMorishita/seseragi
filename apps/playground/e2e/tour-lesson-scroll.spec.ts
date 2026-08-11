import { expect, type Page, test } from "@playwright/test"

const lessonPane = ".tour-lesson"

test("resets only the desktop lesson pane for every lesson route", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 })
  await page.goto("/tour/?lesson=web-feature-state")
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Component分割とstate ownershipを保つ"
  )

  await recordElementScrollTargets(page)
  await scrollLessonToEnd(page)
  await page.locator("#tour-next-button").click()
  await expectLessonAtTop(page)
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Step 1: input dataを定義する"
  )

  await scrollLessonToEnd(page)
  await page.locator("#tour-previous-button").click()
  await expectLessonAtTop(page)

  await scrollLessonToEnd(page)
  await page.locator('[data-lesson-id="web-link-image"]').click()
  await expectLessonAtTop(page)
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "External linkとimageを表示する"
  )

  await scrollLessonToEnd(page)
  await page.goBack()
  await expectLessonAtTop(page)
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Component分割とstate ownershipを保つ"
  )

  const scrollTargets = await page.evaluate(
    () =>
      (
        window as Window & {
          __tourScrollTargets?: string[]
        }
      ).__tourScrollTargets ?? []
  )
  expect(scrollTargets).toContain("tour-lesson")
  expect(scrollTargets).not.toContain("cm-scroller")
  expect(scrollTargets).not.toContain("tour-output")
  expect(scrollTargets).not.toContain("tour-navigation")
})

test("keeps the mobile page scroll reset", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto("/tour/?lesson=web-feature-state")
  await page.evaluate(() => {
    document.body.scrollTop = document.body.scrollHeight
  })
  expect(await page.evaluate(() => document.body.scrollTop)).toBeGreaterThan(0)

  await page.locator("#tour-next-button").click()
  await expect.poll(() => page.evaluate(() => document.body.scrollTop)).toBe(0)
  await expect(page.locator("#tour-lesson-title")).toContainText(
    "Step 1: input dataを定義する"
  )
})

async function scrollLessonToEnd(page: Page): Promise<void> {
  const scrollTop = await page.locator(lessonPane).evaluate((element) => {
    element.scrollTop = element.scrollHeight
    return element.scrollTop
  })
  expect(scrollTop).toBeGreaterThan(0)
}

async function expectLessonAtTop(page: Page): Promise<void> {
  await expect
    .poll(() =>
      page.locator(lessonPane).evaluate((element) => element.scrollTop)
    )
    .toBe(0)
}

async function recordElementScrollTargets(page: Page): Promise<void> {
  await page.evaluate(() => {
    const browserWindow = window as Window & {
      __tourScrollTargets?: string[]
    }
    browserWindow.__tourScrollTargets = []
    const originalScrollTo = Element.prototype.scrollTo
    Object.defineProperty(Element.prototype, "scrollTo", {
      configurable: true,
      writable: true,
      value(this: Element, ...args: unknown[]): void {
        browserWindow.__tourScrollTargets?.push(
          this.id || [...this.classList].join(" ") || this.tagName
        )
        Reflect.apply(originalScrollTo, this, args)
      },
    })
  })
}
