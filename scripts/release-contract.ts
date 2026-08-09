import { mkdir, readFile, writeFile } from "node:fs/promises"
import path from "node:path"

export const repositoryRoot = path.resolve(import.meta.dir, "..")

const PACKAGE_MANIFESTS = [
  "package.json",
  "runtime/ts/package.json",
  "apps/playground/package.json",
  "extensions/seseragi/package.json",
  "extensions/seseragi-legacy/package.json",
]
const RUST_MANIFESTS = [
  "crates/seseragi-cli/Cargo.toml",
  "crates/seseragi-conformance/Cargo.toml",
  "crates/seseragi-driver/Cargo.toml",
  "crates/seseragi-formatter/Cargo.toml",
  "crates/seseragi-lowering/Cargo.toml",
  "crates/seseragi-lsp/Cargo.toml",
  "crates/seseragi-project/Cargo.toml",
  "crates/seseragi-release/Cargo.toml",
  "crates/seseragi-runtime/Cargo.toml",
  "crates/seseragi-semantics/Cargo.toml",
  "crates/seseragi-source/Cargo.toml",
  "crates/seseragi-syntax/Cargo.toml",
  "crates/seseragi-wasm/Cargo.toml",
]
const WASM_PACKAGE_MANIFEST = "apps/playground/src/wasm/pkg/package.json"
const CHANGELOG = "CHANGELOG.md"

export type ReleaseContract = {
  version: string
  tag: string
}

function fail(message: string): never {
  throw new Error(`release contract: ${message}`)
}

export function assertSemver(version: string): void {
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u.test(version)) {
    fail(`version must be SemVer without build metadata, received ${version}`)
  }
}

export function workspaceVersion(cargoToml: string): string {
  const header = /^\[workspace\.package\]\s*$/mu.exec(cargoToml)
  if (!header || header.index === undefined) {
    fail("Cargo.toml is missing [workspace.package]")
  }
  const remaining = cargoToml.slice(header.index + header[0].length)
  const nextSection = remaining.search(/^\[/mu)
  const section =
    nextSection === -1 ? remaining : remaining.slice(0, nextSection)
  const version = section.match(/^version\s*=\s*"([^"]+)"\s*$/mu)?.[1]
  if (!version) fail("Cargo.toml is missing [workspace.package].version")
  assertSemver(version)
  return version
}

export function releaseTag(version: string): string {
  assertSemver(version)
  return `v${version}`
}

export function replaceManifestVersion(source: string, version: string): string {
  assertSemver(version)
  const matches = source.match(/^([ \t]*"version"\s*:\s*)"[^"]+"/mu)
  if (!matches) fail("package manifest is missing version")
  return source.replace(
    /^([ \t]*"version"\s*:\s*)"[^"]+"/mu,
    (_match, prefix: string) => `${prefix}"${version}"`
  )
}

export function replaceWorkspaceVersion(source: string, version: string): string {
  workspaceVersion(source)
  const header = /^\[workspace\.package\]\s*$/mu.exec(source)
  if (!header || header.index === undefined) {
    fail("Cargo.toml is missing [workspace.package]")
  }
  const sectionStart = header.index + header[0].length
  const remaining = source.slice(sectionStart)
  const nextSection = remaining.search(/^\[/mu)
  const section =
    nextSection === -1 ? remaining : remaining.slice(0, nextSection)
  const tail = nextSection === -1 ? "" : remaining.slice(nextSection)
  const updated = section.replace(
    /^([ \t]*version\s*=\s*)"[^"]+"\s*$/mu,
    (_match, prefix: string) => `${prefix}"${version}"`
  )
  return `${source.slice(0, sectionStart)}${updated}${tail}`
}

export function releaseNotes(changelog: string, version: string): string {
  const escaped = version.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&")
  const heading = new RegExp(`^## \\[${escaped}\\][^\\n]*\\n`, "mu").exec(
    changelog
  )
  if (!heading || heading.index === undefined) {
    fail(`${CHANGELOG} has no entry for ${version}`)
  }
  const remaining = changelog.slice(heading.index + heading[0].length)
  const nextHeading = remaining.search(/^## \[/mu)
  const entry = nextHeading === -1 ? remaining : remaining.slice(0, nextHeading)
  return `# Seseragi ${releaseTag(version)}\n\n${entry.trim()}\n`
}

async function text(root: string, relative: string): Promise<string> {
  return readFile(path.join(root, relative), "utf8")
}

async function manifestVersion(root: string, relative: string): Promise<string> {
  const manifest = JSON.parse(await text(root, relative)) as { version?: unknown }
  if (typeof manifest.version !== "string") {
    fail(`${relative} has no string version`)
  }
  return manifest.version
}

function lockedWorkspaceVersions(cargoLock: string): Map<string, string> {
  const versions = new Map<string, string>()
  for (const entry of cargoLock.split("[[package]]")) {
    const name = entry.match(/^\s*name\s*=\s*"([^"]+)"/mu)?.[1]
    const version = entry.match(/^\s*version\s*=\s*"([^"]+)"/mu)?.[1]
    if (name?.startsWith("seseragi-") && version) versions.set(name, version)
  }
  return versions
}

export async function readReleaseContract(
  root = repositoryRoot
): Promise<ReleaseContract> {
  const version = workspaceVersion(await text(root, "Cargo.toml"))
  return { version, tag: releaseTag(version) }
}

export async function syncReleaseVersions(root = repositoryRoot): Promise<string[]> {
  const { version } = await readReleaseContract(root)
  const updated: string[] = []
  for (const relative of PACKAGE_MANIFESTS) {
    const source = await text(root, relative)
    const next = replaceManifestVersion(source, version)
    if (next !== source) {
      await writeFile(path.join(root, relative), next)
      updated.push(relative)
    }
  }
  return updated
}

