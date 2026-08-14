import { expect, test } from "bun:test"
import path from "node:path"
import { installedExtensionVersion } from "./local-dogfood"
import { repositoryRoot } from "./release-contract"

test("finds the official installed extension version exactly", () => {
  expect(
    installedExtensionVersion(
      "publisher.other@1.0.0\nseseragi-dev.seseragi@0.5.0\n"
    )
  ).toBe("0.5.0")
  expect(
    installedExtensionVersion(
      ["seseragi-dev.seseragi", "spec-preview@0.3.0\n"].join("-")
    )
  ).toBeNull()
})

test("installs and verifies only published canonical artifacts", async () => {
  const source = await Bun.file(
    path.join(repositoryRoot, "scripts", "local-dogfood.ts")
  ).text()

  expect(source).toContain(["releases/download/v", "$", "{version}"].join(""))
  expect(source).toContain("verifyNativeRelease")
  expect(source).toContain('"--install-extension"')
  expect(source).toContain('"--locate-extension"')
  expect(source).toContain("extension bundled LSP and installed LSP")
  expect(source).not.toContain('cargo", "build')
})
