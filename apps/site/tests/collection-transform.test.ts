import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { delimiter, dirname, join, resolve } from "node:path"
import {
  collectionTransformCases,
  collectionTransformExamples,
  collectionTransformRoutes,
} from "../scripts/collection-transform"
import { compilerReferenceModules } from "../scripts/reference"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = realpathSync(
  resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
)
const bun = realpathSync(process.env.SESERAGI_BUN ?? process.execPath)
// Supply SESERAGI_NODE with the actual Node executable, not a Bun alias.
const node = process.env.SESERAGI_NODE ?? "node"
const fixtures = join(root, "apps/site/tests/fixtures/collection-transform")
const examples = collectionTransformExamples("https://seseragi.vercel.app/")
const base = resolve(
  process.env.COLLECTION_TRANSFORM_EVIDENCE_DIR ??
    join(tmpdir(), "seseragi-collection-transform-evidence")
)
mkdirSync(base, { recursive: true })
const evidence = mkdtempSync(join(base, "run-"))
for (const part of ["home", "tmp"]) mkdirSync(join(evidence, part))
const resourceMonitor =
  process.env.COLLECTION_TRANSFORM_RESOURCE_MONITOR === "1"
const environment = {
  PATH: [dirname(bun), dirname(node), process.env.PATH ?? ""].join(delimiter),
  HOME: join(evidence, "home"),
  TMPDIR: join(evidence, "tmp"),
  LANG: "C.UTF-8",
}
const sha256 = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")
function save(name: string, value: unknown) {
  writeFileSync(join(evidence, name), `${JSON.stringify(value, null, 2)}\n`)
}
const commands: unknown[] = []
function command(
  label: string,
  executable: string,
  args: string[],
  cwd = evidence
) {
  const boundsPath = join(evidence, `${label}.bounds.json`)
  // Normal runs are portable. Linux proof runs opt into sampled child-tree RSS.
  const result = spawnSync(
    resourceMonitor
      ? (process.env.SESERAGI_PYTHON ?? "/usr/bin/python3")
      : executable,
    resourceMonitor
      ? [join(fixtures, "bounded-command.py"), boundsPath, executable, ...args]
      : args,
    {
      cwd,
      env: environment,
      input: Buffer.alloc(0),
      timeout: resourceMonitor ? 65_000 : 60_000,
      maxBuffer: 16 * 1024 * 1024,
    }
  )
  const streams = {
    stdout: result.stdout ?? Buffer.alloc(0),
    stderr: result.stderr ?? Buffer.alloc(0),
  }
  const files = Object.entries(streams).map(([stream, bytes]) => {
    const path = `${label}.${stream}.bin`
    writeFileSync(join(evidence, path), bytes)
    return { stream, path, bytes: bytes.length, sha256: sha256(bytes) }
  })
  const bounds = resourceMonitor
    ? JSON.parse(readFileSync(boundsPath, "utf8"))
    : {
        argv: [executable, ...args],
        stdinClosedByHarness: true,
        timeoutSeconds: 60,
        residentMemoryLimitBytes: null,
        residentMemoryScope:
          "not measured; portable spawnSync timeout/maxBuffer only",
        outputLimitBytesPerStream: 16 * 1024 * 1024,
        peakObservedResidentBytes: null,
        terminationReason: result.error?.message ?? result.signal ?? null,
        exitStatus: result.status,
      }
  commands.push({
    label,
    executable,
    args,
    cwd,
    environment,
    ...bounds,
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    files,
  })
  save("commands.json", commands)
  expect(result.error, streams.stderr.toString()).toBeUndefined()
  expect(bounds.terminationReason).toBeNull()
  if (resourceMonitor) {
    expect(bounds.peakObservedResidentBytes).toBeLessThanOrEqual(
      bounds.residentMemoryLimitBytes
    )
  } else {
    expect(bounds.peakObservedResidentBytes).toBeNull()
    expect(bounds.residentMemoryLimitBytes).toBeNull()
  }
  return result
}
function succeeded(result: ReturnType<typeof command>, output?: string) {
  expect(result.status, result.stderr?.toString()).toBe(0)
  expect(result.stderr).toEqual(Buffer.alloc(0))
  if (output !== undefined) expect(result.stdout).toEqual(Buffer.from(output))
  return result.stdout!.toString()
}
function sample(slug: string) {
  const item = examples.find((e) => e.id === `collection-transform-${slug}`)
  if (!item) throw new Error(`Missing canonical source: ${slug}`)
  return item
}
const nativeEntries = new Map<string, string>()
function buildSource(slug: string, source: string) {
  const directory = join(evidence, "native", slug)
  mkdirSync(directory, { recursive: true })
  const path = join(directory, "main.ssrg")
  writeFileSync(path, source)
  for (const [label, args] of [
    ["format", ["format", "--check", path]],
    ["lint", ["lint", path, "--deny-warnings"]],
    [
      "build",
      [
        "build",
        path,
        "--target",
        "process",
        "--profile",
        "release",
        "--out-dir",
        join(directory, "release"),
      ],
    ],
  ] as const)
    succeeded(command(`${slug}.${label}`, cli, [...args]))
  expect(readFileSync(path, "utf8")).toBe(source)
  const manifest = JSON.parse(
    readFileSync(join(directory, "release/.seseragi-build.json"), "utf8")
  )
  expect(manifest).toMatchObject({
    profile: "release",
    target: "process",
    runtime: "bundled",
    entry: "entry.js",
  })
  const entry = join(directory, "release", manifest.entry)
  nativeEntries.set(slug, entry)
  return entry
}
function strict(
  label: string,
  paths: string[],
  runtimeProbe = false,
  expectedCode?: number
) {
  const config = join(evidence, `${label}.config.json`)
  writeFileSync(
    config,
    JSON.stringify({
      paths,
      typeRoots: [join(root, "node_modules/@types")],
      runtimeProbe,
      expectedCode,
      result: join(evidence, `${label}.result.json`),
    })
  )
  succeeded(
    command(label, bun, [join(fixtures, "strict-typescript.ts"), config])
  )
}
const boundaries = JSON.parse(
  readFileSync(join(fixtures, "boundary-cases.json"), "utf8")
) as Array<{ slug: string; output: string }>
const negatives = JSON.parse(
  readFileSync(join(fixtures, "negative-cases.json"), "utf8")
) as Array<{
  slug: string
  output: string
  code: string
  messageKey: string
  actualType: string
  expectedType: string
  primary: { start: number; end: number }
  sourceSha256: string
}>

