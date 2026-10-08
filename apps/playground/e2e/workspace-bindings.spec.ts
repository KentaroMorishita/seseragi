import { readFile } from "node:fs/promises"
import { expect, test } from "@playwright/test"

async function inputWorkspace() {
  const root = new URL(
    "../../../examples/spec/fixtures/projects/dts-basic-conversion/",
    import.meta.url
  )
  const files = await Promise.all(
    [
      "seseragi.toml",
      "seseragi.bindings.toml",
      "host/index.d.ts",
      "host/package.json",
    ].map(async (path) => ({
      path,
      source: await readFile(new URL(path, root), "utf8"),
    }))
  )
  files.push({
    path: "main.ssrg",
    source:
      'pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println "ordinary source"\n',
  })
  return {
    schema: 1,
    sampleId: "playground-blank",
    sampleHash: "workspace:blank-v1",
    stdin: "",
    workspace: {
      files,
      folders: ["host"],
      dirtyFiles: [],
      entryFile: "main.ssrg",
      activeFile: "host/index.d.ts",
      openFiles: ["main.ssrg", "host/index.d.ts"],
      expandedFolders: ["host"],
      explorer: { visible: true, width: 280 },
    },
  }
}

test.beforeEach(async ({ page }) => {
  const seed = await inputWorkspace()
  await page.addInitScript((seed) => {
    const key = "seseragi.playground.workspace.schema-1"
    if (localStorage.getItem(key) === null)
      localStorage.setItem(key, JSON.stringify(seed))
  }, seed)
})

test("external workspace edits survive reload, diagnose rename/delete, and preserve ordinary Run", async ({
  page,
}) => {
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  await page.goto("/")
  await expect(page.locator("#active-file-name")).toHaveText("host/index.d.ts")
  await expect(page.locator("#format-source-button")).toBeDisabled()
  await expect(
    page.getByRole("button", {
      name: "Set host/index.d.ts as entry",
      exact: true,
    })
  ).toHaveCount(0)
  for (const path of [
    "host/index.d.ts",
    "seseragi.bindings.toml",
    "host/package.json",
  ]) {
    await page
      .locator(`[data-explorer-path="${path}"] .explorer-row-label`)
      .click()
    await expect(page.locator("#active-file-name")).toHaveText(path)
    await page.locator("#run-button").click()
    await expect(page.locator("#output")).toHaveText("ordinary source")
    await expect(page.locator("#active-file-name")).toHaveText(path)
  }
  await page
    .locator('[data-explorer-path="host/index.d.ts"] .explorer-row-label')
    .click()
  const editor = page.getByRole("textbox", { name: "Seseragi source editor" })
  await editor.fill("export declare function café(): number;")
  await page
    .getByRole("button", { name: "Convert bindings", exact: true })
    .click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await expect(page.locator("#output")).toContainText("café")
  await page.reload()
  await expect(editor).toHaveText("export declare function café(): number;")
  await page
    .getByRole("button", { name: "Convert bindings", exact: true })
    .click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await page.getByRole("button", { name: "Rename host", exact: true }).click()
  const name = page.getByRole("textbox", { name: "New name", exact: true })
  await name.fill("vendor")
  await name.press("Enter")
  await page
    .getByRole("button", { name: "Convert bindings", exact: true })
    .click()
  await expect(page.locator("#status-text")).toHaveText(
    "Cannot convert bindings"
  )
  await expect(page.locator("#output")).toContainText("host/package.json")
  page.once("dialog", (dialog) => dialog.accept())
  await page.getByRole("button", { name: "Delete vendor", exact: true }).click()
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("ordinary source")
  expect(errors).toEqual([])
})

test("editing while WASM initializes discards the old conversion result", async ({
  page,
}) => {
  let release: () => void = () => {}
  const wait = new Promise<void>((resolve) => {
    release = resolve
  })
  await page.route(/seseragi_wasm_bg\.wasm/, async (route) => {
    await wait
    await route.continue()
  })
  await page.goto("/")
  await page
    .getByRole("button", { name: "Convert bindings", exact: true })
    .click()
  await expect(page.locator("#status-text")).toHaveText("Converting bindings…")
  await page
    .getByRole("textbox", { name: "Seseragi source editor" })
    .fill("export declare function stale(value: any): string;")
  release()
  await page
    .getByRole("button", { name: "Convert bindings", exact: true })
    .click()
  await expect(page.locator("#status-text")).toHaveText(
    "Cannot convert bindings"
  )
  await expect(page.locator("#output")).toContainText("any")
  await expect(page.locator("#output")).not.toContainText("Entry: api")
})

