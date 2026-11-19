import { expect, test } from "@playwright/test"

test.use({ viewport: { width: 393, height: 852 } })

test("a match diagnostic stays opaque beside its type hover", async ({
  page,
}) => {
  await page.goto("/")
  const editor = page.getByRole("textbox", { name: "Seseragi source editor" })
  await expect(page.locator("#status-text")).toHaveText("Analysis ready")
  await editor.click()
  await page.keyboard.press("ControlOrMeta+A")
  await page.keyboard.insertText(
    [
      "type Parcel =",
      "  | Packing",
      "  | Sent String",
      "  | Arrived",
      "",
      "fn label parcel: Parcel -> String =",
      "  match parcel {",
      '    Packing -> "Preparing"',
      "    Sent city -> city",
      "  }",
      "",
      'pub effect fn main = Sent "Kyoto" |> label |> println',
    ].join("\n")
  )
  await expect(page.locator("#status-text")).toHaveText("1 problem")
  await page
    .locator(".cm-line")
    .filter({ hasText: "match parcel {" })
    .locator(".cm-lintRange-error")
    .first()
    .hover()
  const diagnostic = page.locator(".cm-tooltip-lint")
  await expect(diagnostic).toBeVisible()
  await expect(diagnostic).toContainText("SES-T0301")
  await expect(page.locator(".analysis-hover")).toBeVisible()
  // The tooltip paints an opaque surface, so the hover beside it never shows through.
  const paint = await diagnostic.evaluate((element) => {
    const style = getComputedStyle(element)
    return { background: style.backgroundColor, color: style.color }
  })
  expect(paint.background).toMatch(/^rgb\(\d+, \d+, \d+\)$/)
  expect(paint.color).not.toBe(paint.background)
  const bounds = await diagnostic.boundingBox()
  expect(bounds?.x).toBeGreaterThanOrEqual(0)
  expect((bounds?.x ?? 0) + (bounds?.width ?? 0)).toBeLessThanOrEqual(393)
  const pageHeight = await page.evaluate(() => ({
    body: document.body.scrollHeight,
    viewport: window.innerHeight,
    editor: document.querySelector(".cm-editor")?.getBoundingClientRect()
      .height,
  }))
  expect(pageHeight.body).toBeLessThanOrEqual(pageHeight.viewport)
  expect(pageHeight.editor).toBeGreaterThan(0)
  await page.screenshot({ path: "/tmp/seseragi-diagnostic-tooltip-fixed.png" })
})
