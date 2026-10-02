import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  filesystemReaderCases,
  filesystemReaderExamples,
  filesystemReaderFailures,
  filesystemReaderRoutes,
} from "../scripts/filesystem-reader"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const bun = process.env.SESERAGI_BUN ?? process.execPath
const examples = filesystemReaderExamples("https://seseragi.vercel.app/")
const fixtureRoot = join(root, "apps/site/tests/fixtures/filesystem-reader")

function command(
  executable: string,
  args: string[],
  cwd = root,
  temporary = cwd
) {
  return spawnSync(executable, args, {
    cwd,
    env: { ...process.env, TMPDIR: temporary },
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  })
}

function sample(slug: string) {
  const result = examples.find(
    (example) => example.id === `filesystem-reader-${slug}`
  )
  if (!result) throw new Error(`Missing filesystem example: ${slug}`)
  return result
}

function ownedTemporary<T>(work: (directory: string) => T): T {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-filesystem-reader-"))
  try {
    return work(directory)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
}

function runSource(source: string, directory: string) {
  writeFileSync(join(directory, "main.ssrg"), source)
  return command(cli, ["run", "main.ssrg", "--target", "process"], directory)
}

test("the exact sixteen existing identities reuse ten source programs and four honest comparisons", () => {
  expect(filesystemReaderRoutes).toHaveLength(16)
  expect(
    new Set(filesystemReaderRoutes.map((item) => item.identity)).size
  ).toBe(16)
  expect(filesystemReaderCases).toHaveLength(10)
  expect(examples).toHaveLength(14)
  for (const route of filesystemReaderRoutes) {
    expect(sample(route.example).source).toContain("pub effect fn main")
    if (route.comparison)
      expect(sample(`${route.comparison}-ts`).source).not.toBe("")
  }
})

test("filesystem panels keep Seseragi highlighting but no unproved Playground launch link", () => {
  for (const item of filesystemReaderCases) {
    const example = sample(item.slug)
    expect(example.highlighted.map((span) => span.text).join("")).toBe(
      example.source
    )
    if (item.filesystem) {
      expect(example.playgroundUrl).toBe("")
      expect(example.source).toContain("fs.withTemporaryDirectory")
      expect(example.source).not.toContain('"/tmp/')
    } else {
      expect(new URL(example.playgroundUrl).searchParams.get("source")).toBe(
        example.source
      )
    }
  }
})

test("official process CLI runs every exact source and leaves no owned temporary workspace", () => {
  ownedTemporary((directory) => {
    for (const item of filesystemReaderCases) {
      const before = readdirSync(directory).filter((name) =>
        name.startsWith("fs16-")
      )
      expect(before).toEqual([])
      const executed = runSource(sample(item.slug).source, directory)
      expect(
        executed.status,
        `${item.slug}: ${executed.stdout}${executed.stderr}`
      ).toBe(0)
      expect(executed.stderr, item.slug).toBe("")
      expect(executed.stdout, item.slug).toBe(item.output)
      expect(
        readdirSync(directory).filter((name) => name.startsWith("fs16-")),
        item.slug
      ).toEqual([])
    }
  })
})

test("real String-as-Path diagnostics retain their wording and the named repairs execute", () => {
  ownedTemporary((directory) => {
    for (const item of filesystemReaderFailures) {
      writeFileSync(
        join(directory, "main.ssrg"),
        readFileSync(join(fixtureRoot, `${item.slug}.ssrg`), "utf8")
      )
      const rejected = command(cli, ["lint", "main.ssrg"], directory)
      expect(rejected.status, item.slug).toBe(2)
      for (const diagnostic of item.diagnostics)
        expect(rejected.stderr).toContain(diagnostic)
      const corrected = runSource(sample(item.correction).source, directory)
      expect(corrected.status, corrected.stderr).toBe(0)
      expect(corrected.stdout).toBe(
        filesystemReaderCases.find(
          (example) => example.slug === item.correction
        )!.output
      )
    }
  })
})

test("portable spelling and rejected segments execute without path normalization claims", () => {
  ownedTemporary((directory) => {
    const result = runSource(
      readFileSync(join(fixtureRoot, "path-boundaries.ssrg"), "utf8"),
      directory
    )
    expect(result.status, result.stderr).toBe(0)
    expect(result.stdout).toBe(
      'reports/../report.txt\n/reports/report.txt\nC:/reports/report.txt\n//server/share/report.txt\nPathContainsNul { offset: 1 }\nInvalidDriveRoot\nInvalidUncRoot\nInvalidPathSegment "a/b"\nInvalidPathSegment "a\\\\b"\nInvalidPathSegment "a\\u0000b"\n'
    )
  })
})

test("all TypeScript counterparts pass strict checking and run on Bun and Node with real cleanup", () => {
  ownedTemporary((directory) => {
    const comparators = filesystemReaderCases.filter(
      (item) => "typescript" in item
    )
    const output = join(directory, "javascript")
    mkdirSync(output)
    const checked = command(
      bun,
      [
        join(root, "node_modules/.bin/tsc"),
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
        ...comparators.map((item) =>
          join(root, sample(`${item.typescript}-ts`).sourcePath)
        ),
      ],
      directory
    )
    expect(checked.status, checked.stdout + checked.stderr).toBe(0)
    writeFileSync(join(output, "package.json"), '{"type":"module"}\n')
    for (const item of comparators) {
      for (const runtime of [bun, "node"]) {
        const result = command(
          runtime,
          [join(output, `${item.typescript}.js`)],
          directory
        )
        expect(result.status, `${item.slug}/${runtime}: ${result.stderr}`).toBe(
          0
        )
        expect(result.stdout).toBe(item.typescriptOutput)
        expect(result.stderr).toBe("")
        expect(
          readdirSync(directory).filter((name) => name.startsWith("fs16-"))
        ).toEqual([])
      }
    }
  })
})

test("committed WASM compiles and executes only the exact pure Path Playground seeds", async () => {
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
  for (const item of filesystemReaderCases.filter(
    (entry) => !entry.filesystem
  )) {
    const seed = new URL(sample(item.slug).playgroundUrl).searchParams.get(
      "source"
    )!
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    )
    expect(compiled.status, JSON.stringify(compiled)).toBe("success")
    expect(
      compiled.entry.environment.some(
        (entry: { service: string }) => entry.service === "fileSystem"
      )
    ).toBe(false)
    const result = await executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(result.stdout, item.slug).toBe(item.output.trimEnd())
  }
})

