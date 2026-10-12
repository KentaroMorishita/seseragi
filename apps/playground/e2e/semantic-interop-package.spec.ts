import { readFile } from "node:fs/promises"
import { expect, test } from "@playwright/test"

async function realPackageWorkspace() {
  const root = new URL(
    "../tests/fixtures/interop-base64-js-1.5.1/",
    import.meta.url
  )
  const mapping = JSON.parse(
    await readFile(new URL("workspace-files.json", root), "utf8")
  ) as { path: string; fixture: string }[]
  const files = await Promise.all(
    mapping.map(async ({ path, fixture }) => ({
      path,
      source: await readFile(new URL(fixture, root), "utf8"),
    }))
  )
  const manifest = files.find(({ path }) => path === "seseragi.toml")!.source
  return {
    schema: 1,
    sampleId: "playground-blank",
    sampleHash: "workspace:blank-v1",
    stdin: "",
    workspace: {
      files,
      folders: [],
      dirtyFiles: [],
      entryFile: "main.ssrg",
      activeFile: "host/node_modules/base64-js/index.d.ts",
      packageManifest: manifest,
      packageEntryFile: "main.ssrg",
      openFiles: ["main.ssrg", "host/node_modules/base64-js/index.d.ts"],
      expandedFolders: [
        "host",
        "host/node_modules",
        "host/node_modules/base64-js",
      ],
      explorer: { visible: true, width: 280 },
    },
  }
}

test("#487 real npm declarations convert through Inspector and generated imports into browser execution", async ({
  page,
}) => {
  const seed = await realPackageWorkspace()
  await page.addInitScript((seed) => {
    const key = "seseragi.playground.workspace.schema-1"
    if (localStorage.getItem(key) === null)
      localStorage.setItem(key, JSON.stringify(seed))
  }, seed)
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
  await page.getByRole("button", { name: "Interop", exact: true }).click()
  const inspector = page.getByRole("dialog", { name: "Interop Inspector" })
  await expect(inspector.locator("pre").nth(0)).toContainText(
    "export function toByteArray"
  )
  await expect(inspector.locator("pre").nth(1)).toContainText(
    'import { Bytes } from "std/bytes"'
  )
  await expect(inspector.locator("pre").nth(1)).toContainText(
    "task fn toByteArray"
  )
  await expect(
    inspector.locator('[contenteditable="true"], textarea')
  ).toHaveCount(0)
  await inspector.getByRole("tab", { name: "Report", exact: true }).click()
  await expect(inspector.getByRole("tabpanel")).toContainText("Added (3)")
  await inspector.getByRole("tab", { name: "Semantic IR", exact: true }).click()
  await expect(inspector.getByRole("tabpanel")).toContainText(
    "Source provenance"
  )
  await expect(inspector.getByRole("tabpanel")).toContainText("callable")
  await expect(inspector.getByRole("tabpanel")).toContainText(
    "host/node_modules/base64-js/index.d.ts"
  )
  await page.keyboard.press("Escape")
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("SGVsbG8=:5.0")
  const published = await page.evaluate(() => {
    const saved = JSON.parse(
      localStorage.getItem("seseragi.playground.workspace.schema-1")!
    )
    return saved.workspace.generatedBindings
  })
  const metadata = JSON.parse(
    published.artifacts.find((file: { path: string }) =>
      file.path.endsWith("binding.json")
    ).source
  )
  expect(Object.keys(metadata.runtimeLinkage.contract.exports)).toHaveLength(3)
  const artifacts = Object.values(metadata.runtimeLinkage.contract.exports) as {
    targets: { browser: { exactIdentity: string; contentDigest: string } }
  }[]
  expect(artifacts[0]!.targets.browser.exactIdentity).toBe(
    "npm:base64-js@1.5.1/index.js"
  )
  expect(artifacts[0]!.targets.browser.contentDigest).toMatch(
    /^sha256:[a-f0-9]{64}$/
  )
  await page.reload()
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("SGVsbG8=:5.0")
  expect(
    await page.evaluate(() => {
      const saved = JSON.parse(
        localStorage.getItem("seseragi.playground.workspace.schema-1")!
      )
      return saved.workspace.generatedBindings
    })
  ).toEqual(published)

  // Persisted raw artifacts retain their source proof across reloads.
  await page.evaluate(() => {
    const key = "seseragi.playground.workspace.schema-1"
    const saved = JSON.parse(localStorage.getItem(key)!)
    saved.workspace.generatedBindings.files[0].source += "\n"
    localStorage.setItem(key, JSON.stringify(saved))
  })
  await page.reload()
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toContainText("SES-F0201")
  await expect(page.locator("#output")).toContainText(
    "generated binding source"
  )
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("SGVsbG8=:5.0")

  // Changed inputs revoke the Inspector snapshot and generated binding until
  // the actual declaration is converted again; undo cannot revive old evidence.
  const declaration = seed.workspace.files.find(({ path }) =>
    path.endsWith("index.d.ts")
  )!.source
  await page
    .locator(
      '[data-explorer-path="host/node_modules/base64-js/index.d.ts"] .explorer-row-label'
    )
    .click()
  await page
    .locator(".cm-content")
    .fill(`${declaration}\nexport function unsupported(value: any): string;\n`)
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Cannot convert bindings"
  )
  await expect(page.locator("#output")).toContainText("any")
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).not.toHaveText("SGVsbG8=:5.0")
  await page.locator(".cm-content").fill(declaration)
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toHaveText("SGVsbG8=:5.0")

  // Semantics still convert when package exports only select Node. Browser
  // execution must produce the runtime target diagnostic before loading JS.
  const packageFile = seed.workspace.files.find(
    ({ path }) => path === "host/node_modules/base64-js/package.json"
  )!
  const nodeOnly = {
    ...JSON.parse(packageFile.source),
    exports: { node: "./index.js" },
  }
  await page
    .locator(
      '[data-explorer-path="host/node_modules/base64-js/package.json"] .explorer-row-label'
    )
    .click()
  await page.locator(".cm-content").fill(JSON.stringify(nodeOnly))
  await convert.click()
  await expect(page.locator("#status-text")).toHaveText(
    "Converted 1 binding(s)"
  )
  await page.locator("#run-button").click()
  await expect(page.locator("#output")).toContainText("SES-F0203")
  await expect(page.locator("#output")).toContainText("browser")
  expect(errors).toEqual([])
})
