import type { WorkspacePathRename } from "./editor-session"
import {
  activateWorkspaceFile,
  createWorkspaceFile,
  createWorkspaceFolder,
  defaultExplorerWidth,
  deleteWorkspacePath,
  maximumExplorerWidth,
  minimumExplorerWidth,
  renameWorkspacePath,
  setWorkspaceEntryFile,
  setWorkspaceExplorer,
  setWorkspaceFolderExpanded,
  type WorkspacePath,
  type WorkspaceState,
} from "./model"

const explorerWidthStorageKey = "seseragi.playground.explorer-width"
const explorerWidthStep = 12

export type WorkspaceTreeRow = Readonly<{
  path: WorkspacePath
  name: string
  kind: "file" | "folder"
  level: number
  parent?: WorkspacePath
  expanded?: boolean
  active: boolean
  dirty: boolean
  entry: boolean
}>

export type WorkspaceExplorerElements = Readonly<{
  codeWorkspace: HTMLElement
  panel: HTMLElement
  tree: HTMLElement
  message: HTMLElement
  resizer: HTMLElement
  toggleButtons: readonly HTMLButtonElement[]
  newFileButton: HTMLButtonElement
  newFolderButton: HTMLButtonElement
  collapseAllButton: HTMLButtonElement
  closeButton: HTMLButtonElement
}>

export type WorkspaceExplorerChange = Readonly<{
  message?: string
  focusEditor?: boolean
  rename?: WorkspacePathRename
}>

export type WorkspaceExplorerController = Readonly<{
  render: (state: WorkspaceState) => void
  toggle: () => void
}>

type Draft =
  | Readonly<{
      kind: "create-file"
      parent?: WorkspacePath
    }>
  | Readonly<{
      kind: "create-folder"
      parent?: WorkspacePath
    }>
  | Readonly<{
      kind: "rename-file"
      path: WorkspacePath
      parent?: WorkspacePath
    }>
  | Readonly<{
      kind: "rename-folder"
      path: WorkspacePath
      parent?: WorkspacePath
    }>

export function workspaceTreeRows(
  state: WorkspaceState
): readonly WorkspaceTreeRow[] {
  const files = new Set(state.files.map(({ path }) => path))
  const folders = new Set(state.folders)
  const children = new Map<WorkspacePath | undefined, WorkspacePath[]>()
  for (const path of [...folders, ...files]) {
    const parent = workspaceParent(path)
    const siblings = children.get(parent) ?? []
    siblings.push(path)
    children.set(parent, siblings)
  }
  for (const siblings of children.values()) {
    siblings.sort((left, right) => {
      const leftFolder = folders.has(left)
      const rightFolder = folders.has(right)
      if (leftFolder !== rightFolder) return leftFolder ? -1 : 1
      return workspaceName(left).localeCompare(workspaceName(right))
    })
  }

  const rows: WorkspaceTreeRow[] = []
  const visit = (parent: WorkspacePath | undefined, level: number): void => {
    for (const path of children.get(parent) ?? []) {
      const folder = folders.has(path)
      const expanded = folder && state.expandedFolders.includes(path)
      rows.push({
        path,
        name: workspaceName(path),
        kind: folder ? "folder" : "file",
        level,
        ...(parent === undefined ? {} : { parent }),
        ...(folder ? { expanded } : {}),
        active: !folder && state.activeFile === path,
        dirty: !folder && state.dirtyFiles.includes(path),
        entry: !folder && state.entryFile === path,
      })
      if (expanded) visit(path, level + 1)
    }
  }
  visit(undefined, 1)
  return rows
}

export function readExplorerWidth(
  storage: Pick<Storage, "getItem">,
  fallback = defaultExplorerWidth
): number {
  const stored = storage.getItem(explorerWidthStorageKey)
  if (stored === null) return clampExplorerWidth(fallback)
  const value = Number(stored)
  return Number.isFinite(value)
    ? clampExplorerWidth(value)
    : clampExplorerWidth(fallback)
}

export function workspaceDeletePrompt(
  state: WorkspaceState,
  path: WorkspacePath,
  kind: "file" | "folder"
): string {
  if (kind === "file") return `Delete file ${path}?`
  const descendants =
    state.files.filter(({ path: file }) => file.startsWith(`${path}/`)).length +
    state.folders.filter((folder) => folder.startsWith(`${path}/`)).length
  return descendants > 0
    ? `Folder ${path} is not empty and contains ${descendants} item(s). Delete the entire subtree?`
    : `Delete folder ${path}?`
}

