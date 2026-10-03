import type { ProjectRequest } from "../compiler/types"
import { workspaceBindingsAreCurrent } from "./binding-conversion"
import {
  isGeneratedWorkspaceFile,
  isWorkspaceSourceFile,
  type WorkspaceState,
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
    (state.activeFile !== undefined &&
    isWorkspaceSourceFile(state.activeFile) &&
    !isGeneratedWorkspaceFile(state.activeFile)
      ? state.activeFile
      : state.files.find(
          ({ path }) =>
            isWorkspaceSourceFile(path) && !isGeneratedWorkspaceFile(path)
        )?.path)
  if (entry === undefined) throw new Error("Workspace has no source file")
  const manifest = state.files.find(
    ({ path }) => path === "seseragi.toml"
  )?.source
  const generated = state.files.some(
    ({ path }) => isGeneratedWorkspaceFile(path) && isWorkspaceSourceFile(path)
  )
  if (generated && !workspaceBindingsAreCurrent(state))
    throw new Error(
      "Binding inputs changed. Convert bindings again before compiling."
    )
  return {
    schema: 1,
    manifest:
      manifest ??
      (state.packageManifest !== undefined && entry === state.packageEntryFile
        ? state.packageManifest
        : workspaceManifest(entry)),
    files: state.files
      .filter(({ path }) => isWorkspaceSourceFile(path))
      .map(({ path, source }) => ({
        ...(isGeneratedWorkspaceFile(path)
          ? { root: "generated" as const }
          : {}),
        path: workspaceSourcePath(path),
        source,
      })),
  }
}

export function runnableWorkspaceProjectRequest(
  state: WorkspaceState
): ProjectRequest {
  if (
    state.entryFile === undefined &&
    !state.files.some(({ path }) => path === "seseragi.toml")
  ) {
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
