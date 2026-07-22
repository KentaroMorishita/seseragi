import path from "node:path"
import { strFromU8, unzipSync } from "fflate"

const supportedTargets = new Set([
  "darwin-arm64",
  "darwin-x64",
  "linux-x64",
  "win32-x64",
])

export async function verifyPackage(
  file: string,
  target: string
): Promise<void> {
  if (!supportedTargets.has(target)) {
    throw new Error(`unsupported VSIX target: ${target}`)
  }
  const archive = Bun.file(file)
  if (!(await archive.exists())) throw new Error(`VSIX does not exist: ${file}`)
  if (archive.size > 40 * 1024 * 1024) {
    throw new Error(
      `VSIX exceeds the 40 MiB package limit: ${archive.size} bytes`
    )
  }

  const entries = unzipSync(new Uint8Array(await archive.arrayBuffer()))
  const names = Object.keys(entries)
  const required = [
    "extension/package.json",
    "extension/dist/extension.js",
    "extension/LICENSE.txt",
    "extension/readme.md",
    "extension/changelog.md",
    "extension/syntaxes/seseragi.tmLanguage.json",
  ]
  for (const name of required) {
    if (!(name in entries)) throw new Error(`VSIX is missing ${name}`)
  }

  const executable =
    target === "win32-x64" ? "seseragi-lsp.exe" : "seseragi-lsp"
  const expectedServer = `extension/server/${target}/${executable}`
  const servers = names.filter((name) => name.startsWith("extension/server/"))
  if (servers.length !== 1 || servers[0] !== expectedServer) {
    throw new Error(
      `VSIX must contain only ${expectedServer}; found ${servers.join(", ") || "none"}`
    )
  }

  const manifest = JSON.parse(strFromU8(entries["extension/package.json"]))
  const language = manifest.contributes?.languages?.find(
    (entry: { id?: string }) => entry.id === "seseragi"
  )
  if (!language?.extensions?.includes(".ssrg")) {
    throw new Error("VSIX does not register every .ssrg file as Seseragi")
  }
  const commands = new Set(
    (manifest.contributes?.commands || []).map(
      (entry: { command?: string }) => entry.command
    )
  )
  for (const command of [
    "seseragi.restartLanguageServer",
    "seseragi.showLanguageServerOutput",
  ]) {
    if (!commands.has(command))
      throw new Error(`VSIX is missing command ${command}`)
  }
}

if (import.meta.main) {
  const packageRoot = path.resolve(import.meta.dir, "..")
  const target = process.argv[3] || process.env.SESERAGI_EXTENSION_TARGET
  if (!target) throw new Error("usage: verify-package.ts VSIX TARGET")
  const file = process.argv[2]
  if (!file) throw new Error("usage: verify-package.ts VSIX TARGET")
  await verifyPackage(path.resolve(packageRoot, file), target)
  console.log(`Verified ${file} for ${target}.`)
}
