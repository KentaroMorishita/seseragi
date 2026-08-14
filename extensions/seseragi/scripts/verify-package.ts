import {
  chmodSync,
  mkdtempSync,
  mkdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import path from "node:path"
import { strFromU8, unzipSync } from "fflate"

const { SERVER_TARGET_TRIPLES, platformTarget } = require("../extension-core")

export const targetContracts = {
  "darwin-arm64": {
    binary: "seseragi-lsp",
    rustTarget: SERVER_TARGET_TRIPLES["darwin-arm64"],
  },
  "darwin-x64": {
    binary: "seseragi-lsp",
    rustTarget: SERVER_TARGET_TRIPLES["darwin-x64"],
  },
  "linux-x64": {
    binary: "seseragi-lsp",
    rustTarget: SERVER_TARGET_TRIPLES["linux-x64"],
  },
  "win32-x64": {
    binary: "seseragi-lsp.exe",
    rustTarget: SERVER_TARGET_TRIPLES["win32-x64"],
  },
} as const

type ArchiveEntry = {
  hostSystem: number
  name: string
  unixMode?: number
}

const CENTRAL_DIRECTORY_SIGNATURE = 0x02014b50
const END_OF_CENTRAL_DIRECTORY_SIGNATURE = 0x06054b50
const UNIX_HOST_SYSTEM = 3
const EXECUTABLE_MODE = 0o755
const REGULAR_FILE_MODE = 0o100000

function targetContract(target: string) {
  const contract = targetContracts[target as keyof typeof targetContracts]
  if (contract === undefined)
    throw new Error(`unsupported VSIX target: ${target}`)
  return contract
}

function readUint16(bytes: Uint8Array, offset: number): number {
  return bytes[offset] | (bytes[offset + 1] << 8)
}

function readUint32(bytes: Uint8Array, offset: number): number {
  return (
    (bytes[offset] |
      (bytes[offset + 1] << 8) |
      (bytes[offset + 2] << 16) |
      (bytes[offset + 3] << 24)) >>>
    0
  )
}

function findEndOfCentralDirectory(bytes: Uint8Array): number {
  const minimumOffset = Math.max(0, bytes.length - 0xffff - 22)
  for (let offset = bytes.length - 22; offset >= minimumOffset; offset -= 1) {
    if (readUint32(bytes, offset) === END_OF_CENTRAL_DIRECTORY_SIGNATURE) {
      return offset
    }
  }
  throw new Error("VSIX ZIP central directory is missing")
}

export function readArchiveEntries(
  data: Uint8Array
): Map<string, ArchiveEntry> {
  const end = findEndOfCentralDirectory(data)
  const count = readUint16(data, end + 10)
  const size = readUint32(data, end + 12)
  let offset = end - size
  if (offset < 0) throw new Error("VSIX ZIP central directory is invalid")

  const entries = new Map<string, ArchiveEntry>()
  for (let index = 0; index < count; index += 1) {
    if (readUint32(data, offset) !== CENTRAL_DIRECTORY_SIGNATURE) {
      throw new Error("VSIX ZIP central directory entry is invalid")
    }
    const versionMadeBy = readUint16(data, offset + 4)
    const nameLength = readUint16(data, offset + 28)
    const extraLength = readUint16(data, offset + 30)
    const commentLength = readUint16(data, offset + 32)
    const nameStart = offset + 46
    const nameEnd = nameStart + nameLength
    if (nameEnd > data.length) {
      throw new Error("VSIX ZIP central directory filename is invalid")
    }
    const hostSystem = versionMadeBy >>> 8
    const attributes = readUint32(data, offset + 38)
    entries.set(strFromU8(data.subarray(nameStart, nameEnd)), {
      hostSystem,
      name: strFromU8(data.subarray(nameStart, nameEnd)),
      unixMode: hostSystem === UNIX_HOST_SYSTEM ? attributes >>> 16 : undefined,
    })
    offset = nameEnd + extraLength + commentLength
  }
  return entries
}

export function assertArchiveExecutable(
  entry: ArchiveEntry | undefined,
  target: string
): void {
  if (entry === undefined) {
    throw new Error(`VSIX archive is missing native LSP metadata for ${target}`)
  }
  if (target.startsWith("win32-")) return
  if (entry.hostSystem !== UNIX_HOST_SYSTEM || entry.unixMode === undefined) {
    throw new Error(
      `VSIX archive does not preserve Unix file mode for ${entry.name}`
    )
  }
  if ((entry.unixMode & 0o777) !== EXECUTABLE_MODE) {
    throw new Error(
      `VSIX archive stores ${entry.name} with mode ${(entry.unixMode & 0o777).toString(8)}; expected 755`
    )
  }
  if ((entry.unixMode & 0o170000) !== REGULAR_FILE_MODE) {
    throw new Error(`VSIX archive stores ${entry.name} as a non-file entry`)
  }
}

function archivePath(root: string, name: string): string {
  const output = path.resolve(root, name)
  if (!output.startsWith(`${root}${path.sep}`)) {
    throw new Error(`VSIX contains an unsafe extraction path: ${name}`)
  }
  return output
}

function extractArchive(
  entries: Record<string, Uint8Array>,
  attributes: Map<string, ArchiveEntry>,
  root: string
): void {
  for (const [name, contents] of Object.entries(entries)) {
    const output = archivePath(root, name)
    mkdirSync(path.dirname(output), { recursive: true })
    writeFileSync(output, contents)
    const mode = attributes.get(name)?.unixMode
    if (process.platform !== "win32" && mode !== undefined) {
      chmodSync(output, mode & 0o777)
    }
  }
}

function outputText(output: Uint8Array | undefined): string {
  return output === undefined ? "" : new TextDecoder().decode(output).trim()
}

export function validateSmokeMetadata(
  metadata: {
    analysisSchemaVersion?: unknown
    name?: unknown
    protocolVersion?: unknown
    target?: unknown
    version?: unknown
  },
  extensionVersion: string,
  target: string
): void {
  const contract = targetContract(target)
  if (metadata.name !== "seseragi-lsp") {
    throw new Error("--version-json did not identify seseragi-lsp")
  }
  if (metadata.version !== extensionVersion) {
    throw new Error(
      `--version-json reported version ${String(metadata.version)}; expected ${extensionVersion}`
    )
  }
  if (metadata.target !== contract.rustTarget) {
    throw new Error(
      `--version-json reported target ${String(metadata.target)}; expected ${contract.rustTarget}`
    )
  }
  if (metadata.protocolVersion !== 1 || metadata.analysisSchemaVersion !== 1) {
    throw new Error(
      `--version-json reported protocol ${String(metadata.protocolVersion)} and analysis schema ${String(metadata.analysisSchemaVersion)}; expected 1/1`
    )
  }
}

async function smokeExtractedPackage({
  archiveEntries,
  archiveFile,
  entries,
  expectedServer,
  extensionVersion,
  target,
}: {
  archiveEntries: Map<string, ArchiveEntry>
  archiveFile: string
  entries: Record<string, Uint8Array>
  expectedServer: string
  extensionVersion: string
  target: string
}): Promise<void> {
  const extractedRoot = mkdtempSync(path.join(tmpdir(), "seseragi-vsix-"))
  const extractedBinary = archivePath(extractedRoot, expectedServer)
  try {
    extractArchive(entries, archiveEntries, extractedRoot)
    if (!target.startsWith("win32-")) {
      const mode = statSync(extractedBinary).mode & 0o777
      if (mode !== EXECUTABLE_MODE) {
        throw new Error(
          `extraction produced mode ${mode.toString(8)}; expected 755`
        )
      }
    }

    const hostTarget = platformTarget(process.platform, process.arch)
    if (hostTarget !== target) {
      console.log(
        `Verified archive and extraction for ${target}; execution smoke runs on its ${target} CI runner.`
      )
      return
    }

    const result = Bun.spawnSync([extractedBinary, "--version-json"], {
      stderr: "pipe",
      stdout: "pipe",
    }) as {
      error?: Error
      exitCode?: number
      stderr?: Uint8Array
      stdout?: Uint8Array
      success: boolean
    }
    if (!result.success) {
      throw new Error(
        outputText(result.stderr) ||
          result.error?.message ||
          `exit code ${result.exitCode ?? "unknown"}`
      )
    }
    let metadata: { name?: unknown; target?: unknown; version?: unknown }
    try {
      metadata = JSON.parse(outputText(result.stdout))
    } catch (error) {
      throw new Error(`invalid --version-json: ${error.message || error}`)
    }
    validateSmokeMetadata(metadata, extensionVersion, target)
    console.log(
      `Execution smoke passed for ${target}: ${extractedBinary} --version-json`
    )
  } catch (error) {
    throw new Error(
      `VSIX smoke test failed for ${archiveFile}; extracted binary ${extractedBinary}: ${error.message || error}`
    )
  } finally {
    rmSync(extractedRoot, { recursive: true, force: true })
  }
}

export async function verifyPackage(
  file: string,
  target: string
): Promise<void> {
  const contract = targetContract(target)
  const archive = Bun.file(file)
  if (!(await archive.exists())) throw new Error(`VSIX does not exist: ${file}`)
  if (archive.size > 40 * 1024 * 1024) {
    throw new Error(
      `VSIX exceeds the 40 MiB package limit: ${archive.size} bytes`
    )
  }

  const bytes = new Uint8Array(await archive.arrayBuffer())
  const entries = unzipSync(bytes)
  const archiveEntries = readArchiveEntries(bytes)
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

  const expectedServer = `extension/server/${target}/${contract.binary}`
  const servers = names.filter((name) => name.startsWith("extension/server/"))
  if (servers.length !== 1 || servers[0] !== expectedServer) {
    throw new Error(
      `VSIX must contain only ${expectedServer}; found ${servers.join(", ") || "none"}`
    )
  }
  assertArchiveExecutable(archiveEntries.get(expectedServer), target)

  const manifest = JSON.parse(strFromU8(entries["extension/package.json"]))
  if (`${manifest.publisher}.${manifest.name}` !== "seseragi-dev.seseragi") {
    throw new Error("VSIX does not have the official Seseragi extension ID")
  }
  const language = manifest.contributes?.languages?.find(
    (entry: { id?: string }) => entry.id === "seseragi"
  )
  if (!language?.extensions?.includes(".ssrg")) {
    throw new Error("VSIX does not register every .ssrg file as Seseragi")
  }
  const formatterDefaults =
    manifest.contributes?.configurationDefaults?.["[seseragi]"]
  if (
    formatterDefaults?.["editor.defaultFormatter"] !== "seseragi-dev.seseragi"
  ) {
    throw new Error("VSIX does not select the Seseragi default formatter")
  }
  const commands = new Set(
    (manifest.contributes?.commands || []).map(
      (entry: { command?: string }) => entry.command
    )
  )
  for (const command of [
    "seseragi.restartLanguageServer",
    "seseragi.showLanguageServerOutput",
    "seseragi.runProject",
    "seseragi.buildWebApp",
    "seseragi.startDevelopmentServer",
    "seseragi.stopDevelopmentServer",
    "seseragi.openInBrowser",
  ]) {
    if (!commands.has(command)) {
      throw new Error(`VSIX is missing command ${command}`)
    }
  }
  if (
    manifest.contributes?.configuration?.properties?.["seseragi.cli.path"]
      ?.default !== ""
  ) {
    throw new Error("VSIX does not expose optional Seseragi CLI discovery")
  }

  await smokeExtractedPackage({
    archiveEntries,
    archiveFile: file,
    entries,
    expectedServer,
    extensionVersion: manifest.version,
    target,
  })
}

if (import.meta.main) {
  const packageRoot = path.resolve(import.meta.dir, "..")
  const target = process.argv[3] || process.env.SESERAGI_EXTENSION_TARGET
  if (!target) throw new Error("usage: verify-package.ts VSIX TARGET")
  const file = process.argv[2]
  if (!file) throw new Error("usage: verify-package.ts VSIX TARGET")
  await verifyPackage(path.resolve(packageRoot, file), target)
  console.log(
    `Verified ${file} for ${target}, including archive/extraction smoke.`
  )
}
