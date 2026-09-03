import { createHash } from "node:crypto"
import {
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  stat,
  writeFile,
} from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import { readReleaseContract, repositoryRoot } from "./release-contract"

export const nativeReleaseTargets = {
  "darwin-arm64": {
    archiveExtension: "tar.gz",
    binarySuffix: "",
    rustTarget: "aarch64-apple-darwin",
  },
  "darwin-x64": {
    archiveExtension: "tar.gz",
    binarySuffix: "",
    rustTarget: "x86_64-apple-darwin",
  },
  "linux-x64": {
    archiveExtension: "tar.gz",
    binarySuffix: "",
    rustTarget: "x86_64-unknown-linux-gnu",
  },
  "win32-x64": {
    archiveExtension: "zip",
    binarySuffix: ".exe",
    rustTarget: "x86_64-pc-windows-msvc",
  },
} as const

export type NativeReleaseTarget = keyof typeof nativeReleaseTargets

type NativeReleaseOptions = {
  archive?: string
  cli?: string
  lsp?: string
  output?: string
  release?: boolean
  target?: NativeReleaseTarget
  version?: string
}

type NativeReleaseArtifact = {
  archive: string
  checksum: string
  target: NativeReleaseTarget
  version: string
}

function fail(message: string): never {
  throw new Error(`native release: ${message}`)
}

function targetContract(
  target: string
): (typeof nativeReleaseTargets)[NativeReleaseTarget] {
  if (!(target in nativeReleaseTargets)) {
    fail(`unsupported target ${target}`)
  }
  return nativeReleaseTargets[target as NativeReleaseTarget]
}

export function hostNativeReleaseTarget(
  platform = process.platform,
  architecture = process.arch
): NativeReleaseTarget {
  const target = {
    "darwin:arm64": "darwin-arm64",
    "darwin:x64": "darwin-x64",
    "linux:x64": "linux-x64",
    "win32:x64": "win32-x64",
  }[`${platform}:${architecture}`]
  if (!target) fail(`unsupported host ${platform}/${architecture}`)
  return target as NativeReleaseTarget
}

export function nativeArchiveName(
  version: string,
  target: NativeReleaseTarget
): string {
  return `seseragi-v${version}-${target}.${targetContract(target).archiveExtension}`
}

export function nativeChecksumName(
  version: string,
  target: NativeReleaseTarget
): string {
  return `${nativeArchiveName(version, target)}.sha256`
}

function run(command: string[], cwd = repositoryRoot): string {
  const result = Bun.spawnSync(command, {
    cwd,
    env: { ...process.env, COPYFILE_DISABLE: "1" },
    stdout: "pipe",
    stderr: "pipe",
  })
  const stdout = result.stdout.toString()
  const stderr = result.stderr.toString()
  if (!result.success) {
    fail(`${command.join(" ")} failed${stderr ? `:\n${stderr.trim()}` : ""}`)
  }
  return stdout
}

function powershellLiteral(value: string): string {
  return `'${value.replaceAll("'", "''")}'`
}

async function sha256(file: string): Promise<string> {
  return createHash("sha256")
    .update(await readFile(file))
    .digest("hex")
}

async function writeChecksum(archive: string): Promise<string> {
  const checksum = `${archive}.sha256`
  await writeFile(
    checksum,
    `${await sha256(archive)}  ${path.basename(archive)}\n`
  )
  return checksum
}

async function verifyChecksum(
  archive: string,
  checksum: string
): Promise<void> {
  const source = await readFile(checksum, "utf8")
  const expected = `${await sha256(archive)}  ${path.basename(archive)}\n`
  if (source !== expected) {
    fail(`${path.basename(checksum)} does not match ${path.basename(archive)}`)
  }
}

function binaryNames(target: NativeReleaseTarget): [string, string] {
  const { binarySuffix } = targetContract(target)
  return [`seseragi${binarySuffix}`, `seseragi-lsp${binarySuffix}`]
}

async function releaseInputs(options: NativeReleaseOptions): Promise<
  Required<Pick<NativeReleaseOptions, "target" | "version">> & {
    cli: string
    lsp: string
    output: string
  }