export function connectWorkspaceExplorer(
  elements: WorkspaceExplorerElements,
  options: Readonly<{
    getState: () => WorkspaceState
    onChange: (state: WorkspaceState, change: WorkspaceExplorerChange) => void
    confirmDelete?: (message: string) => boolean
  }>
): WorkspaceExplorerController {
  let selected: WorkspacePath | undefined
  let draft: Draft | undefined
  let dragging = false

  const report = (message: string | undefined): void => {
    elements.message.hidden = message === undefined
    elements.message.textContent = message ?? ""
  }

  const commit = (
    state: WorkspaceState,
    change: WorkspaceExplorerChange = {}
  ): void => {
    report(undefined)
    options.onChange(state, change)
    render(state)
  }

  const fail = (error: unknown): void => {
    report(error instanceof Error ? error.message : String(error))
  }

  const render = (state: WorkspaceState): void => {
    const rows = workspaceTreeRows(state)
    if (selected === undefined || !rows.some(({ path }) => path === selected)) {
      selected =
        rows.find(({ path }) => path === state.activeFile)?.path ??
        rows[0]?.path
    }
    elements.codeWorkspace.dataset.explorerVisible = String(
      state.explorer.visible
    )
    elements.codeWorkspace.style.setProperty(
      "--explorer-width",
      `${state.explorer.width}px`
    )
    elements.panel.hidden = !state.explorer.visible
    elements.resizer.hidden = !state.explorer.visible
    elements.resizer.setAttribute(
      "aria-valuenow",
      String(Math.round(state.explorer.width))
    )
    for (const button of elements.toggleButtons) {
      if (button.getAttribute("role") === "menuitemcheckbox") {
        button.setAttribute("aria-checked", String(state.explorer.visible))
      } else {
        button.setAttribute("aria-pressed", String(state.explorer.visible))
      }
      button.title = state.explorer.visible
        ? "Explorerを閉じる"
        : "Explorerを開く"
    }

    const rendered = rows.map((row) =>
      draft !== undefined &&
      (draft.kind === "rename-file" || draft.kind === "rename-folder") &&
      draft.path === row.path
        ? renderDraft(draft)
        : renderTreeRow(row, row.path === selected)
    )
    if (
      draft !== undefined &&
      (draft.kind === "create-file" || draft.kind === "create-folder")
    ) {
      const createDraft = draft
      const parentIndex = rows.findIndex(
        ({ path }) => path === createDraft.parent
      )
      let insertAt = rendered.length
      if (parentIndex >= 0) {
        const parentLevel = rows[parentIndex]?.level ?? 1
        const nextSibling = rows.findIndex(
          ({ level }, index) => index > parentIndex && level <= parentLevel
        )
        insertAt = nextSibling < 0 ? rendered.length : nextSibling
      }
      rendered.splice(insertAt, 0, renderDraft(draft))
    }
    elements.tree.replaceChildren(...rendered)
  }

  const toggle = (): void => {
    const state = options.getState()
    commit(setWorkspaceExplorer(state, { visible: !state.explorer.visible }), {
      message: state.explorer.visible ? "Explorer closed" : "Explorer opened",
    })
  }

  for (const button of elements.toggleButtons) {
    button.addEventListener("click", toggle)
  }
  elements.closeButton.addEventListener("click", () => {
    const state = options.getState()
    if (!state.explorer.visible) return
    commit(setWorkspaceExplorer(state, { visible: false }), {
      message: "Explorer closed",
    })
  })
  elements.newFileButton.addEventListener("click", () => {
    startCreate("create-file")
  })
  elements.newFolderButton.addEventListener("click", () => {
    startCreate("create-folder")
  })
  elements.collapseAllButton.addEventListener("click", () => {
    let state = options.getState()
    for (const folder of state.expandedFolders) {
      state = setWorkspaceFolderExpanded(state, folder, false)
    }
    commit(state, { message: "All folders collapsed" })
  })

  elements.tree.addEventListener("click", (event) => {
    const target = event.target
    if (!(target instanceof Element)) return
    const row = target.closest<HTMLElement>("[data-explorer-path]")
    if (row === null) return
    const path = row.dataset.explorerPath
    const kind = row.dataset.explorerKind
    if (path === undefined || (kind !== "file" && kind !== "folder")) return
    selected = path
    const action = target.closest<HTMLButtonElement>("[data-explorer-action]")
      ?.dataset.explorerAction
    if (action === "rename") {
      startRename(path, kind)
      return
    }
    if (action === "delete") {
      deleteSelected(path, kind)
      return
    }
    if (action === "entry" && kind === "file") {
      commit(setWorkspaceEntryFile(options.getState(), path), {
        message: `Entry set: ${path}`,
      })
      return
    }
    if (target instanceof HTMLInputElement) return
    activate(path, kind)
  })

  elements.tree.addEventListener("keydown", (event) => {
    const target = event.target
    if (target instanceof HTMLInputElement) {
      if (event.key === "Enter") {
        event.preventDefault()
        applyDraft(target.value)
      } else if (event.key === "Escape") {
        event.preventDefault()
        draft = undefined
        report(undefined)
        render(options.getState())
        focusSelected()
      }
      return
    }
    if (!(target instanceof HTMLElement)) return
    const rows = workspaceTreeRows(options.getState())
    const path = target.dataset.explorerPath
    if (path === undefined) return
    selected = path
    const index = rows.findIndex((row) => row.path === path)
    const row = rows[index]
    if (row === undefined) return
    let next: WorkspaceTreeRow | undefined
    if (event.key === "ArrowDown") next = rows[index + 1]
    if (event.key === "ArrowUp") next = rows[index - 1]
    if (event.key === "Home") next = rows[0]
    if (event.key === "End") next = rows.at(-1)
    if (event.key === "ArrowRight" && row.kind === "folder") {
      event.preventDefault()
      if (!row.expanded) activate(row.path, "folder", true)
      else next = rows[index + 1]
    }
    if (event.key === "ArrowLeft") {
      event.preventDefault()
      if (row.kind === "folder" && row.expanded) {
        activate(row.path, "folder", false)
      } else if (row.parent !== undefined) {
        next = rows.find(({ path: candidate }) => candidate === row.parent)
      }
    }
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault()
      activate(row.path, row.kind)
    }
    if (event.key === "F2") {
      event.preventDefault()
      startRename(row.path, row.kind)
    }
    if (event.key === "Delete") {
      event.preventDefault()
      deleteSelected(row.path, row.kind)
    }
    if (event.key === "Escape") {
      event.preventDefault()
      const state = options.getState()
      commit(setWorkspaceExplorer(state, { visible: false }), {
        message: "Explorer closed",
      })
      elements.toggleButtons[0]?.focus()
      return
    }
    if (next !== undefined) {
      event.preventDefault()
      selected = next.path
      render(options.getState())
      focusSelected()
    }
  })

  const startCreate = (kind: "create-file" | "create-folder"): void => {
    const state = options.getState()
    const selectedFolder =
      selected !== undefined && state.folders.includes(selected)
        ? selected
        : selected === undefined
          ? undefined
          : workspaceParent(selected)
    if (
      selectedFolder !== undefined &&
      !state.expandedFolders.includes(selectedFolder)
    ) {
      options.onChange(
        setWorkspaceFolderExpanded(state, selectedFolder, true),
        {}
      )
    }
    draft = { kind, ...(selectedFolder ? { parent: selectedFolder } : {}) }
    report(undefined)
    render(options.getState())
    focusDraft()
  }

  const startRename = (path: WorkspacePath, kind: "file" | "folder"): void => {
    selected = path
    draft = {
      kind: kind === "file" ? "rename-file" : "rename-folder",
      path,
      ...(workspaceParent(path) === undefined
        ? {}
        : { parent: workspaceParent(path) }),
    }
    report(undefined)
    render(options.getState())
    focusDraft(true)
  }

  const applyDraft = (value: string): void => {
    if (draft === undefined) return
    const current = draft
    try {
      const name = workspaceNodeName(value, current.kind.includes("file"))
      const target =
        current.parent === undefined ? name : `${current.parent}/${name}`
      let state = options.getState()
      if (current.kind === "create-file") {
        state = createWorkspaceFile(state, target)
        if (!isDesktopExplorer()) {
          state = setWorkspaceExplorer(state, { visible: false })
        }
        selected = target
      } else if (current.kind === "create-folder") {
        state = createWorkspaceFolder(state, target)
        selected = target
      } else {
        state = renameWorkspacePath(state, current.path, target)
        selected = target
      }
      const focusEditor = current.kind === "create-file"
      const message = current.kind.startsWith("create")
        ? `${current.kind === "create-file" ? "File" : "Folder"} created: ${target}`
        : `Renamed to ${target}`
      const rename =
        current.kind === "rename-file" || current.kind === "rename-folder"
          ? { from: current.path, to: target }
          : undefined
      draft = undefined
      commit(state, {
        message,
        focusEditor,
        ...(rename === undefined ? {} : { rename }),
      })
      if (focusEditor) return
      focusSelected()
    } catch (error) {
      fail(error)
      focusDraft()
    }
  }

  const deleteSelected = (
    path: WorkspacePath,
    kind: "file" | "folder"
  ): void => {
    const state = options.getState()
    const confirmDelete =
      options.confirmDelete ?? ((message: string) => window.confirm(message))
    if (!confirmDelete(workspaceDeletePrompt(state, path, kind))) return
    try {
      const next = deleteWorkspacePath(state, path)
      selected = next.activeFile ?? workspaceTreeRows(next)[0]?.path
      commit(next, {
        message: `${kind === "file" ? "File" : "Folder"} deleted: ${path}`,
        focusEditor: state.activeFile !== next.activeFile,
      })
      focusSelected()
    } catch (error) {
      fail(error)
    }
  }

  const activate = (
    path: WorkspacePath,
    kind: "file" | "folder",
    expanded?: boolean
  ): void => {
    try {
      const state = options.getState()
      let next =
        kind === "file"
          ? activateWorkspaceFile(state, path)
          : setWorkspaceFolderExpanded(
              state,
              path,
              expanded ?? !state.expandedFolders.includes(path)
            )
      if (kind === "file" && !isDesktopExplorer()) {
        next = setWorkspaceExplorer(next, { visible: false })
      }
      commit(next, {
        ...(kind === "file"
          ? { message: `Opened ${path}`, focusEditor: true }
          : {}),
      })
    } catch (error) {
      fail(error)
    }
  }

  const focusSelected = (): void => {
    elements.tree
      .querySelector<HTMLElement>(
        `[data-explorer-path="${CSS.escape(selected ?? "")}"]`
      )
      ?.focus()
  }

  const focusDraft = (select = false): void => {
    queueMicrotask(() => {
      const input = elements.tree.querySelector<HTMLInputElement>(
        ".explorer-name-input"
      )
      input?.focus()
      if (select) input?.select()
    })
  }

  const applyWidth = (width: number, persist: boolean): void => {
    const state = options.getState()
    const next = setWorkspaceExplorer(state, { width })
    commit(next)
    if (persist) writeExplorerWidth(next.explorer.width)
  }

  elements.resizer.addEventListener("pointerdown", (event) => {
    if (!isDesktopExplorer()) return
    event.preventDefault()
    dragging = true
    elements.resizer.setPointerCapture(event.pointerId)
    elements.resizer.dataset.dragging = "true"
  })
  elements.resizer.addEventListener("pointermove", (event) => {
    if (!dragging || !elements.resizer.hasPointerCapture(event.pointerId))
      return
    const bounds = elements.codeWorkspace.getBoundingClientRect()
    applyWidth(event.clientX - bounds.left, false)
  })
  const finishResize = (event: PointerEvent): void => {
    if (!elements.resizer.hasPointerCapture(event.pointerId)) return
    dragging = false
    elements.resizer.releasePointerCapture(event.pointerId)
    delete elements.resizer.dataset.dragging
    writeExplorerWidth(options.getState().explorer.width)
  }
  elements.resizer.addEventListener("pointerup", finishResize)
  elements.resizer.addEventListener("pointercancel", finishResize)
  elements.resizer.addEventListener("keydown", (event) => {
    let width: number | undefined
    const current = options.getState().explorer.width
    if (event.key === "ArrowLeft") width = current - explorerWidthStep
    if (event.key === "ArrowRight") width = current + explorerWidthStep
    if (event.key === "Home") width = minimumExplorerWidth
    if (event.key === "End") width = maximumExplorerWidth
    if (event.key === "Enter") width = defaultExplorerWidth
    if (width === undefined) return
    event.preventDefault()
    applyWidth(width, true)
  })

  render(options.getState())
  return { render, toggle }

  function renderTreeRow(
    row: WorkspaceTreeRow,
    isSelected: boolean
  ): HTMLElement {
    const element = document.createElement("div")
    element.className = `explorer-row explorer-row--${row.kind}`
    element.dataset.explorerPath = row.path
    element.dataset.explorerKind = row.kind
    element.setAttribute("role", "treeitem")
    element.setAttribute("aria-level", String(row.level))
    element.setAttribute("aria-selected", String(row.active))
    element.dataset.dirty = String(row.dirty)
    element.dataset.entry = String(row.entry)
    element.setAttribute(
      "aria-label",
      [
        row.path,
        ...(row.entry ? ["entry file"] : []),
        ...(row.dirty ? ["unsaved changes"] : []),
      ].join(", ")
    )
    if (row.kind === "folder") {
      element.setAttribute("aria-expanded", String(row.expanded))
    }
    element.tabIndex = isSelected ? 0 : -1
    element.style.setProperty("--tree-level", String(row.level))

    const marker = document.createElement("span")
    marker.className = "explorer-row-marker"
    marker.setAttribute("aria-hidden", "true")
    marker.textContent =
      row.kind === "folder" ? (row.expanded ? "⌄" : "›") : row.dirty ? "●" : "·"
    const label = document.createElement("span")
    label.className = "explorer-row-label"
    label.textContent = row.name
    element.append(marker, label)

    if (row.entry) {
      const badge = document.createElement("span")
      badge.className = "explorer-entry-badge"
      badge.textContent = "entry"
      badge.setAttribute("aria-hidden", "true")
      element.append(badge)
    }

    const actions = document.createElement("span")
    actions.className = "explorer-row-actions"
    actions.append(
      ...(row.kind === "file" && !row.entry
        ? [rowAction("entry", `Set ${row.path} as entry`, "▶")]
        : []),
      rowAction("rename", `Rename ${row.path}`, "✎"),
      rowAction("delete", `Delete ${row.path}`, "×")
    )
    element.append(actions)
    return element
  }

  function renderDraft(current: Draft): HTMLElement {
    const row = document.createElement("div")
    row.className = "explorer-draft"
    row.setAttribute("role", "none")
    const level = current.parent?.split("/").length ?? 0
    row.style.setProperty("--tree-level", String(level + 1))
    const input = document.createElement("input")
    input.className = "explorer-name-input"
    input.type = "text"
    input.autocomplete = "off"
    input.spellcheck = false
    input.setAttribute(
      "aria-label",
      current.kind.startsWith("rename") ? "New name" : "Name"
    )
    input.placeholder = current.kind.includes("file") ? "name.ssrg" : "folder"
    if (current.kind === "rename-file" || current.kind === "rename-folder")
      input.value = workspaceName(current.path)
    input.addEventListener("blur", () => {
      queueMicrotask(() => {
        if (document.activeElement?.closest(".explorer-draft") !== null) return
        draft = undefined
        render(options.getState())
      })
    })
    row.append(input)
    return row
  }
}

