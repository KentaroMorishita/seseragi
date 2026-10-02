import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import ts from "typescript"
import {
  terminalReaderCases,
  terminalReaderExamples,
  terminalReaderRoutes,
} from "../scripts/terminal-reader"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const runtimeRegistryPath = join(
  root,
  "apps/playground/src/runtime/runtime-modules.ts"
)
const { runtimeModules } = await import(runtimeRegistryPath)
const browserHostPath = join(root, "runtime/ts/src/browser/host.ts")
const effectRuntimePath = join(root, "runtime/ts/src/effect.ts")
const { createBrowserEnvironment } = await import(browserHostPath)
const { createEffectExecution, run } = await import(effectRuntimePath)
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const node = process.env.SESERAGI_NODE ?? "node"
const evidenceRoot = resolve(
  process.env.TERMINAL_READER_EVIDENCE_DIR ??
    join(tmpdir(), "seseragi-terminal-reader-evidence")
)
mkdirSync(evidenceRoot, { recursive: true })
const evidence = mkdtempSync(join(evidenceRoot, "run-"))
const fixtures = join(root, "apps/site/tests/fixtures/terminal-reader")
const examples = terminalReaderExamples("https://seseragi.vercel.app/")
const demoDirectory = "/tmp/seseragi-process-demo"
// The demo directory may preexist: never read its files or remove it.
mkdirSync(demoDirectory, { recursive: true })
for (const part of ["home", "tmp"]) {
  mkdirSync(join(evidence, part))
}
// Never inherit the user's environment or inspect real settings in subprocesses.
const baseEnvironment: Record<string, string> = {
  PATH: `${dirname(bun)}:${dirname(node)}:/usr/bin:/bin`,
  HOME: join(evidence, "home"),
  TMPDIR: join(evidence, "tmp"),
  LANG: "C.UTF-8",
  SESERAGI_DOCS_MODE: "preview",
}
const commands: unknown[] = []
const sha256 = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")
function save(name: string, value: unknown) {
  writeFileSync(join(evidence, name), `${JSON.stringify(value, null, 2)}\n`)
}
function command(
  label: string,
  executable: string,
  args: string[],
  cwd = evidence,
  environment = baseEnvironment
) {
  const result = spawnSync(executable, args, {
    cwd,
    env: environment,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  })
  commands.push({
    label,
    executable,
    args,
    cwd,
    environment,
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    stdout: result.stdout,
    stderr: result.stderr,
  })
  save("commands.json", commands)
  writeFileSync(join(evidence, `${label}.stdout.txt`), result.stdout ?? "")
  writeFileSync(join(evidence, `${label}.stderr.txt`), result.stderr ?? "")
  return result
}
function sample(slug: string) {
  const result = examples.find((item) => item.id === `terminal-reader-${slug}`)
  if (!result) throw new Error(`Missing terminal source: ${slug}`)
  return result
}
function succeeded(result: ReturnType<typeof command>, output?: string) {
  expect(result.error, result.stderr).toBeUndefined()
  expect(result.status, result.stdout + result.stderr).toBe(0)
  expect(result.stderr).toBe("")
  if (output !== undefined) expect(result.stdout).toBe(output)
}
const entries = new Map<string, string>()
function buildSource(slug: string, source = sample(slug).source) {
  const prior = entries.get(slug)
  if (prior) return prior
  const directory = join(evidence, "native", slug)
  mkdirSync(directory, { recursive: true })
  writeFileSync(join(directory, "main.ssrg"), source)
  succeeded(
    command(
      `${slug}.format`,
      cli,
      ["format", "--check", "main.ssrg"],
      directory
    )
  )
  succeeded(
    command(
      `${slug}.lint`,
      cli,
      ["lint", "main.ssrg", "--deny-warnings"],
      directory
    )
  )
  succeeded(
    command(
      `${slug}.build`,
      cli,
      [
        "build",
        "main.ssrg",
        "--target",
        "process",
        "--profile",
        "release",
        "--out-dir",
        "release",
      ],
      directory
    )
  )
  const manifest = JSON.parse(
    readFileSync(join(directory, "release/.seseragi-build.json"), "utf8")
  )
  expect(manifest).toMatchObject({
    profile: "release",
    target: "process",
    entry: "entry.js",
  })
  const entry = join(directory, "release", manifest.entry)
  entries.set(slug, entry)
  return entry
}
const argv = ["alpha", "two words", "", "--preview", "設定"]
const sourcePaths = [
  ...examples.map((item) => item.sourcePath),
  "apps/site/scripts/terminal-reader.ts",
  "apps/site/tests/terminal-reader.test.ts",
  "apps/site/tests/fixtures/terminal-reader/invalid/maybe-as-string.ssrg",
  "apps/site/tests/fixtures/terminal-reader/repairs/maybe-as-string.ssrg",
  "examples/spec/artifacts/stdlib-schema-1/reference/module.json",
  "apps/playground/src/wasm/pkg/seseragi_wasm.js",
  "apps/playground/src/wasm/pkg/seseragi_wasm_bg.wasm",
  "apps/playground/src/runtime/browser-execution.ts",
  "apps/playground/src/runtime/runtime-modules.ts",
  "runtime/ts/src/browser/host.ts",
  "runtime/ts/src/browser/console.ts",
  "runtime/ts/src/effect.ts",
]
save(
  "source-hashes.json",
  sourcePaths.map((path) => ({
    path,
    sha256: sha256(readFileSync(join(root, path))),
  }))
)