export async function checkReleaseContract(root = repositoryRoot): Promise<void> {
  const { version } = await readReleaseContract(root)
  const failures: string[] = []

  for (const relative of PACKAGE_MANIFESTS) {
    try {
      if ((await manifestVersion(root, relative)) !== version) {
        failures.push(`${relative} must use ${version}`)
      }
    } catch (error) {
      failures.push(error instanceof Error ? error.message : String(error))
    }
  }

  for (const relative of RUST_MANIFESTS) {
    try {
      const manifest = await text(root, relative)
      if (!/^version\.workspace\s*=\s*true\s*$/mu.test(manifest)) {
        failures.push(`${relative} must inherit version.workspace = true`)
      }
    } catch (error) {
      failures.push(error instanceof Error ? error.message : String(error))
    }
  }

  try {
    if ((await manifestVersion(root, WASM_PACKAGE_MANIFEST)) !== version) {
      failures.push(
        `${WASM_PACKAGE_MANIFEST} must use ${version}; run bun run build:playground:wasm`
      )
    }
  } catch (error) {
    failures.push(error instanceof Error ? error.message : String(error))
  }

  try {
    const locked = lockedWorkspaceVersions(await text(root, "Cargo.lock"))
    for (const relative of RUST_MANIFESTS) {
      const manifest = await text(root, relative)
      const name = manifest.match(/^name\s*=\s*"([^"]+)"\s*$/mu)?.[1]
      if (name && locked.get(name) !== version) {
        failures.push(`Cargo.lock must record ${name} ${version}`)
      }
    }
  } catch (error) {
    failures.push(error instanceof Error ? error.message : String(error))
  }

  try {
    releaseNotes(await text(root, CHANGELOG), version)
  } catch (error) {
    failures.push(error instanceof Error ? error.message : String(error))
  }

  if (failures.length > 0) {
    throw new Error(`release contract drift:\n- ${failures.join("\n- ")}`)
  }
}

export async function checkReleaseTag(
  tag: string | undefined,
  root = repositoryRoot
): Promise<void> {
  const contract = await readReleaseContract(root)
  if (tag !== contract.tag) {
    fail(
      `release tag ${tag || "<missing>"} must equal ${contract.tag} from Cargo.toml`
    )
  }
}

export async function releaseInfo(root = repositoryRoot): Promise<object> {
  const { version, tag } = await readReleaseContract(root)
  return {
    schemaVersion: 1,
    canonicalVersionSource: "Cargo.toml [workspace.package].version",
    version,
    tag,
    components: {
      cli: `seseragi ${version}`,
      lsp: `seseragi-lsp ${version}`,
      runtime: `@seseragi/runtime ${version}`,
      wasm: `seseragi-wasm ${version}`,
      vscode: `seseragi ${version}`,
      vscodeLegacyMigration: `legacy migration stub ${version}`,
    },
    artifacts: [
      `seseragi-v${version}-darwin-arm64.tar.gz`,
      `seseragi-v${version}-darwin-x64.tar.gz`,
      `seseragi-v${version}-linux-x64.tar.gz`,
      `seseragi-v${version}-win32-x64.zip`,
      `seseragi-v${version}-darwin-arm64.tar.gz.sha256`,
      `seseragi-v${version}-darwin-x64.tar.gz.sha256`,
      `seseragi-v${version}-linux-x64.tar.gz.sha256`,
      `seseragi-v${version}-win32-x64.zip.sha256`,
      `seseragi-v${version}-vscode-<target>.vsix`,
      `seseragi-legacy-migration-v${version}.vsix`,
      `seseragi-runtime-v${version}.tar.gz`,
      `seseragi-wasm-v${version}.tar.gz`,
    ],
  }
}

async function bump(version: string | undefined): Promise<void> {
  if (!version) fail("usage: release-contract.ts bump VERSION")
  assertSemver(version)
  const cargoToml = await text(repositoryRoot, "Cargo.toml")
  const next = replaceWorkspaceVersion(cargoToml, version)
  await writeFile(path.join(repositoryRoot, "Cargo.toml"), next)
  const updated = await syncReleaseVersions()
  console.log(
    `Set canonical toolchain version to ${version}; updated ${updated.length || "no"} package manifests.`
  )
  console.log(
    "Regenerate the WASM package, add the CHANGELOG entry, then run bun run release:check."
  )
}

async function main(): Promise<void> {
  const [command, argument] = process.argv.slice(2)
  switch (command) {
    case "info":
      console.log(JSON.stringify(await releaseInfo(), null, 2))
      return
    case "sync": {
      const updated = await syncReleaseVersions()
      console.log(
        updated.length > 0
          ? `Updated ${updated.join(", ")}.`
          : "Package manifests already match the canonical version."
      )
      return
    }
    case "check":
      await checkReleaseContract()
      console.log("Release contract is synchronized.")
      return
    case "check-tag":
      await checkReleaseTag(argument || process.env.GITHUB_REF_NAME)
      console.log(`Release tag ${argument || process.env.GITHUB_REF_NAME} is valid.`)
      return
    case "notes": {
      const notes = releaseNotes(
        await text(repositoryRoot, CHANGELOG),
        (await readReleaseContract()).version
      )
      if (argument) {
        const output = path.resolve(repositoryRoot, argument)
        await mkdir(path.dirname(output), { recursive: true })
        await writeFile(output, notes)
        console.log(`Wrote release notes to ${path.relative(repositoryRoot, output)}.`)
      } else {
        console.log(notes)
      }
      return
    }
    case "bump":
      await bump(argument)
      return
    default:
      fail("usage: release-contract.ts <info|sync|check|check-tag|notes|bump>")
  }
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  })
}
