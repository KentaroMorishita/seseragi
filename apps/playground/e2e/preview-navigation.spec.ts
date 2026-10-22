import { expect, type Page, test } from "@playwright/test"
import {
  heldPreviewCount,
  holdPreviewReadiness,
  installListenerAudit,
  listenerCount,
  releasePreviewReadiness,
} from "./preview-navigation-support"

async function selectInteractive(page: Page): Promise<void> {
  await page.locator("#sample-browser-button").click()
  await page.locator('[data-sample-id="form-todo"]').click()
  await expect(page.locator("#status-text")).toHaveText("Analysis ready")
}

for (const action of ["sample", "reset", "clear"] as const) {
  test(`pending interactive preview can be cancelled by ${action} and Run works again`, async ({
    page,
  }) => {
    await installListenerAudit(page)
    await page.goto("/")
    await selectInteractive(page)
    await holdPreviewReadiness(page)
    await page.locator("#run-button").click()
    await expect.poll(() => heldPreviewCount(page)).toBe(1)
    await expect(page.locator("#run-button")).toBeDisabled()
    expect(await listenerCount(page)).toBe(1)
    if (action === "clear") {
      await page.locator("#clear-output-button").click()
    } else {
      await page.locator("#sample-browser-button").click()
      await page
        .locator(
          action === "reset" ? "#reset-sample-button" : "#sample-starter-button"
        )
        .click()
    }
    await expect(page.locator("#run-button")).toBeEnabled()
    expect(await listenerCount(page)).toBe(0)
    await expect(page.locator("#html-preview")).not.toHaveAttribute("srcdoc")
    await releasePreviewReadiness(page)
    if (action !== "clear")
      await expect(page.locator("#status-text")).toHaveText("Analysis ready")
    await page.locator("#run-button").click()
    if (action === "sample") {
      await expect(page.locator("#output")).toContainText("Hello")
    } else {
      await expect(
        page
          .frameLocator("#html-preview")
          .getByRole("heading", { name: "Plan a calmer launch." })
      ).toBeVisible()
      await expect(page.locator("#status-text")).toHaveText("Interactive")
    }
    await expect(page.locator("#run-button")).toBeEnabled()
    expect(await listenerCount(page)).toBe(0)
  })
}

test("cancellation during readiness capture cannot bootstrap the old preview", async ({
  page,
}) => {
  await installListenerAudit(page)
  await page.goto("/")
  await selectInteractive(page)
  await page.evaluate(() => {
    window.previewReadiness.cancelButton = "clear-output-button"
  })
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Output cleared")
  await expect(page.locator("#run-button")).toBeEnabled()
  await expect(page.locator("#html-preview")).toBeHidden()
  await expect(page.locator("#html-preview")).not.toHaveAttribute("srcdoc")
  await expect(page.locator("#output")).toHaveText("")
  expect(await listenerCount(page)).toBe(0)
  await page.locator("#run-button").click()
  await expect(
    page
      .frameLocator("#html-preview")
      .getByRole("heading", { name: "Plan a calmer launch." })
  ).toBeVisible()
})

test("a cancelled Run finally cannot unlock the next pending Run", async ({
  page,
}) => {
  await installListenerAudit(page)
  await page.goto("/")
  await selectInteractive(page)
  await holdPreviewReadiness(page)
  await page.locator("#run-button").click()
  await expect.poll(() => heldPreviewCount(page)).toBe(1)
  await page.evaluate(() => {
    ;(
      document.querySelector("#clear-output-button") as HTMLButtonElement
    ).click()
    ;(document.querySelector("#run-button") as HTMLButtonElement).click()
  })
  await expect.poll(() => heldPreviewCount(page)).toBe(2)
  await expect(page.locator("#run-button")).toBeDisabled()
  expect(await listenerCount(page)).toBe(1)
  await releasePreviewReadiness(page, 0, false)
  await expect(page.locator("#run-button")).toBeDisabled()
  await releasePreviewReadiness(page, 1)
  await expect(
    page
      .frameLocator("#html-preview")
      .getByRole("heading", { name: "Plan a calmer launch." })
  ).toBeVisible()
  await expect(page.locator("#run-button")).toBeEnabled()
  expect(await listenerCount(page)).toBe(0)
})

test("an explicit Run recovers from a failed actual WASM fetch in the same Worker", async ({
  page,
}) => {
  let fetches = 0
  const workers: string[] = []
  // Program runs now own separate short-lived Workers. This assertion is
  // specifically about retrying the WASM fetch in the same compiler Worker.
  page.on("worker", (worker) => {
    if (worker.url().includes("compiler.worker")) workers.push(worker.url())
  })
  await page.route(
    /\/seseragi_wasm_bg(?:-[A-Za-z0-9_-]+)?\.wasm(?:\?.*)?$/,
    (route) => {
      fetches++
      return fetches === 1 ? route.abort("failed") : route.continue()
    }
  )
  await page.goto("/")
  await expect(page.locator("#status-dot")).toHaveAttribute(
    "data-state",
    "error"
  )
  expect(fetches).toBe(1)
  expect(workers).toHaveLength(1)
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toContainText("Hello")
  await expect(page.locator("#run-button")).toBeEnabled()
  expect(fetches).toBe(2)
  expect(workers).toHaveLength(1)
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed")
  expect(fetches).toBe(2)
  expect(workers).toHaveLength(1)
})