function rowAction(
  action: string,
  label: string,
  text: string
): HTMLButtonElement {
  const button = document.createElement("button")
  button.type = "button"
  button.dataset.explorerAction = action
  button.setAttribute("aria-label", label)
  button.title = label
  button.textContent = text
  return button
}

function workspaceNodeName(value: string, file: boolean): string {
  if (value === "" || value !== value.trim()) {
    throw new Error("Name must not be empty or have surrounding whitespace")
  }
  if (value === "." || value === ".." || /[\\/\0]/u.test(value)) {
    throw new Error("Name must be one workspace path segment")
  }
  if (file && !value.endsWith(".ssrg")) {
    throw new Error("Seseragi file name must end in .ssrg")
  }
  return value
}

function workspaceParent(path: WorkspacePath): WorkspacePath | undefined {
  const separator = path.lastIndexOf("/")
  return separator < 0 ? undefined : path.slice(0, separator)
}

function workspaceName(path: WorkspacePath): string {
  return path.slice(path.lastIndexOf("/") + 1)
}

function clampExplorerWidth(width: number): number {
  return Math.min(maximumExplorerWidth, Math.max(minimumExplorerWidth, width))
}

function writeExplorerWidth(width: number): void {
  try {
    window.localStorage.setItem(explorerWidthStorageKey, String(width))
  } catch {
    // Storage may be unavailable in hardened or private browser contexts.
  }
}

function isDesktopExplorer(): boolean {
  return !window.matchMedia(
    "(max-width: 760px), (max-width: 960px) and (max-height: 520px)"
  ).matches
}
