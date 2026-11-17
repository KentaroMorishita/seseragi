import { describe, expect, test } from "bun:test"
import { mkdtempSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import {
  archiveName,
  formulaTargets,
  readChecksums,
  renderFormula,
} from "./homebrew-formula"

const checksum = (digit: string) => digit.repeat(64)

function withChecksums(
  version: string,
  contents: Partial<Record<string, string>>,
  check: (directory: string) => void
) {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-formula-"))
  try {
    for (const target of formulaTargets) {
      const content = contents[target]
      if (content !== undefined) {
        writeFileSync(
          join(directory, `${archiveName(version, target)}.sha256`),
          content
        )
      }
    }
    check(directory)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
}

describe("Homebrew formula", () => {
  test("takes each archive's checksum from its .sha256 file", () => {
    withChecksums(
      "0.61.178",
      {
        "darwin-arm64": `${checksum("a")}  seseragi-v0.61.178-darwin-arm64.tar.gz\n`,
        "darwin-x64": `${checksum("b")}\n`,
        "linux-x64": checksum("c"),
      },
      (directory) => {
        expect(readChecksums("0.61.178", directory)).toEqual({
          "darwin-arm64": checksum("a"),
          "darwin-x64": checksum("b"),
          "linux-x64": checksum("c"),
        })
      }
    )
  })

  test("refuses a missing or malformed checksum", () => {
    withChecksums(
      "0.61.178",
      { "darwin-arm64": checksum("a"), "darwin-x64": "not-a-checksum" },
      (directory) => {
        expect(() => readChecksums("0.61.178", directory)).toThrow(
          "does not hold a SHA-256 checksum"
        )
      }
    )
    withChecksums(
      "0.61.178",
      { "darwin-arm64": checksum("a"), "darwin-x64": checksum("b") },
      (directory) => {
        expect(() => readChecksums("0.61.178", directory)).toThrow()
      }
    )
  })

  test("pins every archive to the release and its checksum", () => {
    const formula = renderFormula("0.61.178", {
      "darwin-arm64": checksum("a"),
      "darwin-x64": checksum("b"),
      "linux-x64": checksum("c"),
    })
    expect(formula).toContain('version "0.61.178"')
    for (const [target, digit] of [
      ["darwin-arm64", "a"],
      ["darwin-x64", "b"],
      ["linux-x64", "c"],
    ] as const) {
      const url = `https://github.com/KentaroMorishita/seseragi/releases/download/v0.61.178/seseragi-v0.61.178-${target}.tar.gz`
      expect(formula).toContain(
        `url "${url}"\n      sha256 "${checksum(digit)}"`
      )
    }
    expect(formula).toContain('bin.install "seseragi", "seseragi-lsp"')
    expect(formula).toContain("Bun 1.4.4 or later")
    expect(formula).toContain('"seseragi #{version} "')
  })

  test("refuses a version that is not plain numbers", () => {
    const checksums = {
      "darwin-arm64": checksum("a"),
      "darwin-x64": checksum("b"),
      "linux-x64": checksum("c"),
    }
    expect(() => renderFormula('1.0"; system "x', checksums)).toThrow(
      "invalid version"
    )
  })
})
