import path from "node:path"
import { strFromU8, unzipSync } from "fflate"

export const LEGACY_EXTENSION_ID = "seseragi-dev.seseragi-spec-preview"

export async function verifyLegacyPackage(file: string): Promise<void> {
  const archive = Bun.file(file)
  if (!(await archive.exists())) {
    throw new Error(`legacy migration VSIX does not exist: ${file}`)
  }

  const entries = unzipSync(new Uint8Array(await archive.arrayBuffer()))
  const names = Object.keys(entries)
  for (const name of [
    "extension/package.json",
    "extension/extension.js",
    "extension/LICENSE.txt",
    "extension/readme.md",
    "extension/changelog.md",
  ]) {
    if (!(name in entries)) {
      throw new Error(`legacy migration VSIX is missing ${name}`)
    }
  }
  if (names.some((name) => name.startsWith("extension/server/"))) {
    throw new Error("legacy migration VSIX must not bundle a language server")
  }

  const manifest = JSON.parse(strFromU8(entries["extension/package.json"]))
  if (`${manifest.publisher}.${manifest.name}` !== LEGACY_EXTENSION_ID) {
    throw new Error("legacy migration VSIX has the wrong extension ID")
  }
  if (manifest.main !== "./extension.js") {
    throw new Error(
      "legacy migration VSIX is missing its migration entry point"
    )
  }
  const source = strFromU8(entries["extension/extension.js"])
  if (source.includes("seseragi-lsp")) {
    throw new Error("legacy migration VSIX must not start seseragi-lsp")
  }
}

if (import.meta.main) {
  const file = process.argv[2]
  if (!file) {
    throw new Error("usage: verify-legacy-package.ts LEGACY_VSIX")
  }
  await verifyLegacyPackage(path.resolve(file))
  console.log(`Verified legacy migration package ${file}.`)
}
