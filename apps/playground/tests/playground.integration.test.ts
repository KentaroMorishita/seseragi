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
  test("keeps the stdin prompt independent from the selected sample", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()

    expect(html).toContain('placeholder="プログラムへ渡す標準入力"')
    expect(html).not.toContain("rock&#10;scissors")
  })

  test("keeps source and output clear controls independent", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()

    expect(html).toContain('id="clear-source-button"')
    expect(html).toContain('aria-label="本文をクリア"')
    expect(html).toContain('id="clear-output-button"')
    expect(main).toContain('clearSourceButton.addEventListener("click"')
    expect(main).toContain("replaceEditorSource(editor, source)")
    expect(main).toContain('clearOutputButton.addEventListener("click"')
  })

  test("keeps the catalog curated and grouped by learning purpose", () => {
    expect(new Set(sampleCatalog.map((sample) => sample.id)).size).toBe(
      sampleCatalog.length
    )
    expect(new Set(sampleCatalog.map((sample) => sample.category))).toEqual(
      new Set(["基本", "アプリ", "型と抽象化", "Web"])
    )
    expect(new Set(sampleCatalog.map((sample) => sample.sourcePath)).size).toBe(
      sampleCatalog.length
    )
  })

  test("renders HTML output in an isolated preview", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()
    const sample = sampleCatalog.find(
      (candidate) => candidate.id === "web-html-ssr"
    )

    expect(sample?.outputMode).toBe("html")
    expect(html).toContain('id="show-text-output-button"')
    expect(html).toContain('id="show-html-preview-button"')
    expect(html).toContain('id="html-preview"')
    expect(html).toMatch(/<iframe[\s\S]*?\ssandbox(?:\s|>)/)
    expect(html).not.toContain("allow-scripts")
    expect(main).toContain("htmlPreview.srcdoc = stdout")
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
