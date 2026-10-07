import { resolve } from "node:path"
import { portableParserEnvironment } from "./portable-parser-environment"

const root = resolve(import.meta.dir, "..")
const { environment } = portableParserEnvironment(root)
const command = process.argv.slice(2)
if (command.length === 0) throw new Error("Expected a build command")
const child = Bun.spawnSync(command, {
  cwd: root,
  env: environment,
  stdout: "inherit",
  stderr: "inherit",
})
process.exit(child.exitCode)