> {
  const target = options.target ?? hostNativeReleaseTarget()
  const version = options.version ?? (await readReleaseContract()).version
  const [cliName, lspName] = binaryNames(target)
  const releaseDirectory = path.join(repositoryRoot, "target", "release")
  return {
    target,
    version,
    cli: path.resolve(options.cli ?? path.join(releaseDirectory, cliName)),
    lsp: path.resolve(options.lsp ?? path.join(releaseDirectory, lspName)),
    output: path.resolve(
      options.output ?? path.join(repositoryRoot, "target", "native-release")
    ),
  }
}

export async function packageNativeRelease(
  options: NativeReleaseOptions = {}
): Promise<NativeReleaseArtifact> {
  const { target, version, cli, lsp, output } = await releaseInputs(options)
  const contract = targetContract(target)
  const [cliName, lspName] = binaryNames(target)
  const temporary = await mkdtemp(
    path.join(tmpdir(), "seseragi-native-package-")
  )
  const archive = path.join(output, nativeArchiveName(version, target))

  try {
    const staging = path.join(temporary, "staging")
    await mkdir(staging)
    await copyFile(cli, path.join(staging, cliName))
    await copyFile(lsp, path.join(staging, lspName))
    await copyFile(
      path.join(repositoryRoot, "runtime/unicode/LICENSE"),
      path.join(staging, "UNICODE-LICENSE")
    )
    if (contract.archiveExtension === "tar.gz") {
      await chmod(path.join(staging, cliName), 0o755)
      await chmod(path.join(staging, lspName), 0o755)
    }

    await mkdir(output, { recursive: true })
    await rm(archive, { force: true })
    await rm(`${archive}.sha256`, { force: true })

    if (contract.archiveExtension === "tar.gz") {
      run([
        "tar",
        "-C",
        staging,
        "-czf",
        archive,
        cliName,
        lspName,
        "UNICODE-LICENSE",
      ])
    } else {
      run([
        "powershell.exe",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        `Compress-Archive -LiteralPath ${[cliName, lspName, "UNICODE-LICENSE"].map((name) => powershellLiteral(path.join(staging, name))).join(",")} -DestinationPath ${powershellLiteral(archive)} -Force`,
      ])
    }

    return {
      archive,
      checksum: await writeChecksum(archive),
      target,
      version,
    }
  } finally {
    await rm(temporary, { recursive: true, force: true })
  }
}

async function extractedFiles(directory: string): Promise<string[]> {
  const files: string[] = []
  const visit = async (current: string, prefix: string): Promise<void> => {
    for (const entry of await readdir(current, { withFileTypes: true })) {
      const relative = prefix ? `${prefix}/${entry.name}` : entry.name
      if (!entry.isFile()) fail(`archive contains non-file entry ${relative}`)
      files.push(relative)
    }
  }
  await visit(directory, "")
  return files.sort()
}

function verifyExecutable(executable: string, arguments_: string[]): string {
  const result = Bun.spawnSync([executable, ...arguments_], {
    stdout: "pipe",
    stderr: "pipe",
  })
  if (!result.success) {
    fail(
      `${path.basename(executable)} ${arguments_.join(" ")} failed: ${result.stderr.toString().trim()}`
    )
  }
  return result.stdout.toString().trim()
}

