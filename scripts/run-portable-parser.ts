import { resolve } from "node:path"
import { portableParserEnvironment } from "./portable-parser-environment"

const root = resolve(import.meta.dir, "..")
const { environment, metadata } = portableParserEnvironment(root)
const command = process.argv.slice(2)
if (command.length === 0) throw new Error("Expected a build command")
if (command[0] === "wasm-pack") {
  const version = metadata.packages.find(
    (pkg) => pkg.name === "wasm-bindgen"
  )?.version
  const binary = Bun.which("wasm-bindgen")
  const installed = binary
    ? Bun.spawnSync([binary, "--version"], { stdout: "pipe", stderr: "pipe" })
    : undefined
  if (
    !version ||
    installed?.exitCode !== 0 ||
    installed.stdout.toString().trim() !== `wasm-bindgen ${version}`
  ) {
    throw new Error(
      `Install the matching locked CLI before building: cargo install wasm-bindgen-cli --version ${version ?? "<Cargo.lock version>"} --locked`
    )
  }
}

const child = Bun.spawnSync(command, {
  cwd: root,
  env: environment,
  stdout: "inherit",
  stderr: "inherit",
})
process.exit(child.exitCode)
