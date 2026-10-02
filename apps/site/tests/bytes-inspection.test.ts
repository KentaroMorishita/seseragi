import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import {
  bytesInspectionCases,
  bytesInspectionExamples,
  bytesInspectionRoutes,
} from "../scripts/bytes-inspection"
import { compilerReferenceModules } from "../scripts/reference"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const node = process.env.SESERAGI_NODE ?? "node"
const base = resolve(
  process.env.BYTES_INSPECTION_EVIDENCE_DIR ??
    join(root, "target/bytes-inspection-evidence")
)
mkdirSync(base, { recursive: true })
const evidence = mkdtempSync(join(base, "semantic-"))
const examples = bytesInspectionExamples("https://seseragi.vercel.app/")
const fixtures = join(root, "apps/site/tests/fixtures/bytes-inspection")
const commands: unknown[] = []
function command(
  label: string,
  executable: string,
  args: string[],
  cwd = evidence
) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    timeout: 90_000,
    maxBuffer: 16 * 1024 * 1024,
  })
  commands.push({
    label,
    executable,
    args,
    cwd,
    status: result.status,
    stdoutSha256: createHash("sha256")
      .update(result.stdout ?? "")
      .digest("hex"),
    stderrSha256: createHash("sha256")
      .update(result.stderr ?? "")
      .digest("hex"),
    error: result.error?.message,
    stdout: result.stdout,
    stderr: result.stderr,
  })
  writeFileSync(
    join(evidence, "commands.json"),
    JSON.stringify(commands, null, 2)
  )
  writeFileSync(join(evidence, `${label}.stdout.txt`), result.stdout ?? "")
  writeFileSync(join(evidence, `${label}.stderr.txt`), result.stderr ?? "")
  return result
}
function sample(slug: string, suffix = "") {
  const value = examples.find(
    (item) => item.id === `bytes-inspection-${slug}${suffix}`
  )
  if (!value) throw new Error(`Missing source ${slug}${suffix}`)
  return value
}
function checked(label: string, executable: string, args: string[]) {
  const result = command(label, executable, args)
  expect(result.status, result.stdout + result.stderr).toBe(0)
  return result.stdout
}
function strict(label: string, types: "bun" | "node", paths: string[]) {
  checked(label, node, [
    join(root, "node_modules/typescript/bin/tsc"),
    "--strict",
    "--noUncheckedIndexedAccess",
    "--noEmit",
    "--skipLibCheck",
    "--allowImportingTsExtensions",
    "--types",
    types,
    "--typeRoots",
    join(root, "node_modules/@types"),
    "--target",
    "ES2022",
    "--module",
    "Preserve",
    "--moduleResolution",
    "Bundler",
    ...paths,
  ])
}

test("Bytes inspection11 selects eleven existing portable identities and six exact source pairs", () => {
  expect(bytesInspectionRoutes).toHaveLength(11)
  expect(bytesInspectionCases).toHaveLength(6)
  expect(examples).toHaveLength(12)
  expect(new Set(bytesInspectionRoutes.map((item) => item.identity)).size).toBe(
    11
  )
  expect(
    bytesInspectionRoutes.filter((item) => (item.kind as string) === "module")
  ).toHaveLength(0)
  expect(
    bytesInspectionRoutes.filter((item) => item.kind === "opaque-type")
  ).toHaveLength(2)
  expect(
    bytesInspectionRoutes.filter((item) => item.kind === "function")
  ).toHaveLength(9)
  const prior = JSON.parse(
    readFileSync(join(fixtures, "prior-authored-pages.json"), "utf8")
  ) as Array<{ route: string }>
  expect(prior).toHaveLength(344)
  for (const selected of bytesInspectionRoutes)
    expect(prior.some((entry) => entry.route === selected.route)).toBe(false)
  const modules = compilerReferenceModules()
  for (const route of bytesInspectionRoutes) {
    const module = modules.find((item) => item.specifier === route.module)!
    expect(module.targets).toEqual(["process", "browser"])
    if ((route.kind as string) !== "module") {
      expect(
        module.items.filter(
          (item) =>
            item.identity === route.identity &&
            item.namespace === route.namespace &&
            item.itemKind === route.kind
        )
      ).toHaveLength(1)
    }
    expect(route.route).toStartWith(
      `/docs/library/${route.module.replace("std/", "")}/`
    )
    expect(
      bytesInspectionCases.some((item) => item.slug === route.example)
    ).toBe(true)
  }
  for (const item of examples) {
    expect(item.sha256).toBe(
      createHash("sha256").update(item.source).digest("hex")
    )
    expect(item.highlighted.map((token) => token.text).join("")).toBe(
      item.source
    )
    if (item.id.endsWith("-ts")) expect(item.playgroundUrl).toBe("")
    else
      expect(new URL(item.playgroundUrl).searchParams.get("source")).toBe(
        item.source
      )
  }
  writeFileSync(
    join(evidence, "source-manifest.json"),
    JSON.stringify(
      examples.map(({ id, sourcePath, sha256 }) => ({
        id,
        sourcePath,
        sha256,
      })),
      null,
      2
    )
  )
})

