import { expect, test } from "bun:test"
import { createHash } from "node:crypto"
import { readFileSync } from "node:fs"
import { resolve } from "node:path"
import type {
  CompileResponse,
  EntryContract,
} from "../../playground/src/compiler/types"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { comparisonExamples, entranceComparison } from "../scripts/comparisons"

const root = resolve(import.meta.dir, "../../..")
const examples = comparisonExamples("https://seseragi.example/playground/")

test("comparison panels and Playground preserve the verified source bytes", () => {
  expect(examples.map(({ id }) => id)).toEqual([
    "comparison-shipping-total-seseragi",
    "comparison-shipping-total-typescript",
  ])
  for (const example of examples) {
    const source = readFileSync(resolve(root, example.sourcePath), "utf8")
    expect(example.source).toBe(source)
    expect(example.highlighted.map(({ text }) => text).join("")).toBe(source)
    expect(example.sha256).toBe(
      createHash("sha256").update(source).digest("hex")
    )
  }
  expect(sourceFromPlaygroundUrl(examples[0].playgroundUrl)).toBe(
    examples[0].source
  )
  // TS is highlighted with its own grammar, never sent to the Seseragi Playground.
  expect(examples[1].playgroundUrl).toBe("")
  expect(examples[1].highlighted).toContainEqual({
    text: "function",
    className: "tok-keyword",
  })
  expect(examples[1].highlighted).toContainEqual({
    text: "number",
    className: "tok-standardType",
  })
  const builder = readFileSync(
    resolve(root, "apps/site/scripts/build.ts"),
    "utf8"
  )
  expect(builder).toContain("...comparisonExamples(playgroundUrl)")
})

test("the linked entrance source compiles and runs through the Playground runtime", async () => {
  const bindingsUrl = new URL(
    "../../playground/src/wasm/pkg/seseragi_wasm.js",
    import.meta.url
  )
  const bindings = await import(bindingsUrl.href)
  const wasm = await Bun.file(
    new URL(
      "../../playground/src/wasm/pkg/seseragi_wasm_bg.wasm",
      import.meta.url
    )
  ).arrayBuffer()
  await bindings.default({ module_or_path: wasm })
  const source = sourceFromPlaygroundUrl(examples[0].playgroundUrl)
  expect(source).toBeDefined()
  const compiled = JSON.parse(
    bindings.compile_single_file("main.ssrg", "playground/main", source)
  ) as CompileResponse
  expect(compiled.status).toBe("success")
  if (compiled.status !== "success") return
  expect(compiled.entry).toBeDefined()
  if (!compiled.entry) return
  // Exercise the existing Playground runner as an integration boundary. Its
  // runtime tree is typechecked by the Playground lane with its own tsconfig.
  const runtimeUrl = new URL(
    "../../playground/src/runtime/browser-execution.ts",
    import.meta.url
  )
  const runtime: {
    executeGeneratedModule(
      typescript: string,
      entry: EntryContract
    ): Promise<{ stdout: string }>
  } = await import(runtimeUrl.href)
  const result = await runtime.executeGeneratedModule(
    compiled.generated.typescript,
    compiled.entry
  )
  expect(result.stdout).toBe(
    readFileSync(
      resolve(root, entranceComparison.expectedOutput),
      "utf8"
    ).trimEnd()
  )
})
