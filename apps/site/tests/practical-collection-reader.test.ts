import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import {
  practicalCollectionCases,
  practicalCollectionExamples,
  practicalCollectionRoutes,
} from "../scripts/practical-collection"
import { compilerReferenceModules } from "../scripts/reference"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const node = process.env.SESERAGI_NODE ?? "node"
const base = resolve(
  process.env.PRACTICAL_COLLECTION_EVIDENCE_DIR ??
    join(root, "target/practical-collection-evidence")
)
mkdirSync(base, { recursive: true })
const evidence = mkdtempSync(join(base, "semantic-"))
const fixtures = join(root, "apps/site/tests/fixtures/practical-collection")
const examples = practicalCollectionExamples("https://seseragi.vercel.app/")
const sha = (source: string) =>
  createHash("sha256").update(source).digest("hex")
const commands: unknown[] = []
function command(label: string, executable: string, args: string[]) {
  const result = spawnSync(executable, args, {
    cwd: evidence,
    encoding: "utf8",
    timeout: 90_000,
    maxBuffer: 16 * 1024 * 1024,
  })
  commands.push({
    label,
    executable,
    args,
    cwd: evidence,
    status: result.status,
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
function checked(label: string, executable: string, args: string[]) {
  const result = command(label, executable, args)
  expect(result.status, result.stdout + result.stderr).toBe(0)
  return result.stdout
}
function strict(label: string, paths: string[]) {
  checked(label, node, [
    join(root, "node_modules/typescript/bin/tsc"),
    "--strict",
    "--noUncheckedIndexedAccess",
    "--noEmit",
    "--skipLibCheck",
    "--allowImportingTsExtensions",
    "--types",
    "node",
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
const boundaryCases = JSON.parse(
  readFileSync(join(fixtures, "boundary-cases.json"), "utf8")
) as Array<{ slug: string; expected: string }>
const cases = [
  ...practicalCollectionCases.map((item) => {
    const source = examples.find(
      (e) => e.id === `practical-collection-${item.slug}`
    )!
    const comparison = examples.find(
      (e) => e.id === `practical-collection-${item.slug}-ts`
    )!
    return {
      slug: item.slug,
      source: source.source,
      output: item.output,
      comparison: join(root, comparison.sourcePath),
      displayed: true,
    }
  }),
  ...boundaryCases.map((item) => ({
    slug: item.slug,
    source: readFileSync(
      join(fixtures, "boundaries", `${item.slug}.ssrg`),
      "utf8"
    ),
    output: item.expected,
    comparison: join(fixtures, "boundaries", `${item.slug}.ts`),
    displayed: false,
  })),
]

test("Collection20 selects exactly twenty existing portable functions and fourteen task pairs", () => {
  expect(practicalCollectionRoutes).toHaveLength(20)
  expect(practicalCollectionCases).toHaveLength(14)
  expect(examples).toHaveLength(28)
  expect(cases).toHaveLength(17)
  expect(new Set(practicalCollectionRoutes.map((r) => r.identity)).size).toBe(
    20
  )
  const modules = compilerReferenceModules()
  const counts = Object.fromEntries(
    ["std/array", "std/list", "std/map", "std/set"].map((module) => [
      module,
      practicalCollectionRoutes.filter((r) => r.module === module).length,
    ])
  )
  expect(counts).toEqual({
    "std/array": 8,
    "std/list": 8,
    "std/map": 2,
    "std/set": 2,
  })
  for (const route of practicalCollectionRoutes) {
    const module = modules.find((m) => m.specifier === route.module)!
    expect(module.targets).toEqual(["process", "browser"])
    expect(route.namespace).toBe("value")
    expect(route.kind).toBe("function")
    expect(
      module.items.filter(
        (item) =>
          item.identity === route.identity &&
          item.module === route.module &&
          item.namespace === route.namespace &&
          item.itemKind === route.kind
      )
    ).toHaveLength(1)
    expect(
      practicalCollectionCases.some((item) => item.slug === route.example)
    ).toBe(true)
    expect(String(route.route)).toBe(
      `/docs/library/${route.module.replace("std/", "")}/function/${route.name.toLowerCase()}/`
    )
  }
  for (const item of examples) {
    expect(item.sha256).toBe(sha(item.source))
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

test("seventeen exact task and boundary pairs compile and agree on process, Bun and Node", () => {
  checked("cli-version", cli, ["--version-json"])
  checked("bun-version", bun, ["--version"])
  checked("node-version", node, ["--version"])
  checked("typescript-version", node, [
    join(root, "node_modules/typescript/bin/tsc"),
    "--version",
  ])
  const results: unknown[] = []
  for (const item of cases) {
    const directory = join(evidence, "sources", item.slug)
    mkdirSync(directory, { recursive: true })
    const path = join(directory, "main.ssrg")
    writeFileSync(path, item.source)
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
    const bunOutput = checked(`${item.slug}.bun`, bun, [item.comparison])
    const nodeOutput = checked(`${item.slug}.node`, node, [item.comparison])
    expect(native).toBe(item.output)
    expect(bunOutput).toBe(item.output)
    expect(nodeOutput).toBe(item.output)
    const sourceMap = JSON.parse(
      readFileSync(join(directory, "compiled/main.ts.map"), "utf8")
    )
    expect(sourceMap.sourcesContent).toEqual([item.source])
    results.push({
      slug: item.slug,
      displayed: item.displayed,
      sourceSha256: sha(item.source),
      typescriptSha256: sha(readFileSync(item.comparison, "utf8")),
      native,
      bunOutput,
      nodeOutput,
    })
  }
  strict(
    "strict-comparisons",
    cases.map((item) => item.comparison)
  )
  writeFileSync(
    join(evidence, "exact-source-results.json"),
    JSON.stringify(results, null, 2)
  )
})

test("seventeen exact sources compile and execute through committed WASM with Console only", async () => {
  const bindings = await import(
    new URL("../../playground/src/wasm/pkg/seseragi_wasm.js", import.meta.url)
      .href
  )
  const wasmPath = new URL(
    "../../playground/src/wasm/pkg/seseragi_wasm_bg.wasm",
    import.meta.url
  )
  const bytes = await Bun.file(wasmPath).arrayBuffer()
  await bindings.default({ module_or_path: bytes })
  const { executeGeneratedModule } = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  const results: unknown[] = []
  for (const item of cases) {
    const source = item.displayed
      ? new URL(
          examples.find((e) => e.id === `practical-collection-${item.slug}`)!
            .playgroundUrl
        ).searchParams.get("source")!
      : item.source
    expect(source).toBe(item.source)
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", source)
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
      displayed: item.displayed,
      sourceSha256: sha(source),
      stdout: result.stdout,
      entry: compiled.entry,
    })
  }
  writeFileSync(
    join(evidence, "wasm-seeds.json"),
    JSON.stringify(
      {
        scope:
          "Committed WASM and current browser-runtime path under Bun; no actual browser-host session",
        version: JSON.parse(bindings.toolchain_version_json()),
        wasmSha256: createHash("sha256")
          .update(Buffer.from(bytes))
          .digest("hex"),
        results,
      },
      null,
      2
    )
  )
})

test("seven isolated type mistakes reject and complete repairs produce their checked result", () => {
  const negatives = JSON.parse(
    readFileSync(join(fixtures, "negative-cases.json"), "utf8")
  ) as Array<{
    slug: string
    expected: string
    actualType: string
    expectedType: string
    messageKey: string
  }>
  expect(negatives).toHaveLength(7)
  const results: unknown[] = []
  for (const item of negatives) {
    const source = readFileSync(
      join(fixtures, "invalid", `${item.slug}.ssrg`),
      "utf8"
    )
    const path = join(evidence, `${item.slug}.ssrg`)
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
        diagnostics: {
          diagnostics: Array<{
            code: string
            messageKey: string
            actualType: string
            expectedType: string
            primary: { start: number; end: number }
          }>
        }
      }) => file.diagnostics.diagnostics
    )
    expect(diagnostics).toHaveLength(1)
    expect(diagnostics[0]).toMatchObject({
      code: "SES-T0101",
      messageKey: item.messageKey,
      actualType: item.actualType,
      expectedType: item.expectedType,
    })
    expect(diagnostics[0].primary.end).toBeGreaterThan(
      diagnostics[0].primary.start
    )
    const repaired = readFileSync(
      join(fixtures, "repairs", `${item.slug}.ssrg`),
      "utf8"
    )
    const repairPath = join(evidence, `${item.slug}-repair.ssrg`)
    writeFileSync(repairPath, repaired)
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
    const output = checked(`repair-${item.slug}.process`, cli, [
      "run",
      repairPath,
    ])
    expect(output).toBe(item.expected)
    results.push({
      slug: item.slug,
      sourceSha256: sha(source),
      repairedSha256: sha(repaired),
      diagnostics,
      output,
    })
  }
  writeFileSync(
    join(evidence, "negative-repairs.json"),
    JSON.stringify(results, null, 2)
  )
})

test("runtime probes preserve callback order, short-circuiting, empty cases and persistent inputs", () => {
  const path = join(fixtures, "runtime-contracts.ts")
  strict("runtime-contracts-strict", [path])
  const output = checked("runtime-contracts", bun, [path])
  const result = JSON.parse(output)
  expect(result.sequence.arrayFindCalls).toEqual([0, 2])
  expect(result.sequence.listFindCalls).toEqual([0, 2])
  expect(result.sequence.arrayFilterCalls).toEqual([0, 2, 0, 3])
  expect(result.sequence.listFilterCalls).toEqual([0, 2, 0, 3])
  expect(result.sequence.originalPreserved).toBe(true)
  expect(result.sequence.listDropSharesTail).toBe(true)
  expect(result.mapSet.mapCalls).toEqual([
    ["tea", 2],
    ["hold", 5],
    ["coffee", 3],
    ["empty", 0],
  ])
  expect(result.mapSet.setCalls).toEqual([4, 1, 3, 0])
  expect(result.emptyCallbacks).toBe(0)
  writeFileSync(
    join(evidence, "runtime-results.json"),
    JSON.stringify(
      { sourceSha256: sha(readFileSync(path, "utf8")), ...result },
      null,
      2
    )
  )
})
