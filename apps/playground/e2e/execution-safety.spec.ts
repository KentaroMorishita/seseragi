import type { BrowserExecution } from "../src/runtime/browser-execution"
import { expect, test, type Page } from "@playwright/test"

async function runFixture(
  page: Page,
  script: string,
  options: {
    network?: boolean
    dom?: boolean
    foreign?: boolean
    console?: boolean
    timeout?: number
  } = {}
) {
  return page.evaluate(
    async ({ script, options }) => {
      const load = (path: string) => import(/* @vite-ignore */ path)
      const { compileSingleFile } = await load("/src/compiler/wasm-driver.ts")
      const { startIsolatedProject } = await load(
        "/src/runtime/isolated-execution.ts"
      )
      const compiled = await compileSingleFile(
        "pub effect fn main -> Unit = succeed ()\n"
      )
      if (compiled.status !== "success" || !compiled.entry)
        throw new Error("fixture did not compile")
      const frame = document.createElement("iframe")
      document.body.append(frame)
      let execution: BrowserExecution | undefined
      try {
        execution = await startIsolatedProject(
          frame,
          {
            modules: [{ path: "main.ssrg", typescript: script }],
            entryPath: "main.ssrg",
            entry: {
              ...compiled.entry,
              environment: options.dom
                ? [{ field: "dom", service: "dom" }]
                : options.console
                  ? [{ field: "console", service: "console" }]
                  : [],
            },
            input: "",
            foreignRuntime: options.foreign
              ? {
                  snapshotDigest: "fixture",
                  modules: {
                    "external.js": {
                      path: "external.js",
                      source: "export const marker = 1",
                      exactIdentity: "fixture",
                      contentDigest: "fixture",
                      imports: {},
                    },
                  },
                  roots: {},
                }
              : undefined,
            initialHref: new URL("/runtime-preview.html", location.href).href,
          },
          {
            signal: new AbortController().signal,
            networkAllowed: options.network,
            timeoutMs: options.timeout,
          }
        )
        await execution!.completion
        return "completed"
      } catch (error) {
        return error instanceof Error ? error.message : String(error)
      } finally {
        await execution?.cancel()
        frame.remove()
      }
    },
    { script, options }
  )
}

test("a shared workspace does not inherit input or overwrite local edits, and reload retains its edits", async ({
  page,
}) => {
  await page.goto("/")
  await expect(page.locator("#status-text")).toHaveText("Analysis ready")
  const original = 'pub effect fn main = println "local-marker"\n'
  const shared = 'pub effect fn main = println "shared-marker"\n'
  const edited = 'pub effect fn main = println "edited-marker"\n'
  const editor = page.getByRole("textbox", { name: "Seseragi source editor" })
  await editor.fill(original)
  await page.locator("#stdin-toggle-button").click()
  await page.locator("#stdin-input").fill("private-input-marker")
  const saved = await page.evaluate(() =>
    localStorage.getItem("seseragi.playground.workspace.schema-1")
  )
  await page.goto(`/?source=${encodeURIComponent(shared)}`)
  await expect(editor).toContainText("shared-marker")
  await expect(page.locator("#stdin-input")).toHaveValue("")
  await expect(page).toHaveURL(/workspace=[0-9a-f-]+/)
  expect(page.url()).not.toContain("source=")
  await editor.fill(edited)
  await page.reload()
  await expect(editor).toContainText("edited-marker")
  expect(
    await page.evaluate(() =>
      localStorage.getItem("seseragi.playground.workspace.schema-1")
    )
  ).toBe(saved)
  await page.goto("/")
  await expect(editor).toContainText("local-marker")
  await expect(page.locator("#stdin-input")).toHaveValue("private-input-marker")
})

test("a busy program worker is stopped and the editor can run again", async ({
  page,
}) => {
  await page.goto("/")
  expect(
    await runFixture(
      page,
      "export function main() { return () => { while (true) {} }; }",
      { timeout: 2000 }
    )
  ).toContain("Program time limit exceeded")
  expect(
    await runFixture(page, "export function main() { return () => undefined; }")
  ).toBe("completed")
  await expect(page.locator("#run-button")).toBeEnabled()
})