test("exact sources pass process compilation, Bun and Node comparisons", () => {
  const results: unknown[] = []
  for (const item of bytesInspectionCases) {
    const directory = join(evidence, "sources", item.slug)
    mkdirSync(directory, { recursive: true })
    const source = sample(item.slug)
    const path = join(directory, "main.ssrg")
    writeFileSync(path, source.source)
    checked(`${item.slug}.format`, cli, ["format", "--check", path])
    checked(`${item.slug}.lint`, cli, ["lint", path, "--deny-warnings"])
    checked(`${item.slug}.build`, cli, [
      "build",
      path,
      "--out-dir",
      join(directory, "compiled"),
    ])
    const native = checked(`${item.slug}.process`, cli, [
      "run",
      path,
      "--target",
      "process",
    ])
    const bunOutput = checked(`${item.slug}.bun`, bun, [
      join(root, sample(item.slug, "-ts").sourcePath),
    ])
    const nodeOutput = checked(`${item.slug}.node`, node, [
      join(root, sample(item.slug, "-ts").sourcePath),
    ])
    expect(native).toBe(item.output)
    expect(bunOutput).toBe(item.output)
    expect(nodeOutput).toBe(item.output)
    results.push({
      slug: item.slug,
      sourceSha256: source.sha256,
      native,
      bunOutput,
      nodeOutput,
    })
  }
  strict(
    "strict-bun",
    "bun",
    bytesInspectionCases.map((item) =>
      join(root, sample(item.slug, "-ts").sourcePath)
    )
  )
  strict(
    "strict-node",
    "node",
    bytesInspectionCases.map((item) =>
      join(root, sample(item.slug, "-ts").sourcePath)
    )
  )
  writeFileSync(
    join(evidence, "exact-source-results.json"),
    JSON.stringify(results, null, 2)
  )
})

test("six exact Playground seeds compile and execute through committed WASM", async () => {
  const bindings = await import(
    new URL("../../playground/src/wasm/pkg/seseragi_wasm.js", import.meta.url)
      .href
  )
  await bindings.default({
    module_or_path: await Bun.file(
      new URL(
        "../../playground/src/wasm/pkg/seseragi_wasm_bg.wasm",
        import.meta.url
      )
    ).arrayBuffer(),
  })
  const { executeGeneratedModule } = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  const results: unknown[] = []
  for (const item of bytesInspectionCases) {
    const source = sample(item.slug)
    const compiled = JSON.parse(
      bindings.compile_single_file(
        "main.ssrg",
        "playground/main",
        new URL(source.playgroundUrl).searchParams.get("source")!
      )
    )
    writeFileSync(
      join(evidence, `${item.slug}.wasm.json`),
      JSON.stringify(compiled, null, 2)
    )
    expect(compiled.status, JSON.stringify(compiled)).toBe("success")
    expect(compiled.entry.environment).toEqual([
      { field: "console", service: "console" },
    ])
    expect(compiled.entry.providers ?? []).toEqual([])
    const result = await executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(result.stdout).toBe(item.output.trimEnd())
    results.push({
      slug: item.slug,
      sourceSha256: source.sha256,
      stdout: result.stdout,
      entry: compiled.entry,
    })
  }
  writeFileSync(
    join(evidence, "wasm-seeds.json"),
    JSON.stringify(results, null, 2)
  )
})

