export type WorkspaceFocusRegion = "explorer" | "tabs" | "editor" | "io"

export function nextWorkspaceFocusRegion(
  regions: readonly WorkspaceFocusRegion[],
  current: WorkspaceFocusRegion | undefined,
  reverse: boolean
): WorkspaceFocusRegion | undefined {
  if (regions.length === 0) return undefined
  const currentIndex = current === undefined ? -1 : regions.indexOf(current)
  if (currentIndex < 0) return reverse ? regions.at(-1) : regions[0]
  const offset = reverse ? -1 : 1
  return regions[(currentIndex + offset + regions.length) % regions.length]
}

export function connectWorkspaceFocusNavigation(
  elements: Readonly<{
    document: Document
    explorerPanel: HTMLElement
    workspaceTabs: HTMLElement
    editorSurface: HTMLElement
    ioPanel: HTMLElement
  }>,
  actions: Readonly<{
    focusExplorer: () => void
    focusEditor: () => void
    showCode: () => void
    showIo: () => void
  }>
): void {
  elements.document.addEventListener("keydown", (event) => {
    if (
      event.key.toLowerCase() === "e" &&
      (event.ctrlKey || event.metaKey) &&
      event.shiftKey &&
      !event.altKey
    ) {
      event.preventDefault()
      actions.showCode()
      actions.focusExplorer()
      return
    }
    if (event.key !== "F6" || event.ctrlKey || event.metaKey || event.altKey) {
      return
    }
    const regions = availableRegions(elements)
    const current = regionContaining(elements, elements.document.activeElement)
    const next = nextWorkspaceFocusRegion(regions, current, event.shiftKey)
    if (next === undefined) return
    event.preventDefault()
    focusRegion(next, elements, actions)
  })
}

function availableRegions(
  elements: Readonly<{
    explorerPanel: HTMLElement
    workspaceTabs: HTMLElement
    editorSurface: HTMLElement
    ioPanel: HTMLElement
  }>
): readonly WorkspaceFocusRegion[] {
  return [
    ...(elements.explorerPanel.hidden ? [] : (["explorer"] as const)),
    ...(elements.workspaceTabs.hidden ? [] : (["tabs"] as const)),
    "editor",
    "io",
  ]
}

function regionContaining(
  elements: Readonly<{
    explorerPanel: HTMLElement
    workspaceTabs: HTMLElement
    editorSurface: HTMLElement
    ioPanel: HTMLElement
  }>,
  active: Element | null
): WorkspaceFocusRegion | undefined {
  if (active === null) return undefined
  if (elements.explorerPanel.contains(active)) return "explorer"
  if (elements.workspaceTabs.contains(active)) return "tabs"
  if (elements.editorSurface.contains(active)) return "editor"
  if (elements.ioPanel.contains(active)) return "io"
  return undefined
}

function focusRegion(
  region: WorkspaceFocusRegion,
  elements: Readonly<{
    workspaceTabs: HTMLElement
    ioPanel: HTMLElement
  }>,
  actions: Readonly<{
    focusExplorer: () => void
    focusEditor: () => void
    showCode: () => void
    showIo: () => void
  }>
): void {
  if (region === "explorer") {
    actions.showCode()
    actions.focusExplorer()
    return
  }
  if (region === "tabs") {
    actions.showCode()
    elements.workspaceTabs
      .querySelector<HTMLElement>('[role="tab"][aria-selected="true"]')
      ?.focus()
    return
  }
  if (region === "editor") {
    actions.showCode()
    actions.focusEditor()
    return
  }
  actions.showIo()
  elements.ioPanel.focus()
}
