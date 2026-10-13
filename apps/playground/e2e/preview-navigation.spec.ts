import { expect, type Page, type Route, test } from "@playwright/test"
import {
  installListenerAudit,
  listenerCount,
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
    let release!: () => void
    const held = new Promise<void>((resolve) => {
      release = resolve
    })
    let requested!: () => void
    const arrived = new Promise<void>((resolve) => {
      requested = resolve
    })
    const pendingRoutes: Promise<void>[] = []
    const hold = async (route: Route): Promise<void> => {
      requested()
      await held
      // Cancellation aborts the old iframe request; only transport cleanup is ignored.
      await route.continue().catch(() => {})
    }
    await page.route("**/runtime-preview.html", (route) => {
      const pending = hold(route)
      pendingRoutes.push(pending)
      return pending
    })
    await page.locator("#run-button").click()
    await arrived
    await expect(page.locator("#run-button")).toBeDisabled()
    expect(await listenerCount(page)).toBe(2)
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
    await expect(page.locator("#html-preview")).not.toHaveAttribute("src")
    release()
    await Promise.all(pendingRoutes)
    await page.unroute("**/runtime-preview.html")
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

test("cancellation during load capture cannot bootstrap the old preview", async ({
  page,
}) => {
  await installListenerAudit(page)
  await page.goto("/")
  await selectInteractive(page)
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
    await route.continue()
  })
  await page.locator("#run-button").click()
  await requested
  // Capture cancels before the production load listener can publish readiness,
  // even though the real navigation has reached its expected document.
  await page.evaluate(() => {
    document.querySelector("#html-preview")?.addEventListener(
      "load",
      () => {
        Object.assign(window, {
          cancelledPreviewDocument: (
            document.querySelector("#html-preview") as HTMLIFrameElement
          ).contentDocument,
        })
        ;(
          document.querySelector("#clear-output-button") as HTMLButtonElement
        ).click()
      },
      { once: true, capture: true }
    )
  })
  release()
  await expect(page.locator("#status-text")).toHaveText("Output cleared")
  await expect(page.locator("#run-button")).toBeEnabled()
  await expect(page.locator("#html-preview")).toBeHidden()
  await expect(page.locator("#html-preview")).not.toHaveAttribute("src")
  await expect(page.locator("#output")).toHaveText("")
  expect(
    await page.evaluate(() =>
      (
        window as unknown as { cancelledPreviewDocument: Document }
      ).cancelledPreviewDocument.querySelector("#app")
    )
  ).toBeNull()
  expect(await listenerCount(page)).toBe(0)
  await page.unroute("**/runtime-preview.html")
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
  const releases: (() => void)[] = []
  await page.route("**/runtime-preview.html", async (route) => {
    await new Promise<void>((resolve) => {
      releases.push(resolve)
    })
    await route.continue().catch(() => {})
  })
  await page.locator("#run-button").click()
  await expect.poll(() => releases.length).toBe(1)
  await page.evaluate(() => {
    ;(
      document.querySelector("#clear-output-button") as HTMLButtonElement
    ).click()
    ;(document.querySelector("#run-button") as HTMLButtonElement).click()
  })
  await expect.poll(() => releases.length).toBe(2)
  await expect(page.locator("#run-button")).toBeDisabled()
  expect(await listenerCount(page)).toBe(2)
  releases[0]?.()
  await expect(page.locator("#run-button")).toBeDisabled()
  releases[1]?.()
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
  page.on("worker", (worker) => workers.push(worker.url()))
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
