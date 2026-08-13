import type { ProjectRequest } from "../compiler/types"
import { workspaceSourcePath, type WorkspaceState } from "./model"

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
  const entry = state.entryFile ?? state.activeFile
  if (entry === undefined) throw new Error("Workspace has no source file")
  return {
    schema: 1,
    manifest: workspaceManifest(entry),
    files: state.files.map(({ path, source }) => ({
      path: workspaceSourcePath(path),
      source,
    })),
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
  if (state.activeFile === undefined) {
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
