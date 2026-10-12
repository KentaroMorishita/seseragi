import { expect, test } from "@playwright/test"

const source = [
  "type Parcel =",
  "  | Packing",
  "  | Sent String",
  "",
  "fn label parcel: Parcel -> String =",
  "  match parcel {",
  '    Packing -> "Preparing"',
  `    Sent city -> \`Sent to \${city}\``,
  "  }",
  "",
  "pub effect fn main =",
  '  Sent "Kyoto"',
  "  |> label",
  "  |> println",
].join("\n")

test("editing an ADT keeps closed Reference cards off the input path", async ({
  page,
}) => {
  await page.goto("/")
  const status = page.locator("#status-text")
  const cards = page.locator(".reference-card")
  const editor = page.getByRole("textbox", { name: "Seseragi source editor" })
  await expect(status).toHaveText("Analysis ready")
  await expect(cards).toHaveCount(0)

  for (const variant of ["", "\n  |", "\n  | Ar", "\n  | Arrived", ""]) {
    await editor.click()
    await page.keyboard.press("ControlOrMeta+A")
    await page.keyboard.insertText(
      source.replace("| Sent String", `| Sent String${variant}`)
    )
    await expect(status).toHaveText(
      variant === "" ? "Analysis ready" : "1 diagnostic(s)"
    )
    await expect(cards).toHaveCount(0)
  }

  await page.locator("#mobile-tools-button").click()
  await page.locator("#reference-browser-button").click()
  await expect(page.locator("#reference-browser-dialog")).toBeVisible()
  await expect(cards).not.toHaveCount(0)
  const allCount = await cards.count()
  await page.locator("#reference-search").fill("println")
  await expect(cards).not.toHaveCount(0)
  expect(await cards.count()).toBeLessThan(allCount)
  await expect(cards.first()).toContainText("println")
  await page.locator("#reference-browser-close").click()
  await expect(cards).toHaveCount(0)

  await editor.click()
  await page.keyboard.press("End")
  await page.keyboard.insertText("\n")
  await expect(status).toHaveText("Analysis ready")
  await expect(cards).toHaveCount(0)
  await page.locator("#mobile-tools-button").click()
  await page.locator("#reference-browser-button").click()
  await expect(page.locator("#reference-search")).toHaveValue("println")
  await expect(cards).not.toHaveCount(0)
  await expect(cards.first()).toContainText("println")
})
