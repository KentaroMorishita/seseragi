import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
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

    const project = path.join(temporary, "hello-web")
    run([cli, "new", "web", project])
    for (const source of ["app.ssrg", "main.ssrg"]) {
      const generated = await readFile(path.join(project, "src", source))
      const canonical = await readFile(
        path.join(
          repositoryRoot,
          "examples",
          "samples",
          "web-starter",
          "src",
          source
        )
      )
      if (!generated.equals(canonical)) {
        fail(`generated ${source} differs from the canonical Web starter`)
      }
    }
    const webFixture = path.join(
      repositoryRoot,
      "examples/spec/fixtures/projects/web-assets"
    )
    await cp(path.join(webFixture, "web"), path.join(project, "web"), {
      recursive: true,
    })
    await cp(path.join(webFixture, "public"), path.join(project, "public"), {
      recursive: true,
    })
    const manifestPath = path.join(project, "seseragi.toml")
    await writeFile(
      manifestPath,
      `${await readFile(manifestPath, "utf8")}\n[web]\nindex = "web/index.html"\npublic = "public"\n`
    )
    run([cli, "lock", "update", project])
    const extensionsDirectory = path.join(temporary, "extensions")
    const userDataDirectory = path.join(temporary, "user-data")
    await mkdir(extensionsDirectory)
    await mkdir(userDataDirectory)

    // Run the SDK with Node: Bun's child_process stdin pipe can lose buffered
    // archive bytes under backpressure (#824). The installed-product assertions
    // still run in the actual downloaded VS Code Extension Host.
    const configuration = path.join(temporary, "product-journey.json")
    await writeFile(
      configuration,
      JSON.stringify({
        temporary,
        repositoryRoot,
        cli,
        output,
        project,
        version,
        vsix,
        vscodeVersion,
        extensionsDirectory,
        userDataDirectory,
      })
    )
    run([
      "node",
      path.join(repositoryRoot, "scripts", "local-web-product-e2e-node.cjs"),
      configuration,
    ])

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

if (import.meta.main) {
  await main()
  // Only a fully completed installed-product journey reaches this point.
  process.exit(0)
}
