import type { ProjectRequest } from "../compiler/types"
import {
  isWorkspaceSourcePath,
  type WorkspaceState,
  workspaceBindingInputRevision,
  workspaceSourcePath,
} from "./model"

const virtualPackageName = "playground/workspace"

function workspaceManifest(entryFile: string): string {
  const entry = workspaceSourcePath(entryFile).slice(0, -".ssrg".length)
  return [
    "[package]",
    `name = "${virtualPackageName}"`,
    'version = "0.0.0"',
    'language = "^0.1.0"',
    "",
    "[run]",
    `entry = ${JSON.stringify(entry)}`,
    "",
  ].join("\n")
}

export function workspaceProjectRequest(state: WorkspaceState): ProjectRequest {
  const entry =
    state.entryFile ??
    (isWorkspaceSourcePath(state.activeFile)
      ? state.activeFile
      : state.files.find(({ path }) => isWorkspaceSourcePath(path))?.path)
  if (entry === undefined) throw new Error("Workspace has no source file")
  return {
    schema: 1,
    manifest:
      state.packageManifest !== undefined && entry === state.packageEntryFile
        ? state.packageManifest
        : workspaceManifest(entry),
    files: state.files
      .filter(({ path }) => isWorkspaceSourcePath(path))
      .map(({ path, source }) => ({
        path: workspaceSourcePath(path),
        source,
      })),
    ...(state.generatedBindings !== undefined &&
    state.generatedBindings.inputRevision ===
      workspaceBindingInputRevision(state)
      ? { generatedFiles: state.generatedBindings.files }
      : {}),
  }
}

export function runnableWorkspaceProjectRequest(
  state: WorkspaceState
): ProjectRequest {
  if (state.entryFile === undefined) {
    throw new Error("Select an entry file in Explorer before Run")
  }
  return workspaceProjectRequest(state)
}

export function workspaceProjectRevision(state: WorkspaceState): string {
  return JSON.stringify(workspaceProjectRequest(state))
}

export type WorkspaceAnalysisRequest = Readonly<{
  active: string
  project: ProjectRequest
}>

export function workspaceAnalysisRequest(
  state: WorkspaceState
): WorkspaceAnalysisRequest {
  if (
    !isWorkspaceSourcePath(state.activeFile) ||
    state.activeFile === undefined
  ) {
    throw new Error("Workspace has no active file")
  }
  return {
    active: state.activeFile,
    project: workspaceProjectRequest(state),
  }
}

export function workspaceAnalysisRevision(state: WorkspaceState): string {
  return JSON.stringify(workspaceAnalysisRequest(state))
}
