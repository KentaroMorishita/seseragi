import type { Page } from "@playwright/test"

/**
 * 初めて開いたときの案内（最初の例の自動実行、ひとこと、Try next）を止めて開く。
 * Settings の Hints をオフにしたのと同じ。案内そのものは e2e/first-example.spec.ts で確かめる。
 */
export async function startQuietly(page: Page): Promise<void> {
  await page.addInitScript(() => {
    if (window === window.top)
      localStorage.setItem("seseragi.hints.enabled", "off")
  })
}
