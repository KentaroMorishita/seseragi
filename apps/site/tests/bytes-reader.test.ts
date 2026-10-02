import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import {
  bytesReaderCases,
  bytesReaderExamples,
  bytesReaderRoutes,
} from "../scripts/bytes-reader"
import { compilerReferenceModules } from "../scripts/reference"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const node = process.env.SESERAGI_NODE ?? "node"
const base = resolve(
  process.env.BYTES_READER_EVIDENCE_DIR ??
    join(root, "target/bytes-reader-evidence")
)
mkdirSync(base, { recursive: true })
const evidence = mkdtempSync(join(base, "semantic-"))
const examples = bytesReaderExamples("https://seseragi.vercel.app/")
const fixtures = join(root, "apps/site/tests/fixtures/bytes-reader")
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
    (item) => item.id === `bytes-reader-${slug}${suffix}`
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

test("Bytes20 selects twenty existing portable identities and fourteen exact source triples", () => {
  expect(bytesReaderRoutes).toHaveLength(20)
  expect(bytesReaderCases).toHaveLength(14)
  expect(examples).toHaveLength(42)
  expect(new Set(bytesReaderRoutes.map((item) => item.identity)).size).toBe(20)
  expect(
    bytesReaderRoutes.filter((item) => item.kind === "module")
  ).toHaveLength(3)
  expect(
    bytesReaderRoutes.filter((item) => item.kind === "opaque-type")
  ).toHaveLength(5)
  expect(
    bytesReaderRoutes.filter((item) => item.kind === "function")
  ).toHaveLength(12)
  const modules = compilerReferenceModules()
  for (const route of bytesReaderRoutes) {
    const module = modules.find((item) => item.specifier === route.module)!
    expect(module.targets).toEqual(["process", "browser"])
    if (route.kind !== "module") {
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
    expect(bytesReaderCases.some((item) => item.slug === route.example)).toBe(
      true
    )
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

test("exact sources pass process compilation, Bun-native and separate Node Buffer comparisons", () => {
  const results: unknown[] = []
  for (const item of bytesReaderCases) {
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
      join(root, sample(item.slug, "-node-ts").sourcePath),
    ])
    expect(native).toBe(item.output)
    expect(bunOutput).toBe(item.typescriptOutput)
    expect(nodeOutput).toBe(item.typescriptOutput)
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
    bytesReaderCases.map((item) =>
      join(root, sample(item.slug, "-ts").sourcePath)
    )
  )
  strict(
    "strict-node",
    "node",
    bytesReaderCases.map((item) =>
      join(root, sample(item.slug, "-node-ts").sourcePath)
    )
  )
  writeFileSync(
    join(evidence, "exact-source-results.json"),
    JSON.stringify(results, null, 2)
  )
})

test("fourteen exact Playground seeds compile and execute through committed WASM", async () => {
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
  for (const item of bytesReaderCases) {
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

test("five precise type mistakes reject and their complete repairs run", () => {
  const negatives = [
    [
      "string-to-base64",
      "call.argument-type-mismatch",
      "String",
      "Bytes",
      "aGVsbG8=\n",
    ],
    [
      "string-to-utf8-decoder",
      "call.argument-type-mismatch",
      "String",
      "Bytes",
      "Right hello\n",
    ],
    [
      "unhandled-either",
      "call.argument-type-mismatch",
      "Either<ByteError, Bytes>",
      "Bytes",
      "41\n",
    ],
    ["float-byte-input", "array.element-type-mismatch", "Float", "Int", "1\n"],
    ["int-is-not-byte", "call.argument-type-mismatch", "Int", "Byte", "[65]\n"],
  ] as const
  const results: unknown[] = []
  for (const [slug, key, actual, expected, output] of negatives) {
    const path = join(evidence, `${slug}.ssrg`)
    const source = readFileSync(
      join(fixtures, "invalid", `${slug}.ssrg`),
      "utf8"
    )
    writeFileSync(path, source)
    const result = command(`negative-${slug}`, cli, [
      "lint",
      path,
      "--diagnostic-format",
      "json",
    ])
    expect(result.status, result.stderr).toBe(2)
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
      messageKey: key,
      actualType: actual,
      expectedType: expected,
    })
    expect(diagnostics[0].primary.end).toBeGreaterThan(
      diagnostics[0].primary.start
    )
    const repairPath = join(evidence, `${slug}-repair.ssrg`)
    writeFileSync(
      repairPath,
      readFileSync(join(fixtures, "repairs", `${slug}.ssrg`), "utf8")
    )
    checked(`repair-${slug}.format`, cli, ["format", "--check", repairPath])
    expect(checked(`repair-${slug}`, cli, ["run", repairPath])).toBe(output)
    results.push({
      slug,
      diagnostics,
      output,
      sourceSha256: createHash("sha256").update(source).digest("hex"),
    })
  }
  writeFileSync(
    join(evidence, "negative-repairs.json"),
    JSON.stringify(results, null, 2)
  )
})

test("strict codec, UTF-8, BOM, copy and all-byte boundaries retain their observed contracts", () => {
  const matrix = JSON.parse(
    readFileSync(join(fixtures, "contract-cases.json"), "utf8")
  )
  const path = join(evidence, "runtime-contracts.ssrg")
  writeFileSync(
    path,
    readFileSync(join(fixtures, "runtime-contracts.ssrg"), "utf8")
  )
  const output = checked("contracts-seseragi", cli, ["run", path])
  expect(output).toBe(matrix.expectedSeseragi)
  expect(output.split("\n").filter(Boolean)).toHaveLength(57)
  strict("contracts-strict-bun", "bun", [
    join(fixtures, "runtime-contracts.ts"),
    join(fixtures, "copies-and-roundtrips.ts"),
  ])
  strict("contracts-strict-node", "node", [
    join(fixtures, "runtime-contracts.node.ts"),
  ])
  const bunOutput = checked("contracts-bun", bun, [
    join(fixtures, "runtime-contracts.ts"),
  ])
  const nodeOutput = checked("contracts-node", node, [
    join(fixtures, "runtime-contracts.node.ts"),
  ])
  expect(bunOutput).toBe(nodeOutput)
  expect(bunOutput).toContain("utf-bom|ok:efbbbf41|efbbbf41")
  expect(bunOutput).toContain("default-bom|41")
  const copies = checked("copies-and-roundtrips", bun, [
    join(fixtures, "copies-and-roundtrips.ts"),
  ])
  expect(copies).toContain("base64: 65 deterministic lengths")
  writeFileSync(
    join(evidence, "runtime-results.json"),
    JSON.stringify(
      {
        status: "passed",
        seseragiRows: 57,
        output,
        bunOutput,
        nodeOutput,
        copies,
      },
      null,
      2
    )
  )
})
