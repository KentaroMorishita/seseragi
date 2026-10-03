import { existsSync } from "node:fs"
import { dirname, join, resolve } from "node:path"

/** Configure the C grammar build with Tree-sitter's own WASM libc headers. */
export async function configureWasmCToolchain(
  environment: Record<string, string | undefined>,
  languageManifest: string,
  command: (argv: string[]) => Promise<string>
): Promise<void> {
  const headers = join(dirname(languageManifest), "wasm/include")
  if (!existsSync(headers))
    throw new Error("Locked tree-sitter-language must provide WASM C headers")
  const configured =
    environment.CC_wasm32_unknown_unknown ??
    environment["CC_wasm32-unknown-unknown"]
  let compiler = configured ?? "clang"
  if (!(await command([compiler, "--print-targets"])).includes("wasm32")) {
    if (configured || !Bun.which("brew"))
      throw new Error(
        "Set CC_wasm32_unknown_unknown to a clang with the wasm32 target"
      )
    compiler = join(await command(["brew", "--prefix", "llvm"]), "bin/clang")
    if (!(await command([compiler, "--print-targets"])).includes("wasm32"))
      throw new Error("LLVM clang lacks the wasm32 target")
  }
  environment.CC_wasm32_unknown_unknown = compiler
  const siblingAr = join(dirname(compiler), "llvm-ar")
  environment.AR_wasm32_unknown_unknown ??= existsSync(siblingAr)
    ? siblingAr
    : (Bun.which("llvm-ar") ?? "llvm-ar")
  // cc parses quoted flags so dependency-cache paths may contain spaces.
  environment.CC_SHELL_ESCAPED_FLAGS = "1"
  environment.CFLAGS_wasm32_unknown_unknown = [
    environment.CFLAGS_wasm32_unknown_unknown ?? "",
    `-I'${headers.replaceAll("'", "'\\''")}'`,
  ]
    .filter(Boolean)
    .join(" ")
}

if (import.meta.main) {
  const environment = { ...process.env }
  const cwd = resolve(import.meta.dir, "..")
  const command = async (argv: string[]): Promise<string> => {
    const child = Bun.spawn(argv, { cwd, env: environment, stderr: "inherit" })
    const output = await new Response(child.stdout).text()
    if ((await child.exited) !== 0) throw new Error(`${argv.join(" ")} failed`)
    return output.trim()
  }
  const metadata = JSON.parse(
    await command(["cargo", "metadata", "--locked", "--format-version", "1"])
  ) as { packages: { name: string; manifest_path: string }[] }
  const language = metadata.packages.find(
    (pkg) => pkg.name === "tree-sitter-language"
  )
  if (!language) throw new Error("Locked tree-sitter-language is required")
  await configureWasmCToolchain(environment, language.manifest_path, command)
  const argv = process.argv.slice(2)
  if (argv.length === 0) throw new Error("Provide a WASM build command")
  const child = Bun.spawn(argv, {
    cwd,
    env: environment,
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  })
  process.exit(await child.exited)
}
