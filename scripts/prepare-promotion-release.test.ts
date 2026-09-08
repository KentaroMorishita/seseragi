import { describe, expect, test } from "bun:test"
import {
  changelogEntry,
  nextPatchVersion,
  normalizeIssueTitle,
  prependChangelogEntry,
} from "./prepare-promotion-release"

describe("promotion release preparation", () => {
  test("increments only the patch component", () => {
    expect(nextPatchVersion("0.61.0")).toBe("0.61.1")
    expect(nextPatchVersion("2.3.9")).toBe("2.3.10")
    expect(() => nextPatchVersion("2.3.9-beta.1")).toThrow()
  })

  test("normalizes issue titles to one changelog line", () => {
    expect(normalizeIssueTitle(" fix  event\n snapshot ")).toBe(
      "fix event snapshot"
    )
  })

  test("creates deterministic changelog metadata without inventing prose", () => {
    expect(changelogEntry("0.62.0", "2026-09-06", 528, "Fix change event")).toBe(
      [
        "## [0.62.0] - 2026-09-06",
        "",
        "### Changed",
        "",
        "- #528 Fix change event",
        "",
      ].join("\n")
    )
  })

  test("prepends after the canonical changelog heading", () => {
    const current = "# Change Log\n\n## [0.61.0] - 2026-09-05\n\nold\n"
    const entry = changelogEntry("0.62.0", "2026-09-06", 528, "Fix")
    expect(prependChangelogEntry(current, entry, "0.62.0")).toBe(
      `# Change Log\n\n${entry}## [0.61.0] - 2026-09-05\n\nold\n`
    )
    expect(() =>
      prependChangelogEntry(
        `# Change Log\n\n${entry}`,
        entry,
        "0.62.0"
      )
    ).toThrow("already contains")
  })
})
