import { describe, expect, test } from "bun:test"
import type { CompileResponse } from "../src/compiler/types"
import { executeGeneratedModule } from "../src/runtime/browser-execution"
import { sampleCatalog } from "../src/sample-catalog"

type WasmBindings = {
  readonly default: (input: {
    readonly module_or_path: ArrayBuffer
  }) => Promise<unknown>
  readonly compile_single_file: (
    sourceName: string,
    moduleId: string,
    source: string
  ) => string
}

let bindings: WasmBindings | undefined

async function loadBindings(): Promise<WasmBindings> {
  if (bindings) return bindings
  const bindingsUrl = new URL(
    "../src/wasm/pkg/seseragi_wasm.js",
    import.meta.url
  ).href
  bindings = (await import(bindingsUrl)) as WasmBindings
  const wasm = await Bun.file(
    new URL("../src/wasm/pkg/seseragi_wasm_bg.wasm", import.meta.url)
  ).arrayBuffer()
  await bindings.default({ module_or_path: wasm })
  return bindings
}

async function compile(
  sourceName: string,
  source: string
): Promise<CompileResponse> {
  const wasm = await loadBindings()
  return JSON.parse(
    wasm.compile_single_file(sourceName, `playground/${sourceName}`, source)
  ) as CompileResponse
}

describe("new Playground product gate", () => {
  test("keeps the catalog curated and grouped by learning purpose", () => {
    expect(new Set(sampleCatalog.map((sample) => sample.id)).size).toBe(
      sampleCatalog.length
    )
    expect(new Set(sampleCatalog.map((sample) => sample.category))).toEqual(
      new Set(["基本", "アプリ", "型と抽象化"])
    )
    expect(new Set(sampleCatalog.map((sample) => sample.sourcePath)).size).toBe(
      sampleCatalog.length
    )
  })

  for (const sample of sampleCatalog) {
    test(`executes bundled sample: ${sample.label}`, async () => {
      const source = await Bun.file(
        new URL(`../../../${sample.sourcePath}`, import.meta.url)
      ).text()
      const response = await compile(`${sample.id}.ssrg`, source)

      expect(response.status).toBe("success")
      if (response.status !== "success" || !response.entry)
        throw new Error("missing entry")
      expect(
        await executeGeneratedModule(
          response.generated.typescript,
          response.entry,
          sample.stdin
        )
      ).toEqual({ stdout: sample.expectedOutput })
    })
  }

  test("returns structured diagnostics for invalid source", async () => {
    const response = await compile("broken.ssrg", "pub let broken: Int =\n")
    expect(response.status).toBe("failure")
    expect(response.diagnostics.diagnostics.length).toBeGreaterThan(0)
    expect(response.diagnostics.diagnostics[0]?.primary).toBeDefined()
  })
})
