import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import {
  urlReaderCases,
  urlReaderExamples,
  urlReaderRoutes,
} from "../scripts/url-reader"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const evidenceRoot = resolve(
  process.env.URL_READER_EVIDENCE_DIR ??
    join(root, "target/url-reader-evidence")
)
mkdirSync(evidenceRoot, { recursive: true })
const evidence = mkdtempSync(join(evidenceRoot, "run-"))
const examples = urlReaderExamples("https://seseragi.vercel.app/")
const packagePath = join(root, "apps/site/examples/projects/url-reader")
const fixtures = join(root, "apps/site/tests/fixtures/url-reader")
const commands: unknown[] = []
function command(
  label: string,
  executable: string,
  args: string[],
  cwd = root
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
function sample(slug: string) {
  const result = examples.find((item) => item.id === `url-reader-${slug}`)
  if (!result) throw new Error(`Missing URL reader source: ${slug}`)
  return result
}
const wasm = (async () => {
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
  return bindings
})()
const browser = import(
  new URL("../../playground/src/runtime/browser-execution.ts", import.meta.url)
    .href
)

test("URL20 keeps twenty exact routes and thirteen short canonical pairs", () => {
  expect(urlReaderRoutes).toHaveLength(20)
  expect(new Set(urlReaderRoutes.map((item) => item.identity)).size).toBe(20)
  expect(urlReaderCases).toHaveLength(13)
  expect(examples).toHaveLength(26)
  expect(
    urlReaderRoutes.every((item) => item.module === "std/web/navigation")
  ).toBe(true)
  expect(urlReaderRoutes.map((item) => item.name)).toEqual([
    "std/web/navigation",
    "Url",
    "Query",
    "UrlBuildError",
    "parseUrl",
    "resolveUrl",
    "renderUrl",
    "emptyQuery",
    "parseQuery",
    "appendQuery",
    "setQuery",
    "removeQuery",
    "queryValues",
    "queryEntries",
    "renderQuery",
    "urlQuery",
    "withQuery",
    "urlFragment",
    "withFragment",
    "withoutFragment",
  ])
  for (const route of urlReaderRoutes) {
    expect(route.identity).toBe(
      route.namespace === "module"
        ? route.module
        : `${route.module}::${route.name}`
    )
    expect(String(route.route)).toBe(
      route.namespace === "module"
        ? "/docs/library/web/navigation/"
        : `/docs/library/web/navigation/${route.kind}/${route.slug}/`
    )
    expect(urlReaderCases.some((item) => item.slug === route.example)).toBe(
      true
    )
  }
  expect(sample("share-search-link").sha256).toBe(
    "1f79418621f5d6575557ab9c9bfc488c23ddadf68a367ed989c51dea0ed79a59"
  )
  const manifest = readFileSync(join(packagePath, "seseragi.toml"), "utf8")
  expect(manifest).toContain('target = "web"')
  expect(manifest).toContain('entry = "share-search-link"')
  expect(
    readFileSync(join(root, "apps/site/examples/src/main.ssrg"), "utf8")
  ).not.toContain("url-reader")
  for (const item of examples) {
    expect(item.sha256).toBe(
      createHash("sha256").update(item.source).digest("hex")
    )
    expect(item.highlighted.map((token) => token.text).join("")).toBe(
      item.source
    )
    if (item.id.endsWith("-ts")) {
      expect(item.playgroundUrl).toBe("")
    } else {
      expect(new URL(item.playgroundUrl).searchParams.get("source")).toBe(
        item.source
      )
      expect(item.sourcePath).toStartWith(
        "apps/site/examples/projects/url-reader/src/"
      )
    }
  }
})

test("every exact source has an official isolated web package build and strict TS parity", () => {
  for (const item of urlReaderCases) {
    const source = sample(item.slug)
    const directory = join(evidence, "packages", item.slug)
    mkdirSync(join(directory, "src"), { recursive: true })
    writeFileSync(join(directory, "src/main.ssrg"), source.source)
    writeFileSync(
      join(directory, "seseragi.toml"),
      `[package]\nname = "proof/url-reader-${item.slug}"\nversion = "0.0.0"\nlanguage = ">=0.1.0 <0.2.0"\n\n[run]\nentry = "main"\ntarget = "web"\n`
    )
    const lint = command(
      `${item.slug}.lint`,
      cli,
      ["lint", join(directory, "src/main.ssrg"), "--deny-warnings"],
      directory
    )
    expect(lint.status, lint.stderr).toBe(0)
    const format = command(
      `${item.slug}.format`,
      cli,
      ["format", "--check", join(directory, "src/main.ssrg")],
      directory
    )
    expect(format.status, format.stderr).toBe(0)
    const lock = command(
      `${item.slug}.lock`,
      cli,
      ["lock", "update", directory],
      directory
    )
    expect(lock.status, lock.stderr).toBe(0)
    const built = command(
      `${item.slug}.web-build`,
      cli,
      ["build", directory, "--target", "web", "--out-dir", "web-artifact"],
      directory
    )
    expect(built.status, built.stderr).toBe(0)
    const artifact = JSON.parse(
      readFileSync(
        join(directory, "web-artifact/artifact-manifest.json"),
        "utf8"
      )
    )
    expect(artifact.target).toBe("web")
    expect(artifact.entryModule).toBe(
      `proof/url-reader-${item.slug}@0.0.0::main`
    )
    const ts = command(`${item.slug}.typescript`, bun, [
      join(root, `apps/site/examples/comparisons/url-reader/${item.slug}.ts`),
    ])
    expect(ts.status, ts.stderr).toBe(0)
    expect(ts.stdout).toBe(item.output)
  }
  const checked = command("strict-typescript", bun, [
    join(root, "node_modules/typescript/bin/tsc"),
    "--noEmit",
    "--strict",
    "--skipLibCheck",
    "--types",
    "bun",
    "--target",
    "ES2022",
    "--lib",
    "ESNext,DOM,DOM.Iterable",
    "--module",
    "Preserve",
    "--moduleResolution",
    "Bundler",
    ...urlReaderCases.map((item) =>
      join(root, `apps/site/examples/comparisons/url-reader/${item.slug}.ts`)
    ),
    join(fixtures, "runtime-contracts.ts"),
  ])
  expect(checked.status, checked.stdout + checked.stderr).toBe(0)
})

test("the checked-in browser package builds for web and rejects the process target", () => {
  const lockPath = join(packagePath, "seseragi.lock")
  const lockBefore = readFileSync(lockPath, "utf8")
  const build = command("canonical-package-web", cli, [
    "build",
    packagePath,
    "--target",
    "web",
    "--out-dir",
    join(evidence, "canonical-package-web"),
  ])
  expect(build.status, build.stderr).toBe(0)
  // This is a negative target-boundary check, never a positive process run.
  const processBoundary = command("package-process-rejected", cli, [
    "run",
    packagePath,
    "--target",
    "process",
    "--diagnostic-format",
    "json",
  ])
  expect(processBoundary.status, processBoundary.stderr).toBe(2)
  // Package provider diagnostics are terminal text even when JSON is requested.
  expect(processBoundary.stderr).toContain("SES-K0203 provider.target-mismatch")
  expect(processBoundary.stderr).toContain(
    "standard module `std/web/navigation` is not available for target `bun-process`"
  )
  expect(processBoundary.stderr).toContain("compatible targets: browser")
  expect(readFileSync(lockPath, "utf8")).toBe(lockBefore)
})

test("thirteen exact Playground seeds execute through committed WASM with Console only", async () => {
  const bindings = await wasm
  const { executeGeneratedModule } = await browser
  const results: unknown[] = []
  for (const item of urlReaderCases) {
    const source = sample(item.slug)
    const seed = new URL(source.playgroundUrl).searchParams.get("source")!
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
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
      entry: compiled.entry,
      stdout: result.stdout,
    })
    writeFileSync(
      join(evidence, "wasm-seeds.json"),
      JSON.stringify(results, null, 2)
    )
  }
})

