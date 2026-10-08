import type { WorkspaceDiagnostic } from "../diagnostics/workspace-diagnostics"
import type { WorkspaceState } from "./model"

export type BindingRequest = Readonly<{
  schema: 1
  revision: string
  files: readonly Readonly<{ path: string; source: string }>[]
  entry?: string
}>
export type BindingResponse = Readonly<{
  schema: 1
  revision: string
  status: "success" | "failure"
  generated: readonly Readonly<{
    id: string
    output: string
    source: string
    metadata: string
    report: string
  }>[]
  diagnostics: readonly Readonly<{
    code: string
    severity: "Error" | "Warning"
    message: string
    file: string
    start: number
    end: number
  }>[]
}>

export function workspaceBindingRequest(
  state: WorkspaceState,
  entry?: string
): BindingRequest {
  const files = state.files.map(({ path, source }) => ({ path, source }))
  return {
    schema: 1,
    revision: JSON.stringify({ files, entry }),
    files,
    ...(entry === undefined ? {} : { entry }),
  }
}

/** Revision plus a generation counter rejects both stale success/failure and ABA edits. */
export function createBindingConversion(
  convert: (request: BindingRequest) => Promise<BindingResponse>,
  currentRevision: () => string
) {
  let generation = 0
  return {
    invalidate() {
      generation += 1
    },
    async run(request: BindingRequest): Promise<BindingResponse | undefined> {
      const requested = ++generation
      try {
        const response = await convert(request)
        if (requested !== generation || currentRevision() !== request.revision)
          return undefined
        if (response.schema !== 1 || response.revision !== request.revision)
          throw new Error(
            "Conversion response does not match its workspace snapshot"
          )
        return response
      } catch (error) {
        if (requested !== generation || currentRevision() !== request.revision)
          return undefined
        throw error
      }
    },
  }
}

export function bindingDiagnostics(
  request: BindingRequest,
  response: BindingResponse
): readonly WorkspaceDiagnostic[] {
  const sources = new Map(
    request.files.map(({ path, source }) => [path, source])
  )
  return response.diagnostics.map((item) => ({
    path: item.file.normalize("NFC"),
    source: sources.get(item.file.normalize("NFC")) ?? "",
    diagnostic: {
      code: item.code,
      messageKey: "bindings.conversion",
      message: item.message,
      severity: item.severity,
      primary: { start: item.start, end: item.end },
      related: [],
      labels: [],
      notes: [],
      helps: [],
      fixes: [],
      expectedType: null,
      actualType: null,
    },
  }))
}
