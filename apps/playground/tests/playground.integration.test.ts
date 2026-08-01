import { describe, expect, test } from "bun:test"
import { createHash } from "node:crypto"
import { readdir } from "node:fs/promises"
import { queryAnalysisAt } from "../src/analysis/document"
import { analysisHoverAt } from "../src/analysis/hover"
import type {
  AnalysisDocument,
  CompileResponse,
  EntryContract,
  FormatResponse,
  ProjectAnalysisResponse,
  ProjectCompileResponse,
  ProjectFormatResponse,
  ProjectRequest,
} from "../src/compiler/types"
import {
  executeGeneratedModule,
  executeGeneratedProject,
} from "../src/runtime/browser-execution"
import {
  sampleArchitectures,
  sampleCapabilities,
  sampleDifficulties,
  sampleExperiences,
  sampleFocuses,
  sampleKinds,
  validateSampleCatalog,
} from "../src/sample-catalog"
import { discoverGroups, samples } from "../src/samples"
import { tourLessons } from "../src/tour/curriculum"
import { createWorkspace } from "../src/workspace/model"
import { runnableWorkspaceProjectRequest } from "../src/workspace/project-request"

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
  readonly format_single_file: (sourceName: string, source: string) => string
  readonly compile_project: (request: string) => string
  readonly analyze_project: (request: string) => string
  readonly format_project_file: (request: string, path: string) => string
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

async function format(source: string): Promise<FormatResponse> {
  const wasm = await loadBindings()
  return JSON.parse(wasm.format_single_file("main.ssrg", source))
}

async function compileProject(
  request: ProjectRequest
): Promise<ProjectCompileResponse> {
  const wasm = await loadBindings()
  return JSON.parse(wasm.compile_project(JSON.stringify(request)))
}

async function analyzeProject(
  request: ProjectRequest
): Promise<ProjectAnalysisResponse> {
  const wasm = await loadBindings()
  return JSON.parse(wasm.analyze_project(JSON.stringify(request)))
}

async function formatProjectFile(
  request: ProjectRequest,
  path: string
): Promise<ProjectFormatResponse> {
  const wasm = await loadBindings()
  return JSON.parse(wasm.format_project_file(JSON.stringify(request), path))
}

