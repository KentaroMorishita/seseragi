import { access, chmod, copyFile, mkdir, mkdtemp, rm } from "node:fs/promises"
import { homedir, tmpdir } from "node:os"
import path from "node:path"
import {
  hostNativeReleaseTarget,
  nativeArchiveName,
  nativeChecksumName,
  nativeReleaseTargets,
  verifyNativeRelease,
} from "./native-release"
import { readReleaseContract, repositoryRoot } from "./release-contract"

const EXTENSION_ID = "seseragi-dev.seseragi"

function fail(message: string): never {
  throw new Error(`local dogfood: ${message}`)
}

function run(command: string[], options: { cwd?: string } = {}): string {
  const result = Bun.spawnSync(command, {
    cwd: options.cwd ?? repositoryRoot,
    stdout: "pipe",
    stderr: "pipe",
  })
  if (!result.success) {
    fail(
      `${command.join(" ")} failed: ${result.stderr.toString().trim() || `exit ${result.exitCode}`}`
    )
  }
  return result.stdout.toString().trim()
}

function commandPath(command: string): string {
  const executable = Bun.which(command)
  if (!executable) fail(`required command is unavailable: ${command}`)
  return executable
}

function parseJson(command: string[], label: string): Record<string, unknown> {
  try {
    return JSON.parse(run(command)) as Record<string, unknown>
  } catch (error) {
    fail(`${label} did not return valid version JSON: ${String(error)}`)
  }
}

function assertTool(
  metadata: Record<string, unknown>,
  name: string,
  version: string,
  target: string
): void {
  if (
    metadata.name !== name ||
    metadata.version !== version ||
    metadata.target !== target ||
    metadata.channel !== "release" ||
    metadata.dirty !== false ||
    metadata.releaseTag !== `v${version}`
  ) {
    fail(`${name} is not the installed v${version} release for ${target}`)
  }
}

export function installedExtensionVersion(output: string): string | null {
  const prefix = `${EXTENSION_ID}@`
  const entry = output.split(/\r?\n/u).find((line) => line.startsWith(prefix))
  return entry?.slice(prefix.length) ?? null
}

async function checkInstalled(version: string): Promise<void> {
  const target = hostNativeReleaseTarget()
  const rustTarget = nativeReleaseTargets[target].rustTarget
  const code = commandPath("code")
  const cli = commandPath("seseragi")
  const lsp = commandPath("seseragi-lsp")
  const cliMetadata = parseJson([cli, "--version-json"], "seseragi")
  const lspMetadata = parseJson([lsp, "--version-json"], "seseragi-lsp")
  assertTool(cliMetadata, "seseragi", version, rustTarget)
  assertTool(lspMetadata, "seseragi-lsp", version, rustTarget)

  const extensionVersion = installedExtensionVersion(
    run([code, "--list-extensions", "--show-versions"])
  )
  if (extensionVersion !== version) {
    fail(
      `${EXTENSION_ID} is ${extensionVersion ?? "not installed"}; expected ${version}`
    )
  }
  const extensionRoot = run([code, "--locate-extension", EXTENSION_ID])
  if (!extensionRoot) fail(`${EXTENSION_ID} install path is unavailable`)
  const manifest = (await Bun.file(
    path.join(extensionRoot, "package.json")
  ).json()) as {
    version?: unknown
  }
  if (manifest.version !== version) {
    fail(
      `installed extension manifest is ${String(manifest.version)}; expected ${version}`
    )
  }
  const lspName = target.startsWith("win32-")
    ? "seseragi-lsp.exe"
    : "seseragi-lsp"
  const bundledLsp = path.join(extensionRoot, "server", target, lspName)
  const bundledMetadata = parseJson(
    [bundledLsp, "--version-json"],
    "extension bundled seseragi-lsp"
  )
  assertTool(bundledMetadata, "seseragi-lsp", version, rustTarget)
  if (
    bundledMetadata.protocolVersion !== lspMetadata.protocolVersion ||
    bundledMetadata.analysisSchemaVersion !== lspMetadata.analysisSchemaVersion
  ) {
    fail("extension bundled LSP and installed LSP handshake metadata differ")
  }

  console.log(
    `Local dogfood is synchronized: CLI, LSP, and ${EXTENSION_ID} ${version} (${target}).`
  )
}

async function download(url: string, output: string): Promise<void> {
  const response = await fetch(url, { redirect: "follow" })
  if (!response.ok) fail(`download ${url} failed with HTTP ${response.status}`)
  await Bun.write(output, response)
}

async function sync(version: string): Promise<void> {
  const target = hostNativeReleaseTarget()
  const contract = nativeReleaseTargets[target]
  if (contract.archiveExtension === "zip") {
    fail("Windows local dogfood install is not implemented")
  }
  const code = commandPath("code")
  const currentCli = Bun.which("seseragi")
  const cargoBin = path.join(homedir(), ".cargo", "bin")
  const cargoBinExists = await access(cargoBin).then(
    () => true,
    () => false
  )
  const installDirectory = currentCli
    ? path.dirname(currentCli)
    : cargoBinExists
      ? cargoBin
      : path.join(homedir(), ".local", "bin")
  const temporary = await mkdtemp(path.join(tmpdir(), "seseragi-dogfood-"))
  const archiveName = nativeArchiveName(version, target)
  const checksumName = nativeChecksumName(version, target)
  const vsixName = `seseragi-v${version}-vscode-${target}.vsix`
  const base = `https://github.com/KentaroMorishita/seseragi/releases/download/v${version}`

  try {
    const archive = path.join(temporary, archiveName)
    const checksum = path.join(temporary, checksumName)
    const vsix = path.join(temporary, vsixName)
    await Promise.all([
      download(`${base}/${archiveName}`, archive),
      download(`${base}/${checksumName}`, checksum),
      download(`${base}/${vsixName}`, vsix),
    ])
    run(["bun", "install", "--frozen-lockfile"], {
      cwd: path.join(repositoryRoot, "extensions", "seseragi"),
    })
    await verifyNativeRelease({
      archive,
      output: temporary,
      release: true,
      target,
      version,
    })
    run(["bun", "scripts/verify-package.ts", vsix, target], {
      cwd: path.join(repositoryRoot, "extensions", "seseragi"),
    })

    const extracted = path.join(temporary, "native")
    await mkdir(extracted)
    await mkdir(installDirectory, { recursive: true })
    run(["tar", "-xzf", archive, "-C", extracted])
    for (const binary of ["seseragi", "seseragi-lsp"]) {
      const destination = path.join(installDirectory, binary)
      try {
        await copyFile(path.join(extracted, binary), destination)
        await chmod(destination, 0o755)
      } catch (error) {
        fail(`cannot install ${binary} to ${destination}: ${String(error)}`)
      }
    }
    run([code, "--install-extension", vsix, "--force"])
    await checkInstalled(version)
  } finally {
    await rm(temporary, { recursive: true, force: true })
  }
}

async function main(): Promise<void> {
  const command = process.argv[2]
  const { version } = await readReleaseContract()
  if (command === "check") return checkInstalled(version)
  if (command === "sync") return sync(version)
  fail("usage: local-dogfood.ts <sync|check>")
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  })
}