test("seven focused mistakes reject with precise diagnostics and canonical working repairs", () => {
  const cases = [
    ["render-string", "call.argument-type-mismatch", "check-absolute-link"],
    ["render-either", "call.argument-type-mismatch", "check-absolute-link"],
    ["query-order", "call.argument-type-mismatch", "replace-link-query"],
    ["call-empty-query", "expression.invalid", "build-filter-query"],
    [
      "wrong-error-formatter",
      "call.argument-type-mismatch",
      "url-build-errors",
    ],
    [
      "query-private-fields-annotated",
      "record.access-on-non-record",
      "read-selected-tags",
    ],
    ["construct-query", "function.return-type-mismatch", "build-filter-query"],
  ] as const
  const directory = mkdtempSync(join(evidence, "negative-"))
  for (const [slug, key, repair] of cases) {
    writeFileSync(
      join(directory, "main.ssrg"),
      readFileSync(join(fixtures, "invalid", `${slug}.ssrg`), "utf8")
    )
    const result = command(
      `negative-${slug}`,
      cli,
      ["lint", "main.ssrg", "--diagnostic-format", "json"],
      directory
    )
    expect(result.status, result.stderr).toBe(2)
    const diagnostics = JSON.parse(result.stderr).diagnostics.flatMap(
      (file: any) => file.diagnostics.diagnostics
    )
    expect(diagnostics).toHaveLength(1)
    expect(diagnostics[0].code).toBe("SES-T0101")
    expect(diagnostics[0].messageKey).toBe(key)
    // The repair is one of the exact-source WASM executions and official web
    // builds above, not a fabricated successful diagnostic or a process run.
    expect(sample(repair).source).toContain('from "std/web/navigation"')
  }
  writeFileSync(
    join(evidence, "negative-repairs.json"),
    JSON.stringify(
      cases.map(([slug, key, repair]) => ({ slug, key, repair })),
      null,
      2
    )
  )
})

