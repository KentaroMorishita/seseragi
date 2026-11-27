import { expect, test } from "@playwright/test"

test("opens /en/ and /en/tour/ in English and stays English across reload", async ({
  page,
}) => {
  await page.goto("/en/")
  await expect(page.locator("html")).toHaveAttribute("lang", "en")
  await expect(page.locator("#lang-en-button")).toHaveAttribute(
    "aria-pressed",
    "true"
  )
  await expect(page.locator("#lang-ja-button")).toHaveAttribute(
    "aria-pressed",
    "false"
  )
  await expect(page.locator("#lang-ja-button")).toHaveAttribute(
    "aria-label",
    "日本語"
  )
  await expect(page.locator("#lang-en-button")).toHaveAttribute(
    "aria-label",
    "English"
  )

  await page.reload()
  await expect(page).toHaveURL(/\/en\/$/u)
  await expect(page.locator("html")).toHaveAttribute("lang", "en")
  await expect(page.locator("#auto-run-note")).toContainText(
    "Ran automatically when opened."
  )

  await page.goto("/en/tour/")
  await expect(page.locator("html")).toHaveAttribute("lang", "en")
  await expect(page.locator("#tour-untranslated-notice")).toBeVisible()
  await expect(page.locator("#tour-untranslated-notice")).toHaveText(
    "This lesson isn't translated yet. Its text is shown in the original Japanese. Buttons, code, and output work the same."
  )
  await expect(page.locator("#tour-lesson-title")).toHaveAttribute("lang", "ja")
  await expect(page.locator("#tour-guide")).toHaveAttribute("lang", "ja")

  await page.reload()
  await expect(page).toHaveURL(/\/en\/tour\//u)
  await expect(page.locator("html")).toHaveAttribute("lang", "en")
  await expect(page.locator("#tour-untranslated-notice")).toBeVisible()
})

test("switching JA and EN preserves example, edited code, and output", async ({
  page,
}) => {
  await page.goto("/?example=fizzbuzz")
  await expect(page.locator("#status-text")).toHaveText("Completed", {
    timeout: 60_000,
  })
  const initialOutput = await page.locator("#output").textContent()
  expect(initialOutput).toContain("FizzBuzz")

  // コードにコメントを足して状態を作る
  const editor = page.locator("#editor .cm-content")
  await editor.click()
  await page.keyboard.press("End")
  await page.keyboard.type(" // kept")
  const editedSource = await editor.textContent()
  expect(editedSource).toContain("// kept")

  await page.locator("#lang-en-button").click()
  await expect(page).toHaveURL(/\/en\/$/u)
  await expect(page.locator("html")).toHaveAttribute("lang", "en")
  await expect(page.locator("#current-sample-title")).toHaveText(
    "規則を足せる FizzBuzz"
  )
  await expect(page.locator("#current-sample-title")).toHaveAttribute(
    "lang",
    "ja"
  )
  await expect(editor).toContainText("// kept")
  await expect(page.locator("#output")).toHaveText(initialOutput ?? "")

  // Search の未訳項目に Japanese only 印が付く
  await page.locator("#search-button").click()
  await expect(
    page.locator('#search-results [data-sample-id="fizzbuzz"] .ui-tag').first()
  ).toHaveText("Japanese only")
  await page.keyboard.press("Escape")

  // Guide を開くと未訳案内と lang="ja" が付く
  await page.locator("#sample-guide-button").click()
  await expect(page.locator("#sample-guide-untranslated")).toBeVisible()
  await expect(page.locator("#sample-guide-body")).toHaveAttribute("lang", "ja")
  await page.locator("#sample-guide-close").click()

  // 日本語に戻してもコードと出力が残る
  await page.locator("#lang-ja-button").click()
  await expect(page).toHaveURL(/\/$/u)
  await expect(page.locator("html")).toHaveAttribute("lang", "ja")
  await expect(editor).toContainText("// kept")
  await expect(page.locator("#output")).toHaveText(initialOutput ?? "")
})

test("shows English banner once for non-Japanese browsers and never auto-redirects", async ({
  browser,
}) => {
  const context = await browser.newContext({ locale: "en-US" })
  const page = await context.newPage()
  try {
    await page.goto("/")
    // 自動で /en/ へ移動しない
    await expect(page).toHaveURL(/\/$/u)
    await expect(page.locator("html")).toHaveAttribute("lang", "ja")

    const banner = page.locator("#english-available-banner")
    await expect(banner).toBeVisible()
    await expect(banner).toContainText("English is available.")

    await page.locator("#english-banner-dismiss").click()
    await expect(banner).toBeHidden()

    await page.reload()
    await expect(page).toHaveURL(/\/$/u)
    await expect(banner).toBeHidden()
  } finally {
    await context.close()
  }
})