// These hashes lock the exact candidate panel sources, not generated output.
const protectedSources = JSON.parse(
  readFileSync(join(fixtures, "source-manifest.json"), "utf8")
) as Array<{
  slug: string
  sourceSha256: string
  typescriptSourceSha256: string
  outputSha256: string
}>

test("Collection transformations14 has exact identities, canonical files, outputs, and decoded seeds", () => {
  const expectedNames = [
    "append",
    "concat",
    "filterMap",
    "flatMap",
    "reverse",
    "tail",
  ]
  const identities = [
    ...[...expectedNames, "toList"].map((name) => `std/array::${name}`),
    ...[...expectedNames, "toArray"].map((name) => `std/list::${name}`),
  ]
  expect(collectionTransformRoutes.map<string>((r) => r.identity)).toEqual(
    identities
  )
  expect(collectionTransformCases).toHaveLength(14)
  expect(protectedSources).toHaveLength(14)
  expect(examples).toHaveLength(28)
  expect(new Set(examples.map((e) => e.id)).size).toBe(28)
  const modules = compilerReferenceModules()
  for (const [index, route] of collectionTransformRoutes.entries()) {
    const [module, name] = identities[index]!.split("::")
    const slug = `${module!.slice(4)}-${name!.toLowerCase()}`
    expect<Record<keyof typeof route, string>>(route).toEqual({
      identity: identities[index],
      module,
      namespace: "value",
      kind: "function",
      name,
      slug,
      example: slug,
      route: `/docs/library/${module!.slice(4)}/function/${name!.toLowerCase()}/`,
    })
    const owner = modules.find((m) => m.specifier === module)!
    expect(owner.targets).toEqual(["process", "browser"])
    expect(
      owner.items.filter(
        (item) =>
          item.identity === route.identity &&
          item.module === route.module &&
          item.namespace === route.namespace &&
          item.itemKind === route.kind
      )
    ).toHaveLength(1)
    expect<string | undefined>(collectionTransformCases[index]?.slug).toBe(slug)
    const source = sample(slug)
    const comparison = sample(`${slug}-ts`)
    for (const [item, suffix] of [
      [source, "ssrg"],
      [comparison, "ts"],
    ] as const) {
      expect(item.sourcePath).toBe(
        `apps/site/examples/src/collection-transform/${slug}.${suffix}`
      )
      expect(item.source).toBe(
        readFileSync(join(root, item.sourcePath), "utf8")
      )
      expect(item.sha256).toBe(sha256(item.source))
      expect(item.highlighted.map((token) => token.text).join("")).toBe(
        item.source
      )
      expect(item.source.endsWith("\n")).toBe(true)
    }
    expect(new URL(source.playgroundUrl).searchParams.get("source")).toBe(
      source.source
    )
    expect(comparison.playgroundUrl).toBe("")
    const protectedSource = protectedSources[index]!
    expect(protectedSource.slug).toBe(slug)
    expect(source.sha256).toBe(protectedSource.sourceSha256)
    expect(comparison.sha256).toBe(protectedSource.typescriptSourceSha256)
    expect(sha256(collectionTransformCases[index]!.output)).toBe(
      protectedSource.outputSha256
    )
  }
  expect(
    readdirSync(
      join(root, "apps/site/examples/src/collection-transform")
    ).sort()
  ).toEqual(
    collectionTransformCases
      .flatMap(({ slug }) => [`${slug}.ssrg`, `${slug}.ts`])
      .sort()
  )
  save(
    "source-manifest.json",
    examples.map(({ id, sourcePath, sha256, playgroundUrl }) => ({
      id,
      sourcePath,
      sha256,
      playgroundUrl,
    }))
  )
})

