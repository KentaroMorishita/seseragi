import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import path from "node:path"
import manifest from "../package.json"
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

function packagedReadme(source: string): string {
  const repositoryAsset = "../../assets/brand/source/seseragi-icon.svg"
  const packageAsset = "./images/icon.png"
  if (!source.includes(repositoryAsset)) {
    throw new Error(`extension README is missing ${repositoryAsset}`)
  }
  return source.replace(repositoryAsset, packageAsset)
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
    `../../target/seseragi-v${manifest.version}-vscode-${target}.vsix`
)
mkdirSync(path.dirname(output), { recursive: true })

const readmePath = path.join(packageRoot, "README.md")
const repositoryReadme = readFileSync(readmePath, "utf8")
try {
  // GitHub renders the canonical SVG. VSCE rejects SVG references in the
  // packaged README, so only the transient package input uses the PNG copy.
  writeFileSync(readmePath, packagedReadme(repositoryReadme))
  run([
    "bun",
    "run",
    "vsce",
    "package",
    "--target",
    target,
    "--out",
    output,
  ])
} finally {
  writeFileSync(readmePath, repositoryReadme)
}

await verifyPackage(output, target)
console.log(`Packaged and smoke-verified ${output}.`)
