import { describe, expect, test } from "bun:test"
import manifest from "../package.json"
import {
  assertArchiveExecutable,
  readArchiveEntries,
  validateSmokeMetadata,
} from "../scripts/verify-package"

function archiveMetadataFixture(
  name: string,
  hostSystem: number,
  unixMode: number
): Uint8Array {
  const encodedName = new TextEncoder().encode(name)
  const centralDirectorySize = 46 + encodedName.length
  const bytes = new Uint8Array(centralDirectorySize + 22)
  const view = new DataView(bytes.buffer)
  view.setUint32(0, 0x02014b50, true)
  view.setUint16(4, (hostSystem << 8) | 20, true)
  view.setUint16(28, encodedName.length, true)
  view.setUint32(38, (unixMode << 16) >>> 0, true)
  bytes.set(encodedName, 46)

  const end = centralDirectorySize
  view.setUint32(end, 0x06054b50, true)
  view.setUint16(end + 8, 1, true)
  view.setUint16(end + 10, 1, true)
  view.setUint32(end + 12, centralDirectorySize, true)
  return bytes
}

describe("VSIX native LSP archive contract", () => {
  const server = "extension/server/darwin-arm64/seseragi-lsp"

  test("requires a Unix executable mode for non-Windows servers", () => {
    const entries = readArchiveEntries(
      archiveMetadataFixture(server, 3, 0o100755)
    )
    const entry = entries.get(server)

    expect(entry?.unixMode).toBe(0o100755)
    expect(() => assertArchiveExecutable(entry, "darwin-arm64")).not.toThrow()

    const nonExecutable = readArchiveEntries(
      archiveMetadataFixture(server, 3, 0o100644)
    ).get(server)
    expect(() =>
      assertArchiveExecutable(nonExecutable, "darwin-arm64")
    ).toThrow("mode 644")
  })

  test("requires the extracted binary to identify its packaged target", () => {
    expect(() =>
      validateSmokeMetadata(
        {
          name: "seseragi-lsp",
          target: "aarch64-apple-darwin",
          version: manifest.version,
        },
        manifest.version,
        "darwin-arm64"
      )
    ).not.toThrow()
    expect(() =>
      validateSmokeMetadata(
        {
          name: "seseragi-lsp",
          target: "x86_64-apple-darwin",
          version: manifest.version,
        },
        manifest.version,
        "darwin-arm64"
      )
    ).toThrow("expected aarch64-apple-darwin")
  })
})