test("strict compiler API checks the exact fourteen TS sources and rejects an isolated type error", () => {
  strict(
    "strict-panels",
    collectionTransformCases.map(({ slug }) =>
      join(root, sample(`${slug}-ts`).sourcePath)
    )
  )
  const invalid = join(evidence, "strict-invalid.ts")
  writeFileSync(invalid, 'export const count: number = "wrong"\n')
  strict("strict-negative-control", [invalid], false, 2322)
  const valid = join(evidence, "strict-repair.ts")
  writeFileSync(valid, "export const count: number = 1\n")
  strict("strict-positive-control", [valid])
})

test("all exact displayed native and TS sources match byte-for-byte on Node and Bun", () => {
  const versions: Record<string, unknown> = {}
  for (const [name, executable] of [
    ["cli", cli],
    ["node", node],
    ["bun", bun],
  ]) {
    const version = succeeded(
      command(`${name}.version`, executable!, ["--version"])
    )
    const identity =
      name === "cli"
        ? executable!
        : succeeded(
            command(`${name}.executable`, executable!, [
              "-p",
              "process.execPath",
            ])
          ).trim()
    if (name === "node") expect(version).toMatch(/^v\d+\./)
    versions[name!] = {
      path: identity,
      version: version.trim(),
      sha256: sha256(readFileSync(identity)),
    }
  }
  save("toolchain.json", versions)
  const results = []
  for (const item of collectionTransformCases) {
    const source = sample(item.slug)
    const comparison = sample(`${item.slug}-ts`)
    const entry = buildSource(item.slug, source.source)
    for (const [host, executable] of [
      ["node", node],
      ["bun", bun],
    ] as const) {
      for (const [language, path] of [
        ["native", entry],
        ["typescript", join(root, comparison.sourcePath)],
      ] as const) {
        const label = `${item.slug}.${language}.${host}`
        succeeded(command(label, executable, [path]), item.output)
        results.push({
          slug: item.slug,
          host,
          language,
          command: label,
          sourceSha256:
            language === "native" ? source.sha256 : comparison.sha256,
          stdoutSha256: sha256(item.output),
          stderrSha256: sha256(""),
        })
      }
    }
  }
  expect(results).toHaveLength(56)
  save("exact-source-results.json", results)
})

test("native boundary fixtures preserve one-level flattening, empty shapes, order, and inputs", () => {
  expect(boundaries.map((item) => item.slug)).toEqual(["array", "list"])
  const results = []
  for (const item of boundaries) {
    const source = readFileSync(
      join(fixtures, "boundaries", `${item.slug}.ssrg`),
      "utf8"
    )
    const entry = buildSource(`boundary-${item.slug}`, source)
    for (const [host, executable] of [
      ["node", node],
      ["bun", bun],
    ] as const) {
      const label = `boundary-${item.slug}.${host}`
      succeeded(command(label, executable, [entry]), item.output)
      results.push({
        slug: item.slug,
        host,
        command: label,
        sourceSha256: sha256(source),
        stdoutSha256: sha256(item.output),
      })
    }
  }
  save("boundary-results.json", results)
})