test("Terminal9 preserves exact compiler metadata and fourteen source panels", () => {
  expect(terminalReaderCases).toHaveLength(7)
  expect(terminalReaderRoutes).toHaveLength(9)
  expect(examples).toHaveLength(14)
  expect(terminalReaderRoutes.map((item) => item.identity)).toEqual([
    "std/prelude::println",
    "std/prelude::print",
    "std/prelude::printValue",
    "std/process",
    "std/process::Process",
    "std/process::ProcessError",
    "std/process::environment",
    "std/process::arguments",
    "std/process::currentDirectory",
  ])
  expect(terminalReaderRoutes.map((item) => item.slug)).toEqual([
    "prelude-println",
    "prelude-print",
    "prelude-printvalue",
    "process-module",
    "process-type",
    "process-error",
    "environment",
    "arguments",
    "current-directory",
  ])
  expect(new Set(terminalReaderRoutes.map((item) => item.identity)).size).toBe(
    9
  )
  const catalog = JSON.parse(
    readFileSync(
      join(
        root,
        "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
      ),
      "utf8"
    )
  )
  for (const route of terminalReaderRoutes) {
    const module = catalog.modules.find(
      (item: { specifier: string }) => item.specifier === route.module
    )
    expect(module).toBeDefined()
    if (route.namespace === "module") {
      expect([
        route.identity,
        route.module,
        route.namespace,
        route.kind,
        route.name,
        route.route,
      ]).toEqual([
        "std/process",
        "std/process",
        "module",
        "module",
        "std/process",
        "/docs/library/process/",
      ])
    } else {
      const item = module.items.find(
        (item: { identity: string }) => item.identity === route.identity
      )
      expect(item).toMatchObject({
        identity: route.identity,
        module: route.module,
        name: route.name,
        namespace: route.namespace,
        kind: route.kind,
      })
      expect(route.route).toBe(
        `/docs/library/${route.module.slice(4)}/${route.kind}/${route.name.toLowerCase()}/`
      )
    }
    expect(sample(route.example).source).toContain("pub effect fn main")
  }
  for (const item of terminalReaderCases) {
    for (const suffix of ["", "-ts"]) {
      const example = sample(`${item.slug}${suffix}`)
      expect(example.sha256).toBe(sha256(example.source))
      expect(example.highlighted.map((span) => span.text).join("")).toBe(
        example.source
      )
      expect(example.sourcePath).toStartWith(
        "apps/site/examples/src/terminal-reader/"
      )
      if (!item.portable || suffix) expect(example.playgroundUrl).toBe("")
      else
        expect(new URL(example.playgroundUrl).searchParams.get("source")).toBe(
          example.source
        )
    }
    if (!item.portable) {
      expect(sample(item.slug).source).toContain(
        "with Console, process: process.Process fails ConsoleError"
      )
      expect(sample(item.slug).source.match(/effects\.attempt/g)).toHaveLength(
        1
      )
      expect(sample(item.slug).source.match(/match result/g)).toHaveLength(1)
    }
  }
  save("routes.json", terminalReaderRoutes)
})

test("each exact source formats, compiles and runs as a release entry on Node and Bun", () => {
  succeeded(command("cli-version", cli, ["--version-json"]))
  succeeded(command("node-version", node, ["--version"]))
  succeeded(command("bun-version", bun, ["--version"]))
  for (const item of terminalReaderCases) {
    const entry = buildSource(item.slug)
    for (const [runtime, executable] of [
      ["node", node],
      ["bun", bun],
    ]) {
      succeeded(
        command(
          `${item.slug}.${runtime}`,
          executable!,
          [entry, ...(item.slug === "arguments" ? argv : [])],
          demoDirectory
        ),
        item.output
      )
    }
  }
})

