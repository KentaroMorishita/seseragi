import { readFile } from "node:fs/promises"
import { expect, test } from "@playwright/test"

for (const name of ["domains", "relations", "structures", "callables"]) {
  test(`Inspector presents semantic corpus ${name} and compatibility evidence`, async ({
    page,
  }) => {
    const project = new URL(
      "../../../examples/spec/fixtures/projects/dts-basic-conversion/",
      import.meta.url
    )
    const files = await Promise.all(
      ["seseragi.toml", "seseragi.bindings.toml", "host/package.json"].map(
        async (path) => ({
          path,
          source: await readFile(new URL(path, project), "utf8"),
        })
      )
    )
    const source = await readFile(
      new URL(
        `../../../examples/spec/fixtures/semantic-interop-schema-1/${name}.d.ts`,
        import.meta.url
      ),
      "utf8"
    )
    files.push({ path: "host/index.d.ts", source })
    files.push({
      path: "main.ssrg",
      source:
        'pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  println "semantic corpus"\n',
    })
    const seed = {
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
    await page.addInitScript(
      (seed) =>
        localStorage.setItem(
          "seseragi.playground.workspace.schema-1",
          JSON.stringify(seed)
        ),
      seed
    )
    const errors: string[] = []
    page.on("pageerror", (error) => errors.push(error.message))
    await page.goto("/")
    await page
      .getByRole("button", { name: "Convert bindings", exact: true })
      .click()
    await expect(page.locator("#status-text")).not.toHaveText(
      "Converting bindings…"
    )
    await page.getByRole("button", { name: "Interop", exact: true }).click()
    await page.getByRole("tab", { name: "Semantic IR", exact: true }).click()
    await expect(
      page.getByRole("combobox", { name: "Semantic type" })
    ).toBeVisible()
    await page.getByRole("tab", { name: "Compatibility", exact: true }).click()
    await expect(
      page.getByRole("heading", { name: "Semantic compatibility · api" })
    ).toBeVisible()
    const summaries = page.locator("#interop-inspector summary")
    await expect(summaries).toHaveCount(7)
    await summaries
      .filter({ hasText: /: [1-9][0-9]*$/ })
      .first()
      .click()
    await expect(page.locator("#interop-inspector h4").first()).toBeVisible()
    await expect(page.locator("#interop-inspector li").first()).toBeVisible()
    expect(errors).toEqual([])
  })
}
