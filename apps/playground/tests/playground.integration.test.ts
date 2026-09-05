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
import { collectWorkspaceDiagnostics } from "../src/diagnostics/workspace-diagnostics"
import {
  executeGeneratedModule,
  executeGeneratedProject,
  ProjectExecutionError,
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
import {
  activateWorkspaceFile,
  createWorkspace,
  renameWorkspacePath,
  updateWorkspaceFileSource,
} from "../src/workspace/model"
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
  readonly format_single_file_with_options: (
    sourceName: string,
    source: string,
    lineWidth: number
  ) => string
  readonly compile_project: (request: string) => string
  readonly analyze_project: (request: string) => string
  readonly format_project_file: (request: string, path: string) => string
  readonly format_project_file_with_options: (
    request: string,
    path: string,
    lineWidth: number
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

async function format(
  source: string,
  lineWidth?: number
): Promise<FormatResponse> {
  const wasm = await loadBindings()
  return JSON.parse(
    lineWidth === undefined
      ? wasm.format_single_file("main.ssrg", source)
      : wasm.format_single_file_with_options("main.ssrg", source, lineWidth)
  )
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
  path: string,
  lineWidth?: number
): Promise<ProjectFormatResponse> {
  const wasm = await loadBindings()
  const serialized = JSON.stringify(request)
  return JSON.parse(
    lineWidth === undefined
      ? wasm.format_project_file(serialized, path)
      : wasm.format_project_file_with_options(serialized, path, lineWidth)
  )
}

function memoryWebStorage(): Storage {
  const values = new Map<string, string>()
  return {
    get length() {
      return values.size
    },
    clear: () => values.clear(),
    getItem: (key) => values.get(key) ?? null,
    key: (index) => [...values.keys()][index] ?? null,
    removeItem: (key) => {
      values.delete(key)
    },
    setItem: (key, value) => {
      values.set(key, value)
    },
  }
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

  test("executes namespaced Console and structured Logger as separate browser services", async () => {
    const source = [
      'import * as arrays from "std/array"',
      'import * as console from "std/console"',
      'import * as effects from "std/effect"',
      'import * as logs from "std/log"',
      'import { LogEvent } from "std/log"',
      "",
      "type AppError deriving Show =",
      "  | OutputFailure ConsoleError",
      "  | LogFailure logs.LogError",
      "",
      "pub effect fn main -> Unit",
      "with Console, logger: logs.Logger",
      "fails AppError =",
      "  do {",
      "    logs.log (LogEvent {",
      "      level: logs.LogInfo,",
      '      message: "browser",',
      '      fields: arrays.toList [("ordered", logs.LogInt 1)],',
      "    })",
      "      |> effects.mapError LogFailure",
      '    console.println "console" |> effects.mapError OutputFailure',
      "  }",
      "",
    ].join("\n")
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing Console and Logger execution entry")
    }
    expect(response.entry.contract.environment).toEqual([
      { field: "console", service: "console" },
      { field: "logger", service: "logger" },
    ])
    expect(
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({
      stdout:
        '{"level":"info","message":"browser","fields":[["ordered",1]]}\nconsole',
      debug: "()",
    })
  })

  test("preserves grouped arithmetic through the Playground WASM compiler", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/typescript-precedence-grouping/main.ssrg",
        import.meta.url
      )
    ).text()
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    expect(response.status).toBe("success")
    if (response.status !== "success") return
    const typescript = response.modules[0]?.generated.typescript ?? ""
    expect(typescript).toContain(
      '_ssrg_float_div_dictionary["div"](_ssrg_float_sub_dictionary["sub"](value)(1.0))(_ssrg_float_add_dictionary["add"](value)(1.0))'
    )
    expect(typescript).toContain(
      '_ssrg_float_div_dictionary["div"](_ssrg_float_sub_dictionary["sub"](positive)(negative))(2.0)'
    )
    expect(typescript).toContain(
      'double(_ssrg_float_div_dictionary["div"](_ssrg_float_sub_dictionary["sub"](9.0)(1.0))(2.0))'
    )
  })

  test("keeps a renamed nested entry and its diagnostic tab on canonical paths", async () => {
    const initial = createWorkspace({
      files: [
        {
          path: "feature/main.ssrg",
          source:
            'import { value } from "./value"\n\n' +
            "pub effect fn main = value () |> debug |> println\n",
        },
        {
          path: "feature/value.ssrg",
          source: "pub fn value unit: Unit -> Int = 42\n",
        },
      ],
      entryFile: "feature/main.ssrg",
      activeFile: "feature/value.ssrg",
      openFiles: ["feature/main.ssrg", "feature/value.ssrg"],
      dirtyFiles: ["feature/value.ssrg"],
      expandedFolders: ["feature"],
    })
    const renamed = renameWorkspacePath(initial, "feature", "application")
    const request = runnableWorkspaceProjectRequest(renamed)
    const compiled = await compileProject(request)

    expect(request.manifest).toContain('entry = "application/main"')
    expect(renamed.activeFile).toBe("application/value.ssrg")
    expect(renamed.openFiles).toEqual([
      "application/main.ssrg",
      "application/value.ssrg",
    ])
    expect(renamed.dirtyFiles).toEqual(["application/value.ssrg"])
    expect(renamed.expandedFolders).toEqual(["application"])
    expect(compiled.status).toBe("success")
    if (compiled.status !== "success") return
    expect(compiled.entry.path).toBe("application/main.ssrg")

    const broken = updateWorkspaceFileSource(
      renamed,
      "application/value.ssrg",
      'pub fn value unit: Unit -> Int = "wrong"\n'
    )
    const brokenRequest = runnableWorkspaceProjectRequest(broken)
    const failure = await compileProject(brokenRequest)

    expect(failure.status).toBe("failure")
    if (failure.status !== "failure") return
    const diagnostics = collectWorkspaceDiagnostics(
      brokenRequest,
      failure.diagnostics,
      failure.problems
    )
    expect(diagnostics[0]?.path).toBe("application/value.ssrg")
    expect(diagnostics[0]?.source).toBe(
      'pub fn value unit: Unit -> Int = "wrong"\n'
    )
    expect(
      activateWorkspaceFile(broken, diagnostics[0]?.path ?? "").activeFile
    ).toBe("application/value.ssrg")
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

  for (const fixtureName of [
    "effect-match",
    "effect-until",
    "monoid-wrappers",
    "transformers",
    "array-index",
  ]) {
    test(`executes ${fixtureName} across named and namespace imports`, async () => {
      const fixture = new URL(
        `../../../examples/spec/fixtures/projects/${fixtureName}/`,
        import.meta.url
      )
      const request: ProjectRequest = {
        schema: 1,
        manifest: await Bun.file(new URL("seseragi.toml", fixture)).text(),
        files: await Promise.all(
          ["domain.ssrg", "main.ssrg"].map(async (path) => ({
            path,
            source: await Bun.file(new URL(`src/${path}`, fixture)).text(),
          }))
        ),
      }
      const analysis = await analyzeProject(request)
      const compiled = await compileProject(request)
      expect(analysis.status).toBe("success")
      expect(compiled.status).toBe("success")
      if (analysis.status !== "success" || compiled.status !== "success") return
      if (compiled.entry.contract === undefined)
        throw new Error("missing execution entry")
      const result = await executeGeneratedProject(
        compiled.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        compiled.entry.path,
        compiled.entry.contract
      )
      expect(result.stdout).toBe(
        (await Bun.file(new URL("expected.stdout", fixture)).text()).trimEnd()
      )
    })
  }
  test("preserves concrete inline polymorphic payloads across project modules", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/inline-polymorphic-inference/",
      import.meta.url
    )
    const request: ProjectRequest = {
      schema: 1,
      manifest: await Bun.file(new URL("seseragi.toml", fixture)).text(),
      files: await Promise.all(
        ["domain.ssrg", "main.ssrg"].map(async (path) => ({
          path,
          source: await Bun.file(new URL(`src/${path}`, fixture)).text(),
        }))
      ),
    }
    const analysis = await analyzeProject(request)
    const compiled = await compileProject(request)
    expect(analysis.status).toBe("success")
    expect(compiled.status).toBe("success")
    if (analysis.status !== "success" || compiled.status !== "success") return
    const document = analysis.documents.find(
      ({ path }) => path === "main.ssrg"
    )?.document
    for (const [name, type] of [
      ["inlineArray", "Array<Int>"],
      ["inlineList", "List<Int>"],
      ["arrayValue", "Array<Int>"],
      ["listValue", "List<Int>"],
      ["shuffled", "Array<Int>"],
    ]) {
      expect(
        document?.symbols.find((symbol) => symbol.name === name)?.typeName
      ).toBe(type)
    }
    if (compiled.entry.contract === undefined)
      throw new Error("missing execution entry")
    const result = await executeGeneratedProject(
      compiled.modules.map(({ path, generated }) => ({
        path,
        typescript: generated.typescript,
      })),
      compiled.entry.path,
      compiled.entry.contract
    )
    expect(result.stdout).toBe(
      (await Bun.file(new URL("expected.stdout", fixture)).text()).trimEnd()
    )
  })

  test("analyzes, compiles, and executes the portable std parity package", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/std-parity-portable/",
      import.meta.url
    )
    const source = await Bun.file(new URL("src/main.ssrg", fixture)).text()
    const request: ProjectRequest = {
      schema: 1,
      manifest: await Bun.file(new URL("seseragi.toml", fixture)).text(),
      files: [{ path: "main.ssrg", source }],
    }
    const analysis = await analyzeProject(request)
    const response = await compileProject(request)

    expect(analysis.status).toBe("success")
    if (analysis.status !== "success") return
    expect(
      analysis.documents[0]?.document.standardLibrary
        .filter(({ module }) =>
          [
            "std/iterator",
            "std/array",
            "std/bytes",
            "std/float",
            "std/int",
            "std/list",
            "std/map",
            "std/maybe",
            "std/either",
            "std/validation",
            "std/set",
            "std/number",
            "std/text",
            "std/char",
            "std/text/grapheme",
            "std/text/unicode",
          ].includes(module)
        )
        .map(({ identity }) => identity)
    ).toEqual(
      expect.arrayContaining([
        "std/iterator::unfold",
        "std/iterator::next",
        "std/array::filter",
        "std/array::length",
        "std/array::toList",
        "std/bytes::length",
        "std/float::toInt",
        "std/int::saturatingAdd",
        "std/list::length",
        "std/map::fromEntries",
        "std/maybe::sequence",
        "std/either::mapRight",
        "std/validation::invalid",
        "std/set::fromIterable",
        "std/number::HalfEven",
        "std/text::decodeUtf8",
        "std/text::encodeUtf8",
        "std/text::lengthScalars",
        "std/char::codePoint",
        "std/text/grapheme::length",
        "std/text/unicode::normalize",
      ])
    )
    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing std parity execution entry")
    }
    for (const binding of [
      "filter as _ssrg_array_filter",
      "toList as _ssrg_array_toList",
      "arrayIterable as _ssrg_array_iterable",
      "groupBy as _ssrg_array_groupBy",
      "windows as _ssrg_array_windows",
      "length as _ssrg_array_length",
      "arrayTraversable as _ssrg_array_traversable",
      "maybeSequence as _ssrg_maybe_sequence",
      "mapRight as _ssrg_either_mapRight",
      "invalid as _ssrg_validation_invalid",
    ]) {
      expect(response.modules[0]?.generated.typescript).toContain(binding)
    }
    expect(response.modules[0]?.generated.typescript).toContain(
      'length as _ssrg_list_length, type List as List } from "@seseragi/runtime/list"'
    )
    expect(response.modules[0]?.generated.typescript).toContain(
      'length as _ssrg_bytes_length, type Bytes as Bytes } from "@seseragi/runtime/bytes"'
    )
    expect(response.modules[0]?.generated.typescript).toContain(
      [
        "encodeUtf8 as _ssrg_text_encodeUtf8",
        "lengthScalars as _ssrg_text_lengthScalars",
        "lengthBytes as _ssrg_text_lengthBytes",
        "scalarAt as _ssrg_text_scalarAt",
        "decodeUtf8 as _ssrg_text_decodeUtf8",
      ].join(", ")
    )
    expect(response.modules[0]?.generated.typescript).toContain(
      'from "@seseragi/runtime/text"'
    )
    expect(
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({
      stdout: (
        await Bun.file(new URL("expected.stdout", fixture)).text()
      ).trimEnd(),
      debug: "()",
    })
  })

  test("resolves and executes browser Clock and HTTP client providers", async () => {
    const server = Bun.serve({
      port: 0,
      fetch: () => new Response("seseragi"),
    })
    server.unref()
    const url = new URL("seseragi", server.url).href
    const source = [
      'import * as clock from "std/clock"',
      'import * as effects from "std/effect"',
      'import * as http from "std/http"',
      'import * as text from "std/text"',
      'import * as time from "std/time"',
      "",
      "type AppError deriving Show =",
      "  | BuildFailure http.HttpBuildError",
      "  | HttpFailure String",
      "  | TextFailure text.Utf8DecodeError",
      "  | ConsoleFailure ConsoleError",
      "",
      "fn httpFailure error: http.HttpError -> AppError =",
      "  HttpFailure (http.errorMessage error)",
      "",
      "fn preserve instant: time.Instant -> time.Instant = instant",
      "",
      "pub effect fn main -> Unit",
      "with Console, clock: clock.Clock, httpClient: http.HttpClient",
      "fails AppError =",
      "  do {",
      "    current <- clock.now ()",
      "    let instant = preserve current",
      `    url <- http.parseUrl "${url}"`,
      "      |> effects.fromEither",
      "      |> effects.mapError BuildFailure",
      "    response <- http.request http.get url",
      "      |> http.sendEmpty (http.defaultBodyLimit ())",
      "      |> mapError httpFailure",
      "    body <- http.responseBody response",
      "      |> text.decodeUtf8",
      "      |> effects.fromEither",
      "      |> effects.mapError TextFailure",
      "    println `browser providers: $" + "{body}`",
      "      |> mapError ConsoleFailure",
      "  }",
      "",
    ].join("\n")
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing browser provider execution entry")
    }
    expect(response.entry.contract.providers).toEqual([
      expect.objectContaining({
        service: "std/clock::Clock",
        target: "browser",
        entryModule: "seseragi/runtime-browser/clock",
      }),
      expect.objectContaining({
        service: "std/http::HttpClient",
        target: "browser",
        entryModule: "seseragi/runtime-browser/http-client",
      }),
    ])
    expect(response.modules[0]?.generated.typescript).toContain(
      'type Instant as Instant, type Clock as Clock } from "@seseragi/runtime/clock"'
    )
    expect(source).not.toContain("runtime-browser")
    const execution = await executeGeneratedProject(
      response.modules.map(({ path, generated }) => ({
        path,
        typescript: generated.typescript,
      })),
      response.entry.path,
      response.entry.contract
    )
    server.stop(true)
    expect(execution).toEqual({
      stdout: "browser providers: seseragi",
      debug: "()",
    })
  })

  test("executes Navigation push and replace against the preview window", async () => {
    const source = [
      'import * as effects from "std/effect"',
      'import * as navigation from "std/web/navigation"',
      "",
      "type AppError deriving Show =",
      "  | BuildFailure navigation.UrlBuildError",
      "  | NavigationFailure navigation.NavigationError",
      "",
      "pub effect fn main -> Unit",
      "with navigation: navigation.Navigation",
      "fails AppError =",
      "  do {",
      '    first <- navigation.parseUrl "https://example.test/first?tag=one&tag=two"',
      "      |> effects.fromEither",
      "      |> effects.mapError BuildFailure",
      "    _ <- navigation.push first |> mapError NavigationFailure",
      '    final <- navigation.resolveUrl "../final?step=2#done" first',
      "      |> effects.fromEither",
      "      |> effects.mapError BuildFailure",
      "    _ <- navigation.replace final |> mapError NavigationFailure",
      "    succeed ()",
      "  }",
      "",
    ].join("\n")
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing browser navigation execution entry")
    }
    expect(response.entry.contract.providers).toEqual([
      expect.objectContaining({
        service: "std/web/navigation::Navigation",
        target: "browser",
        entryModule: "seseragi/runtime-browser/navigation",
      }),
    ])

    let href = "https://example.test/start"
    const pushes: string[] = []
    const replacements: string[] = []
    const previewWindow = {
      location: {
        get href() {
          return href
        },
      },
      history: {
        pushState: (_state: unknown, _title: string, value: string) => {
          href = new URL(value, href).href
          pushes.push(href)
        },
        replaceState: (_state: unknown, _title: string, value: string) => {
          href = new URL(value, href).href
          replacements.push(href)
        },
        back: () => undefined,
        forward: () => undefined,
      },
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
    } as unknown as Window
    const execution = await executeGeneratedProject(
      response.modules.map(({ path, generated }) => ({
        path,
        typescript: generated.typescript,
      })),
      response.entry.path,
      response.entry.contract,
      "",
      { domDocument: { defaultView: previewWindow } as unknown as Document }
    )

    expect(execution).toEqual({ stdout: "", debug: "()" })
    expect(pushes).toEqual(["https://example.test/first?tag=one&tag=two"])
    expect(replacements).toEqual(["https://example.test/final?step=2#done"])
    expect(href).toBe("https://example.test/final?step=2#done")
  })

  test("executes explicit JSON through browser local and session Storage", async () => {
    const source = [
      'import * as effects from "std/effect"',
      'import * as json from "std/json"',
      'import * as storage from "std/web/storage"',
      "",
      "struct Profile deriving JsonEncode, JsonDecode {",
      "  name: String,",
      "}",
      "",
      "fn decodeProfile text: String -> Either<json.JsonReadError, Profile> =",
      "  json.decodeString text",
      "",
      "type AppError deriving Show =",
      "  | StorageFailure storage.StorageError",
      "  | MissingProfile",
      "  | ConsoleFailure ConsoleError",
      "",
      "pub effect fn main -> Unit",
      "with Console, storage: storage.Storage",
      "fails AppError =",
      "  do {",
      '    encoded <- succeed (json.encodeString (Profile { name: "Mio" }))',
      "    _ <- storage.clear storage.Local",
      "      |> mapError StorageFailure",
      '    _ <- storage.set storage.Local "profile" encoded',
      "      |> mapError StorageFailure",
      '    _ <- storage.set storage.Session "draft" "open"',
      "      |> mapError StorageFailure",
      '    stored <- storage.get storage.Local "profile"',
      "      |> mapError StorageFailure",
      "    text <- effects.fromMaybe MissingProfile stored",
      "    let profileName = match decodeProfile text {",
      '      Left _ -> "decode error"',
      "      Right profile -> profile.name",
      "    }",
      "    _ <- println profileName |> mapError ConsoleFailure",
      '    _ <- storage.remove storage.Session "draft"',
      "      |> mapError StorageFailure",
      "    succeed ()",
      "  }",
      "",
    ].join("\n")
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    if (response.status === "failure") {
      throw new Error(JSON.stringify(response))
    }
    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing browser storage execution entry")
    }
    expect(response.entry.contract.providers).toEqual([
      expect.objectContaining({
        service: "std/web/storage::Storage",
        target: "browser",
        entryModule: "seseragi/runtime-browser/storage",
      }),
    ])

    const localStorage = memoryWebStorage()
    const sessionStorage = memoryWebStorage()
    localStorage.setItem("seseragi:playground:workspace", "keep")
    const previewWindow = { localStorage, sessionStorage } as unknown as Window
    const execution = await executeGeneratedProject(
      response.modules.map(({ path, generated }) => ({
        path,
        typescript: generated.typescript,
      })),
      response.entry.path,
      response.entry.contract,
      "",
      { domDocument: { defaultView: previewWindow } as unknown as Document }
    )

    expect(execution).toEqual({ stdout: "Mio", debug: "()" })
    expect(localStorage.getItem("seseragi:playground:workspace")).toBe("keep")
    expect(localStorage.getItem("profile")).toBeNull()
    expect(
      Array.from({ length: localStorage.length }, (_, index) =>
        localStorage.getItem(localStorage.key(index) ?? "")
      )
    ).toContain('{"name":"Mio"}')
    expect(sessionStorage.getItem("draft")).toBeNull()
  })

  test("renders browser quota failure from normal Seseragi source", async () => {
    const source = [
      'import * as storage from "std/web/storage"',
      "",
      "type AppError deriving Show =",
      "  | StorageFailure storage.StorageError",
      "",
      "pub effect fn main -> Unit",
      "with storage: storage.Storage",
      "fails AppError =",
      '  storage.set storage.Local "profile" "large"',
      "    |> mapError StorageFailure",
      "",
    ].join("\n")
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing browser storage quota execution entry")
    }
    const quotaStorage = {
      length: 0,
      clear: () => undefined,
      getItem: () => null,
      key: () => null,
      removeItem: () => undefined,
      setItem: () => {
        throw Object.freeze({
          name: "QuotaExceededError",
          message: "storage quota reached",
        })
      },
    } as Storage
    const previewWindow = {
      localStorage: quotaStorage,
      sessionStorage: memoryWebStorage(),
    } as unknown as Window

    try {
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract,
        "",
        { domDocument: { defaultView: previewWindow } as unknown as Document }
      )
      throw new Error("browser quota failure unexpectedly succeeded")
    } catch (error) {
      expect(error).toBeInstanceOf(Error)
      const message = (error as Error).message
      expect(message).toContain("StorageQuotaExceeded")
      expect(message).toContain('key: "profile"')
      expect(message).toContain('message: "storage quota reached"')
    }
  })

  test("executes effect-temporal-control through WASM and browser Clock", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/effect-temporal-control/",
      import.meta.url
    )
    const source = await Bun.file(new URL("src/main.ssrg", fixture)).text()
    const expectedOutput = await Bun.file(
      new URL("expected.stdout", fixture)
    ).text()
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing effect temporal execution entry")
    }
    expect(response.entry.contract.providers).toEqual([
      expect.objectContaining({
        service: "std/clock::Clock",
        target: "browser",
        entryModule: "seseragi/runtime-browser/clock",
      }),
    ])
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

  test("executes effect-resource-scope through WASM", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/effect-resource-scope/",
      import.meta.url
    )
    const source = await Bun.file(new URL("src/main.ssrg", fixture)).text()
    const expectedOutput = await Bun.file(
      new URL("expected.stdout", fixture)
    ).text()
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing effect resource execution entry")
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

  test("executes Stream and typeclass operator fixtures through WASM", async () => {
    for (const name of [
      "stream-cold-resource",
      "effect-stream-simultaneous-failure",
      "typeclass-operator-parity",
    ]) {
      const fixture = new URL(
        `../../../examples/spec/fixtures/projects/${name}/`,
        import.meta.url
      )
      const source = await Bun.file(new URL("src/main.ssrg", fixture)).text()
      const expectedOutput = await Bun.file(
        new URL("expected.stdout", fixture)
      ).text()
      const response = await compileProject({
        schema: 1,
        entry: "main.ssrg",
        files: [{ path: "main.ssrg", source }],
      })

      expect(response.status).toBe("success")
      if (
        response.status !== "success" ||
        response.entry.contract === undefined
      ) {
        throw new Error(`missing Stream execution entry for ${name}`)
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
    }
  })

  test("executes imported standard evidence through WASM", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/standard-evidence-parity/",
      import.meta.url
    )
    const [main, evidence, expectedOutput] = await Promise.all([
      Bun.file(new URL("src/main.ssrg", fixture)).text(),
      Bun.file(new URL("src/evidence.ssrg", fixture)).text(),
      Bun.file(new URL("expected.stdout", fixture)).text(),
    ])
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        { path: "main.ssrg", source: main },
        { path: "evidence.ssrg", source: evidence },
      ],
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing standard evidence execution entry")
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

  test("executes canonical Random shuffle through WASM and browser providers", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/random-shuffle/",
      import.meta.url
    )
    const source = await Bun.file(new URL("src/main.ssrg", fixture)).text()
    const expected = await Bun.file(new URL("expected.stdout", fixture)).text()
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })
    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing Random shuffle execution entry")
    }
    expect(response.modules[0]?.generated.typescript).toContain(
      "@seseragi/runtime/random"
    )
    const previousSeed = globalThis.__SESERAGI_RANDOM_SEED__
    globalThis.__SESERAGI_RANDOM_SEED__ = "42"
    try {
      const result = await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
      expect(result).toEqual({ stdout: expected.trimEnd(), debug: "()" })
      expect(JSON.parse(result.stdout).sort()).toEqual([
        1, 2, 3, 4, 5, 6, 7, 8, 9,
      ])
    } finally {
      globalThis.__SESERAGI_RANDOM_SEED__ = previousSeed
    }
  })

  test("keeps intentionally unavailable Float Eq as SES-T0201", async () => {
    const source = await Bun.file(
      new URL(
        "../../../crates/seseragi-cli/tests/fixtures/standard-evidence-float-eq-negative.ssrg",
        import.meta.url
      )
    ).text()
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    expect(response.status).toBe("failure")
    if (response.status !== "failure") return
    const diagnostics = response.diagnostics.flatMap(
      ({ diagnostics }) => diagnostics.diagnostics
    )
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "SES-T0201",
          related: expect.arrayContaining([
            expect.objectContaining({
              message: expect.stringContaining(
                "no Eq instance matches the inferred call arguments"
              ),
            }),
          ]),
        }),
      ])
    )
  })

  test("keeps intentionally unavailable Float Hash as SES-T0201", async () => {
    const source = await Bun.file(
      new URL(
        "../../../crates/seseragi-cli/tests/fixtures/standard-evidence-float-hash-negative.ssrg",
        import.meta.url
      )
    ).text()
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    })

    expect(response.status).toBe("failure")
    if (response.status !== "failure") return
    const diagnostics = response.diagnostics.flatMap(
      ({ diagnostics }) => diagnostics.diagnostics
    )
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "SES-T0201",
          related: expect.arrayContaining([
            expect.objectContaining({
              message: expect.stringContaining(
                "no Hash instance matches the inferred call arguments"
              ),
            }),
          ]),
        }),
      ])
    )
  })

  test("diagnoses browser-unsupported standard imports before execution", async () => {
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "main.ssrg",
          source: [
            'import * as server from "std/http/server"',
            "",
            "pub effect fn main -> Unit with httpServer: server.HttpServer =",
            "  succeed ()",
            "",
          ].join("\n"),
        },
      ],
    })

    expect(response.status).toBe("failure")
    if (response.status !== "failure") return
    expect(response.problems).toEqual([
      expect.objectContaining({
        code: "SES-K0203",
        label: "provider.target-mismatch",
        details: expect.objectContaining({
          target: "browser",
          required: ["std/http/server"],
          compatibleTargets: ["process"],
          reasons: ["standard-module-target"],
        }),
      }),
    ])
  })

  test("diagnoses Navigation on a non-browser target before execution", async () => {
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "main.ssrg",
          source: [
            'import * as navigation from "std/web/navigation"',
            "",
            "pub effect fn main -> Unit",
            "with navigation: navigation.Navigation",
            "fails navigation.NavigationError =",
            "  navigation.back ()",
            "",
          ].join("\n"),
        },
      ],
      provider: {
        target: "bun-process",
        backendFamily: "typescript",
        backendAbiMajor: 1,
      },
    })

    expect(response.status).toBe("failure")
    if (response.status !== "failure") return
    expect(response.problems).toEqual([
      expect.objectContaining({
        code: "SES-K0203",
        label: "provider.target-mismatch",
        details: expect.objectContaining({
          target: "bun-process",
          required: ["std/web/navigation"],
          compatibleTargets: ["browser"],
          reasons: ["standard-module-target"],
        }),
      }),
    ])
  })

  test("diagnoses Storage on a non-browser target before execution", async () => {
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "main.ssrg",
          source: [
            'import * as storage from "std/web/storage"',
            "",
            "pub effect fn main -> Unit",
            "with storage: storage.Storage",
            "fails storage.StorageError =",
            "  storage.clear storage.Local",
            "",
          ].join("\n"),
        },
      ],
      provider: {
        target: "bun-process",
        backendFamily: "typescript",
        backendAbiMajor: 1,
      },
    })

    expect(response.status).toBe("failure")
    if (response.status !== "failure") return
    expect(response.problems).toEqual([
      expect.objectContaining({
        code: "SES-K0203",
        label: "provider.target-mismatch",
        details: expect.objectContaining({
          target: "bun-process",
          required: ["std/web/storage"],
          compatibleTargets: ["browser"],
          reasons: ["standard-module-target"],
        }),
      }),
    ])
  })

  test("shares the target diagnostic contract with the CLI package", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/std-parity-target/",
      import.meta.url
    )
    const request: ProjectRequest = {
      schema: 1,
      manifest: await Bun.file(new URL("seseragi.toml", fixture)).text(),
      files: [
        {
          path: "main.ssrg",
          source: await Bun.file(new URL("src/main.ssrg", fixture)).text(),
        },
      ],
      provider: {
        target: "bun-process",
        backendFamily: "typescript",
        backendAbiMajor: 1,
      },
    }
    const analysis = await analyzeProject(request)
    const compilation = await compileProject(request)

    expect(analysis.status).toBe("failure")
    expect(compilation.status).toBe("failure")
    if (analysis.status !== "failure" || compilation.status !== "failure") {
      return
    }
    expect(compilation.problems).toEqual(analysis.problems)
    expect(compilation.problems).toEqual([
      expect.objectContaining({
        code: "SES-K0203",
        label: "provider.target-mismatch",
        details: expect.objectContaining({
          target: "bun-process",
          required: ["std/web/dom"],
          compatibleTargets: ["browser"],
          reasons: ["standard-module-target"],
        }),
      }),
    ])
  })

  test("executes only modules reachable from the project entry", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/entry-rooted-runtime/src/",
      import.meta.url
    )
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: await Promise.all(
        ["dependency.ssrg", "main.ssrg", "unused.ssrg"].map(async (path) => ({
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
      throw new Error("missing entry-rooted project execution entry")
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
    ).toEqual({ stdout: "entry graph only", debug: "()" })
  })

  test("keeps compile diagnostics for an unreachable source file", async () => {
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "main.ssrg",
          source: 'pub effect fn main = println "ok"\n',
        },
        {
          path: "unused.ssrg",
          source: 'pub let broken: Int = "not an Int"\n',
        },
      ],
    })

    expect(response.status).toBe("failure")
    if (response.status !== "failure") return
    expect(
      response.diagnostics.find(({ path }) => path === "unused.ssrg")
        ?.diagnostics.diagnostics
    ).toContainEqual(expect.objectContaining({ code: "SES-T0101" }))
  })

  test("evaluates a shared dependency once through a diamond", async () => {
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "shared.ssrg",
          source: "pub fn value _unit: Unit -> Int = 21\n",
        },
        {
          path: "left.ssrg",
          source:
            'import { value } from "./shared"\n\n' +
            "pub fn left _unit: Unit -> Int = value ()\n",
        },
        {
          path: "right.ssrg",
          source:
            'import { value } from "./shared"\n\n' +
            "pub fn right _unit: Unit -> Int = value ()\n",
        },
        {
          path: "main.ssrg",
          source:
            'import { left } from "./left"\n' +
            'import { right } from "./right"\n\n' +
            `pub effect fn main = println $ \`\${left ()}:\${right ()}\`\n`,
        },
      ],
    })

    expect(response.status).toBe("success")
    if (
      response.status !== "success" ||
      response.entry.contract === undefined
    ) {
      throw new Error("missing diamond project execution entry")
    }

    const evaluationKey = "__seseragiIssue197SharedEvaluations"
    const host = globalThis as typeof globalThis &
      Record<string, number | undefined>
    host[evaluationKey] = 0
    try {
      const counter = JSON.stringify(evaluationKey)
      expect(
        await executeGeneratedProject(
          response.modules.map(({ path, generated }) => ({
            path,
            typescript:
              path === "shared.ssrg"
                ? `globalThis[${counter}] = (globalThis[${counter}] ?? 0) + 1\n${generated.typescript}`
                : generated.typescript,
          })),
          response.entry.path,
          response.entry.contract
        )
      ).toEqual({ stdout: "21:21", debug: "()" })
      expect(host[evaluationKey]).toBe(1)
    } finally {
      delete host[evaluationKey]
    }
  })

  test("returns a structured error when generated output omits the entry", async () => {
    const entry: EntryContract = {
      environment: [],
      failureRenderer: { kind: "never" },
    }

    try {
      await executeGeneratedProject([], "main.ssrg", entry)
      throw new Error("expected the missing entry to reject")
    } catch (error) {
      expect(error).toBeInstanceOf(ProjectExecutionError)
      expect(error).toMatchObject({
        code: "missing-entry",
        entryPath: "main.ssrg",
        message: "generated project omitted entry module: main.ssrg",
      })
    }
  })

  test("executes every persistent Map / Set operation and dictionary through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/map-set/main.ssrg",
        import.meta.url
      )
    ).text()
    const expected = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/map-set/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("map-set.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error("missing Map / Set execution entry")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
  })

  test("executes Maybe/Either APIs and conditional Monoid through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/maybe-either-apis/main.ssrg",
        import.meta.url
      )
    ).text()
    const expected = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/maybe-either-apis/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("maybe-either-apis.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error("missing Maybe/Either entry")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
  })

  test("executes canonical Validation accumulation and conditional dictionaries through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/validation-apis/main.ssrg",
        import.meta.url
      )
    ).text()
    const expected = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/validation-apis/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("validation-apis.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error("missing Validation entry")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
  })

  test("executes pinned Unicode scalar, byte and grapheme APIs through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/unicode-text/main.ssrg",
        import.meta.url
      )
    ).text()
    const expected = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/unicode-text/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("unicode-text.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error("missing Unicode execution entry")
    expect(response.generated.typescript).toContain(
      '$ssrg$assertUnicodeVersion("17.0.0")'
    )
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
    const formatted = await format(source)
    expect(formatted.status).toBe("success")
  })

  test("executes imported Unicode APIs and enforces dependency guards in the browser runtime", async () => {
    const root =
      "../../../examples/spec/artifacts/project-schema-1/imported-unicode/"
    const files = await Promise.all(
      ["main", "operations"].map(async (name) => ({
        path: `${name}.ssrg`,
        source: await Bun.file(
          new URL(`${root}src/${name}.ssrg`, import.meta.url)
        ).text(),
      }))
    )
    const expected = await Bun.file(
      new URL(`${root}stdout.txt`, import.meta.url)
    ).text()
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files,
    })
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry.contract)
      throw new Error("missing imported Unicode execution entry")
    const modules = response.modules.map(({ path, generated }) => ({
      path,
      typescript: generated.typescript,
    }))
    expect(
      await executeGeneratedProject(
        modules,
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
    const incompatible = modules.map((module) => ({
      ...module,
      typescript:
        module.path === "operations.ssrg"
          ? module.typescript.replace(
              '$ssrg$assertUnicodeVersion("17.0.0")',
              '$ssrg$assertUnicodeVersion("18.0.0")'
            )
          : module.typescript,
    }))
    await expect(
      executeGeneratedProject(
        incompatible,
        response.entry.path,
        response.entry.contract
      )
    ).rejects.toThrow("runtime ABI mismatch")
  })

  test("rejects Monad for Validation through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/validation-no-monad/main.ssrg",
        import.meta.url
      )
    ).text()
    const response = await compile("validation-no-monad.ssrg", source)
    expect(response.status).not.toBe("success")
    expect(JSON.stringify(response)).toContain("SES-T0201")
  })

  test("executes generic short-circuit traversal through WASM and the runtime registry", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/collection-reduce-until/main.ssrg",
        import.meta.url
      )
    ).text()
    const response = await compile("collection-reduce-until.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error("missing short-circuit entry")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({
      stdout:
        "(3, 3)\n(6, 6, 6)\n(6, 6)\n42\nabc\n(2, ab)\n(7, done)\n(12, 13)",
      debug: "()",
    })
  })

  test("executes all Array / List APIs, SizeError and Ord evidence through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/array-list-apis/main.ssrg",
        import.meta.url
      )
    ).text()
    const expected = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/array-list-apis/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("array-list-apis.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry)
      throw new Error("missing sequence entry")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
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

  test("executes Eq-distinct Signal publications through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/signal-distinct/main.ssrg",
        import.meta.url
      )
    ).text()
    const expectedOutput = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/signal-distinct/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("signal-distinct.ssrg", source)

    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing Signal.distinct execution entry")
    }
    expect(response.generated.typescript).toContain("_ssrg_signal_distinct")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expectedOutput.trimEnd(), debug: "()" })
  })

  test("executes logical conditions with branch values through WASM", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/logical-short-circuit/",
      import.meta.url
    )
    const source = await Bun.file(new URL("src/main.ssrg", fixture)).text()
    const expectedOutput = await Bun.file(
      new URL("expected.stdout", fixture)
    ).text()
    const response = await compile("logical-short-circuit.ssrg", source)

    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing logical short-circuit execution entry")
    }
    expect(response.generated.typescript).toContain(
      '(a ? b : false) ? "both" : "not-both"'
    )
    expect(response.generated.typescript).toContain("(a ? true : b) ? 1 : 2")
    expect(response.generated.typescript).toContain(
      '((a ? true : b) ? c : false) ? "mixed-left" : "mixed-left-no"'
    )
    expect(response.generated.typescript).toContain(
      '(false ? unavailable(undefined) : false) ? "wrong" : "and-safe"'
    )
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expectedOutput.trimEnd(), debug: "()" })
  })

  test("rejects malformed namespaced reduce before WASM lowering", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/fixtures/projects/namespaced-reduce-rejection/src/main.ssrg",
        import.meta.url
      )
    ).text()
    const response = await compile("namespaced-reduce-rejection.ssrg", source)

    expect(response.status).toBe("failure")
    if (response.status !== "failure") {
      throw new Error("malformed namespaced reduce unexpectedly compiled")
    }
    expect(response.diagnostics.diagnostics).toHaveLength(1)
    expect(response.diagnostics.diagnostics[0]).toMatchObject({
      code: "SES-P0001",
      messageKey: "parser.expected-expression",
      primary: { start: 84, end: 159 },
    })
  })

  test("executes canonical reduce with curried lambdas through WASM", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/prelude-reduce-lambda/",
      import.meta.url
    )
    const source = await Bun.file(new URL("src/main.ssrg", fixture)).text()
    const expectedOutput = await Bun.file(
      new URL("expected.stdout", fixture)
    ).text()
    const response = await compile("prelude-reduce-lambda.ssrg", source)

    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing canonical reduce execution entry")
    }
    expect(response.generated.typescript).toContain("_ssrg_array_reduce")
    expect(response.generated.typescript).toContain("_ssrg_list_reduce")
    expect(response.generated.typescript).not.toContain(" = _")
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

    if (response.status !== "success") {
      throw new Error(JSON.stringify(response))
    }
    expect(response.status).toBe("success")
    if (response.entry.contract === undefined) {
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

  test("keeps module nominal identity inside nested generics", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/module-generic-nominal-identity/vendor/ui/src/",
      import.meta.url
    )
    const main = `import * as model from "./model"
import { child, items, signaled, view, wrapped } from "./component"
import * as html from "std/web/html"
import * as signals from "std/signal"

fn direct unit: Unit -> html.Html<model.Action> = view ()
fn userGeneric unit: Unit -> model.Envelope<html.Html<model.Action>> = wrapped ()
fn nested unit: Unit -> Array<html.Html<model.Action>> = items ()
fn reactive unit: Unit -> signals.Signal<html.Html<model.Action>> = signaled ()
fn children unit: Unit -> html.Html<model.Action> = child ()

pub effect fn main = println "module generic identity: ok"
`
    const aliasConsumer = `import { Action as LocalAction } from "./model"
import { view } from "./component"
import * as html from "std/web/html"

pub fn aliased unit: Unit -> html.Html<LocalAction> = view ()
`
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: [
        {
          path: "model.ssrg",
          source: await Bun.file(new URL("model.ssrg", fixture)).text(),
        },
        {
          path: "component.ssrg",
          source: await Bun.file(new URL("component.ssrg", fixture)).text(),
        },
        { path: "alias-consumer.ssrg", source: aliasConsumer },
        { path: "main.ssrg", source: main },
      ],
    })

    if (response.status !== "success") {
      throw new Error(JSON.stringify(response))
    }
    expect(response.status).toBe("success")
    if (response.entry.contract === undefined) {
      throw new Error(response.entry.error ?? "missing project execution entry")
    }
    expect(response.diagnostics).toEqual([])
    expect(
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({ stdout: "module generic identity: ok", debug: "()" })
  })

  test("keeps imported generic identity in public struct fields", async () => {
    const fixture = new URL(
      "../../../examples/spec/fixtures/projects/struct-field-generic-identity/src/",
      import.meta.url
    )
    const response = await compileProject({
      schema: 1,
      entry: "main.ssrg",
      files: await Promise.all(
        ["domain.ssrg", "context.ssrg", "qualified.ssrg", "main.ssrg"].map(
          async (path) => ({
            path,
            source: await Bun.file(new URL(path, fixture)).text(),
          })
        )
      ),
    })

    if (response.status !== "success") {
      throw new Error(JSON.stringify(response))
    }
    if (response.entry.contract === undefined) {
      throw new Error(response.entry.error ?? "missing project execution entry")
    }
    expect(response.diagnostics).toEqual([])
    expect(
      await executeGeneratedProject(
        response.modules.map(({ path, generated }) => ({
          path,
          typescript: generated.typescript,
        })),
        response.entry.path,
        response.entry.contract
      )
    ).toEqual({ stdout: "struct field identity: ok", debug: "()" })
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

  test("passes an explicit line width through the committed project WASM boundary", async () => {
    const source =
      'let labels = ["formatter", "playground", "curriculum", "diagnostics"]\n'
    const formatRequest: ProjectRequest = {
      schema: 1,
      entry: "main.ssrg",
      files: [{ path: "main.ssrg", source }],
    }

    const wide = await formatProjectFile(formatRequest, "main.ssrg")
    const narrow = await formatProjectFile(formatRequest, "main.ssrg", 48)

    expect(wide.status).toBe("success")
    expect(narrow.status).toBe("success")
    if (wide.status !== "success" || narrow.status !== "success") return
    expect(narrow.source).not.toBe(wide.source)
    expect(narrow.source).toContain("[\n")
    expect(
      await formatProjectFile(
        {
          ...formatRequest,
          files: [{ path: "main.ssrg", source: narrow.source }],
        },
        "main.ssrg",
        48
      )
    ).toMatchObject({
      status: "success",
      source: narrow.source,
      changed: false,
    })
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
      if (lesson.expectedFailure !== "") {
        await expect(
          executeGeneratedModule(
            response.generated.typescript,
            response.entry,
            lesson.stdin
          )
        ).rejects.toThrow(lesson.expectedFailure)
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
      expect(sample.sourcePath).toBe(
        sample.manifestPath === undefined
          ? `examples/samples/${sample.id}/main.ssrg`
          : `examples/samples/${sample.id}/src/main.ssrg`
      )
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
            `${sample.manifest === "" ? "" : `${sample.manifest}\0`}${sample.workspace.files
              .map(({ path, source }) => `${path}\0${source}\0`)
              .join("")}`
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
    const byId = new Map(samples.map((sample) => [sample.id, sample]))
    expect(
      discoverGroups.flatMap((group) =>
        group.samples.filter((id) => byId.get(id)?.featured)
      )
    ).toEqual([
      "collections",
      "project-greeting",
      "html-components",
      "interactive-app",
      "signal-run-route",
      "seseragi-landing-page",
      "form-todo",
      "project-flow-app",
    ])
    expect(
      samples
        .filter((sample) => sample.featured)
        .every((sample) => sample.kind !== "lesson")
    ).toBe(true)
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
        activeFile: "app.ssrg",
        openFiles: [
          "main.ssrg",
          "app.ssrg",
          "ui/components.ssrg",
          "focus/model.ssrg",
          "notes/model.ssrg",
        ],
        expandedFolders: ["ui", "focus", "notes"],
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

  test("keeps surface switching global and Discover workspace-local", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()

    expect(html).toContain('id="sample-browser-button"')
    expect(html).toContain('aria-label="Workspaceとsampleを選ぶ"')
    expect(html).toContain('id="surface-switcher-button"')
    expect(html).toContain('href="./tour/" role="menuitem"')
    expect(html).toContain('class="workspace-editor-chrome"')
    expect(html).toContain("順序を持つ学習はTour")
    expect(html).toContain("Minimal / GuidedはWeb UI例の説明量")
    expect(html).not.toContain('id="sample-browser-learn-tab"')
    expect(html).not.toContain('id="sample-browser-discover-tab"')
    expect(html).not.toContain('class="sample-browser-tabs"')
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
    expect(main).not.toContain("sampleBrowserLearnTab")
    const browser = await Bun.file(
      new URL("../src/ui/sample-browser.ts", import.meta.url)
    ).text()
    expect(browser).toContain('kind !== "lesson"')
    expect(browser).toContain("sample-discover-group")
    expect(browser).toContain("experienceLabel(sample.experience)")
    expect(browser).toContain("architectureLabel(sample.architecture)")
    expect(browser).toContain("sample-card-prerequisite")
    expect(browser).toContain("sample-card-comparison")
    expect(browser).not.toContain("setMode")
    expect(browser).not.toContain("前提:")
    expect(browser).not.toContain("次:")
  })

  test("starts from the canonical starter and keeps Blank distinct", async () => {
    const html = await Bun.file(
      new URL("../index.html", import.meta.url)
    ).text()
    const main = await Bun.file(
      new URL("../src/main.ts", import.meta.url)
    ).text()
    const hello = samples.find((sample) => sample.id === "hello-world")

    expect(hello).toBeDefined()
    expect(html).toContain('id="sample-new-blank-button"')
    expect(html).toContain('id="sample-starter-button"')
    expect(html).toContain('id="reset-sample-button"')
    expect(html).toContain("空のmain.ssrgから書く")
    expect(main).toContain('sample.id === "hello-world"')
    expect(main).toContain("createWorkspace(defaultSample.workspace)")
    expect(main).toContain('createSingleFileWorkspace("")')
    expect(main).toContain("currentSample ?? blankWorkspaceOrigin")
    expect(main).toContain('restoredWorkspace.status === "restored"')
    expect(main).toContain("editorSessions.reset(workspaceState)")
    expect(main).toContain("editor.dispatch(setDiagnostics(editor.state, []))")
    expect(main).toContain("cancelActiveExecution()")
    expect(main).toContain("showTextOutput(")
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
      formItems.find((item) => item.name === "ChangeEvent")?.description
    ).toContain("Just for checkbox/radio, Nothing for value controls")
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
    const fixtureRoot = new URL(
      "../../../crates/seseragi-formatter/tests/fixtures/",
      import.meta.url
    )
    const source = await Bun.file(
      new URL("canonical-layout.input.ssrg", fixtureRoot)
    ).text()
    const expected = await Bun.file(
      new URL("canonical-layout.expected.ssrg", fixtureRoot)
    ).text()

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

  test("keeps explicit-width single-file WASM formatting option-idempotent", async () => {
    const source =
      'let labels = ["formatter", "playground", "curriculum", "diagnostics"]\n'
    const narrow = await format(source, 48)

    expect(narrow.status).toBe("success")
    if (narrow.status !== "success") return
    expect(narrow.source).toContain("[\n")
    expect(await format(narrow.source, 48)).toEqual({
      ...narrow,
      changed: false,
    })
  })

  test("preserves bodyless declaration boundaries through WASM formatting", async () => {
    const fixtureRoot = new URL(
      "../../../crates/seseragi-formatter/tests/fixtures/",
      import.meta.url
    )
    const source = await Bun.file(
      new URL("declaration-boundaries.input.ssrg", fixtureRoot)
    ).text()
    const expected = await Bun.file(
      new URL("declaration-boundaries.expected.ssrg", fixtureRoot)
    ).text()

    expect(await format(source)).toEqual({
      status: "success",
      schema: 1,
      source: expected,
      changed: true,
    })
    expect(await format(expected)).toEqual({
      status: "success",
      schema: 1,
      source: expected,
      changed: false,
    })
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

  test("preserves arbitrary-precision BigInt through WASM execution", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/big-int-apis/main.ssrg",
        import.meta.url
      )
    ).text()

    const response = await compile("big-int-apis.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing BigInt execution entry")
    }
    expect(response.generated.typescript).toContain(
      'from "@seseragi/runtime/big-int"'
    )
    expect(response.generated.typescript).not.toContain("Number(")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({
      stdout: [
        "exact: 2999999999999999999999999999999999998",
        "subtract: 1999999999999999999999999999999999999",
        "radix: c097ce7bc90715b34b9f0fffffffff",
        "division: -3 / -2 / -3 / -2 / BigIntDivisionByZero",
        "power: 1267650600228229401496703205376 / NegativeBigIntExponent -1",
        "conversion: 42 / BigIntOutsideIntRange",
        "parse-errors: Left EmptyBigInt / Left InvalidBigIntDigit { offset: 1, radix: 10 } / Left InvalidBigIntDigit { offset: 2, radix: 10 }",
        "magnitude: 17 / -1",
        "display: 999999999999999999999999999999999999 / 999999999999999999999999999999999999",
        "instances: True / Less / True / 0 / 1",
        "typed-errors: True / True",
      ].join("\n"),
      debug: "()",
    })
  })

  test("dispatches comparison operators through Ord in WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/user-ord-operator/main.ssrg",
        import.meta.url
      )
    ).text()
    const expected = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/execution-schema-1/user-ord-operator/stdout.txt",
        import.meta.url
      )
    ).text()
    const response = await compile("user-ord-operator.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing Ord execution entry")
    }
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expected.trimEnd(), debug: "()" })
    for (const operator of ["<", "<=", ">", ">="]) {
      const negative = await compile(
        "missing-ord.ssrg",
        `pub let invalid = 1.0 ${operator} 2.0`
      )
      expect(negative.status).not.toBe("success")
    }
  })

  test("preserves exact Decimal values and rounding through WASM", async () => {
    const source = await Bun.file(
      new URL(
        "../../../examples/spec/artifacts/schema-1/decimal-apis/main.ssrg",
        import.meta.url
      )
    ).text()
    const expectedOutput = JSON.parse(
      await Bun.file(
        new URL(
          "../../../examples/spec/artifacts/execution-schema-1/decimal-apis/stdout.txt",
          import.meta.url
        )
      ).text()
    ) as string[]

    const response = await compile("decimal-apis.ssrg", source)
    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing Decimal execution entry")
    }
    expect(response.generated.typescript).toContain(
      'from "@seseragi/runtime/decimal"'
    )
    expect(response.generated.typescript).not.toContain("Number(")
    expect(
      await executeGeneratedModule(
        response.generated.typescript,
        response.entry
      )
    ).toEqual({ stdout: expectedOutput.join("\n"), debug: "()" })
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

  test("builds and executes the DomRuntimeError<Never> renderer", async () => {
    const response = await compile(
      "dom-runtime-never.ssrg",
      `import * as dom from "std/web/dom"

pub effect fn main -> Unit
fails dom.DomRuntimeError<Never> =
  succeed ()
`
    )

    expect(response.status).toBe("success")
    if (response.status !== "success" || !response.entry) {
      throw new Error("missing DomRuntimeError<Never> execution entry")
    }
    expect(response.entry.failureRenderer).toEqual({
      kind: "show",
      module: "@seseragi/runtime/show",
      export: "domRuntimeErrorShow",
      arguments: [
        {
          module: "@seseragi/runtime/show",
          export: "neverShow",
        },
      ],
    })

    const source = `
      import { fail } from "@seseragi/runtime/effect"
      export const main = (_unit: undefined) =>
        fail({ tag: "DomFailure", value: { tag: "DomTargetRemoved" } })
    `
    await expect(
      executeGeneratedModule(source, response.entry)
    ).rejects.toThrow("DomFailure DomTargetRemoved")
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
    const runtimePreview = await Bun.file(
      new URL("../runtime-preview.html", import.meta.url)
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
    expect(main).toContain("runtime-preview.html")
    expect(main).toContain("document.head.replaceChildren")
    expect(runtimePreview).toContain("Seseragi Runtime Preview")
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
fails String =
  do {
    imageUrl <-
      parseSampleUrl "https://images.unsplash.com/photo-1441974231531-c6227db76b6e?fit=crop&w=960&h=480&q=80"
    println (html.renderToString (view imageUrl initialState))
    |> mapError (\\error: ConsoleError -> show error)
    println (html.renderToString (view imageUrl (update ChooseRiverside initialState)))
    |> mapError (\\error: ConsoleError -> show error)
    println (html.renderToString (view imageUrl (update ChooseWoodland initialState)))
    |> mapError (\\error: ConsoleError -> show error)
    println (html.renderToString (view imageUrl (update ChooseRidge initialState)))
    |> mapError (\\error: ConsoleError -> show error)
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

  test("integrates the advanced Launch Loop surface in one feature-owned sample", () => {
    const sample = samples.find((candidate) => candidate.id === "form-todo")

    expect(sample?.interactive).toBe(true)
    expect(sample?.source).toContain("MutableSignal<Model>")
    expect(sample?.source).toContain("Signal<html.Html<Task<Unit>>>")
    expect(sample?.source).toContain("onSubmit: dispatch state Submitted")
    expect(sample?.source).toContain("html.img {")
    expect(sample?.source).toContain("html.a {")
    expect(sample?.source).toContain("html.textarea {")
    expect(sample?.source).toContain("TrackChanged")
    expect(sample?.source).toContain("ToggleComplete")
    expect(sample?.source).toContain("TogglePinned")
    expect(sample?.source).toContain("ClearCompleted")
    expect(sample?.source).toContain("fn planCard")
    expect(sample?.source).toContain("fn emptyState")
    expect(sample?.source).toContain("onKeyDown: filterKeyTask state")
    expect(sample?.source).toContain("onPointerDown: pointerTask state")
    expect(sample?.source).toContain("stopClickPropagation: True")
    expect(sample?.source).toContain('role: "alert"')
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