describe("Playground project compiler boundary", () => {
  const request: ProjectRequest = {
    schema: 1,
    entry: "main.ssrg",
    files: [
      {
        path: "feature/counter.ssrg",
        source: "pub fn increment value: Int -> Int = value + 1\n",
      },
      {
        path: "main.ssrg",
        source:
          'import { increment } from "./feature/counter"\n\n' +
          "pub effect fn main = increment 41 |> debug |> println\n",
      },
    ],
  }

  test("compiles generated modules and an entry contract in dependency order", async () => {
    const response = await compileProject(request)

    expect(response.status).toBe("success")
    if (response.status !== "success") return
    expect(response.modules.map(({ path }) => path)).toEqual([
      "feature/counter.ssrg",
      "main.ssrg",
    ])
    expect(response.modules[1]?.generated.typescript).toContain(
      'from "./feature/counter.js"'
    )
    expect(response.entry).toMatchObject({
      path: "main.ssrg",
      module: "playground/main",
      contract: {
        environment: [{ field: "console", service: "console" }],
      },
    })
  })

  test("stages and executes generated modules through relative imports", async () => {
    const response = await compileProject(request)

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing project execution entry")
    }
    expect(
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({ stdout: "42", debug: "()" })
  })

  test("executes shared pattern bindings across standard generic ADTs", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/pattern-binding-matrix/main.ssrg",
        import.meta.url
      )
    ).text()
    const expectedOutput = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/pattern-binding-matrix/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("pattern-binding-matrix.ssrg", source)

    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing pattern binding execution entry")
    }
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expectedOutput.trimEnd(), debug: "()" })
  })

  test("executes line-leading unary pipelines with evidence through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/unary-operators/main.ssrg",
        import.meta.url
      )
    ).text()
    const expectedOutput = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/unary-operators/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("unary-operators.ssrg", source)

    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing unary operator execution entry")
    }
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expectedOutput.trimEnd(), debug: "()" })
  })

  test("executes imported generic ADT patterns through the project boundary", async () => {
    const fixture = new URL(
      "../../../examples/spec/artifacts/project-schema-1/imported-generic-adt-monad/src/",
      import.meta.url
    )
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: await Promise.all(
        ["domain.ssrg", "main.ssrg"].map(async (path) => ({
          path,
          source: await Bun.file(new URL(path, fixture)).text(),
        }))
      ),
    })
    const expectedOutput = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/project-schema-1/imported-generic-adt-monad/execution.stdout.txt",
        import.meta.url
      )
    ).text()

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing imported pattern execution entry")
    }
    expect(
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({ stdout: expectedOutput.trimEnd(), debug: "()" })
  })

  test("executes imported top-level values across generated modules", async () => {
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "markup.ssrg",
          source: 'pub let markup: String = "<h1>Workspace preview</h1>"\n',
        },
        {
          path: "main.ssrg",
          source:
            'import { markup } from "./markup"\n\n' +
            "pub effect fn main = println markup\n",
        },
      ],
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing project execution entry")
    }
    expect(
      response.modules.find(({ path }) => path === "main.ssrg")?.generated
        .typescript
    ).toContain('from "./markup.js"')
    expect(
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({ stdout: "<h1>Workspace preview</h1>", debug: "()" })
  })

  test("resolves a failure renderer from another generated module", async () => {
    const fixture = new URL(
      "../../../examples/spec/artifacts/project-schema-1/transitive-effect-failure/src/",
      import.meta.url
    )
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: await Promise.all(
        ["provider.ssrg", "facade.ssrg", "main.ssrg"].map(async (path) => ({
          path,
          source: await Bun.file(new URL(path, fixture)).text(),
        }))
      ),
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error(
        response.status === "success"
          ? (response.entry.error ?? "missing project execution entry")
          : JSON.stringify(response.problems)
      )
    }
    expect(response.entry.contract.failureRenderer).toMatchObject({
      kind: "show",
      module: "./provider.ts",
    })
    await expect(
      executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).rejects.toThrow("InvalidInput lizard")
  })

  test("returns file-addressed analysis for imported symbols and types", async () => {
    const response = await analyzeProject(request)

    expect(response.status).toBe("success")
    if (response.status !== "success") return
    const main = response.documents.find(({ path }) => path === "main.ssrg")
    expect(main?.document.diagnostics.diagnostics).toEqual([])
    expect(
      main?.document.symbols.some(
        ({ name, module }) =>
          name === "increment" && module === "playground/feature/counter"
      )
    ).toBe(true)
    expect(
      main?.document.typeOccurrences.some(
        ({ type: typeName }) => typeName === "Int -> Int"
      )
    ).toBe(true)
  })

  test("reports graph failures with file paths and ranges", async () => {
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "main.ssrg",
          source: 'import { missing } from "./missing"\n',
        },
      ],
    })

    expect(response.status).toBe("failure")
    if (response.status !== "failure") return
    expect(response.problems[0]).toMatchObject({
      code: "SES-N0104",
      path: "main.ssrg",
    })
    expect(response.problems[0]?.primary?.end).toBeGreaterThan(0)
  })

  test("keeps aggregated project diagnostics identical across Analyze and Compile", async () => {
    const invalid: ProjectRequest = {
      schema: 1,
      entry: "z-semantic.ssrg",
      files: [
        {
          path: "z-semantic.ssrg",
          source: 'pub fn wrong unit: Unit -> Int = "wrong"\n',
        },
        {
          path: "a-parse.ssrg",
          source: "pub let broken: Int =\n",
        },
      ],
    }

    const [analyzed, compiled] = await Promise.all([
      analyzeProject(invalid),
      compileProject(invalid),
    ])
    expect(analyzed.status).toBe("failure")
    expect(compiled.status).toBe("failure")
    if (analyzed.status !== "failure" || compiled.status !== "failure") return
    expect(compiled.diagnostics).toEqual(analyzed.diagnostics)
    expect(compiled.diagnostics.map(({ path }) => path)).toEqual([
      "a-parse.ssrg",
      "z-semantic.ssrg",
    ])
    expect(
      compiled.diagnostics[0]?.diagnostics.diagnostics[0]?.messageKey
    ).toMatch(/^parser\./)
    expect(compiled.diagnostics[1]?.diagnostics.diagnostics[0]?.code).toBe(
      "SES-T0101"
    )
    expect(compiled.problems).toEqual([])
  })

  test("formats an active path through the same workspace request", async () => {
    const response = await formatProjectFile(
      {
        ...request,
        files: request.files.map((file) =>
          file.path === "main.ssrg"
            ? { ...file, source: `${file.source.trimEnd()}   \r\n` }
            : file
        ),
      },
      "main.ssrg"
    )

    expect(response.status).toBe("success")
    if (response.status !== "success") return
    expect(response.path).toBe("main.ssrg")
    expect(response.changed).toBe(true)
    expect(response.source.endsWith("   \r\n")).toBe(false)
  })

  test("routes the single-file adapter through the project boundary", async () => {
    const driver = await Bun.file(
      new URL("../src/compiler/wasm-driver.ts", import.meta.url)
    ).text()

    expect(driver).toContain("compileProject(singleFileRequest(source))")
    expect(driver).toContain("analyzeProject(singleFileRequest(source))")
    expect(driver).toMatch(
      /formatProjectFile\(\s*singleFileRequest\(source\),\s*"main\.ssrg"/
    )
  })
})

