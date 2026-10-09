import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import type { CompileResponse } from "../../playground/src/compiler/types"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import {
  chapterExamples,
  functionChapterExamples,
} from "../scripts/function-chapter-examples"

setDefaultTimeout(120_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = chapterExamples("https://seseragi.vercel.app/")

test("chapter sources retain bytes, highlighting and runnable Playground seeds", () => {
  expect(examples).toHaveLength(2)
  for (const example of examples) {
    const source = readFileSync(resolve(root, example.sourcePath), "utf8")
    expect(example.source).toBe(source)
    expect(example.sha256).toBe(
      createHash("sha256").update(source).digest("hex")
    )
    expect(example.highlighted.map((part) => part.text).join("")).toBe(source)
    expect(sourceFromPlaygroundUrl(example.playgroundUrl)).toBe(source)
  }
})

test("chapter snippets execute natively as displayed and remain formatted", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-function-chapter-"))
  try {
    for (const { id, output } of functionChapterExamples) {
      const example = examples.find((value) => value.id === id)
      expect(example).toBeDefined()
      if (!example) continue
      writeFileSync(join(temporary, "main.ssrg"), example.source)
      for (const args of [
        ["format", "--check", "main.ssrg"],
        ["lint", "main.ssrg"],
        ["run", "main.ssrg"],
      ]) {
        const result = spawnSync(cli, args, {
          cwd: temporary,
          encoding: "utf8",
          timeout: 30_000,
        })
        expect(
          result.status,
          `${id}: ${result.stderr || result.error?.message}`
        ).toBe(0)
        if (args[0] === "run") expect(result.stdout, id).toBe(output)
      }
    }
    // Test the distinction the prose asks readers to make: a configured
    // predicate is still a function, while the completed call returns Bool.
    const values =
      examples.find((example) => example.id === "chapter-function-values")
        ?.source ?? ""
    writeFileSync(
      join(temporary, "main.ssrg"),
      values.replace(
        'let builds = matches "b"',
        'let builds: Bool = matches "b"'
      )
    )
    const rejected = spawnSync(cli, ["lint", "main.ssrg"], {
      cwd: temporary,
      encoding: "utf8",
      timeout: 30_000,
    })
    expect(rejected.status).not.toBe(0)
    expect(rejected.stderr).toContain("SES-T0101")
    expect(rejected.stderr).toContain("String -> Bool")
    writeFileSync(
      join(temporary, "main.ssrg"),
      values.replace(
        "pub effect fn main = do {",
        'pub effect fn main = println (show (matches "b" "Build"))\n\neffect fn unused = do {'
      )
    )
    const repaired = spawnSync(cli, ["run", "main.ssrg"], {
      cwd: temporary,
      encoding: "utf8",
      timeout: 30_000,
    })
    expect(repaired.status, repaired.stderr).toBe(0)
    expect(repaired.stdout).toBe("True\n")
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test("new chapter Playground programs compile in WASM and execute with matching output", async () => {
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
  const runtime = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  for (const { id, output } of functionChapterExamples) {
    const example = examples.find((value) => value.id === id)
    expect(example).toBeDefined()
    if (!example) continue
    const source = sourceFromPlaygroundUrl(example.playgroundUrl)
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", source)
    ) as CompileResponse
    expect(compiled.status, JSON.stringify(compiled)).toBe("success")
    if (compiled.status !== "success") continue
    expect(compiled.entry).toBeDefined()
    if (!compiled.entry) continue
    const result = await runtime.executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(result.stdout, id).toBe(output.trimEnd())
  }
})

test("the chapter's follow-up search and missing-value distinction behave as stated", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-chapter-followup-"))
  try {
    const pipeline = readFileSync(
      resolve(
        root,
        "apps/site/examples/src/language/syntax-reader-pipelines.ssrg"
      ),
      "utf8"
    )
    writeFileSync(
      join(temporary, "main.ssrg"),
      pipeline.replaceAll('matches "b"', 'matches "c"')
    )
    const changed = spawnSync(cli, ["run", "main.ssrg"], {
      cwd: temporary,
      encoding: "utf8",
      timeout: 30_000,
    })
    expect(changed.status, changed.stderr).toBe(0)
    expect(changed.stdout).toBe("[> check]\nTrue\nJust > check\nNothing\n")
    writeFileSync(
      join(temporary, "main.ssrg"),
      pipeline.replace("badge <$> picked", "badge picked")
    )
    const rejected = spawnSync(cli, ["lint", "main.ssrg"], {
      cwd: temporary,
      encoding: "utf8",
      timeout: 30_000,
    })
    expect(rejected.status).not.toBe(0)
    expect(rejected.stderr).toContain("SES-T0101")
    expect(rejected.stderr).toContain("Maybe<String>")
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})
