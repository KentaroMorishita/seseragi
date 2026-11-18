import { expect, test } from "@playwright/test"
import { startQuietly } from "./quiet-start-support"

// 初めて開いたときの案内（最初の例の自動実行など）は first-example.spec.ts で確かめる
test.beforeEach(async ({ page }) => {
  await startQuietly(page)
})

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
      variant === "" ? "Analysis ready" : "1 problem"
    )
    await expect(cards).toHaveCount(0)
  }

  // API は Search（⌘K）の範囲の一つ。開いているあいだだけ説明を描く
  const results = page.locator('#search-results [data-scope="api"]')
  await page.locator("#search-button").click()
  await page.locator('[data-filter="api"]').click()
  await expect(results).not.toHaveCount(0)
  const allCount = await results.count()
  await page.locator("#search-input").fill("println")
  await expect(results).not.toHaveCount(0)
  expect(await results.count()).toBeLessThan(allCount)
  await expect(results.first()).toContainText("println")
  await expect(cards).toHaveCount(1)
  await expect(cards.first()).toContainText("println")
  await page.keyboard.press("Escape")
  await expect(cards).toHaveCount(0)

  await editor.click()
  await page.keyboard.press("End")
  await page.keyboard.insertText("\n")
  await expect(status).toHaveText("Analysis ready")
  await expect(cards).toHaveCount(0)
  // 開き直すと、前に打った言葉が残っている
  await page.locator("#search-button").click()
  await expect(page.locator("#search-input")).toHaveValue("println")
  await page.locator('[data-filter="api"]').click()
  await expect(cards.first()).toContainText("println")
})
