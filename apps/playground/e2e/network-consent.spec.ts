import { expect, type Page, test } from "@playwright/test"
import { startQuietly } from "./quiet-start-support"

// 初めて開いたときの案内（最初の例の自動実行など）は first-example.spec.ts で確かめる
test.beforeEach(async ({ page }) => {
  await startQuietly(page)
})

// 外部通信の許可（#1371）: 初めて要る Run で一度だけ確かめ、例ごとに覚え、
// Settings（Search の Network access）から取り消せる。

let images = 0

test.beforeEach(async ({ context }) => {
  images = 0
  await context.route("https://raw.githubusercontent.com/**", async (route) => {
    images++
    await route.fulfill({
      contentType: "image/svg+xml",
      body: '<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"/>',
    })
  })
})

async function openSample(page: Page, sampleId: string): Promise<void> {
  await page.goto("/")
  await expect(page.locator("#status-text")).toHaveText("Analysis ready", {
    timeout: 60_000,
  })
  await page.locator("#sample-browser-button").click()
  await page.locator(`[data-sample-id="${sampleId}"]`).click()
  await expect(page.locator("#status-text")).toHaveText("Analysis ready")
}

test("renders seseragi-landing-page with default settings without network prompts", async ({
  page,
}) => {
  await openSample(page, "seseragi-landing-page")
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Interactive", {
    timeout: 60_000,
  })
  await expect(page.locator("#network-consent-dialog")).toBeHidden()
  await expect(page.locator("#preview-network-notice")).toBeHidden()
  expect(images).toBe(0)
  const frame = page.frameLocator("#html-preview")
  await expect(
    frame.getByRole("img", { name: "Seseragi ロゴマーク" }).first()
  ).toBeVisible()
  await expect(frame.getByRole("img", { name: "流れる緑の光" })).toBeVisible()
})

test("asks once on the first Run that needs the network, then remembers", async ({
  page,
}) => {
  await openSample(page, "project-flow-app")
  await page.locator("#mobile-tools-button").click()
  await expect(page.getByRole("menuitemcheckbox")).toHaveCount(0)
  await page.keyboard.press("Escape")

  const consent = page.locator("#network-consent-dialog")
  await page.locator("#run-button").click()
  await expect(consent).toBeVisible()
  await expect(consent).toContainText(
    "旅行プランナー（地図ルート・日程・割り勘）"
  )
  await expect(consent.getByRole("button", { name: "Cancel" })).toBeFocused()
  await consent.getByRole("button", { name: "Cancel" }).click()
  const notice = page.locator("#preview-network-notice")
  await expect(notice).toContainText("外部の画像と通信を止めています")
  expect(images).toBe(0)

  // 一度答えた例では、次の Run でもう聞かない
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Interactive")
  await expect(notice).toBeVisible()
  await expect(consent).toBeHidden()

  await notice.getByRole("button", { name: "Allow" }).click()
  await expect.poll(() => images).toBeGreaterThan(0)
  await expect(notice).toBeHidden()

  await page.reload()
  await expect(page.locator("#status-text")).toHaveText("Analysis ready", {
    timeout: 60_000,
  })
  images = 0
  await page.locator("#run-button").click()
  await expect.poll(() => images).toBeGreaterThan(0)
  await expect(consent).toBeHidden()
})

test("removes a remembered answer from Network access", async ({ page }) => {
  await openSample(page, "project-flow-app")
  await page.locator("#run-button").click()
  await page
    .locator("#network-consent-dialog")
    .getByRole("button", { name: "Allow" })
    .click()
  await expect.poll(() => images).toBeGreaterThan(0)

  await page.locator("#search-button").click()
  await page.locator('[data-filter="settings"]').click()
  await expect(
    page.locator('[data-command="network-access"] .ui-tag')
  ).toHaveText("Allowed")
  await page.locator('[data-command="network-access"]').click()
  const settings = page.locator("#network-settings-dialog")
  await expect(settings).toBeVisible()
  await settings
    .getByRole("button", {
      name: "Remove 旅行プランナー（地図ルート・日程・割り勘）",
    })
    .click()
  await expect(settings).toContainText("まだ、どの例にも答えていません。")
  await settings.getByRole("button", { name: "Close network access" }).click()

  await page.locator("#run-button").click()
  await expect(page.locator("#network-consent-dialog")).toBeVisible()
})

test("keeps the consent dialog and its buttons compact on a phone", async ({
  page,
}) => {
  // iPhone で、ダイアログが画面いっぱいに伸び、ボタンが細い帯になった不具合の再発を防ぐ
  await page.setViewportSize({ width: 390, height: 844 })
  await openSample(page, "project-flow-app")
  await page.locator("#run-button").click()
  const consent = page.locator("#network-consent-dialog")
  await expect(consent).toBeVisible()
  const dialog = await consent.boundingBox()
  expect(dialog?.height ?? 0).toBeLessThan(844 * 0.7)
  for (const name of ["Cancel", "Allow"]) {
    const box = await consent.getByRole("button", { name }).boundingBox()
    expect(Math.round(box?.height ?? 0)).toBe(44)
    expect(box?.width ?? 0).toBeGreaterThan(120)
  }
})
