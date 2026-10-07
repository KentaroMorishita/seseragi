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
