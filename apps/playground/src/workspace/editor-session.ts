import type { EditorState } from "@codemirror/state"
import type { WorkspacePath, WorkspaceState } from "./model"

export type WorkspacePathRename = Readonly<{
  from: WorkspacePath
  to: WorkspacePath
}>

export type WorkspaceEditorView = {
  state: EditorState
  readonly scrollDOM: Pick<HTMLElement, "scrollLeft" | "scrollTop">
  setState: (state: EditorState) => void
}

export type WorkspaceEditorSessions = Readonly<{
  transition: (
    previous: WorkspaceState,
    next: WorkspaceState,
    rename?: WorkspacePathRename
  ) => boolean
  reset: (state: WorkspaceState) => void
}>

type EditorSnapshot = Readonly<{
  state: EditorState
  scrollLeft: number
  scrollTop: number
}>

export function createWorkspaceEditorSessions(
  view: WorkspaceEditorView,
  createState: (source: string) => EditorState
): WorkspaceEditorSessions {
  const snapshots = new Map<WorkspacePath, EditorSnapshot>()

  const capture = (path: WorkspacePath): void => {
    snapshots.set(path, {
      state: view.state,
      scrollLeft: view.scrollDOM.scrollLeft,
      scrollTop: view.scrollDOM.scrollTop,
    })
  }

  const restore = (state: WorkspaceState): void => {
    const path = state.activeFile
    const source = activeSource(state)
    const remembered = path === undefined ? undefined : snapshots.get(path)
    const snapshot =
      remembered !== undefined && remembered.state.doc.toString() === source
        ? remembered
        : {
            state: createState(source),
            scrollLeft: 0,
            scrollTop: 0,
          }
    view.setState(snapshot.state)
    if (path !== undefined) snapshots.set(path, snapshot)
    queueMicrotask(() => {
      view.scrollDOM.scrollLeft = snapshot.scrollLeft
      view.scrollDOM.scrollTop = snapshot.scrollTop
    })
  }

  return {
    transition(previous, next, rename) {
      if (previous.activeFile !== undefined) capture(previous.activeFile)
      if (rename !== undefined) remapSnapshots(snapshots, rename)
      const open = new Set(next.openFiles)
      for (const path of snapshots.keys()) {
        if (!open.has(path)) snapshots.delete(path)
      }
      if (previous.activeFile === next.activeFile) return false
      restore(next)
      return true
    },
    reset(state) {
      snapshots.clear()
      restore(state)
    },
  }
}

function remapSnapshots(
  snapshots: Map<WorkspacePath, EditorSnapshot>,
  rename: WorkspacePathRename
): void {
  const moved: [WorkspacePath, EditorSnapshot][] = []
  for (const [path, snapshot] of snapshots) {
    const next = remapPath(path, rename)
    if (next === path) continue
    snapshots.delete(path)
    moved.push([next, snapshot])
  }
  for (const [path, snapshot] of moved) snapshots.set(path, snapshot)
}

function remapPath(
  path: WorkspacePath,
  rename: WorkspacePathRename
): WorkspacePath {
  if (path === rename.from) return rename.to
  const prefix = `${rename.from}/`
  return path.startsWith(prefix)
    ? `${rename.to}/${path.slice(prefix.length)}`
    : path
}

function activeSource(state: WorkspaceState): string {
  if (state.activeFile === undefined) return ""
  return state.files.find(({ path }) => path === state.activeFile)?.source ?? ""
}
