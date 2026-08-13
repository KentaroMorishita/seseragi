import init, {
  analyze_project,
  analyze_single_file,
  compile_project,
  format_project_file,
  format_project_file_with_options,
} from "../wasm/pkg/seseragi_wasm"
import type {
  AnalysisDocument,
  CompileResponse,
  DiagnosticArtifact,
  FormatOptions,
  FormatResponse,
  ProjectAnalysisResponse,
  ProjectCompileResponse,
  ProjectFormatResponse,
  ProjectRequest,
} from "./types"

let initialization: Promise<unknown> | undefined

export async function compileSingleFile(
  source: string
): Promise<CompileResponse> {
  const response = await compileProject(singleFileRequest(source))
  if (response.status === "failure") {
    return {
      status: "failure",
      schema: response.schema,
      diagnostics: firstDiagnostics(response.diagnostics),
    }
  }
  const entry = response.modules.find(
    (module) => module.module === response.entry.module
  )
  if (!entry) throw new Error("project response omitted its entry module")
  return {
    status: "success",
    schema: response.schema,
    diagnostics: firstDiagnostics(response.diagnostics),
    generated: entry.generated,
    entry: response.entry.contract,
    entryError: response.entry.error,
  }
}

export async function analyzeSingleFile(
  source: string
): Promise<AnalysisDocument> {
  const response = await analyzeProject(singleFileRequest(source))
  if (response.status === "success") {
    const entry = response.documents.find(
      (document) => document.path === "main.ssrg"
    )
    if (!entry) throw new Error("project response omitted its entry analysis")
    return entry.document
  }
  // Syntax-invalid sources cannot form a project graph. Preserve the existing
  // recovery document (including the standard-library catalog) for that case.
  initialization ??= init()
  await initialization
  return JSON.parse(
    analyze_single_file("main.ssrg", "playground/main", source)
  ) as AnalysisDocument
}

export async function formatSingleFile(
  source: string,
  options?: FormatOptions
): Promise<FormatResponse> {
  const response = await formatProjectFile(
    singleFileRequest(source),
    "main.ssrg",
    options
  )
  if (response.status === "failure") {
    return {
      status: "failure",
      schema: response.schema,
      diagnostics: firstDiagnostics(response.diagnostics),
    }
  }
  return response
}

export async function compileProject(
  request: ProjectRequest
): Promise<ProjectCompileResponse> {
  initialization ??= init()
  await initialization
  return JSON.parse(
    compile_project(JSON.stringify(request))
  ) as ProjectCompileResponse
}

export async function analyzeProject(
  request: ProjectRequest
): Promise<ProjectAnalysisResponse> {
  initialization ??= init()
  await initialization
  return JSON.parse(
    analyze_project(JSON.stringify(request))
  ) as ProjectAnalysisResponse
}

export async function formatProjectFile(
  request: ProjectRequest,
  path: string,
  options?: FormatOptions
): Promise<ProjectFormatResponse> {
  initialization ??= init()
  await initialization
  const serialized = JSON.stringify(request)
  return JSON.parse(
    options === undefined
      ? format_project_file(serialized, path)
      : format_project_file_with_options(serialized, path, options.lineWidth)
  ) as ProjectFormatResponse
}

function singleFileRequest(source: string): ProjectRequest {
  return {
    schema: 1,
    entry: "main.ssrg",
    files: [{ path: "main.ssrg", source }],
  }
}

function firstDiagnostics(
  diagnostics: readonly {
    readonly diagnostics: DiagnosticArtifact
  }[]
): DiagnosticArtifact {
  return diagnostics[0]?.diagnostics ?? { diagnostics: [] }
}
