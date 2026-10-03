import assert from "node:assert/strict"
import { existsSync } from "node:fs"
import { mkdir, readdir, readFile } from "node:fs/promises"
import { homedir } from "node:os"
import { join, resolve } from "node:path"
import { pathToFileURL } from "node:url"
import { configureWasmCToolchain } from "./wasm-c-toolchain"

const root = resolve(import.meta.dir, "..")
const environment = { ...process.env }

async function command(
  argv: string[],
  input?: string,
  inherit = false
): Promise<string> {
  const child = Bun.spawn(argv, {
    cwd: root,
    env: environment,
    stdin: input === undefined ? "ignore" : new Blob([input]),
    stdout: inherit ? "inherit" : "pipe",
    stderr: "pipe",
  })
  const [code, stdout, stderr] = await Promise.all([
    child.exited,
    inherit ? Promise.resolve("") : new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ])
  if (code !== 0)
    throw new Error(`${argv.join(" ")} failed (${code})\n${stderr}`)
  if (stderr) process.stderr.write(stderr)
  return stdout.trimEnd()
}

// Use the repository's pinned rustup toolchain, including its WASM target.
const cargo = await command(["rustup", "which", "cargo"])
environment.RUSTC = await command(["rustup", "which", "rustc"])
const metadata = JSON.parse(
  await command([
    cargo,
    "metadata",
    "--locked",
    "--offline",
    "--format-version",
    "1",
  ])
) as {
  packages: Array<{ name: string; version: string; manifest_path: string }>
  target_directory: string
}
const language = metadata.packages.find(
  (pkg) => pkg.name === "tree-sitter-language"
)
const bindgen = metadata.packages.find((pkg) => pkg.name === "wasm-bindgen")
assert(
  language && bindgen,
  "locked Tree-sitter and wasm-bindgen dependencies are required"
)
await configureWasmCToolchain(environment, language.manifest_path, command)

async function findBindgen(version: string): Promise<string> {
  if (environment.WASM_BINDGEN) return environment.WASM_BINDGEN
  const onPath = Bun.which("wasm-bindgen")
  if (onPath) return onPath
  const caches = [
    join(homedir(), "Library/Caches/.wasm-pack"),
    join(environment.XDG_CACHE_HOME ?? join(homedir(), ".cache"), ".wasm-pack"),
  ]
  for (const cache of caches) {
    if (!existsSync(cache)) continue
    for (const entry of await readdir(cache)) {
      if (!entry.includes(`wasm-bindgen`) || !entry.includes(version)) continue
      for (const suffix of ["wasm-bindgen", "bin/wasm-bindgen"]) {
        const candidate = join(cache, entry, suffix)
        if (existsSync(candidate)) return candidate
      }
    }
  }
  throw new Error(
    `Install wasm-bindgen-cli ${version}, or set WASM_BINDGEN to its executable`
  )
}
const bindgenCommand = await findBindgen(bindgen.version)
assert.equal(
  await command([bindgenCommand, "--version"]),
  `wasm-bindgen ${bindgen.version}`
)

console.log(
  "Building the shared declaration-text converter without filesystem support..."
)
await command(
  [
    cargo,
    "build",
    "--locked",
    "-p",
    "seseragi-dts",
    "--no-default-features",
    "--example",
    "portable_wasm",
    "--target",
    "wasm32-unknown-unknown",
  ],
  undefined,
  true
)
const output = join(metadata.target_directory, "dts-portable-wasm")
await mkdir(output, { recursive: true })
await command([
  bindgenCommand,
  join(
    metadata.target_directory,
    "wasm32-unknown-unknown/debug/examples/portable_wasm.wasm"
  ),
  "--target",
  "web",
  "--out-dir",
  output,
  "--out-name",
  "portable_wasm",
])
const wasmBytes = await readFile(join(output, "portable_wasm_bg.wasm"))
const wasmModule = await WebAssembly.compile(wasmBytes)
assert(
  !WebAssembly.Module.imports(wasmModule).some((item) =>
    /wasi|filesystem|fd_read|fd_write/.test(item.module + item.name)
  )
)
const wasm = await import(pathToFileURL(join(output, "portable_wasm.js")).href)
await wasm.default({ module_or_path: wasmBytes })

type Request = {
  settings: string
  entry: string
  declaration: string
  hostModule: { specifier: string; exactIdentity: string }
  previousMetadata?: string
}
const fixtures = [
  ["dts-basic-conversion", "api", "fixture-api"],
  ["dts-callback-during-call", "api", "callback-api"],
  ["dts-declaration-merge", "merge", "merge-api"],
  ["dts-generated-name", "naming", "naming-api"],
  ["dts-namespace-runtime", "analytics", "analytics"],
  ["dts-overload-selection", "parser", "parser-api"],
  ["dts-unsupported-any", "api", "unsafe-api"],
  ["dts-callback-missing-release", "api", "watch-api"],
] as const
const requests: Request[] = []
for (const [name, entry, specifier] of fixtures) {
  const fixture = join(root, "examples/spec/fixtures/projects", name)
  requests.push({
    settings: await readFile(join(fixture, "seseragi.bindings.toml"), "utf8"),
    entry,
    declaration: await readFile(join(fixture, "host/index.d.ts"), "utf8"),
    hostModule: { specifier, exactIdentity: `${specifier}@1.0.0` },
  })
}
const initial = requests.map((request) =>
  JSON.parse(wasm.convert_declaration(JSON.stringify(request)))
)
for (const [index, response] of initial.entries()) {
  const [name, , outputName] = fixtures[index]!
  assert.equal(response.status, "success", name)
  if (index < 6) {
    assert(response.result.generated, name)
    assert.equal(
      response.result.generated.source,
      await readFile(
        join(
          root,
          "examples/spec/fixtures/projects",
          name,
          "expected",
          `${outputName}.ssrg`
        ),
        "utf8"
      ),
      name
    )
    requests.push({
      ...requests[index]!,
      previousMetadata: response.result.generated.metadata,
    })
  } else {
    assert.equal(response.result.generated, null, name)
    assert(
      response.result.diagnostics.some(
        (item: { severity: string }) => item.severity === "Error"
      ),
      name
    )
  }
}
const base = requests[0]!
requests.push(
  { ...base, declaration: "export declare function (" },
  { ...base, entry: "missing" },
  { ...base, hostModule: { specifier: "wrong", exactIdentity: "wrong@1.0.0" } },
  { ...base, settings: "schema = 99\n" }
)
const actual = requests.map((request) =>
  JSON.parse(wasm.convert_declaration(JSON.stringify(request)))
)
const native = JSON.parse(
  await command(
    [
      cargo,
      "run",
      "--quiet",
      "--locked",
      "-p",
      "seseragi-dts",
      "--no-default-features",
      "--example",
      "portable_native",
    ],
    JSON.stringify(requests)
  )
)
assert.deepEqual(
  actual,
  native,
  "native and WASM source, metadata, report and diagnostics must match exactly"
)
console.log(
  `Portable .d.ts conversion passed: ${requests.length} native/WASM comparisons; 6 canonical output snapshots; no filesystem imports.`
)
