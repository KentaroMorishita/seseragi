import { describe, expect, test } from "bun:test"
import type { CompileResponse } from "../src/compiler/types"
import { executeGeneratedModule } from "../src/runtime/browser-execution"
import { sampleCatalog, sampleLevels } from "../src/sample-catalog"

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

  test("orders a curriculum by learning level and outcome", () => {
    expect(new Set(sampleCatalog.map((sample) => sample.id)).size).toBe(
      sampleCatalog.length
    )
    expect([...new Set(sampleCatalog.map((sample) => sample.level))]).toEqual(
      [...sampleLevels]
    )
    expect(sampleCatalog.map((sample) => sample.sequence)).toEqual(
      sampleCatalog.map((_, index) => index + 1)
    )
    expect(new Set(sampleCatalog.map((sample) => sample.sourcePath)).size).toBe(
      sampleCatalog.length
    )
    expect(
      sampleCatalog.every(
        (sample) =>
          sample.sourcePath.startsWith("examples/playground/") &&
          sample.summary.trim() !== "" &&
          sample.concepts.length > 0 &&
          sample.concepts.every((concept) => concept.trim() !== "")
      )
    ).toBe(true)
    expect(sampleCatalog.map((sample) => sample.id)).not.toContain(
      "rock-paper-scissors"
    )
    expect(sampleCatalog.map((sample) => sample.id)).not.toContain(
      "mini-adventure"
    )
  })

  test("starts minimal and keeps comments focused on the code", async () => {
    const sources = await Promise.all(
      sampleCatalog.map((sample) =>
        Bun.file(
          new URL(`../../../${sample.sourcePath}`, import.meta.url)
        ).text()
      )
    )

    expect(sources[0]?.trim()).toBe(
      'pub effect fn main = println "Hello, Seseragi!"'
    )
    for (const source of sources) {
      expect(source).not.toContain("Lesson ")
      expect(source).not.toContain("Expected stdout")
      expect(source).not.toContain("前提:")
    }
  })

  test("opens the curriculum from a level-based sample browser", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()

    expect(html).toContain('id="sample-browser-button"')
    expect(html).toContain('aria-haspopup="dialog"')
    expect(html).toContain('id="sample-browser-dialog"')
    expect(html).toContain('id="sample-browser-groups"')
    expect(html).not.toContain('id="sample-select"')
    expect(main).toContain("connectSampleBrowser(")
  })

  test("keeps sample guidance available without growing the workspace rows", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const styles = await Bun.file(
      new URL("../src/styles.css", import.meta.url)
    ).text()

    expect(html).toContain('id="sample-guide-button"')
    expect(html).toContain('aria-controls="sample-guide"')
    expect(html).toContain('id="sample-guide-summary"')
    expect(styles).toMatch(/\.sample-guide \{[\s\S]*?position: absolute;/)
  })

  test("renders HTML output in an isolated preview", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()
    const previewDocument = await Bun.file(
      new URL("../src/preview-document.ts", import.meta.url)
    ).text()
    const sample = sampleCatalog.find(
      (candidate) => candidate.id === "html-components"
    )

    expect(sample?.outputMode).toBe("html")
    expect(html).toContain('id="show-text-output-button"')
    expect(html).toContain('id="show-html-preview-button"')
    expect(html).toContain('id="html-preview"')
    expect(html).toMatch(/<iframe[\s\S]*?\ssandbox(?:=|\s|>)/)
    expect(html).toContain('sandbox="allow-same-origin allow-scripts"')
    expect(main).toContain("createPreviewDocument(html)")
    expect(main).toContain("prepareInteractivePreview()")
    expect(previewDocument).toContain("script-src 'none'")
    expect(main).toContain(`createPreviewDocument('<div id="app"></div>')`)
    expect(main).toContain('htmlPreview.addEventListener(\n    "load"')
    expect(main).toContain('htmlPreview.removeAttribute("src")')
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
      if (sample.interactive) {
        expect(response.entry.environment).toContainEqual({
          field: "dom",
          service: "dom",
        })
        return
      }
      expect(
        await executeGeneratedModule(
          response.generated.typescript,
          response.entry,
          sample.stdin
        )
      ).toEqual({ stdout: sample.expectedOutput })
    })
  }

  test("connects DOM programs to cancellable interactive preview sessions", async () => {
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()
    const sample = sampleCatalog.find(
      (candidate) => candidate.id === "interactive-app"
    )

    expect(sample?.interactive).toBe(true)
    const source = await Bun.file(
      new URL(`../../../${sample!.sourcePath}`, import.meta.url)
    ).text()
    expect(source).toContain("dom.app {")
    expect(source).not.toContain("dom.run (dom.defaultOptions ())")
    expect(source).not.toContain("signals.make")
    expect(main).toContain("prepareInteractivePreview()")
    expect(main).toContain('binding.service === "dom"')
    expect(main).toContain('setStatus("success", "Interactive")')
    expect(main).toContain("execution.cancel()")
  })

  test("returns structured diagnostics for invalid source", async () => {
    const response = await compile("broken.ssrg", "pub let broken: Int =\n")
    expect(response.status).toBe("failure")
    expect(response.diagnostics.diagnostics.length).toBeGreaterThan(0)
    expect(response.diagnostics.diagnostics[0]?.primary).toBeDefined()
  })
})
