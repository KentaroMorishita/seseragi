import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import {
  jsonReaderCases,
  jsonReaderExamples,
  jsonReaderRoutes,
} from "../scripts/json-reader"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const evidence = resolve(
  process.env.JSON_READER_EVIDENCE_DIR ??
    join(root, "target/json-reader-evidence")
)
mkdirSync(evidence, { recursive: true })
const examples = jsonReaderExamples("https://seseragi.vercel.app/")
const fixtures = join(root, "apps/site/tests/fixtures/json-reader")
const commands: unknown[] = []
function command(
  label: string,
  executable: string,
  args: string[],
  cwd: string
) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  })
  commands.push({
    label,
    executable,
    args,
    cwd,
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
  })
  writeFileSync(
    join(evidence, "commands.json"),
    JSON.stringify(commands, null, 2)
  )
  return result
}
function sourceIn(source: string, directory: string) {
  writeFileSync(join(directory, "main.ssrg"), source)
}

test("JSON20 uses twenty exact identities and twelve honest native/strict TypeScript pairs", () => {
  expect(jsonReaderRoutes).toHaveLength(20)
  expect(new Set(jsonReaderRoutes.map((x) => x.identity)).size).toBe(20)
  expect(jsonReaderCases).toHaveLength(12)
  expect(examples).toHaveLength(24)
  const directory = mkdtempSync(join(evidence, "native-"))
  for (const item of jsonReaderCases) {
    const source = examples.find((x) => x.id === `json-reader-${item.slug}`)!
    sourceIn(source.source, directory)
    const lint = command(
      `${item.slug}.lint`,
      cli,
      ["lint", "main.ssrg", "--deny-warnings"],
      directory
    )
    expect(lint.status, lint.stderr).toBe(0)
    const format = command(
      `${item.slug}.format`,
      cli,
      ["format", "--check", "main.ssrg"],
      directory
    )
    expect(format.status, format.stderr).toBe(0)
    const native = command(
      `${item.slug}.native`,
      cli,
      ["run", "main.ssrg"],
      directory
    )
    expect(native.status, native.stderr).toBe(0)
    expect(native.stderr).toBe("")
    expect(native.stdout).toBe(item.output)
    expect(source.sha256).toBe(
      createHash("sha256").update(source.source).digest("hex")
    )
    expect(new URL(source.playgroundUrl).searchParams.get("source")).toBe(
      source.source
    )
    expect(source.highlighted.map((x) => x.text).join("")).toBe(source.source)
    const comparison = examples.find(
      (x) => x.id === `json-reader-${item.slug}-ts`
    )!
    expect(comparison.playgroundUrl).toBe("")
    expect(comparison.highlighted.map((x) => x.text).join("")).toBe(
      comparison.source
    )
    const ts = command(
      `${item.slug}.typescript`,
      bun,
      [join(root, comparison.sourcePath)],
      directory
    )
    expect(ts.status, ts.stderr).toBe(0)
    expect(ts.stdout).toBe(item.output)
  }
  const checked = command(
    "strict-typescript",
    bun,
    [
      join(root, "node_modules/typescript/bin/tsc"),
      "--noEmit",
      "--strict",
      "--skipLibCheck",
      "--target",
      "ES2022",
      "--lib",
      "ESNext,DOM",
      "--module",
      "ESNext",
      "--moduleResolution",
      "bundler",
      ...jsonReaderCases.map((x) =>
        join(root, `apps/site/examples/comparisons/json-reader/${x.slug}.ts`)
      ),
    ],
    directory
  )
  expect(checked.status, checked.stdout + checked.stderr).toBe(0)
})

test("the twelve exact source seeds compile and execute through committed WASM", async () => {
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
  const browser = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  const results: unknown[] = []
  for (const item of jsonReaderCases) {
    const sample = examples.find((x) => x.id === `json-reader-${item.slug}`)!
    const seed = new URL(sample.playgroundUrl).searchParams.get("source")!
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    )
    expect(compiled.status, `${item.slug}: ${JSON.stringify(compiled)}`).toBe(
      "success"
    )
    expect(compiled.entry).toBeDefined()
    expect(compiled.entry.environment).toEqual([
      { field: "console", service: "console" },
    ])
    expect(compiled.entry.providers ?? []).toEqual([])
    const result = await browser.executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(result.stdout).toBe(item.output.trimEnd())
    results.push({
      slug: item.slug,
      sourceSha256: sample.sha256,
      entry: compiled.entry,
      stdout: result.stdout,
    })
    writeFileSync(
      join(evidence, "wasm-seeds.json"),
      JSON.stringify(results, null, 2)
    )
  }
})