test("twelve isolated type-invalid programs reject with the exact diagnosis and nearby repairs execute", () => {
  expect(negatives).toHaveLength(12)
  const results = []
  for (const item of negatives) {
    const source = readFileSync(
      join(fixtures, "invalid", `${item.slug}.ssrg`),
      "utf8"
    )
    // Keep the fixture out of the site package: only this source is invalid.
    const path = join(evidence, `negative-${item.slug}.ssrg`)
    writeFileSync(path, source)
    expect(sha256(source)).toBe(item.sourceSha256)
    const result = command(`negative-${item.slug}`, cli, [
      "lint",
      path,
      "--diagnostic-format",
      "json",
    ])
    expect(result.status).toBe(2)
    expect(result.stdout).toEqual(Buffer.alloc(0))
    const diagnostics = JSON.parse(
      result.stderr!.toString()
    ).diagnostics.flatMap(
      (file: { diagnostics: { diagnostics: unknown[] } }) =>
        file.diagnostics.diagnostics
    )
    expect(diagnostics).toHaveLength(1)
    expect(diagnostics[0]).toMatchObject({
      code: item.code,
      messageKey: item.messageKey,
      actualType: item.actualType,
      expectedType: item.expectedType,
      primary: item.primary,
    })
    expect(item.primary.end).toBeGreaterThan(item.primary.start)
    expect(item.primary.end).toBeLessThanOrEqual(Buffer.byteLength(source))
    const repair = readFileSync(
      join(fixtures, "repairs", `${item.slug}.ssrg`),
      "utf8"
    )
    const entry = buildSource(`repair-${item.slug}`, repair)
    for (const [host, executable] of [
      ["node", node],
      ["bun", bun],
    ] as const) {
      succeeded(
        command(`repair-${item.slug}.${host}`, executable, [entry]),
        item.output
      )
    }
    results.push({
      slug: item.slug,
      sourceSha256: sha256(source),
      repairedSha256: sha256(repair),
      diagnostics,
      output: item.output,
    })
  }
  save("negative-repairs.json", results)
})

test("direct runtime instrumentation observes each pure-callback contract in order on both hosts", () => {
  const path = join(fixtures, "runtime-contracts.ts")
  strict("runtime-contracts-strict", [path], true)
  const bundled = join(evidence, "runtime-contracts.mjs")
  succeeded(
    command("runtime-contracts-bundle", bun, [
      "build",
      path,
      "--target=node",
      "--format=esm",
      "--outfile",
      bundled,
    ])
  )
  const outputs = []
  for (const [host, executable] of [
    ["node", node],
    ["bun", bun],
  ] as const) {
    const output = succeeded(
      command(`runtime-contracts.${host}`, executable, [bundled])
    )
    const records = JSON.parse(output) as Array<{
      family: string
      operation: string
      input: number[]
      visits: number[]
      callbackCount: number
      output: unknown[]
    }>
    expect(records).toHaveLength(12)
    for (const record of records) {
      expect(["array", "list"]).toContain(record.family)
      expect(record.visits).toEqual(record.input)
      expect(record.callbackCount).toBe(record.input.length)
      expect(record.output).toEqual(
        record.operation === "filterMap"
          ? record.input.filter((n) => n % 2 === 0)
          : record.input.flatMap((n) => (n === 0 ? [] : [[n], [n + 10]]))
      )
    }
    outputs.push(output)
  }
  expect(outputs[0]).toBe(outputs[1])
  save("runtime-results.json", {
    scope:
      "Direct runtime instrumentation observes callback order and count; it does not introduce effects into pure Seseragi callbacks",
    sourceSha256: sha256(readFileSync(path)),
    records: JSON.parse(outputs[0]!),
  })
})

test("exact final decoded seeds execute through committed WASM and current browser host under Bun", () => {
  succeeded(
    command("wasm-seeds", bun, [join(fixtures, "wasm-seeds.ts"), evidence])
  )
  const result = JSON.parse(
    readFileSync(join(evidence, "wasm-seeds.json"), "utf8")
  )
  expect(result.results).toHaveLength(14)
  for (const [index, item] of collectionTransformCases.entries()) {
    expect(result.results[index]).toMatchObject({
      slug: item.slug,
      sourceSha256: sample(item.slug).sha256,
      rawStdout: item.output,
      rawStdoutSha256: sha256(item.output),
    })
  }
})
