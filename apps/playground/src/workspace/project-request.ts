import type { ProjectRequest } from "../compiler/types"
import type { WorkspaceState } from "./model"

export function workspaceProjectRequest(state: WorkspaceState): ProjectRequest {
  const entry = state.entryFile ?? state.activeFile
  if (entry === undefined) throw new Error("Workspace has no source file")
  return {
    schema: 1,
    entry,
    files: state.files.map(({ path, source }) => ({ path, source })),
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
