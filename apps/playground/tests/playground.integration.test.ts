import { describe, expect, test } from "bun:test"
import { createHash } from "node:crypto"
import { readdir } from "node:fs/promises"
import { queryAnalysisAt } from "../src/analysis/document"
import type { AnalysisDocument, CompileResponse } from "../src/compiler/types"
import { executeGeneratedModule } from "../src/runtime/browser-execution"
import {
  sampleCapabilities,
  sampleDifficulties,
  sampleKinds,
  validateSampleCatalog,
} from "../src/sample-catalog"
import { learningPaths, samples } from "../src/samples"

type WasmBindings = {
  readonly default: (input: {
    readonly module_or_path: ArrayBuffer
  }) => Promise<unknown>
  readonly compile_single_file: (
    sourceName: string,
    moduleId: string,
    source: string
  ) => string
  readonly analyze_single_file: (
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

async function analyze(source: string): Promise<AnalysisDocument> {
  const wasm = await loadBindings()
  return JSON.parse(
    wasm.analyze_single_file("main.ssrg", "playground/main", source)
  ) as AnalysisDocument
}

describe("Playground sample catalog", () => {
  test("discovers every stable-slug sample directory without a central import map", async () => {
    const entries = await readdir(
      new URL("../../../examples/samples", import.meta.url),
      { withFileTypes: true }
    )
    const directoryIds = entries
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name)
      .sort()
    const samplesSource = await Bun.file(
      new URL("../src/samples.ts", import.meta.url)
    ).text()

    expect(samples.map((sample) => sample.id).sort()).toEqual(directoryIds)
    expect(samplesSource).toContain("generatedSamples.map")
    expect(samplesSource).not.toContain("sourceById")
    expect(samplesSource).not.toContain("examples/playground")
  })

  test("validates identity, metadata, paths and generated source hashes", () => {
    validateSampleCatalog(samples, learningPaths)
    expect(new Set(samples.map((sample) => sample.id)).size).toBe(
      samples.length
    )
    expect(new Set(samples.map((sample) => sample.sourcePath)).size).toBe(
      samples.length
    )
    expect(new Set(samples.map((sample) => sample.kind))).toEqual(
      new Set(sampleKinds)
    )
    expect(new Set(samples.map((sample) => sample.difficulty))).toEqual(
      new Set(sampleDifficulties)
    )
    expect(new Set(samples.flatMap((sample) => sample.capabilities))).toEqual(
      new Set(sampleCapabilities)
    )
    for (const sample of samples) {
      expect(sample.sourcePath).toBe(`examples/samples/${sample.id}/main.ssrg`)
      expect(sample.sourcePath).not.toMatch(/\/\d+-/)
      expect(sample.summary.trim()).not.toBe("")
      expect(sample.guide.trim()).not.toBe("")
      expect(sample.topics.length).toBeGreaterThan(0)
      expect(sample.sourceHash).toBe(
        `sha256:${createHash("sha256").update(sample.source).digest("hex")}`
      )
    }
    expect(learningPaths.length).toBeGreaterThan(1)
    expect(samples.some((sample) => sample.featured)).toBe(true)
    expect(samples.some((sample) => sample.isNew)).toBe(true)
  })

  test("starts minimal and keeps explanatory prose in the guide", () => {
    const hello = samples.find((sample) => sample.id === "hello-world")
    expect(hello?.source.trim()).toBe(
      'pub effect fn main = println "Hello, Seseragi!"'
    )
    for (const sample of samples) {
      expect(sample.source).not.toContain("Lesson ")
      expect(sample.source).not.toContain("Expected stdout")
      expect(sample.source).not.toContain("前提:")
    }
  })

  test("separates guided learning paths from searchable discovery", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()

    expect(html).toContain('id="sample-browser-button"')
    expect(html).toContain('id="sample-browser-learn-tab"')
    expect(html).toContain('id="sample-browser-discover-tab"')
    expect(html).toContain('id="sample-learning-paths"')
    expect(html).toContain('id="sample-search"')
    expect(html).toContain('id="sample-kind-filter"')
    expect(html).toContain('id="sample-topic-filter"')
    expect(html).toContain('id="sample-capability-filter"')
    expect(html).toContain('id="sample-featured-filter"')
    expect(html).toContain('id="sample-new-filter"')
    expect(html).not.toContain('id="sample-select"')
    expect(html).not.toContain("初級 01")
    expect(main).toContain("connectSampleBrowser(")
    expect(main).toContain("learningPaths")
    expect(main).toContain("currentContext: currentSampleContext")
    const browser = await Bun.file(
      new URL("../src/ui/sample-browser.ts", import.meta.url)
    ).text()
    expect(browser).toContain("前提:")
    expect(browser).toContain("次:")
  })

  test("keeps Input and clear controls independent", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()
    const stdinSample = samples.find((sample) => sample.id === "stdin-greeting")

    expect(html).toContain('placeholder="Input passed to the program"')
    expect(html).toContain('aria-label="Program input"')
    expect(stdinSample?.capabilities).toContain("stdin")
    expect(stdinSample?.stdin).not.toBe("")
    expect(html).toContain('id="clear-source-button"')
    expect(html).toContain('id="clear-output-button"')
    expect(main).toContain('clearSourceButton.addEventListener("click"')
    expect(main).toContain('clearOutputButton.addEventListener("click"')
  })

  test("shows per-sample guidance without growing the workspace rows", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const styles = await Bun.file(
      new URL("../src/styles.css", import.meta.url)
    ).text()

    expect(html).toContain('id="sample-guide-button"')
    expect(html).toContain('id="sample-guide-summary"')
    expect(html).toContain('id="sample-guide-body"')
    expect(styles).toMatch(/\.sample-guide \{[\s\S]*?position: absolute;/)
  })

  test("renders navigable human-readable diagnostic cards", async () => {
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()
    const cards = await Bun.file(
      new URL("../src/diagnostics/diagnostic-cards.ts", import.meta.url)
    ).text()

    expect(main).toContain("renderDiagnosticCards(output, diagnostics")
    expect(main).toContain("utf8RangeToUtf16(analyzedSource, byteRange)")
    expect(main).toContain('mobilePanels.show("code")')
    expect(cards).toContain("diagnostic.message")
    expect(cards).not.toContain("diagnostic.messageKey")
    expect(cards).toContain('formatSourceLocation("main.ssrg"')
    expect(cards).toContain("location.dataset.byteStart")
    expect(cards).toContain("Expected")
    expect(cards).toContain("Help:")
    expect(cards).toContain("Fix:")
  })

  test("shares symbol, inferred type, callable and standard Reference metadata", async () => {
    const source =
      "fn add left: Int -> right: Int -> Int = left + right\n" +
      "let addOne = add 1\n"
    const analysis = await analyze(source)
    const addReference = source.lastIndexOf("add 1")
    const partialArgument = addReference + "add ".length

    expect(analysis.diagnostics.diagnostics).toEqual([])
    expect(queryAnalysisAt(analysis, addReference).symbol?.identity).toBe(
      "playground/main::add"
    )
    expect(queryAnalysisAt(analysis, addReference).type).toBe(
      "Int -> Int -> Int"
    )
    expect(
      queryAnalysisAt(analysis, partialArgument).callable?.remainingParameters
    ).toEqual([{ name: "right", type: "Int" }])
    expect(
      analysis.standardLibrary
        .filter((item) => ["join", "sum", "forEach", "map"].includes(item.name))
        .map((item) => item.name)
    ).toEqual(expect.arrayContaining(["join", "sum", "forEach", "map"]))
    const formItems = analysis.standardLibrary.filter(
      (item) => item.module === "std/web/html"
    )
    expect(formItems.map((item) => item.name)).toEqual(
      expect.arrayContaining([
        "InputEvent",
        "ChangeEvent",
        "form",
        "label",
        "input",
        "textarea",
      ])
    )
    expect(
      formItems.find((item) => item.name === "input")?.signature
    ).toContain("onInput?: InputEvent -> Msg")
    expect(formItems.find((item) => item.name === "form")?.signature).toContain(
      "onSubmit?: Msg"
    )
  })

  test("connects live hover and generated Reference UI without running Effects", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()
    const reference = await Bun.file(
      new URL("../src/ui/reference-browser.ts", import.meta.url)
    ).text()

    expect(html).toContain('id="reference-browser-button"')
    expect(html).toContain('id="reference-search"')
    expect(html).toContain('id="reference-category"')
    expect(main).toContain("createLiveAnalysis({")
    expect(main).toContain("analysisHoverAt(latestAnalysis")
    expect(main).toContain(
      "referenceBrowser.setCatalog(analysis.standardLibrary)"
    )
    expect(reference).not.toContain("const referenceItems")
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
    const sample = samples.find(
      (candidate) => candidate.id === "html-components"
    )

    expect(sample?.outputMode).toBe("html")
    expect(html).toContain('id="show-html-preview-button"')
    expect(html).toContain(
      'sandbox="allow-forms allow-same-origin allow-scripts"'
    )
    expect(main).toContain("createPreviewDocument(html)")
    expect(main).toContain("prepareInteractivePreview()")
    expect(previewDocument).toContain("script-src 'none'")
    expect(previewDocument).toContain("form-action 'none'")
  })

  for (const sample of samples) {
    test(`compiles and executes sample: ${sample.title}`, async () => {
      const response = await compile(`${sample.id}.ssrg`, sample.source)

      expect(response.status).toBe("success")
      if (response.status !== "success" || !response.entry) {
        throw new Error("missing entry")
      }
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
    const sample = samples.find(
      (candidate) => candidate.id === "interactive-app"
    )

    expect(sample?.interactive).toBe(true)
    expect(sample?.source).toContain("dom.app {")
    expect(sample?.source).not.toContain("dom.run (dom.defaultOptions ())")
    expect(main).toContain('binding.service === "dom"')
    expect(main).toContain('setStatus("success", "Interactive")')
    expect(main).toContain("execution.cancel()")
  })

  test("returns structured diagnostics for invalid source", async () => {
    const response = await compile("broken.ssrg", "pub let broken: Int =\n")
    expect(response.status).toBe("failure")
    expect(response.diagnostics.diagnostics.length).toBeGreaterThan(0)
    const diagnostic = response.diagnostics.diagnostics[0]
    expect(diagnostic?.primary).toBeDefined()
    expect(diagnostic?.message).not.toBe(diagnostic?.messageKey)
    expect(diagnostic?.message).not.toContain("parser.")
    expect(Array.isArray(diagnostic?.labels)).toBe(true)
    expect(diagnostic?.helps.length).toBeGreaterThan(0)
  })

  test("stops an invalid String escape before browser execution", async () => {
    const response = await compile(
      "invalid-escape.ssrg",
      'pub let message: String = "bad\\qescape"\n'
    )

    expect(response.status).toBe("failure")
    if (response.status !== "failure") {
      throw new Error("invalid escape reached generated output")
    }
    expect(response.diagnostics.diagnostics[0]).toMatchObject({
      code: "SES-P0201",
      messageKey: "literal.invalid-escape",
      message: "Literal contains an invalid or unsupported escape sequence",
    })
  })

  test("exposes type differences and field spelling fixes", async () => {
    const mismatch = await compile(
      "mismatch.ssrg",
      'fn one value: Int -> Int = value\npub fn main -> Int = one "no"\n'
    )
    expect(mismatch.status).toBe("failure")
    expect(mismatch.diagnostics.diagnostics[0]).toMatchObject({
      messageKey: "call.argument-type-mismatch",
      expectedType: "Int",
      actualType: "String",
    })

    const field = await compile(
      "field.ssrg",
      'pub struct User { name: String }\npub fn main -> User = User { nmae: "A" }\n'
    )
    expect(field.status).toBe("failure")
    expect(field.diagnostics.diagnostics[0]?.fixes[0]).toMatchObject({
      title: "Replace with `name`",
      edits: [{ replacement: "name" }],
    })
  })
})