test("twenty-four genuine JSON mistakes produce focused diagnostics and useful repairs execute", () => {
  const directory = mkdtempSync(join(evidence, "negative-"))
  const cases: Array<[string, string, string]> = [
    ["construct-decode-error", "SES-T0101", "struct.representation-private"],
    ["pattern-decode-error", "SES-T0101", "match.pattern-type-mismatch"],
    ["decode-text-directly", "SES-T0101", "call.argument-type-mismatch"],
    ["decoder-wrong-input", "SES-T0101", "binding.annotation-type-mismatch"],
    ["mixed-record", "SES-T0101", "array.element-type-mismatch"],
    ["jsonobject-from-pairs", "SES-T0101", "call.argument-type-mismatch"],
    ...["float-codec", "bigint-codec", "no-derived-codec"].map(
      (x) => [x, "SES-T0201", "instance.missing"] as [string, string, string]
    ),
    ...["eq", "show", "debug"].flatMap((trait) =>
      [
        "decodeerror",
        "decodeerrorkind",
        "jsonparseerror",
        "jsonpathsegment",
        "jsonreaderror",
      ].map(
        (type) =>
          [`${trait}-${type}`, "SES-T0201", "instance.missing"] as [
            string,
            string,
            string,
          ]
      )
    ),
  ]
  expect(cases).toHaveLength(24)
  for (const [slug, code, key] of cases) {
    sourceIn(
      readFileSync(join(fixtures, "invalid", `${slug}.ssrg`), "utf8"),
      directory
    )
    const result = command(
      slug,
      cli,
      ["lint", "main.ssrg", "--diagnostic-format", "json"],
      directory
    )
    expect(result.status, result.stderr).toBe(2)
    const diagnostics = JSON.parse(result.stderr).diagnostics.flatMap(
      (file: any) => file.diagnostics.diagnostics
    )
    expect(diagnostics).toHaveLength(1)
    expect(diagnostics[0].code).toBe(code)
    expect(diagnostics[0].messageKey).toBe(key)
  }
  for (const [slug, output] of [
    [
      "custom-domain-validation",
      "accepted: 2\nretries must be nonnegative\nshape rejected\n",
    ],
    ["decimal-and-bigint-encoding", '1.25\n"42"\n'],
    ["jsonobject-from-map", '{"name":"Mio"}\n'],
  ]) {
    sourceIn(
      readFileSync(join(fixtures, "repairs", `${slug}.ssrg`), "utf8"),
      directory
    )
    const result = command(
      `${slug}.repair`,
      cli,
      ["run", "main.ssrg"],
      directory
    )
    expect(result.status, result.stderr).toBe(0)
    expect(result.stdout).toBe(output)
  }
})

test("JSON runtime contract harness passes its strict check and thirty-four assertions", () => {
  const probe = join(fixtures, "runtime-contracts.ts")
  const checked = command(
    "runtime-contracts-strict",
    bun,
    [
      join(root, "node_modules/typescript/bin/tsc"),
      "--noEmit",
      "--strict",
      "--skipLibCheck",
      "--target",
      "ES2022",
      "--lib",
      "ESNext,DOM",
      "--module",
      "ESNext",
      "--moduleResolution",
      "bundler",
      probe,
    ],
    evidence
  )
  expect(checked.status, checked.stdout + checked.stderr).toBe(0)
  const result = command("runtime-contracts", bun, [probe], evidence)
  expect(result.status, result.stderr).toBe(0)
  const verified = JSON.parse(result.stdout)
  expect(verified.assertions).toBe(34)
  writeFileSync(
    join(evidence, "runtime-contracts.json"),
    JSON.stringify(verified, null, 2)
  )
})

test("the parse-error example reports a repeated key on an explicitly different input", () => {
  const original = examples.find(
    (item) => item.id === "json-reader-parse-errors"
  )!.source
  const source = original.replace(
    JSON.stringify('{"name":}'),
    JSON.stringify('{"name":"Mio","name":"Aki"}')
  )
  expect(source).not.toBe(original)
  const directory = mkdtempSync(join(evidence, "duplicate-field-"))
  sourceIn(source, directory)
  const result = command(
    "parse-errors-duplicate",
    cli,
    ["run", "main.ssrg"],
    directory
  )
  expect(result.status, result.stderr).toBe(0)
  expect(result.stdout).toBe(
    'Valid JSON: {"name":"Mio"}\nRepeated field: name\n'
  )
  writeFileSync(
    join(evidence, "duplicate-field-probe.json"),
    JSON.stringify(
      {
        source,
        sourceSha256: createHash("sha256").update(source).digest("hex"),
        stdout: result.stdout,
      },
      null,
      2
    )
  )
})
