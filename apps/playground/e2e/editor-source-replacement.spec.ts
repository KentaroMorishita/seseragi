import { expect, type Page, test } from "@playwright/test"

const unformatted = 'pub effect fn main=println "1"\n'
const formatted = 'pub effect fn main = println "1"\n'

async function sourceText(page: Page): Promise<string> {
  return (await page.locator(".cm-content .cm-line").allTextContents()).join(
    "\n"
  )
}

async function selectMain(page: Page): Promise<void> {
  const editor = page.locator(".cm-content")
  await editor.focus()
  await editor.press("ControlOrMeta+Home")
  for (let index = 0; index < 14; index++) await editor.press("ArrowRight")
  for (let index = 0; index < 4; index++) await editor.press("Shift+ArrowRight")
  await expect
    .poll(() => page.evaluate(() => getSelection()?.toString()))
    .toBe("main")
}

for (const surface of ["Playground", "Tour"] as const) {
  test(`${surface} formats selected text without losing its editor`, async ({
    page,
  }) => {
    const errors: string[] = []
    page.on("pageerror", (error) => errors.push(error.message))
    const tour = surface === "Tour"
    await page.goto(tour ? "/tour/?lesson=comments-and-tools" : "/")
    await page.locator(".cm-content").fill(unformatted)
    await selectMain(page)
    await page
      .locator(tour ? "#tour-format-button" : "#format-source-button")
      .click()
    await expect.poll(() => sourceText(page)).toBe(formatted)

    await page.locator(".cm-content").press("ControlOrMeta+z")
    await expect.poll(() => sourceText(page)).toBe(unformatted)
    await expect
      .poll(() => page.evaluate(() => getSelection()?.toString()))
      .toBe("main")
    await page.locator(".cm-content").press("ControlOrMeta+Shift+z")
    await expect.poll(() => sourceText(page)).toBe(formatted)
    await page.locator(tour ? "#tour-run-button" : "#run-button").click()
    await expect(
      page.locator(tour ? "#tour-status-text" : "#status-text")
    ).toHaveText("Completed")
    await expect(page.locator(tour ? "#tour-output" : "#output")).toHaveText(
      "1"
    )
    expect(errors).toEqual([])
  })
}

test("Tour changes lesson after an interior text selection", async ({
  page,
}) => {
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  await page.goto("/tour/?lesson=comments-and-tools")
  await page.locator(".cm-content").fill(unformatted)
  await selectMain(page)
  await page.locator("#tour-next-button").click()
  await expect(page).not.toHaveURL(/lesson=comments-and-tools$/)
  await expect(page.locator("#tour-status-text")).toHaveText("Analysis ready")
  expect(await sourceText(page)).not.toBe(unformatted)
  await page.locator("#tour-run-button").click()
  await expect(page.locator("#tour-status-text")).toHaveText("Completed")
  expect(errors).toEqual([])
})

test("diagnostic navigation can be followed by Format and another edit", async ({
  page,
}) => {
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  await page.goto("/")
  await page.locator(".cm-content").fill("pub effect fn main=missing\n")
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toContainText("diagnostic")
  await page.locator("#output button").first().click()
  await expect
    .poll(() => page.evaluate(() => getSelection()?.toString()))
    .toBe("missing")
  await page.locator("#format-source-button").click()
  await expect
    .poll(() => sourceText(page))
    .toBe("pub effect fn main = missing\n")
  await page.locator(".cm-content").fill(formatted)
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed")
  await expect(page.locator("#output")).toHaveText("1")
  expect(errors).toEqual([])
})