test("environment distinguishes preview, present-empty and missing; argv preserves empty values", () => {
  for (const [runtime, executable] of [
    ["node", node],
    ["bun", bun],
  ]) {
    const environmentEntry = buildSource("environment")
    const missing = { ...baseEnvironment }
    delete missing.SESERAGI_DOCS_MODE
    succeeded(
      command(
        `environment.empty.${runtime}`,
        executable!,
        [environmentEntry],
        demoDirectory,
        { ...baseEnvironment, SESERAGI_DOCS_MODE: "" }
      ),
      "Mode: []\n"
    )
    succeeded(
      command(
        `environment.missing.${runtime}`,
        executable!,
        [environmentEntry],
        demoDirectory,
        missing
      ),
      "Mode: preview (default)\n"
    )
    succeeded(
      command(
        `arguments.empty.${runtime}`,
        executable!,
        [buildSource("arguments")],
        demoDirectory
      ),
      "No arguments\n"
    )
  }
})

test("all exact TypeScript counterparts pass strict checking and Node/Bun stream parity", () => {
  const output = join(evidence, "typescript")
  mkdirSync(output)
  succeeded(
    command("strict-typescript", bun, [
      join(root, "node_modules/typescript/bin/tsc"),
      "--strict",
      "--noUncheckedIndexedAccess",
      "--exactOptionalPropertyTypes",
      "--skipLibCheck",
      "--target",
      "ES2022",
      "--lib",
      "ESNext,DOM",
      "--module",
      "ESNext",
      "--moduleResolution",
      "bundler",
      "--outDir",
      output,
      "--types",
      "node",
      "--typeRoots",
      join(root, "node_modules/@types"),
      ...terminalReaderCases.map((item) =>
        join(root, sample(`${item.slug}-ts`).sourcePath)
      ),
    ])
  )
  writeFileSync(join(output, "package.json"), '{"type":"module"}\n')
  for (const item of terminalReaderCases) {
    const entry = join(output, `${item.slug}.js`)
    for (const [runtime, executable] of [
      ["node", node],
      ["bun", bun],
    ]) {
      succeeded(
        command(
          `${item.slug}.typescript.${runtime}`,
          executable!,
          [entry, ...(item.slug === "arguments" ? argv : [])],
          demoDirectory
        ),
        item.output
      )
      if (item.slug === "environment") {
        const missing = { ...baseEnvironment }
        delete missing.SESERAGI_DOCS_MODE
        succeeded(
          command(
            `environment.typescript.empty.${runtime}`,
            executable!,
            [entry],
            demoDirectory,
            { ...baseEnvironment, SESERAGI_DOCS_MODE: "" }
          ),
          "Mode: []\n"
        )
        succeeded(
          command(
            `environment.typescript.missing.${runtime}`,
            executable!,
            [entry],
            demoDirectory,
            missing
          ),
          "Mode: preview (default)\n"
        )
      }
      if (item.slug === "arguments")
        succeeded(
          command(
            `arguments.typescript.empty.${runtime}`,
            executable!,
            [entry],
            demoDirectory
          ),
          "No arguments\n"
        )
    }
  }
})

test("a real Maybe-as-String diagnostic has a complete executable repair", () => {
  const source = readFileSync(
    join(fixtures, "invalid/maybe-as-string.ssrg"),
    "utf8"
  )
  const directory = join(evidence, "negative")
  mkdirSync(directory)
  writeFileSync(join(directory, "main.ssrg"), source)
  const rejected = command(
    "maybe-as-string.rejected",
    cli,
    ["lint", "main.ssrg"],
    directory
  )
  expect(rejected.status, rejected.stderr).toBe(2)
  expect(rejected.stdout).toBe("")
  expect(rejected.stderr).toContain("SES-T0101")
  expect(rejected.stderr).toContain("expected: String")
  expect(rejected.stderr).toContain("actual: Maybe<String>")
  const repair = readFileSync(
    join(fixtures, "repairs/maybe-as-string.ssrg"),
    "utf8"
  )
  expect(repair).toBe(sample("environment").source)
  const entry = buildSource("maybe-as-string-repair", repair)
  for (const [runtime, executable] of [
    ["node", node],
    ["bun", bun],
  ]) {
    succeeded(
      command(
        `maybe-as-string.repaired.${runtime}`,
        executable!,
        [entry],
        demoDirectory
      ),
      "Mode: [preview]\n"
    )
  }
  save("negative-repair.json", {
    diagnostic: "SES-T0101",
    expected: "String",
    actual: "Maybe<String>",
    sourceSha256: sha256(source),
    repairSha256: sha256(repair),
    canonicalRepair: "terminal-reader-environment",
  })
})

