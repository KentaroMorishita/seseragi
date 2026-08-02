import { describe, expect, test } from "bun:test"
import {
  releaseNotes,
  releaseTag,
  replaceManifestVersion,
  replaceWorkspaceVersion,
  workspaceVersion,
} from "./release-contract"

describe("release contract", () => {
  test("uses the workspace package version as the canonical value", () => {
    const source = "[workspace.package]\nversion = \"0.4.0\"\nedition = \"2021\"\n"
    expect(workspaceVersion(source)).toBe("0.4.0")
    expect(releaseTag("0.4.0")).toBe("v0.4.0")
    expect(replaceWorkspaceVersion(source, "0.4.1")).toContain(
      "version = \"0.4.1\""
    )
  })

  test("updates only the derived JavaScript package version", () => {
    const source = '{\n  "name": "@seseragi/runtime",\n  "version": "0.0.0",\n  "private": true\n}\n'
    expect(replaceManifestVersion(source, "0.4.0")).toBe(
      '{\n  "name": "@seseragi/runtime",\n  "version": "0.4.0",\n  "private": true\n}\n'
    )
  })

  test("extracts the current release notes without copying older entries", () => {
    const changelog =
      "# Change Log\n\n## [0.4.0] - 2026-08-02\n\n- Unified release contract.\n\n## [0.3.0]\n\n- Older entry.\n"
    expect(releaseNotes(changelog, "0.4.0")).toBe(
      "# Seseragi v0.4.0\n\n- Unified release contract.\n"
    )
  })
})
