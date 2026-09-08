import { expect, test } from "bun:test"
import { readFileSync } from "node:fs"
import { assertGlibcVersions } from "./linux-native-abi"
import contract from "./linux-native-contract.json"

test("compares required GLIBC versions numerically and fails closed", () => {
  expect(
    assertGlibcVersions(
      "Name: GLIBC_2.2.5 Flags: none\nName: GLIBC_2.34 Flags: none\nName: GCC_3.0"
    )
  ).toEqual(["GLIBC_2.2.5", "GLIBC_2.34"])
  for (const version of [
    "GLIBC_2.35",
    "GLIBC_2.39",
    "GLIBC_2.100",
    "GLIBC_3.0",
    "GLIBC_2.34.1",
    "GLIBC_PRIVATE",
    "GLIBC_ABI_DT_RELR",
  ]) {
    expect(() => assertGlibcVersions(`Name: ${version} Flags: none`)).toThrow()
  }
  expect(() => assertGlibcVersions("no requirements")).toThrow()
  expect(contract.image).toMatch(/@sha256:[a-f0-9]{64}$/u)
})

test("release cannot bypass downloaded baseline smoke or rebuild a separate Linux VSIX LSP", () => {
  const release = readFileSync(
    new URL("../.github/workflows/release.yml", import.meta.url),
    "utf8"
  )
  const linux = readFileSync(
    new URL("../.github/workflows/linux-native.yml", import.meta.url),
    "utf8"
  )
  expect(release).toContain("local-web-product-e2e, linux-native]")
  expect(release).not.toContain("target: linux-x64")
  expect(linux).toContain("actions/download-artifact@v4")
  expect(linux).toContain('bash scripts/linux-native.sh verify "${args[@]}"')
  expect(linux).toContain('bun scripts/linux-native-smoke.ts "${args[@]}"')
  expect(linux).toContain(
    'SESERAGI_LSP_BINARY="$PWD/target/linux-native/x86_64-unknown-linux-gnu/release/seseragi-lsp"'
  )
})
