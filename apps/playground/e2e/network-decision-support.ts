import type { Page } from "@playwright/test"

/**
 * 外部通信の許可（#1371）を、先にこのブラウザへ答えておく。
 * リンクや実行の後片付けを確かめるテストが、許可のモーダルに止められないようにする。
 */
export async function rememberNetworkDecision(
  page: Page,
  scopes: readonly string[],
  decision: "allowed" | "denied"
): Promise<void> {
  await page.addInitScript(
    ({ scopes, decision }) => {
      if (window !== window.top) return
      localStorage.setItem(
        "seseragi.network-permissions.v1",
        JSON.stringify(
          scopes.map((scope) => ({
            scope,
            label: scope,
            decision,
            decidedAt: "2026-01-01T00:00:00.000Z",
          }))
        )
      )
    },
    { scopes, decision }
  )
}