test("real provider verifies bytes and qualified temporary error provenance on Bun and Node", () => {
  ownedTemporary((directory) => {
    const fixture = join(fixtureRoot, "provider-boundaries.ts")
    const bundle = join(directory, "provider-boundaries.mjs")
    const build = command(
      bun,
      [
        "build",
        fixture,
        "--target",
        "node",
        "--format",
        "esm",
        "--outfile",
        bundle,
      ],
      root,
      directory
    )
    expect(build.status, build.stdout + build.stderr).toBe(0)
    for (const [runtime, source, target] of [
      [bun, fixture, "bun-process"],
      ["node", bundle, "node-process"],
    ]) {
      const result = command(
        runtime!,
        [source!, directory, target!],
        root,
        directory
      )
      expect(result.status, result.stdout + result.stderr).toBe(0)
      expect(result.stderr).toBe("")
      const evidence = JSON.parse(result.stdout)
      expect(evidence.target).toBe(target)
      expect(evidence.rows).toHaveLength(8)
      expect(
        evidence.rows.find(
          (row: { scenario: string }) => row.scenario === "combined-failure"
        )
      ).toEqual({
        scenario: "combined-failure",
        callbackRuns: 1,
        outcome: "failure",
        failureSide: "Right",
        shutdown: "separate ProviderPackageDefect",
      })
      expect(
        readdirSync(directory).filter((name) => name.startsWith("scenario-"))
      ).toEqual([])
    }
  })
})