describe("Playground sample catalog", () => {
  test("compiles and executes every canonical Tour lesson", async () => {
    expect(tourLessons.map(({ position }) => position)).toEqual(
      tourLessons.map((_, index) => index + 1)
    )

    for (const lesson of tourLessons) {
      const response = await compile(`${lesson.id}.ssrg`, lesson.source)
      expect(response.status).toBe("success")
      if (response.status !== "success" || !response.entry) {
        throw new Error(`Tour lesson ${lesson.id} has no execution entry`)
      }
      if (lesson.interactive) {
        expect(response.entry.environment).toContainEqual({
          field: "dom",
          service: "dom",
        })
        continue
      }
      const result = await executeGeneratedModule(
        response.generated.typescript,
        response.entry,
        lesson.stdin
      )
      expect(result.stdout).toBe(lesson.expectedOutput)
    }
  })

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

  test("validates identity, metadata, Discover groups and source hashes", () => {
    validateSampleCatalog(samples, discoverGroups)
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
      expect(sample.workspaceHash).toBe(
        `sha256:${createHash("sha256")
          .update(
            sample.workspace.files
              .map(({ path, source }) => `${path}\0${source}\0`)
              .join("")
          )
          .digest("hex")}`
      )
    }
    expect(discoverGroups.length).toBeGreaterThan(1)
    expect(
      discoverGroups.flatMap(({ samples: sampleIds }) => sampleIds).sort()
    ).toEqual(
      samples
        .filter(({ kind }) => kind !== "lesson")
        .map(({ id }) => id)
        .sort()
    )
    expect(
      samples.filter(({ kind }) => kind === "lesson").map(({ id }) => id)
    ).toEqual([
      "data-and-patterns",
      "effects-and-do",
      "functions-and-pipelines",
      "generic-structs",
      "hello-world",
      "signal-composition",
      "strings-and-templates",
      "traits-and-instances",
    ])
    expect(
      samples
        .filter(({ id }) =>
          [
            "collection-patterns",
            "local-functions",
            "newtypes",
            "type-aliases",
          ].includes(id)
        )
        .map(({ kind }) => kind)
    ).toEqual(["recipe", "recipe", "recipe", "recipe"])
    expect(samples.some((sample) => sample.featured)).toBe(true)
    expect(samples.some((sample) => sample.isNew)).toBe(true)
    const webSamples = samples.filter(({ outputMode }) => outputMode === "html")
    expect(new Set(webSamples.map(({ experience }) => experience))).toEqual(
      new Set(sampleExperiences)
    )
    expect(new Set(webSamples.map(({ architecture }) => architecture))).toEqual(
      new Set(
        sampleArchitectures.filter(
          (architecture) => architecture !== "signal-mount"
        )
      )
    )
    expect(new Set(webSamples.map(({ focus }) => focus))).toEqual(
      new Set(sampleFocuses.filter((focus) => !["event"].includes(focus)))
    )
    expect(webSamples).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: "html-components",
          experience: "minimal",
          architecture: "static",
          focus: "component",
        }),
        expect.objectContaining({
          id: "interactive-app",
          experience: "minimal",
          architecture: "dom-app",
          focus: "state",
        }),
        expect.objectContaining({
          id: "feature-composition",
          experience: "guided",
          architecture: "signal-run",
          focus: "composition",
        }),
        expect.objectContaining({
          id: "signal-run-route",
          experience: "minimal",
          architecture: "signal-run",
          focus: "state",
          comparisonSample: "interactive-app",
        }),
        expect.objectContaining({
          id: "form-todo",
          experience: "showcase",
          architecture: "signal-run",
          focus: "form",
        }),
        expect.objectContaining({
          id: "project-flow-app",
          experience: "showcase",
          architecture: "multi-module",
          focus: "project",
        }),
      ])
    )
    for (const sample of webSamples) {
      expect(sample.guide.trimStart().startsWith("このsampleを選ぶ理由:")).toBe(
        true
      )
      expect(sample.title).not.toBe("Interactive Web App")
    }
    expect(samples.find(({ id }) => id === "project-greeting")).toMatchObject({
      project: {
        entryFile: "main.ssrg",
        activeFile: "main.ssrg",
        openFiles: ["main.ssrg", "feature/greeting.ssrg"],
        expandedFolders: ["feature"],
      },
      workspace: {
        entryFile: "main.ssrg",
        activeFile: "main.ssrg",
        openFiles: ["main.ssrg", "feature/greeting.ssrg"],
        expandedFolders: ["feature"],
        explorer: { visible: true },
      },
      stdin: "Seseragi\n",
    })
    expect(samples.find(({ id }) => id === "project-flow-app")).toMatchObject({
      interactive: true,
      outputMode: "html",
      project: {
        entryFile: "main.ssrg",
        activeFile: "main.ssrg",
        openFiles: [
          "main.ssrg",
          "styles.ssrg",
          "app.ssrg",
          "todo/feature.ssrg",
        ],
        expandedFolders: ["todo"],
      },
      workspace: {
        explorer: { visible: true },
      },
    })
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

  test("keeps Tour as Learn and groups each Recipe or Showcase once", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()

    expect(html).toContain('id="sample-browser-button"')
    expect(html).toContain('id="sample-browser-learn-tab"')
    expect(html).toContain('id="sample-browser-discover-tab"')
    expect(html).toContain("CANONICAL LEARNING PATH")
    expect(html).toContain("段階的なlesson")
    expect(html).toContain('class="sample-learn-link" href="./tour/"')
    expect(html).not.toContain('id="sample-learning-paths"')
    expect(html).toContain('id="sample-search"')
    expect(html).toContain('id="sample-kind-filter"')
    expect(html).toContain('id="sample-topic-filter"')
    expect(html).toContain('id="sample-capability-filter"')
    expect(html).toContain('id="sample-featured-filter"')
    expect(html).toContain('id="sample-new-filter"')
    expect(html).not.toContain('<option value="lesson">')
    expect(html).not.toContain('id="sample-select"')
    expect(html).not.toContain("初級 01")
    expect(main).toContain("connectSampleBrowser(")
    expect(main).toContain("discoverGroups")
    expect(main).toContain("currentContext: currentSampleContext")
    const browser = await Bun.file(
      new URL("../src/ui/sample-browser.ts", import.meta.url)
    ).text()
    expect(browser).toContain('kind !== "lesson"')
    expect(browser).toContain("sample-discover-group")
    expect(browser).toContain("experienceLabel(sample.experience)")
    expect(browser).toContain("architectureLabel(sample.architecture)")
    expect(browser).toContain("sample-card-prerequisite")
    expect(browser).toContain("sample-card-comparison")
    expect(browser).not.toContain("前提:")
    expect(browser).not.toContain("次:")
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

    expect(main).toContain("renderWorkspaceDiagnosticCards(")
    expect(main).toContain("activateWorkspaceFile(workspaceState, path)")
    expect(main).toContain("utf8RangeToUtf16(")
    expect(main).toContain('mobilePanels.show("code")')
    expect(cards).toContain("diagnostic.message")
    expect(cards).not.toContain("diagnostic.messageKey")
    expect(cards).toContain("formatSourceLocation(path")
    expect(cards).toContain("location.dataset.byteStart")
    expect(cards).toContain("Expected")
    expect(cards).toContain("difference.message")
    expect(cards).toContain("diagnostic-card-differences")
    expect(cards).toContain("Help:")
    expect(cards).toContain("Fix:")
  })

  test("shares symbol, inferred type, callable and standard Reference metadata", async () => {
    const source =
      'import * as html from "std/web/html"\n' +
      "fn add left: Int -> right: Int -> Int = left + right\n" +
      "let addOne = add 1\n" +
      "let inputElement = html.input\n"
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
    expect(analysisHoverAt(analysis, source, addReference)?.title).toBe(
      queryAnalysisAt(analysis, addReference).callable?.signature
    )
    const inputReference = source.lastIndexOf("html.input") + "html.".length
    const inputCallable = queryAnalysisAt(analysis, inputReference).callable
    const inputCatalog = analysis.standardLibrary.find(
      (item) => item.identity === "std/web/html::input"
    )
    expect(inputCallable?.signature).toBe(inputCatalog?.signature)
    expect(analysisHoverAt(analysis, source, inputReference)?.title).toBe(
      inputCatalog?.signature
    )
    expect(
      analysis.standardLibrary
        .filter((item) =>
          ["join", "sum", "forEach", "map", "Task"].includes(item.name)
        )
        .map((item) => item.name)
    ).toEqual(expect.arrayContaining(["join", "sum", "forEach", "map", "Task"]))
    expect(
      analysis.standardLibrary.find((item) => item.name === "Task")?.signature
    ).toBe("alias Task<A> = Effect<{}, Never, A>")
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
    ).toContain("onInput?: (InputEvent -> Action)")
    expect(formItems.find((item) => item.name === "form")?.signature).toContain(
      "onSubmit?: Action"
    )
    expect(formItems.map((item) => item.signature).join("\n")).not.toMatch(
      /\bMsg\b/
    )
  })

  test("shares generic and nested expected record completion metadata", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/expected-record-completion/main.ssrg",
        import.meta.url
      )
    ).text()
    const analysis = await analyze(source)

    expect(analysis.diagnostics.diagnostics[0]).toMatchObject({
      code: "SES-T0101",
      messageKey: "call.argument-type-mismatch",
    })
    expect(
      (analysis.completionContexts ?? [])
        .filter((context) => (context.recordFields?.length ?? 0) > 0)
        .map((context) => ({
          type: context.type,
          fields: context.recordFields?.map((field) => ({
            name: field.name,
            type: field.type,
          })),
        }))
    ).toEqual([
      {
        type: "{ initial: Int, profile: { label: String, count: Int }, render: (Int -> String) }",
        fields: [{ name: "render", type: "Int -> String" }],
      },
      {
        type: "{ label: String, count: Int }",
        fields: [{ name: "count", type: "Int" }],
      },
    ])
  })

  test("keeps concrete call results across top-level bindings in WASM analysis", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/top-level-call-inference/main.ssrg",
        import.meta.url
      )
    ).text()
    const analysis = await analyze(source)

    expect(analysis.diagnostics.diagnostics).toEqual([])
    for (const [name, expected] of [
      ["success", "Maybe<Int>"],
      ["stopped", "Maybe<Int>"],
      ["wrapped", "Maybe<Int>"],
      ["eitherValue", "Either<String, Int>"],
      ["arrayValue", "Array<Int>"],
      ["listValue", "List<Int>"],
      ["packetValue", "Packet<Int>"],
      ["boxValue", "Box<Int>"],
    ] as const) {
      const sourcePosition = source.indexOf(`let ${name}`) + "let ".length
      const position = new TextEncoder().encode(
        source.slice(0, sourcePosition)
      ).length
      expect(queryAnalysisAt(analysis, position).symbol?.typeName).toBe(
        expected
      )
    }

    const response = await compile("top-level-call-inference.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing entry")
    }
    const expectedOutput = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/top-level-call-inference/stdout.txt",
        import.meta.url
      )
    ).text()
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry,
        ""
      )
    ).toEqual({ stdout: expectedOutput.trimEnd(), debug: "()" })
  })

  test("rejects top-level initializer cycles before browser execution", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/top-level-initialization-cycle/main.ssrg",
        import.meta.url
      )
    ).text()
    const analysis = await analyze(source)

    expect(analysis.diagnostics.diagnostics).toHaveLength(1)
    expect(analysis.diagnostics.diagnostics[0]).toMatchObject({
      code: "SES-N0201",
      messageKey: "module.initialization-cycle",
      message: "Top-level initialization depends recursively on itself",
    })

    const response = await compile(
      "top-level-initialization-cycle.ssrg",
      source
    )
    expect(response.status).toBe("failure")
    if (response.status !== "failure") throw new Error("expected diagnostics")
    expect(response.diagnostics.diagnostics[0]?.code).toBe("SES-N0201")
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
    expect(main).toContain("createLiveAnalysis<WorkspaceAnalysisResult>({")
    expect(main).toMatch(/analysisHoverAt\(\s*latestAnalysis,/)
    expect(main).toContain(
      "referenceBrowser.setCatalog(analysis.activeDocument.standardLibrary)"
    )
    expect(reference).not.toContain("const referenceItems")
  })

  test("keeps pending live analysis from overwriting Run diagnostics", async () => {
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()
    const applyStart = main.indexOf(
      "apply: (analysis, _analyzedSource, identity) => {"
    )
    const applyEnd = main.indexOf("\n    },", applyStart)
    const applyHandler = main.slice(applyStart, applyEnd)
    expect(applyHandler).toContain(
      "if (runButton.disabled && identity === activeRunAnalysisRevision) return"
    )
    expect(
      applyHandler.indexOf("identity === activeRunAnalysisRevision")
    ).toBeLessThan(
      applyHandler.indexOf("setActiveEditorDiagnostics(analysis.diagnostics)")
    )

    const runStart = main.indexOf("async function run(): Promise<void> {")
    const runEnd = main.indexOf("\nfunction cancelActiveExecution", runStart)
    const runHandler = main.slice(runStart, runEnd)
    expect(runHandler.indexOf("liveAnalysis.cancel()")).toBeGreaterThan(0)
    expect(runHandler.indexOf("liveAnalysis.cancel()")).toBeLessThan(
      runHandler.indexOf("await compileProject(request)")
    )
  })

  test("formats source through the shared WASM formatter without rewriting errors", async () => {
    const source = 'pub let greeting: String = "こんにちは🙂"   \r\n' + "\r\n"
    const expected = 'pub let greeting: String = "こんにちは🙂"\n'

    const formatted = await format(source)
    expect(formatted).toEqual({
      status: "success",
      schema: 1,
      source: expected,
      changed: true,
    })
    const canonical = await format(expected)
    expect(canonical).toEqual({
      status: "success",
      schema: 1,
      source: expected,
      changed: false,
    })
    const invalid = await format("pub let broken: Int =\n")
    expect(invalid.status).toBe("failure")
    if (invalid.status !== "failure") throw new Error("invalid source changed")
    expect(invalid.diagnostics.diagnostics.length).toBeGreaterThan(0)
    expect(invalid).not.toHaveProperty("source")
  })

  test("compiles and executes canonical Float literals through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/float-literal-lowering/main.ssrg",
        import.meta.url
      )
    ).text()
    const expectedOutput = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/float-literal-lowering/stdout.txt",
        import.meta.url
      )
    ).text()

    const response = await compile("float-literal-lowering.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing Float execution entry")
    }
    expect(response.generated.typescript).toContain(
      "[1.0, 2.3, -(0.0), 6.022e23]"
    )
    expect(response.generated.typescript).not.toContain(" = _")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expectedOutput.trimEnd(), debug: "()" })
  })

  test("renders generic standard failures from nested Show evidence", async () => {
    const entry: EntryContract = {
      environment: [],
      failureRenderer: {
        kind: "show",
        module: "@seseragi/runtime/show",
        export: "domRuntimeErrorShow",
        arguments: [
          {
            module: "@seseragi/runtime/show",
            export: "stringShow",
          },
        ],
      },
    }
    const source = `
      import { fail } from "@seseragi/runtime/effect"
      export const main = (_unit: undefined) =>
        fail({ tag: "DispatchFailure", value: "denied" })
    `

    await expect(executeGeneratedModule(source, entry)).rejects.toThrow(
      "DispatchFailure denied"
    )
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
      'sandbox="allow-forms allow-popups allow-popups-to-escape-sandbox allow-same-origin allow-scripts"'
    )
    expect(html).toContain('referrerpolicy="no-referrer"')
    expect(main).toContain("createPreviewDocument(html)")
    expect(main).toContain("prepareInteractivePreview()")
    expect(previewDocument).toContain("script-src 'none'")
    expect(previewDocument).toContain("form-action 'none'")
    expect(previewDocument).toContain("img-src 'self' https: data: blob:")
  })

  for (const sample of samples) {
    test(`compiles and executes sample: ${sample.title}`, async () => {
      const response = await compileProject(
        runnableWorkspaceProjectRequest(createWorkspace(sample.workspace))
      )

      expect(response.status).toBe("success")
      if (
        response.status !== "success" ||
        response.entry.contract === undefined
      ) {
        throw new Error("missing entry")
      }
      if (sample.interactive) {
        expect(response.entry.contract.environment).toContainEqual({
          field: "dom",
          service: "dom",
        })
        return
      }
      expect(
        await executeGeneratedProject(
          response.modules.map(({ path, generated }) => ({
            path,
            typescript: generated.typescript,
          })),
          response.entry.path,
          response.entry.contract,
          sample.stdin
        )
      ).toEqual({ stdout: sample.expectedOutput, debug: "()" })
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

  test("keeps dom.app and explicit dom.run as one semantic comparison pair", async () => {
    const domApp = samples.find(({ id }) => id === "interactive-app")
    const signalRun = samples.find(({ id }) => id === "signal-run-route")
    if (domApp === undefined || signalRun === undefined) {
      throw new Error("missing Trail planner comparison samples")
    }

    expect(domApp.comparisonSample).toBe(signalRun.id)
    expect(signalRun.comparisonSample).toBe(domApp.id)
    expect(domApp.architecture).toBe("dom-app")
    expect(signalRun.architecture).toBe("signal-run")

    const sharedMarker = "// Shared comparison model and view."
    const runtimeMarker = "// Runtime boundary:"
    const sharedSource = (source: string) =>
      source.slice(source.indexOf(sharedMarker), source.indexOf(runtimeMarker))
    expect(sharedSource(domApp.source)).toBe(sharedSource(signalRun.source))

    const snapshotSource = (source: string) => `${source.slice(
      0,
      source.indexOf(runtimeMarker)
    )}
pub effect fn main -> Unit
with Console
fails ConsoleError =
  do {
    println (html.renderToString (view initialState))
    println (html.renderToString (view (update ChooseRiverside initialState)))
    println (html.renderToString (view (update ChooseWoodland initialState)))
    println (html.renderToString (view (update ChooseRidge initialState)))
  }
`
    const executeSnapshots = async (id: string, source: string) => {
      const response = await compile(`${id}-snapshots.ssrg`, source)
      expect(response.status).toBe("success")
      if (response.status !== "success" || response.entry === undefined) {
        throw new Error(`missing ${id} snapshot entry`)
      }
      return executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    }

    const appSnapshots = await executeSnapshots(
      domApp.id,
      snapshotSource(domApp.source)
    )
    const runSnapshots = await executeSnapshots(
      signalRun.id,
      snapshotSource(signalRun.source)
    )
    expect(runSnapshots).toEqual(appSnapshots)
    expect(appSnapshots.stdout.split("\n")).toHaveLength(4)
    expect(appSnapshots.stdout).toContain("川辺をゆっくり歩く")
    expect(appSnapshots.stdout).toContain("木陰のloopを巡る")
    expect(appSnapshots.stdout).toContain("尾根の展望へ登る")
    expect(appSnapshots.stdout).toContain("width: 28%")
    expect(appSnapshots.stdout).toContain("width: 62%")
    expect(appSnapshots.stdout).toContain("width: 88%")
  })

  test("shows stateful feature composition without a flattened root Action", () => {
    const sample = samples.find(
      (candidate) => candidate.id === "feature-composition"
    )

    expect(sample?.interactive).toBe(true)
    expect(sample?.source).toContain("Signal<html.Html<Task<Unit>>>")
    expect(sample?.source).not.toContain("Effect<{}, Never, Unit>")
    expect(sample?.source).toContain("signals.switchMap")
    expect(sample?.source).toContain("effect fn mount")
    expect(sample?.source).not.toContain("type RootAction")
  })

  test("integrates the Web UI surface in one feature-owned Todo sample", () => {
    const sample = samples.find((candidate) => candidate.id === "form-todo")

    expect(sample?.interactive).toBe(true)
    expect(sample?.source).toContain("MutableSignal<Model>")
    expect(sample?.source).toContain("Signal<html.Html<Task<Unit>>>")
    expect(sample?.source).toContain("onSubmit: dispatch state Submitted")
    expect(sample?.source).toContain("html.img {")
    expect(sample?.source).toContain("html.a {")
    expect(sample?.source).toContain("html.table {")
    expect(sample?.source).toContain("onKeyDown: filterKeyTask state")
    expect(sample?.source).toContain("onPointerDown: pointerTask state")
    expect(sample?.source).toContain("stopClickPropagation: True")
    expect(sample?.source).toContain('role: "status"')
    expect(sample?.source).not.toContain("dom.app {")
  })

  test("keeps transparent aliases and Task in the Recipe catalog", () => {
    const sample = samples.find((candidate) => candidate.id === "type-aliases")

    expect(sample?.source).toContain("alias Pair<A>")
    expect(sample?.source).toContain("Task<Unit>")
    expect(sample?.expectedOutput).toBe("user: 42, signal: 42")
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

  test("exposes structured type differences and field spelling fixes", async () => {
    const mismatch = await compile(
      "mismatch.ssrg",
      'fn one value: Int -> Int = value\npub fn main -> Int = one "no"\n'
    )
    expect(mismatch.status).toBe("failure")
    expect(mismatch.diagnostics.diagnostics[0]).toMatchObject({
      messageKey: "call.argument-type-mismatch",
      expectedType: "Int",
      actualType: "String",
      typeDifference: {
        entries: [{ message: "expected Int, actual String" }],
      },
    })

    const structuredSource = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/semantic-diagnostics-schema-1/structured-type-differences/main.ssrg",
        import.meta.url
      )
    ).text()
    const structured = await compile("structured.ssrg", structuredSource)
    expect(structured.status).toBe("failure")
    expect(
      structured.diagnostics.diagnostics.map((diagnostic) =>
        diagnostic.typeDifference?.entries.map((entry) => entry.message)
      )
    ).toEqual([
      [
        "profile.score is missing; expected Int",
        "profile.extra is extra; actual type is Bool",
        "enabled is missing; expected Bool",
        "stale is extra; actual type is Int",
      ],
      [
        "parameter 1: expected Int, actual String",
        "return type: expected String, actual Int",
      ],
      [
        "Array type argument 1 > Maybe type argument 1: expected Int, actual String",
      ],
    ])

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