test("generated imports survive reload and regeneration replaces the previous module", async ({
  page,
}) => {
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  await page.goto("/")
  const convert = page.getByRole("button", {
    name: "Convert bindings",
    exact: true,
  })
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await page
    .locator('[data-explorer-path="main.ssrg"] .explorer-row-label')
    .click()
  const editor = page.getByRole("textbox", { name: "Seseragi source editor" })
  const source = (module: string) =>
    `import * as api from "gen/${module}"\npub fn identity config: api.Config -> api.Config = config\n\npub effect fn main -> Unit\nwith Console\nfails ConsoleError = println "generated import compiled"\n`
  await editor.fill(source("fixture-api"))
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("generated import compiled")
  await page.reload()
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("generated import compiled")
  await page
    .locator(
      '[data-explorer-path="seseragi.bindings.toml"] .explorer-row-label'
    )
    .click()
  const settings = await editor.innerText()
  await editor.fill(
    settings.replace('output = "fixture-api"', 'output = "renamed-api"')
  )
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toContainText("gen/fixture-api")
  await expect(page.locator("#output")).toContainText("missing module")
  await page
    .locator('[data-explorer-path="main.ssrg"] .explorer-row-label')
    .click()
  await editor.fill(source("renamed-api"))
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("generated import compiled")
  expect(errors).toEqual([])
})

test("Inspector compares readonly artifacts, reports and navigates Unicode diagnostics", async ({
  page,
}, testInfo) => {
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  await page.goto("/")
  const editor = page.getByRole("textbox", { name: "Seseragi source editor" })
  const convert = page.getByRole("button", {
    name: "Convert bindings",
    exact: true,
  })
  const open = page.getByRole("button", { name: "Interop", exact: true })
  const dialog = page.getByRole("dialog", { name: "Interop Inspector" })
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await open.click()
  await expect(dialog.locator("pre").nth(0)).toContainText(
    "export interface Config"
  )
  await expect(dialog.locator("pre").nth(1)).toContainText("Config")
  await expect(dialog).toContainText(".seseragi/generated/fixture-api.ssrg")
  await expect(
    dialog.locator('[contenteditable="true"], textarea')
  ).toHaveCount(0)
  await page.screenshot({ path: testInfo.outputPath("inspector-desktop.png") })
  await dialog.getByRole("tab", { name: "Bindings", exact: true }).focus()
  await page.keyboard.press("ArrowRight")
  await expect(
    dialog.getByRole("tab", { name: "Report", exact: true })
  ).toBeFocused()
  await expect(dialog.getByRole("tabpanel")).toContainText("Added (4)")
  await expect(dialog.getByRole("tabpanel")).toContainText("Config")
  await page.keyboard.press("Escape")
  await expect(open).toBeFocused()
  const source =
    "// 🙂 café\nexport declare function unsafe(value: any): string;"
  await editor.fill(source)
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Cannot convert bindings"
  )
  await open.click()
  await dialog.getByRole("tab", { name: /^Diagnostics/ }).click()
  await dialog.locator(".diagnostic-card-location").first().click()
  await expect(dialog).not.toBeVisible()
  await expect(page.locator("#active-file-name")).toHaveText("host/index.d.ts")
  expect(await page.evaluate(() => window.getSelection()?.toString())).toBe(
    "any"
  )
  // Already-dirty edits invalidate the snapshot; restoring identical text cannot revive it.
  await editor.fill(`${source}\n`)
  await editor.fill(source)
  await open.click()
  await expect(dialog).toContainText("Inputs changed")
  await expect(dialog.locator("pre")).toHaveCount(0)
  await page.keyboard.press("Escape")
  await editor.fill("export declare function fresh(): string;")
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await page.setViewportSize({ width: 390, height: 844 })
  await open.click()
  await dialog.getByRole("tab", { name: "Bindings", exact: true }).click()
  await expect(dialog.locator("pre").nth(1)).toContainText("fresh")
  expect(
    await dialog.evaluate((node) => node.scrollWidth <= node.clientWidth)
  ).toBe(true)
  await page.screenshot({ path: testInfo.outputPath("inspector-mobile.png") })
  await dialog.getByRole("button", { name: "Open source in editor" }).click()
  await expect(editor).toBeFocused()
  await page.reload()
  await open.click()
  await expect(dialog).toContainText("Convert bindings to compare")
  await expect(dialog.locator("pre")).toHaveCount(0)
  expect(errors).toEqual([])
})