test("generated DOM loops are guarded before they monopolize the editor", async ({
  page,
}) => {
  await page.goto("/")
  expect(
    await runFixture(
      page,
      "export function main() { return () => { while (true) {} }; }",
      { dom: true }
    )
  ).toContain("Program computation limit exceeded")
  await expect(page.locator("#run-button")).toBeEnabled()
})

test("Worker HTTP is blocked by default and available only with explicit permission", async ({
  page,
  context,
}) => {
  let requests = 0
  await context.route("https://preview-network.invalid/**", async (route) => {
    requests++
    await route.fulfill({
      status: 200,
      headers: { "access-control-allow-origin": "*" },
      body: "fixture",
    })
  })
  await page.goto("/")
  const script =
    'export function main() { return async () => { await fetch("https://preview-network.invalid/marker").catch(() => undefined); }; }'
  expect(await runFixture(page, script)).toBe("completed")
  expect(requests).toBe(0)
  expect(await runFixture(page, script, { network: true })).toBe("completed")
  expect(requests).toBe(1)
  const permission = page.getByRole("menuitemcheckbox", {
    name: "Allow preview HTTP, WebSocket and external images",
  })
  await page.locator("#mobile-tools-button").click()
  await permission.click()
  await page
    .getByRole("textbox", { name: "Seseragi source editor" })
    .fill('pub effect fn main = println "changed"')
  await expect(permission).toHaveAttribute("aria-checked", "false")
})

test("output cannot grow past the per-run bound", async ({ page }) => {
  await page.goto("/")
  expect(
    await runFixture(
      page,
      'export function main() { return env => { env.console.print("x".repeat(300000)); }; }',
      { console: true }
    )
  ).toContain("Program output limit exceeded")
})

test("a shared program cannot read another workspace's application storage", async ({
  page,
}) => {
  const head = `import * as storage from "std/web/storage"
type AppError deriving Show =
  | StorageFailure storage.StorageError
  | ConsoleFailure ConsoleError
pub effect fn main -> Unit
with storage: storage.Storage, Console
fails AppError = do {
`
  const write = `${head}  _ <- storage.set storage.Local "profile" "private-app-marker" |> mapError StorageFailure
  succeed ()
}\n`
  const read = `${head}  value <- storage.get storage.Local "profile" |> mapError StorageFailure
  println (show value) |> mapError ConsoleFailure
}\n`
  await page.goto("/")
  await page
    .getByRole("textbox", { name: "Seseragi source editor" })
    .fill(write)
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed")
  await page.goto(`/?source=${encodeURIComponent(read)}`)
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed")
  await expect(page.locator("#output")).toHaveText("Nothing")
})

test("Text output does not load external images until Preview is selected", async ({
  page,
  context,
}) => {
  let images = 0
  await context.route("https://preview-images.invalid/**", async (route) => {
    images++
    await route.fulfill({
      contentType: "image/svg+xml",
      body: '<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>',
    })
  })
  await page.goto("/")
  await page
    .getByRole("textbox", { name: "Seseragi source editor" })
    .fill(
      "pub effect fn main = println \"<img src='https://preview-images.invalid/marker'>\"\n"
    )
  await page.locator("#mobile-tools-button").click()
  await page
    .getByRole("menuitemcheckbox", {
      name: "Allow preview HTTP, WebSocket and external images",
    })
    .click()
  await page.locator("#run-button").click()
  await expect(page.locator("#status-text")).toHaveText("Completed")
  expect(images).toBe(0)
  await page.locator("#show-html-preview-button").click()
  await expect.poll(() => images).toBe(1)
})

test("foreign JavaScript is refused in DOM previews that cannot forcibly terminate it", async ({
  page,
}) => {
  await page.goto("/")
  expect(
    await runFixture(
      page,
      "export function main() { return () => undefined; }",
      { dom: true, foreign: true }
    )
  ).toContain("Interactive foreign JavaScript is not supported")
})
