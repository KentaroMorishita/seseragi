import { readFileSync } from "node:fs"
import { resolve } from "node:path"
import { portableParserEnvironment } from "./portable-parser-environment"

const root = resolve(import.meta.dir, "..")
const { environment, metadata } = portableParserEnvironment(root)
const command = process.argv.slice(2)
if (command.length === 0) throw new Error("Expected a build command")
if (process.env.SESERAGI_WASM_BUILD_EVIDENCE) {
  console.log(
    JSON.stringify({
      command,
      rustflags: process.env.RUSTFLAGS,
      compiler: environment["CC_wasm32-unknown-unknown"],
      compilerVersion: Bun.spawnSync(
        [environment["CC_wasm32-unknown-unknown"], "--version"],
        { stdout: "pipe" }
      ).stdout.toString(),
      wasiSdkVersion: readFileSync(
        resolve(process.env.WASI_SDK_PATH ?? "", "VERSION"),
        "utf8"
      ),
      globalCflags: process.env.CFLAGS,
      targetCflags: process.env.TARGET_CFLAGS,
      cflags: environment["CFLAGS_wasm32-unknown-unknown"],
      targetDirectory: metadata.target_directory,
    })
  )
}
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
  if (process.env.SESERAGI_WASM_BUILD_EVIDENCE) {
    console.log(installed?.stdout.toString().trim())
  }
}

const child = Bun.spawnSync(command, {
  cwd: root,
  env: environment,
  stdout: "inherit",
  stderr: "inherit",
})
process.exit(child.exitCode)
