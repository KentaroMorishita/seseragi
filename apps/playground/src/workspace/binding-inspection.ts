import type { WorkspaceDiagnostic } from "../diagnostics/workspace-diagnostics"
import {
  type BindingRequest,
  type BindingResponse,
  bindingDiagnostics,
  workspaceBindingRequest,
} from "./bindings"
import {
  type CompatibilityInspection,
  inspectCompatibilityReport,
} from "./compatibility-report"
import { type WorkspaceState, workspaceBindingInputRevision } from "./model"
import {
  inspectSemanticGraph,
  type SemanticInspection,
} from "./semantic-inspection"

export type BindingReport = Readonly<{
  added: readonly string[]
  changed: readonly string[]
  removed: readonly string[]
  unsupported: readonly string[]
}>
export type InspectedBinding = Readonly<{
  id: string
  declaration: string
  path: string
  source: string
  report: BindingReport
  adapters?: NonNullable<BindingResponse["generated"][number]["adapters"]>
}>
export type BindingInspection = Readonly<{
  inputRevision: string
  status: "success" | "failure"
  inputs: BindingRequest["files"]
  bindings: readonly InspectedBinding[]
  diagnostics: readonly WorkspaceDiagnostic[]
  semantics: readonly SemanticInspection[]
  compatibility?: readonly CompatibilityInspection[]
}>

function record(json: string, kind: string): Record<string, unknown> {
  const value: unknown = JSON.parse(json)
  if (typeof value !== "object" || value === null || Array.isArray(value))
    throw new Error(`Invalid ${kind}`)
  const result = value as Record<string, unknown>
  if (result.schema !== 1 || result.kind !== kind)
    throw new Error(`Unsupported ${kind} schema`)
  return result
}
function strings(value: unknown, field: string): readonly string[] {
  if (!Array.isArray(value) || !value.every((item) => typeof item === "string"))
    throw new Error(`Invalid conversion report field: ${field}`)
  return [...value]
}

export function inspectWorkspaceBindings(
  state: WorkspaceState,
  request: BindingRequest,
  response: BindingResponse
): BindingInspection {
  if (
    response.schema !== 1 ||
    response.revision !== request.revision ||
    request.revision !== workspaceBindingRequest(state, request.entry).revision
  )
    throw new Error("Inspector result does not match its workspace snapshot")
  const semantics = (response.semantic ?? []).map(({ id, graph }) =>
    inspectSemanticGraph(id, graph, request.files)
  )
  const compatibility = (response.compatibility ?? []).map(({ id, report }) => {
    const semantic = semantics.find((entry) => entry.id === id)
    if (!semantic)
      throw new Error("Compatibility report has no matching semantic graph")
    return inspectCompatibilityReport(id, report, semantic, request.files)
  })
  return {
    compatibility,
    inputRevision: workspaceBindingInputRevision(state),
    status: response.status,
    inputs: request.files
      .filter((file) => !file.path.endsWith(".ssrg"))
      .map((file) => ({ ...file })),
    bindings: response.generated.map((binding) => {
      const metadata = record(binding.metadata, "seseragi-typescript-binding")
      const report = record(
        binding.report,
        "seseragi-typescript-conversion-report"
      )
      if (
        typeof metadata.declaration !== "string" ||
        metadata.entry !== binding.id ||
        report.entry !== binding.id
      )
        throw new Error("Binding inspection metadata does not match its entry")
      if (typeof response.generatedRoot !== "string")
        throw new Error("Missing generated root")
      return {
        id: binding.id,
        declaration: metadata.declaration.normalize("NFC"),
        path: `${response.generatedRoot}/${binding.output}.ssrg`,
        source: binding.source,
        ...(binding.adapters ? { adapters: binding.adapters } : {}),
        report: {
          added: strings(report.added, "added"),
          changed: strings(report.changed, "changed"),
          removed: strings(report.removed, "removed"),
          unsupported: strings(report.unsupported, "unsupported"),
        },
      }
    }),
    diagnostics: bindingDiagnostics(request, response),
    semantics,
  }
}

/** Once invalidated, edit/undo cannot revive the old display. */
export function currentBindingInspection(
  inspection: BindingInspection | undefined,
  state: WorkspaceState
): BindingInspection | undefined {
  return inspection?.inputRevision === workspaceBindingInputRevision(state)
    ? inspection
    : undefined
}
