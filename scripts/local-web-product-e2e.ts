import { cp, mkdir, mkdtemp, readFile, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import {
  downloadAndUnzipVSCode,
  runTests,
  runVSCodeCommand,
} from "@vscode/test-electron"
import {
  hostNativeReleaseTarget,
  type NativeReleaseTarget,
  nativeArchiveName,
  nativeReleaseTargets,
} from "./native-release"
import { readReleaseContract, repositoryRoot } from "./release-contract"

type Options = {
  nativeArchive?: string
  output?: string
  target?: NativeReleaseTarget
  version?: string
  vsix?: string
}

function fail(message: string): never {
  throw new Error(`local Web product E2E: ${message}`)
}

function parseOptions(arguments_: string[]): Options {
  const options: Options = {}
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index]
    const value = arguments_[index + 1]
    if (!value) fail(`${argument} requires a value`)
    index += 1
    switch (argument) {
      case "--native":
        options.nativeArchive = value
        break
      case "--output":
        options.output = value
        break
      case "--target":
        if (!(value in nativeReleaseTargets))
          fail(`unsupported target ${value}`)
        options.target = value as NativeReleaseTarget
        break
      case "--version":
        options.version = value
        break
      case "--vsix":
        options.vsix = value
        break
      default:
        fail(`unknown option ${argument}`)
    }
  }
  return options
}

function run(command: string[], cwd = repositoryRoot): void {
  const result = Bun.spawnSync(command, {
    cwd,
    env: { ...process.env, COPYFILE_DISABLE: "1" },
    stderr: "inherit",
    stdout: "inherit",
  })
  if (!result.success) fail(`${command.join(" ")} failed`)
}

async function extractNativeArchive(
  archive: string,
  target: NativeReleaseTarget,
  destination: string
): Promise<string> {
  await mkdir(destination, { recursive: true })
  if (nativeReleaseTargets[target].archiveExtension === "tar.gz") {
    run(["tar", "-xzf", archive, "-C", destination])
  } else {
    run([
      "powershell.exe",
      "-NoProfile",
      "-NonInteractive",
      "-Command",
      `Expand-Archive -LiteralPath '${archive.replaceAll("'", "''")}' -DestinationPath '${destination.replaceAll("'", "''")}' -Force`,
    ])
  }
  return path.join(
    destination,
    target.startsWith("win32-") ? "seseragi.exe" : "seseragi"
  )
}

async function main(): Promise<void> {
  const options = parseOptions(process.argv.slice(2))
  const release = await readReleaseContract()
  const version = options.version ?? release.version
  const target = options.target ?? hostNativeReleaseTarget()
  const hostTarget = hostNativeReleaseTarget()
  const vscodeVersion = process.env.SESERAGI_E2E_VSCODE_VERSION ?? "1.133.0"
  if (target !== hostTarget) {
    fail(`target ${target} cannot execute on host ${hostTarget}`)
  }

  const nativeArchive = path.resolve(
    options.nativeArchive ??
      path.join(
        repositoryRoot,
        "target",
        "native-release",
        nativeArchiveName(version, target)
      )
  )
  const vsix = path.resolve(
    options.vsix ??
      path.join(
        repositoryRoot,
        "target",
        `seseragi-v${version}-vscode-${target}.vsix`
      )
  )
  for (const file of [nativeArchive, vsix]) {
    if (!(await Bun.file(file).exists()))
      fail(`artifact does not exist: ${file}`)
  }

  const output = path.resolve(
    options.output ??
      path.join(repositoryRoot, "target", "local-web-product-e2e")
  )
  await rm(output, { recursive: true, force: true })
  await mkdir(output, { recursive: true })
  const temporaryRoot = process.platform === "darwin" ? "/tmp" : tmpdir()
  const temporary = await mkdtemp(path.join(temporaryRoot, "seseragi-e2e-"))
  try {
    const nativeRoot = path.join(temporary, "native")
    const cli = await extractNativeArchive(nativeArchive, target, nativeRoot)
    const metadata = Bun.spawnSync([cli, "--version-json"], {
      stderr: "pipe",
      stdout: "pipe",
    })
    if (!metadata.success) fail("extracted CLI failed its version handshake")
    const cliMetadata = JSON.parse(metadata.stdout.toString())
    if (
      cliMetadata.name !== "seseragi" ||
      cliMetadata.version !== version ||
      cliMetadata.target !== nativeReleaseTargets[target].rustTarget
    ) {
      fail(`extracted CLI metadata does not match ${version}/${target}`)
    }

    const project = path.join(temporary, "project-flow-app")
    await cp(
      path.join(repositoryRoot, "examples", "samples", "project-flow-app"),
      project,
      { recursive: true }
    )
    const extensionsDirectory = path.join(temporary, "extensions")
    const userDataDirectory = path.join(temporary, "user-data")
    await mkdir(extensionsDirectory)
    await mkdir(userDataDirectory)

    let vscodeExecutablePath = await downloadAndUnzipVSCode(vscodeVersion)
    if (process.platform === "darwin") {
      const currentExecutable = path.join(
        path.dirname(vscodeExecutablePath),
        "Code"
      )
      if (await Bun.file(currentExecutable).exists()) {
        vscodeExecutablePath = currentExecutable
      }
    }
    await runVSCodeCommand(
      [
        "--install-extension",
        vsix,
        "--force",
        `--extensions-dir=${extensionsDirectory}`,
        `--user-data-dir=${userDataDirectory}`,
      ],
      { reuseMachineInstall: true, version: vscodeVersion }
    )

    await runTests({
      extensionDevelopmentPath: path.join(
        repositoryRoot,
        "scripts",
        "fixtures",
        "local-web-e2e-driver"
      ),
      extensionTestsEnv: {
        SESERAGI_E2E_CLI: cli,
        SESERAGI_E2E_OUTPUT: output,
        SESERAGI_E2E_PLAYWRIGHT: path.join(
          repositoryRoot,
          "apps",
          "playground",
          "node_modules",
          "playwright"
        ),
        SESERAGI_E2E_PROJECT: project,
        SESERAGI_E2E_VERSION: version,
        SESERAGI_E2E_VSIX: vsix,
      },
      extensionTestsPath: path.join(
        repositoryRoot,
        "scripts",
        "local-web-product-e2e-extension.cjs"
      ),
      launchArgs: [
        project,
        `--extensions-dir=${extensionsDirectory}`,
        `--user-data-dir=${userDataDirectory}`,
      ],
      reuseMachineInstall: true,
      vscodeExecutablePath,
    })

    const report = JSON.parse(
      await readFile(path.join(output, "report.json"), "utf8")
    )
    if (report.schema !== 1 || report.result !== "passed") {
      fail("Extension Host did not produce a passing report")
    }
    console.log(`Local Web product E2E passed; artifacts: ${output}`)
  } finally {
    await rm(temporary, { recursive: true, force: true })
  }
}

if (import.meta.main) await main()
