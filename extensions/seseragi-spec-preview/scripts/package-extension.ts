import { chmodSync, copyFileSync, mkdirSync, rmSync } from "node:fs"
import path from "node:path"
import { verifyPackage } from "./verify-package"

const { platformTarget, serverBinaryName } = require("../extension-core")

const packageRoot = path.resolve(import.meta.dir, "..")
const repositoryRoot = path.resolve(packageRoot, "../..")
const hostTarget = platformTarget(process.platform, process.arch)
const target = process.env.SESERAGI_EXTENSION_TARGET || hostTarget
if (!target) {
  throw new Error(
    `unsupported packaging host: ${process.platform}/${process.arch}`
  )
}
if (target !== hostTarget && !process.env.SESERAGI_LSP_BINARY) {
  throw new Error(
    `cross packaging ${target} from ${hostTarget} requires SESERAGI_LSP_BINARY`
  )
}

function run(command: string[], cwd = packageRoot): void {
  const result = Bun.spawnSync(command, {
    cwd,
    stdout: "inherit",
    stderr: "inherit",
  })
  if (!result.success) throw new Error(`${command.join(" ")} failed`)
}

let sourceBinary = process.env.SESERAGI_LSP_BINARY
if (!sourceBinary) {
  run(["cargo", "build", "--release", "-p", "seseragi-lsp"], repositoryRoot)
  sourceBinary = path.join(
    repositoryRoot,
    "target",
    "release",
    serverBinaryName(process.platform)
  )
}
sourceBinary = path.resolve(sourceBinary)
if (!(await Bun.file(sourceBinary).exists())) {
  throw new Error(`seseragi-lsp binary does not exist: ${sourceBinary}`)
}

const serverRoot = path.join(packageRoot, "server")
const stagedDirectory = path.join(serverRoot, target)
const stagedBinary = path.join(
  stagedDirectory,
  target.startsWith("win32-") ? "seseragi-lsp.exe" : "seseragi-lsp"
)
rmSync(serverRoot, { recursive: true, force: true })
mkdirSync(stagedDirectory, { recursive: true })
copyFileSync(sourceBinary, stagedBinary)
if (!target.startsWith("win32-")) chmodSync(stagedBinary, 0o755)
copyFileSync(
  path.join(repositoryRoot, "LICENSE.txt"),
  path.join(packageRoot, "LICENSE.txt")
)

run(["bun", "run", "build"])
const output = path.resolve(
  packageRoot,
  process.env.SESERAGI_EXTENSION_OUTPUT ||
    `../../target/seseragi-vscode-${target}.vsix`
)
mkdirSync(path.dirname(output), { recursive: true })
run(["bunx", "vsce", "package", "--target", target, "--out", output])
await verifyPackage(output, target)
console.log(`Packaged and verified ${output}.`)
