import { copyFileSync, mkdirSync } from "node:fs"
import path from "node:path"
import manifest from "../../seseragi-legacy/package.json"
import { verifyLegacyPackage } from "./verify-legacy-package"

const packageRoot = path.resolve(import.meta.dir, "..")
const repositoryRoot = path.resolve(packageRoot, "../..")
const legacyRoot = path.join(repositoryRoot, "extensions", "seseragi-legacy")

function run(command: string[], cwd: string): void {
  const result = Bun.spawnSync(command, {
    cwd,
    stdout: "inherit",
    stderr: "inherit",
  })
  if (!result.success) throw new Error(`${command.join(" ")} failed`)
}

copyFileSync(
  path.join(repositoryRoot, "LICENSE.txt"),
  path.join(legacyRoot, "LICENSE.txt")
)
const output = path.resolve(
  legacyRoot,
  process.env.SESERAGI_LEGACY_EXTENSION_OUTPUT ||
    `../../target/seseragi-legacy-migration-v${manifest.version}.vsix`
)
mkdirSync(path.dirname(output), { recursive: true })
const vsce = path.join(
  packageRoot,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "vsce.cmd" : "vsce"
)
if (!(await Bun.file(vsce).exists())) {
  throw new Error(
    "local vsce is missing; run bun install --frozen-lockfile in extensions/seseragi"
  )
}
run([vsce, "package", "--out", output], legacyRoot)
await verifyLegacyPackage(output)
console.log(`Packaged and verified legacy migration VSIX ${output}.`)