test("six exact type/export mistakes reject and their complete repairs run", () => {
  const cases = JSON.parse(
    readFileSync(join(fixtures, "negative-cases.json"), "utf8")
  ) as Array<{
    slug: string
    expectedRepairOutput: string
    diagnostic: {
      code: string
      messageKey: string
      actualType: string | null
      expectedType: string | null
    }
  }>
  expect(cases).toHaveLength(6)
  const results = []
  for (const item of cases) {
    const source = readFileSync(
      join(fixtures, "invalid", `${item.slug}.ssrg`),
      "utf8"
    )
    const path = join(evidence, `invalid-${item.slug}.ssrg`)
    writeFileSync(path, source)
    const result = command(`negative-${item.slug}`, cli, [
      "lint",
      path,
      "--diagnostic-format",
      "json",
    ])
    expect(result.status).toBe(2)
    const diagnostics = JSON.parse(result.stderr).diagnostics.flatMap(
      (file: {
        diagnostics: { diagnostics: Array<Record<string, unknown>> }
      }) => file.diagnostics.diagnostics
    )
    expect(diagnostics).toHaveLength(1)
    expect(diagnostics[0]).toMatchObject(item.diagnostic)
    const primary = diagnostics[0].primary as { start: number; end: number }
    expect(primary.end).toBeGreaterThan(primary.start)
    const repair = readFileSync(
      join(fixtures, "repairs", `${item.slug}.ssrg`),
      "utf8"
    )
    const repairPath = join(evidence, `repair-${item.slug}.ssrg`)
    writeFileSync(repairPath, repair)
    checked(`repair-${item.slug}.format`, cli, [
      "format",
      "--check",
      repairPath,
    ])
    checked(`repair-${item.slug}.lint`, cli, [
      "lint",
      repairPath,
      "--deny-warnings",
    ])
    expect(
      checked(`repair-${item.slug}.run`, cli, [
        "run",
        repairPath,
        "--target",
        "process",
      ])
    ).toBe(item.expectedRepairOutput)
    results.push({
      slug: item.slug,
      diagnostics,
      sourceSha256: createHash("sha256").update(source).digest("hex"),
      repairSha256: createHash("sha256").update(repair).digest("hex"),
      output: item.expectedRepairOutput,
    })
  }
  writeFileSync(
    join(evidence, "negative-repairs.json"),
    JSON.stringify(results, null, 2)
  )
})

test("31 byte/range/chunk observations preserve their exact boundaries and native TS differences stay explicit", () => {
  const source = readFileSync(join(fixtures, "boundaries.ssrg"), "utf8")
  const path = join(evidence, "boundaries.ssrg")
  writeFileSync(path, source)
  const expected = readFileSync(join(fixtures, "boundaries.stdout"), "utf8")
  checked("boundaries.format", cli, ["format", "--check", path])
  checked("boundaries.lint", cli, ["lint", path, "--deny-warnings"])
  checked("boundaries.build", cli, [
    "build",
    path,
    "--out-dir",
    join(evidence, "boundaries-build"),
  ])
  const output = checked("boundaries.process", cli, [
    "run",
    path,
    "--target",
    "process",
  ])
  expect(output).toBe(expected)
  expect(output.trimEnd().split("\n")).toHaveLength(31)
  const native = join(fixtures, "native-differences.ts")
  strict("native-differences.strict-bun", "bun", [native])
  strict("native-differences.strict-node", "node", [native])
  const nativeExpected =
    "construction-wraps|[255, 0, 1, 1]\nnative-slice-clamps|[0, 15, 255]\nnative-slice-negative|[255]\nnative-slice-reversed|[]\ncontent-after|[0, 15, 255]\n"
  expect(checked("native-differences.bun", bun, [native])).toBe(nativeExpected)
  expect(checked("native-differences.node", node, [native])).toBe(
    nativeExpected
  )
  writeFileSync(
    join(evidence, "boundaries.json"),
    JSON.stringify(
      {
        sourceSha256: createHash("sha256").update(source).digest("hex"),
        rows: 31,
        output,
        nativeExpected,
      },
      null,
      2
    )
  )
})

test("process and comparison tool versions are retained separately from browser-host evidence", () => {
  const versions = {
    cli: checked("version-cli", cli, ["--version"]),
    bun: checked("version-bun", bun, ["--version"]),
    node: checked("version-node", node, ["--version"]),
    typescript: checked("version-typescript", node, [
      join(root, "node_modules/typescript/bin/tsc"),
      "--version",
    ]),
  }
  expect(versions.node).toStartWith("v")
  writeFileSync(
    join(evidence, "tool-versions.json"),
    JSON.stringify(
      { executables: { cli, bun, node }, versions, actualBrowser: false },
      null,
      2
    )
  )
})
