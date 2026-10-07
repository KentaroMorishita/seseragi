import { resolve } from "node:path"

export function portableParserEnvironment(root: string) {
  const metadataProcess = Bun.spawnSync(
    ["cargo", "metadata", "--locked", "--format-version", "1"],
    { cwd: root, stdout: "pipe", stderr: "inherit" }
  )
  if (metadataProcess.exitCode !== 0) throw new Error("cargo metadata failed")
  const metadata = JSON.parse(metadataProcess.stdout.toString()) as {
    target_directory: string
    packages: { name: string; manifest_path: string }[]
  }
  const language = metadata.packages.find(
    (dependency) => dependency.name === "tree-sitter-language"
  )
  if (!language)
    throw new Error("tree-sitter-language is missing from Cargo.lock")
  // The TypeScript grammar's upstream build script predates the Rust core's
  // wasm32 support. Use the same portable C headers as tree-sitter itself.
  const headers = resolve(language.manifest_path, "..", "wasm", "include")
  const environment = {
    ...process.env,
    CC_SHELL_ESCAPED_FLAGS: "1",
    "CFLAGS_wasm32-unknown-unknown": [
      process.env["CFLAGS_wasm32-unknown-unknown"] ??
        process.env.CFLAGS_wasm32_unknown_unknown,
      `-I${JSON.stringify(headers)}`,
    ]
      .filter(Boolean)
      .join(" "),
  }

  return { environment, metadata }
}
