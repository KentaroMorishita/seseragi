import {
  activateWorkspaceFile,
  closeWorkspaceFile,
  type WorkspacePath,
  type WorkspaceState,
} from "./model"

export type WorkspaceTabItem = Readonly<{
  path: WorkspacePath
  name: string
  active: boolean
  dirty: boolean
}>

export type WorkspaceTabChange = Readonly<{
  message: string
  focusEditor: boolean
}>

export type WorkspaceTabsController = Readonly<{
  render: (state: WorkspaceState) => void
}>

export function workspaceTabItems(
  state: WorkspaceState
): readonly WorkspaceTabItem[] {
  return state.openFiles.map((path) => ({
    path,
    name: path.slice(path.lastIndexOf("/") + 1),
    active: state.activeFile === path,
    dirty: state.dirtyFiles.includes(path),
  }))
}

export function workspaceDirtyClosePrompt(path: WorkspacePath): string {
  return `Close ${path}? Its edits will stay in the workspace and remain marked as unsaved.`
}

export function connectWorkspaceTabs(
  list: HTMLElement,
  options: Readonly<{
    getState: () => WorkspaceState
    onChange: (state: WorkspaceState, change: WorkspaceTabChange) => void
    confirmClose?: (message: string) => boolean
    panel?: HTMLElement
  }>
): WorkspaceTabsController {
  const render = (state: WorkspaceState): void => {
    const items = workspaceTabItems(state)
    list.hidden = items.length <= 1
    list.dataset.compact = String(items.length <= 2)
    list.replaceChildren(...items.map(renderTab))
    const activeIndex = items.findIndex(({ active }) => active)
    if (options.panel !== undefined) {
      const active = items[activeIndex]
      options.panel.setAttribute(
        "aria-label",
        active === undefined
          ? "Seseragi source editor, no file open"
          : `Seseragi source editor, ${active.path}`
      )
      if (activeIndex < 0 || list.hidden) {
        options.panel.removeAttribute("aria-labelledby")
      } else {
        options.panel.setAttribute(
          "aria-labelledby",
          workspaceTabId(activeIndex)
        )
      }
    }
    queueMicrotask(() => {
      for (const tab of list.querySelectorAll<HTMLElement>("[role=tab]")) {
        if (tab.getAttribute("aria-selected") === "true") {
          tab.scrollIntoView({ block: "nearest", inline: "nearest" })
          break
        }
      }
    })
  }

  const commit = (state: WorkspaceState, change: WorkspaceTabChange): void => {
    options.onChange(state, change)
    render(state)
  }

  const activate = (path: WorkspacePath, focusEditor = true): void => {
    const state = options.getState()
    commit(activateWorkspaceFile(state, path), {
      message: `Opened ${path}`,
      focusEditor,
    })
  }

  const close = (path: WorkspacePath): void => {
    const state = options.getState()
    const dirty = state.dirtyFiles.includes(path)
    const confirmClose =
      options.confirmClose ?? ((message: string) => window.confirm(message))
    if (dirty && !confirmClose(workspaceDirtyClosePrompt(path))) return
    commit(closeWorkspaceFile(state, path), {
      message: dirty
        ? `Closed ${path}; unsaved edits remain in the workspace`
        : `Closed ${path}`,
      focusEditor: state.activeFile === path,
    })
  }

  list.addEventListener("click", (event) => {
    const target = event.target
    if (!(target instanceof Element)) return
    const item = target.closest<HTMLElement>("[data-tab-path]")
    const path = item?.dataset.tabPath
    if (path === undefined) return
    if (target.closest("[data-tab-action=close]") !== null) close(path)
    else if (target.closest("[role=tab]") !== null) activate(path)
  })

  list.addEventListener("keydown", (event) => {
    const target = event.target
    if (
      !(target instanceof HTMLElement) ||
      target.getAttribute("role") !== "tab"
    )
      return
    const state = options.getState()
    const path = target.dataset.tabPath
    if (path === undefined) return
    const index = state.openFiles.indexOf(path)
    if (index < 0) return
    let next: WorkspacePath | undefined
    if (event.key === "ArrowLeft") next = state.openFiles[index - 1]
    if (event.key === "ArrowRight") next = state.openFiles[index + 1]
    if (event.key === "Home") next = state.openFiles[0]
    if (event.key === "End") next = state.openFiles.at(-1)
    if (event.key === "Delete") {
      event.preventDefault()
      close(path)
      return
    }
    if (next === undefined) return
    event.preventDefault()
    activate(next, false)
    queueMicrotask(() => {
      list
        .querySelector<HTMLElement>('[role="tab"][aria-selected="true"]')
        ?.focus()
    })
  })

  render(options.getState())
  return { render }

  function renderTab(item: WorkspaceTabItem, index: number): HTMLElement {
    const wrapper = document.createElement("div")
    wrapper.className = "workspace-tab"
    wrapper.dataset.tabPath = item.path
    wrapper.dataset.testid = "workspace-tab"
    wrapper.setAttribute("role", "presentation")

    const tab = document.createElement("button")
    tab.type = "button"
    tab.className = "workspace-tab-select"
    tab.dataset.tabPath = item.path
    tab.id = workspaceTabId(index)
    tab.setAttribute("role", "tab")
    tab.setAttribute("aria-selected", String(item.active))
    tab.setAttribute("aria-controls", options.panel?.id ?? "editor")
    tab.tabIndex = item.active ? 0 : -1
    tab.title = item.path
    tab.setAttribute(
      "aria-label",
      item.dirty ? `${item.path}, unsaved changes` : item.path
    )
    const dirty = document.createElement("span")
    dirty.className = "workspace-tab-dirty"
    dirty.setAttribute("aria-hidden", "true")
    dirty.textContent = item.dirty ? "●" : ""
    const label = document.createElement("span")
    label.className = "workspace-tab-label"
    label.textContent = item.name
    tab.append(dirty, label)

    const close = document.createElement("button")
    close.type = "button"
    close.className = "workspace-tab-close"
    close.dataset.tabAction = "close"
    close.setAttribute("aria-label", `Close ${item.path}`)
    close.title = `Close ${item.path}`
    close.textContent = "×"
    wrapper.append(tab, close)
    return wrapper
  }
}

function workspaceTabId(index: number): string {
  return `workspace-tab-${index}`
}