test("current-runtime edges retain duplicates, encoding differences, offsets and decode failure", () => {
  const result = command("runtime-contracts", bun, [
    join(fixtures, "runtime-contracts.ts"),
    join(evidence, "runtime-results.json"),
  ])
  expect(result.status, result.stderr).toBe(0)
  const observations = JSON.parse(
    readFileSync(join(evidence, "runtime-results.json"), "utf8")
  )
  expect(observations.status).toBe("passed")
  expect(observations.assertions).toBe(52)
})

test("WASM preserves UTF-16 query offsets and the malformed-UTF8 fragment throw boundary", async () => {
  const bindings = await wasm
  const { executeGeneratedModule } = await browser
  for (const slug of ["query-percent-offset", "malformed-fragment"]) {
    const source = readFileSync(
      join(fixtures, "runtime", `${slug}.ssrg`),
      "utf8"
    )
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", source)
    )
    expect(compiled.status, JSON.stringify(compiled)).toBe("success")
    expect(compiled.entry.environment).toEqual([
      { field: "console", service: "console" },
    ])
    expect(compiled.entry.providers ?? []).toEqual([])
    let stdout: string | undefined
    let caught: unknown
    try {
      stdout = (
        await executeGeneratedModule(
          compiled.generated.typescript,
          compiled.entry
        )
      ).stdout
    } catch (error) {
      caught = error
    }
    const record = {
      slug,
      source,
      sourceSha256: createHash("sha256").update(source).digest("hex"),
      entry: compiled.entry,
      stdout,
      errorName: caught instanceof Error ? caught.name : undefined,
      errorMessage: caught instanceof Error ? caught.message : undefined,
    }
    writeFileSync(
      join(evidence, `${slug}.runtime-result.json`),
      JSON.stringify(record, null, 2)
    )
    if (slug === "query-percent-offset") {
      expect(caught).toBeUndefined()
      expect(stdout).toBe("Invalid percent escape at 4")
    } else {
      expect(caught).toBeInstanceOf(URIError)
      expect(stdout).toBeUndefined()
    }
  }
})