export async function verifyNativeRelease(
  options: NativeReleaseOptions = {}
): Promise<NativeReleaseArtifact> {
  const { target, version, output } = await releaseInputs(options)
  const contract = targetContract(target)
  const archive = path.resolve(
    options.archive ?? path.join(output, nativeArchiveName(version, target))
  )
  const checksum = `${archive}.sha256`
  const [cliName, lspName] = binaryNames(target)
  const temporary = await mkdtemp(
    path.join(tmpdir(), "seseragi-native-verify-")
  )

  try {
    await verifyChecksum(archive, checksum)
    if (contract.archiveExtension === "tar.gz") {
      run(["tar", "-xzf", archive, "-C", temporary])
    } else {
      run([
        "powershell.exe",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        `Expand-Archive -LiteralPath ${powershellLiteral(archive)} -DestinationPath ${powershellLiteral(temporary)} -Force`,
      ])
    }

    const expectedFiles = [cliName, lspName, "UNICODE-LICENSE"].sort()
    const actualFiles = await extractedFiles(temporary)
    if (JSON.stringify(actualFiles) !== JSON.stringify(expectedFiles)) {
      fail(
        `${path.basename(archive)} must contain only ${expectedFiles.join(", ")}; received ${actualFiles.join(", ")}`
      )
    }

    const cli = path.join(temporary, cliName)
    if (
      (await readFile(path.join(temporary, "UNICODE-LICENSE"), "utf8")) !==
      (await readFile(
        path.join(repositoryRoot, "runtime/unicode/LICENSE"),
        "utf8"
      ))
    ) {
      fail("archive Unicode license notice differs from the pinned data notice")
    }
    const lsp = path.join(temporary, lspName)
    if (contract.archiveExtension === "tar.gz") {
      for (const executable of [cli, lsp]) {
        const mode = (await stat(executable)).mode & 0o777
        if (mode !== 0o755) {
          fail(
            `${path.basename(executable)} must be mode 755, received ${mode.toString(8)}`
          )
        }
      }
    }

    const cliVersion = verifyExecutable(cli, ["--version"])
    if (
      !cliVersion.includes(`seseragi ${version}`) ||
      !cliVersion.includes(`target ${contract.rustTarget}`)
    ) {
      fail(
        `seseragi --version does not match ${version}/${contract.rustTarget}`
      )
    }
    if (options.release && !cliVersion.includes("(release,")) {
      fail(`seseragi is not a clean v${version} release build`)
    }
    const cliMetadata = JSON.parse(
      verifyExecutable(cli, ["--version-json"])
    ) as Record<string, unknown>
    if (
      cliMetadata.name !== "seseragi" ||
      cliMetadata.version !== version ||
      cliMetadata.target !== contract.rustTarget
    ) {
      fail(
        `seseragi --version-json does not match ${version}/${contract.rustTarget}`
      )
    }
    if (
      options.release &&
      (cliMetadata.channel !== "release" ||
        cliMetadata.dirty !== false ||
        cliMetadata.releaseTag !== `v${version}`)
    ) {
      fail(`seseragi --version-json is not a clean v${version} release build`)
    }

    const lspVersion = JSON.parse(
      verifyExecutable(lsp, ["--version-json"])
    ) as Record<string, unknown>
    if (
      lspVersion.name !== "seseragi-lsp" ||
      lspVersion.version !== version ||
      lspVersion.target !== contract.rustTarget
    ) {
      fail(
        `seseragi-lsp --version-json does not match ${version}/${contract.rustTarget}`
      )
    }
    if (
      options.release &&
      (lspVersion.channel !== "release" ||
        lspVersion.dirty !== false ||
        lspVersion.releaseTag !== `v${version}`)
    ) {
      fail(`seseragi-lsp is not a clean v${version} release build`)
    }

    return { archive, checksum, target, version }
  } finally {
    await rm(temporary, { recursive: true, force: true })
  }
}

function parseOptions(arguments_: string[]): NativeReleaseOptions {
  const options: NativeReleaseOptions = {}
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index]
    if (argument === "--release") {
      options.release = true
      continue
    }
    const value = arguments_[index + 1]
    if (!value) fail(`${argument} requires a value`)
    index += 1
    switch (argument) {
      case "--archive":
        options.archive = value
        break
      case "--cli":
        options.cli = value
        break
      case "--lsp":
        options.lsp = value
        break
      case "--output":
        options.output = value
        break
      case "--target":
        targetContract(value)
        options.target = value as NativeReleaseTarget
        break
      case "--version":
        options.version = value
        break
      default:
        fail(`unknown option ${argument}`)
    }
  }
  return options
}

async function main(): Promise<void> {
  const [command, ...arguments_] = process.argv.slice(2)
  const options = parseOptions(arguments_)
  let artifact: NativeReleaseArtifact
  let result: "Packaged" | "Verified"
  switch (command) {
    case "package":
      artifact = await packageNativeRelease(options)
      result = "Packaged"
      break
    case "verify":
      artifact = await verifyNativeRelease(options)
      result = "Verified"
      break
    case "smoke":
      artifact = await packageNativeRelease(options)
      await verifyNativeRelease({ ...options, archive: artifact.archive })
      result = "Verified"
      break
    default:
      fail(
        "usage: native-release.ts <package|verify|smoke> [--version VERSION] [--target TARGET] [--output DIR] [--archive FILE] [--cli FILE] [--lsp FILE] [--release]"
      )
  }
  console.log(
    `${result} ${path.basename(artifact.archive)} and ${path.basename(artifact.checksum)} for ${artifact.target}.`
  )
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  })
}
