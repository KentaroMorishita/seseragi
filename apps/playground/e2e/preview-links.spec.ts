import { expect, type Page, test } from "@playwright/test"
import { rememberNetworkDecision } from "./network-decision-support"
import { startQuietly } from "./quiet-start-support"

// 初めて開いたときの案内（最初の例の自動実行など）は first-example.spec.ts で確かめる
test.beforeEach(async ({ page }) => {
  await startQuietly(page)
})

// #1375: links the user clicks inside the sandboxed Preview open in a new tab
// through the shell, without relaxing the frame sandbox.

async function openSample(page: Page, id: string): Promise<void> {
  // リンクを確かめるテスト。外部の画像は止めたままにしておく
  await rememberNetworkDecision(page, [`example:${id}`], "denied")
  await page.goto("/")
  await expect(page.locator("#status-text")).toHaveText("Analysis ready", {
    timeout: 60_000,
  })
  await page.locator("#sample-browser-button").click()
  await page.locator(`#search-dialog [data-sample-id="${id}"]`).click()
  await expect(page.locator("#status-text")).toHaveText("Analysis ready", {
    timeout: 60_000,
  })
  await page.locator("#run-button").click()
}

test.beforeEach(async ({ context }) => {
  // Never leave the test machine: answer every external page locally.
  await context.route(
    /^https:\/\/(github\.com|seseragi\.vercel\.app)\//,
    (route) =>
      route.fulfill({ contentType: "text/html", body: "<title>opened</title>" })
  )
})

test("an interactive preview opens an external link in a new tab", async ({
  page,
  context,
}) => {
  await openSample(page, "seseragi-landing-page")
  const preview = page.frameLocator("#html-preview")
  const github = preview.locator("header a", { hasText: "GitHub" }).first()
  await expect(github).toBeVisible({ timeout: 60_000 })

  const popup = context.waitForEvent("page")
  await github.click()
  const opened = await popup
  await opened.waitForLoadState()
  expect(opened.url()).toMatch(/^https:\/\/github\.com\//)
  await expect(page.locator("#preview-link-notice")).toBeVisible()
  await expect(page.locator("#preview-link-notice a")).toHaveAttribute(
    "href",
    opened.url()
  )
  // The frame keeps its strict sandbox.
  await expect(page.locator("#html-preview")).toHaveAttribute(
    "sandbox",
    "allow-scripts allow-forms"
  )
})

test("a static HTML preview opens an external link in a new tab", async ({
  page,
  context,
}) => {
  await openSample(page, "html-components")
  await page.locator("#show-html-preview-button").click()
  const preview = page.frameLocator("#html-preview")
  const link = preview.locator('a[href^="https://github.com/"]').first()
  await expect(link).toBeVisible({ timeout: 60_000 })

  const popup = context.waitForEvent("page")
  await link.click()
  const opened = await popup
  await opened.waitForLoadState()
  expect(opened.url()).toMatch(/^https:\/\/github\.com\//)
  await expect(page.locator("#preview-link-notice")).toBeVisible()
})