test("the real CLI rejects argument forwarding forms and the browser Process target", () => {
  const directory = join(evidence, "cli-boundaries")
  mkdirSync(directory)
  writeFileSync(join(directory, "main.ssrg"), sample("arguments").source)
  for (const [label, args, diagnostic] of [
    [
      "double-dash",
      ["run", "main.ssrg", "--", "alpha"],
      "unknown run option `--`",
    ],
    [
      "positional",
      ["run", "main.ssrg", "alpha"],
      "unexpected run argument `alpha`",
    ],
    [
      "web-target",
      ["run", "main.ssrg", "--target", "web"],
      "run does not support the `web` target",
    ],
    [
      "web-build",
      ["build", "main.ssrg", "--target", "web", "--out-dir", "web-artifact"],
      "SES-K0203 provider.target-mismatch",
    ],
  ] as const) {
    const rejected = command(`cli.${label}`, cli, [...args], directory)
    expect(rejected.status, rejected.stderr).toBe(2)
    expect(rejected.stdout).toBe("")
    expect(rejected.stderr).toContain(diagnostic)
    if (label === "web-build")
      expect(rejected.stderr).toContain("missing capabilities: process")
  }
})

test("committed WASM/current browser execution proves exact portable seeds and raw newline behavior", async () => {
  const bindings = await import(
    new URL("../../playground/src/wasm/pkg/seseragi_wasm.js", import.meta.url)
      .href
  )
  await bindings.default({
    module_or_path: readFileSync(
      join(root, "apps/playground/src/wasm/pkg/seseragi_wasm_bg.wasm")
    ),
  })
  const { executeGeneratedModule } = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  const results: unknown[] = []
  save("wasm-version.json", JSON.parse(bindings.toolchain_version_json()))
  for (const item of terminalReaderCases) {
    const source = sample(item.slug)
    const seed = item.portable
      ? new URL(source.playgroundUrl).searchParams.get("source")!
      : source.source
    expect(seed).toBe(source.source)
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    )
    save(`${item.slug}.wasm-compiled.json`, compiled)
    expect(compiled.status, JSON.stringify(compiled)).toBe("success")
    if (!item.portable) {
      expect(compiled.entry.environment).toContainEqual({
        field: "process",
        service: "process",
      })
      let rejection: unknown
      try {
        await executeGeneratedModule(
          compiled.generated.typescript,
          compiled.entry
        )
      } catch (error) {
        rejection = error
      }
      expect(String(rejection)).toContain(
        "unsupported playground runtime module: @seseragi/runtime/process"
      )
      results.push({
        slug: item.slug,
        sourceSha256: source.sha256,
        browserRejected: true,
        error: String(rejection),
        entry: compiled.entry,
      })
      save("wasm-results.json", results)
      continue
    }
    expect(compiled.entry.environment).toEqual([
      { field: "console", service: "console" },
    ])
    expect(compiled.entry.providers ?? []).toEqual([])
    const execution = await executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(execution).toEqual({ stdout: item.output.trimEnd(), debug: "()" })
    // Production output intentionally trims trailing whitespace. Supplemental
    // raw host callbacks prove newlines using the same generated source/runtime.
    const javascript = ts.transpileModule(compiled.generated.typescript, {
      compilerOptions: {
        target: ts.ScriptTarget.ES2022,
        module: ts.ModuleKind.CommonJS,
        strict: true,
      },
    }).outputText
    const module = { exports: {} as Record<string, unknown> }
    const requireRuntime = (specifier: string) => {
      const resolved = (runtimeModules as Record<string, unknown>)[specifier]
      if (resolved === undefined)
        throw new Error(`Unsupported runtime import: ${specifier}`)
      return resolved
    }
    new Function("require", "module", "exports", javascript)(
      requireRuntime,
      module,
      module.exports
    )
    const chunks: string[] = []
    const effectExecution = createEffectExecution()
    try {
      const environment = createBrowserEnvironment(
        compiled.entry.environment,
        "",
        (text) => chunks.push(text),
        undefined,
        effectExecution.context
      )
      const main = module.exports.main as (
        unit: undefined
      ) => Parameters<typeof run>[0]
      expect(typeof main).toBe("function")
      const outcome = await run(
        main(undefined),
        environment,
        effectExecution.context
      )
      expect(outcome.kind).toBe("success")
      expect(chunks.join("")).toBe(item.output)
      results.push({
        slug: item.slug,
        sourceSha256: source.sha256,
        seedSha256: sha256(seed),
        generatedTypeScriptSha256: sha256(compiled.generated.typescript),
        entry: compiled.entry,
        productionBrowserExecution: execution,
        supplementalRawHost: {
          chunks,
          stdout: chunks.join(""),
          status: outcome.kind,
        },
      })
    } finally {
      await effectExecution.close()
    }
    save("wasm-results.json", results)
  }
})
