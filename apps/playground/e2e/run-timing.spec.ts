import { expect, type Page, test } from "@playwright/test"
import { startQuietly } from "./quiet-start-support"

// Output の状態のチップは main の実行時間を出し、押すと区間ごとの内訳を開く。

const phaseLabels = [
  "Compile",
  "Sandbox start",
  "Worker start",
  "Worker load",
  "TypeScript init",
  "Transpile",
  "Evaluate",
  "Environment",
  "main",
  "Cleanup",
  "Messaging",
]

async function runFizzBuzz(page: Page): Promise<void> {
  await startQuietly(page)
  await page.goto("/?example=fizzbuzz")
  await expect(page.locator("#status-text")).not.toHaveText(
    /Compiling|Analy/u,
    {
      timeout: 60_000,
    }
  )
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed", {
    timeout: 60_000,
  })
}

function milliseconds(text: string): number {
  if (text.includes("<1 ms")) return 0
  return Number(/([\d.]+) ms/u.exec(text)?.[1])
}

test("shows the time of main and opens the breakdown", async ({ page }) => {
  await runFizzBuzz(page)
  const chip = page.locator("#run-status .ui-status-button")
  await expect(chip).toHaveText(/^✓ (<1|\d+) ms$/u)
  const main = milliseconds((await chip.textContent()) ?? "")

  await chip.click()
  const panel = page.getByRole("dialog", { name: "Run timing" })
  await expect(panel).toBeVisible()
  await expect(chip).toHaveAttribute("aria-expanded", "true")
  await expect(panel.locator("dt")).toHaveText(phaseLabels)

  // チップの数字は main の行と同じ。準備の区間は main よりずっと長い
  const mainRow = panel.locator('[data-phase="main"] dd')
  const mainInBreakdown = milliseconds((await mainRow.textContent()) ?? "")
  if (main === 0) expect(mainInBreakdown).toBeLessThan(1)
  else expect(Math.round(mainInBreakdown)).toBe(Math.round(main))
  const total = milliseconds(
    (await panel.locator(".pg-run-timing-total").textContent()) ?? ""
  )
  expect(total).toBeGreaterThan(main)

  // Escape で閉じ、チップに戻る
  await page.keyboard.press("Escape")
  await expect(panel).toBeHidden()
  await expect(chip).toBeFocused()
  await expect(chip).toHaveAttribute("aria-expanded", "false")

  // 外を押しても閉じる
  await chip.click()
  await expect(panel).toBeVisible()
  await page.locator(".pg-foot").click({ position: { x: 8, y: 8 } })
  await expect(panel).toBeHidden()

  // 次の実行が始まったら閉じる
  await chip.click()
  await expect(panel).toBeVisible()
  await page.locator("#run-button").click()
  await expect(panel).toBeHidden()
})

test("keeps the breakdown on screen on a phone", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await runFizzBuzz(page)
  const chip = page.locator("#run-status .ui-status-button")
  await chip.click()
  const panel = page.getByRole("dialog", { name: "Run timing" })
  await expect(panel).toBeVisible()
  const box = (await panel.boundingBox())!
  expect(box.x).toBeGreaterThanOrEqual(16)
  expect(box.x + box.width).toBeLessThanOrEqual(390 - 16)
  expect(box.y).toBeGreaterThanOrEqual(0)
  expect(box.y + box.height).toBeLessThanOrEqual(844)
})
