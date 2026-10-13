import { expect, test } from "@playwright/test"
import {
  installListenerAudit,
  listenerCount,
} from "./preview-navigation-support"

for (const action of ["reset", "lesson", "text"] as const) {
  test(`Tour cancels pending interactive preparation on ${action} and permits the next Run`, async ({
    page,
  }) => {
    await installListenerAudit(page, "tour-html-preview")
    await page.goto("/tour/?lesson=web-dom-run")
    await expect(page.locator("#tour-status-text")).toHaveText("Analysis ready")
    let release!: () => void
    const held = new Promise<void>((resolve) => {
      release = resolve
    })
    let arrived!: () => void
    const requested = new Promise<void>((resolve) => {
      arrived = resolve
    })
    await page.route("**/runtime-preview.html", async (route) => {
      arrived()
      await held
      await route.continue().catch(() => {})
    })
    await page.locator("#tour-run-button").click()
    await requested
    await expect(page.locator("#tour-run-button")).toBeDisabled()
    expect(await listenerCount(page)).toBe(2)
    if (action === "reset") {
      await page.locator("#tour-tools-button").click()
      await page.locator("#tour-reset-button").click()
    } else if (action === "lesson") {
      await page.locator("#tour-next-button").click()
      await expect(page).not.toHaveURL(/lesson=web-dom-run$/)
    } else {
      await page.locator("#tour-show-text-button").click()
    }
    await expect(page.locator("#tour-run-button")).toBeEnabled()
    await expect(page.locator("#tour-html-preview")).not.toHaveAttribute("src")
    expect(await listenerCount(page)).toBe(0)
    release()
    await page.unroute("**/runtime-preview.html")
    if (action !== "text")
      await expect(page.locator("#tour-status-text")).toHaveText(
        "Analysis ready"
      )
    await page.locator("#tour-run-button").click()
    if (action === "lesson") {
      await expect(page.locator("#tour-status-text")).toHaveText("Completed")
      await expect(page.locator("#tour-output")).toHaveText("<p>Ada</p>")
      await expect(page.locator("#tour-run-button")).toBeEnabled()
      return
    }
    await expect(page.locator("#tour-status-text")).toHaveText("Interactive")
    const increment = page
      .frameLocator("#tour-html-preview")
      .getByRole("button", { name: "Count: 0", exact: true })
    await expect(increment).toBeVisible()
    await increment.click()
    await expect(
      page
        .frameLocator("#tour-html-preview")
        .getByRole("button", { name: "Count: 1", exact: true })
    ).toBeVisible()
    await expect(page.locator("#tour-run-button")).toBeEnabled()
  })
}
