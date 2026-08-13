import { describe, expect, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { join, resolve } from "node:path"

const root = resolve(import.meta.dir, "..")

describe("project fixture inventory", () => {
  test("keeps roles, product evidence and workspace policy current", async () => {
    const result = spawnSync("bun", ["scripts/check-project-fixtures.ts"], {
      cwd: root,
      encoding: "utf8",
    })

    expect(result.status).toBe(0)
    expect(result.stderr).toBe("")
    expect(result.stdout).toContain("49 project fixture roles")
    expect(result.stdout).toContain("8 current, 41 contract-only")

    const settings = await Bun.file(join(root, ".vscode/settings.json")).json()
    const [contractOnlyPattern] = Object.keys(settings["files.associations"])
    expect(contractOnlyPattern).toContain("dom-hydration-mismatch")
    expect(contractOnlyPattern).not.toContain("entry-rooted-runtime")
    expect(settings["files.associations"][contractOnlyPattern]).toBe(
      "plaintext"
    )
    expect(settings["files.watcherExclude"][contractOnlyPattern]).toBe(true)
  })
})
