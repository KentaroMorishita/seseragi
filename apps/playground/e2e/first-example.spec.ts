import { expect, test } from "@playwright/test"

// 初めて開いたときの体験（#1350 の全体設計 2、DESIGN.md「案内は初回だけ、消せて、ふだんは静か」）。

test("first visit runs the weather example once and points at what to change", async ({
  page,
}) => {
  await page.goto("/")
  await expect(page.locator("#current-sample-title")).toHaveText(
    "天気のアドバイス"
  )
  await expect(page.locator("#status-text")).toHaveText("Completed", {
    timeout: 60_000,
  })
  await expect(page.locator("#output")).toContainText(
    "傘を持って出かけましょう（3mm）"
  )
  await expect(page.locator("#auto-run-note")).toBeVisible()
  const coach = page.locator(".pg-coach")
  await expect(coach).toContainText("3 を 12 に変えて")
  await expect(page.locator(".cm-coach-anchor")).toHaveText("Rainy 3")

  // Close で消え、同じブラウザでは二度と出さない
  await coach.getByRole("button", { name: "Close" }).click()
  await expect(coach).toHaveCount(0)
  // 保存したコードだけを消して、もう一度「初めて開いた」状態にする
  await page.evaluate(() =>
    localStorage.removeItem("seseragi.playground.workspace.schema-1")
  )
  await page.reload()
  await expect(page.locator("#status-text")).toHaveText("Completed", {
    timeout: 60_000,
  })
  await expect(coach).toHaveCount(0)
})

test("editing the code dismisses the hint", async ({ page }) => {
  await page.goto("/")
  await expect(page.locator(".pg-coach")).toBeVisible({ timeout: 60_000 })
  await page.locator(".cm-content").press("End")
  await page.keyboard.type(" ")
  await expect(page.locator(".pg-coach")).toHaveCount(0)
  await expect(page.locator("#auto-run-note")).toBeHidden()
})

test("a Try next chip rewrites the code and runs it", async ({ page }) => {
  await page.goto("/")
  await expect(page.locator("#status-text")).toHaveText("Completed", {
    timeout: 60_000,
  })
  const chips = page.locator("#try-next")
  await expect(
    chips.getByRole("button", { name: "くもりを足す" })
  ).toBeVisible()
  await expect(
    chips.getByRole("link", { name: "基本から順に学ぶ →" })
  ).toHaveAttribute("href", /\/tour\/$/u)

  await chips.getByRole("button", { name: "くもりを足す" }).click()
  await expect(page.locator(".cm-content")).toContainText("| Cloudy")
  await expect(page.locator("#output")).toContainText(
    "折りたたみ傘があると安心です"
  )
  await expect(chips.locator('[data-try-next="1"]')).toBeDisabled()

  // Hide で一行に畳み、その選択を覚える
  await chips.getByRole("button", { name: "Hide" }).click()
  await expect(chips.locator(".pg-try-next-chips")).toBeHidden()
  await expect(chips.getByRole("button", { name: "Show" })).toBeVisible()
  expect(
    await page.evaluate(() =>
      localStorage.getItem("seseragi.hints.try-next-collapsed")
    )
  ).toBe("true")
})

test("free writing stays quiet: no chips, no auto-run after a restore", async ({
  page,
}) => {
  await page.goto("/")
  await expect(page.locator("#status-text")).toHaveText("Completed", {
    timeout: 60_000,
  })
  // 2 回目: 前回のコードが戻る。自動では実行せず、案内も出さない
  await page.reload()
  await expect(page.locator("#status-text")).toHaveText("Analysis ready", {
    timeout: 60_000,
  })
  await expect(page.locator("#try-next")).toBeHidden()
  await expect(page.locator(".pg-coach")).toHaveCount(0)
  await expect(page.locator("#auto-run-note")).toBeHidden()

  // 例を開き直すと、Try next が戻る
  await page.locator("#sample-browser-button").click()
  await page.locator('[data-sample-id="weather-advice"]').click()
  await expect(page.locator("#try-next")).toBeVisible()
  // New file（白紙）では出さない
  await page.locator("#search-button").click()
  await page.locator('[data-filter="settings"]').click()
  page.once("dialog", (dialog) => void dialog.accept())
  await page.locator('[data-command="new-file"]').click()
  await expect(page.locator("#try-next")).toBeHidden()
})

test("Hints off hides the chips", async ({ page }) => {
  await page.goto("/")
  await expect(page.locator("#try-next")).toBeVisible({ timeout: 60_000 })
  await page.locator("#search-button").click()
  await page.locator('[data-filter="settings"]').click()
  await expect(page.locator('[data-command="hints"] .ui-tag')).toHaveText("On")
  await page.locator('[data-command="hints"]').click()
  await expect(page.locator("#try-next")).toBeHidden()
  await expect(page.locator(".pg-coach")).toHaveCount(0)
})
