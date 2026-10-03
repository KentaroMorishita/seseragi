import type {
  BindingConversionRequest,
  BindingConversionResponse,
} from "../compiler/interop-types"
import type { WorkspaceDiagnostic } from "../diagnostics/workspace-diagnostics"
import {
  createWorkspace,
  isGeneratedWorkspaceFile,
  isWorkspaceSourceFile,
  type WorkspaceState,
  workspaceFilePath,
} from "./model"

/** All document edits and path changes invalidate an in-flight conversion. */
export function workspaceBindingRevision(state: WorkspaceState): string {
  return JSON.stringify({ entry: state.entryFile, files: state.files })
}

/** Generated code remains usable while ordinary Seseragi source is edited. */
export function workspaceBindingInputRevision(state: WorkspaceState): string {
  return JSON.stringify(
    state.files.filter(
      ({ path }) =>
        !isGeneratedWorkspaceFile(path) && !isWorkspaceSourceFile(path)
    )
  )
}

export function workspaceBindingsAreCurrent(state: WorkspaceState): boolean {
  return state.bindingInputRevision === workspaceBindingInputRevision(state)
}

export function workspaceBindingRequest(
  state: WorkspaceState
): BindingConversionRequest {
  const manifest = state.files.find(({ path }) => path === "seseragi.toml")
  if (manifest === undefined) {
    throw new Error(
      "Add seseragi.toml, a binding settings file, .d.ts declarations and host package.json to Explorer, or open the TypeScript bindings workspace."
    )
  }
  return {
    schema: 1,
    revision: workspaceBindingRevision(state),
    manifest: manifest.source,
    files: state.files.map(({ path, source }) => ({ path, source })),
  }
}

export function applyBindingConversion(
  state: WorkspaceState,
  response: BindingConversionResponse
): { status: "applied" | "stale" | "failure"; state: WorkspaceState } {
  if (response.revision !== workspaceBindingRevision(state)) {
    return { status: "stale", state }
  }
  if (response.status !== "success") return { status: "failure", state }
  if (response.schema !== 1) throw new Error("Unsupported binding response")
  const generated = response.generated.flatMap((binding) => [
    {
      path: workspaceFilePath(`gen/${binding.output}.ssrg`),
      source: binding.source,
    },
    {
      path: workspaceFilePath(`gen/${binding.output}.binding.json`),
      source: binding.metadata,
    },
    {
      path: workspaceFilePath(`gen/${binding.output}.report.json`),
      source: binding.report,
    },
  ])
  const files = [
    ...state.files.filter(({ path }) => !isGeneratedWorkspaceFile(path)),
    ...generated,
  ]
  const paths = new Set(files.map(({ path }) => path))
  const activeFile =
    generated[0]?.path ??
    (state.activeFile !== undefined && paths.has(state.activeFile)
      ? state.activeFile
      : state.entryFile)
  return {
    status: "applied",
    state: createWorkspace({
      ...state,
      files,
      folders: state.folders,
      activeFile,
      openFiles: [
        ...new Set([
          ...state.openFiles.filter((path) => paths.has(path)),
          ...(activeFile === undefined ? [] : [activeFile]),
        ]),
      ],
      dirtyFiles: state.dirtyFiles.filter((path) => paths.has(path)),
      expandedFolders: [
        ...new Set([
          ...state.expandedFolders,
          ...(generated.length === 0 ? [] : ["gen"]),
        ]),
      ],
      bindingInputRevision: workspaceBindingInputRevision(state),
      explorer: { ...state.explorer, visible: true },
    }),
  }
}

export function bindingWorkspaceDiagnostics(
  state: WorkspaceState,
  response: BindingConversionResponse
): readonly WorkspaceDiagnostic[] {
  const sources = new Map(state.files.map(({ path, source }) => [path, source]))
  return response.diagnostics.map((diagnostic) => ({
    path: diagnostic.path,
    source: sources.get(diagnostic.path) ?? "",
    diagnostic: {
      code: diagnostic.code,
      messageKey: "binding.conversion",
      message: diagnostic.message,
      severity: diagnostic.severity,
      primary: {
        start: diagnostic.start ?? 0,
        end: diagnostic.end ?? diagnostic.start ?? 0,
      },
      related: [],
      labels: [],
      notes: [
        ...(diagnostic.entry === undefined
          ? []
          : [`Binding entry: ${diagnostic.entry}`]),
        ...(diagnostic.symbol === undefined
          ? []
          : [`Declaration: ${diagnostic.symbol}`]),
      ],
      helps: [],
      fixes: [],
      expectedType: null,
      actualType: null,
    },
  }))
}
