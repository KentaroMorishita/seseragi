const WORKSPACE_RATIO_KEY = "seseragi.playground.workspace-ratio"
const IO_RATIO_KEY = "seseragi.playground.io-ratio"

const DEFAULT_WORKSPACE_RATIO = 0.68
const DEFAULT_IO_RATIO = 0.34
const KEYBOARD_STEP = 0.025

type Axis = "horizontal" | "vertical"

type ResizerOptions = {
  readonly container: HTMLElement
  readonly resizer: HTMLElement
  readonly axis: Axis
  readonly cssProperty: string
  readonly storageKey: string
  readonly defaultRatio: number
  readonly minBefore: number
  readonly minAfter: number
}

export type PanelLayoutElements = {
  readonly workspace: HTMLElement
  readonly workspaceResizer: HTMLElement
  readonly ioPanel: HTMLElement
  readonly ioResizer: HTMLElement
}

export function clampPanelRatio(
  ratio: number,
  size: number,
  minBefore: number,
  minAfter: number
): number {
  if (!Number.isFinite(ratio) || size <= minBefore + minAfter) return 0.5
  return Math.min(1 - minAfter / size, Math.max(minBefore / size, ratio))
}

export function readPanelRatio(
  storage: Pick<Storage, "getItem">,
  key: string,
  fallback: number
): number {
  const value = Number(storage.getItem(key))
  return Number.isFinite(value) && value > 0 && value < 1 ? value : fallback
}

export function connectPanelLayout(elements: PanelLayoutElements): void {
  connectResizer({
    container: elements.workspace,
    resizer: elements.workspaceResizer,
    axis: "horizontal",
    cssProperty: "--editor-panel-ratio",
    storageKey: WORKSPACE_RATIO_KEY,
    defaultRatio: DEFAULT_WORKSPACE_RATIO,
    minBefore: 360,
    minAfter: 300,
  })
  connectResizer({
    container: elements.ioPanel,
    resizer: elements.ioResizer,
    axis: "vertical",
    cssProperty: "--stdin-panel-ratio",
    storageKey: IO_RATIO_KEY,
    defaultRatio: DEFAULT_IO_RATIO,
    minBefore: 104,
    minAfter: 160,
  })
}

function connectResizer(options: ResizerOptions): void {
  let ratio = readStoredRatio(options.storageKey, options.defaultRatio)

  const size = (): number =>
    options.axis === "horizontal"
      ? options.container.getBoundingClientRect().width
      : options.container.getBoundingClientRect().height

  const apply = (nextRatio: number, persist: boolean): void => {
    ratio = clampPanelRatio(
      nextRatio,
      size(),
      options.minBefore,
      options.minAfter
    )
    options.container.style.setProperty(
      options.cssProperty,
      `${(ratio * 100).toFixed(2)}%`
    )
    options.resizer.setAttribute(
      "aria-valuenow",
      String(Math.round(ratio * 100))
    )
    if (persist) writeStoredRatio(options.storageKey, ratio)
  }

  apply(ratio, false)

  options.resizer.addEventListener("pointerdown", (event) => {
    if (!isDesktopLayout()) return
    event.preventDefault()
    options.resizer.setPointerCapture(event.pointerId)
    options.resizer.dataset.dragging = "true"
  })
  options.resizer.addEventListener("pointermove", (event) => {
    if (!options.resizer.hasPointerCapture(event.pointerId)) return
    const bounds = options.container.getBoundingClientRect()
    const position =
      options.axis === "horizontal"
        ? event.clientX - bounds.left
        : event.clientY - bounds.top
    const containerSize =
      options.axis === "horizontal" ? bounds.width : bounds.height
    apply(position / containerSize, false)
  })
  const finishPointerResize = (event: PointerEvent): void => {
    if (!options.resizer.hasPointerCapture(event.pointerId)) return
    options.resizer.releasePointerCapture(event.pointerId)
    delete options.resizer.dataset.dragging
    writeStoredRatio(options.storageKey, ratio)
  }
  options.resizer.addEventListener("pointerup", finishPointerResize)
  options.resizer.addEventListener("pointercancel", finishPointerResize)

  options.resizer.addEventListener("keydown", (event) => {
    const decreaseKey = options.axis === "horizontal" ? "ArrowLeft" : "ArrowUp"
    const increaseKey =
      options.axis === "horizontal" ? "ArrowRight" : "ArrowDown"
    let nextRatio: number | undefined
    if (event.key === decreaseKey) nextRatio = ratio - KEYBOARD_STEP
    if (event.key === increaseKey) nextRatio = ratio + KEYBOARD_STEP
    if (event.key === "Home") nextRatio = 0
    if (event.key === "End") nextRatio = 1
    if (event.key === "Enter") nextRatio = options.defaultRatio
    if (nextRatio === undefined) return
    event.preventDefault()
    apply(nextRatio, true)
  })
  window.addEventListener("resize", () => apply(ratio, false))
}

function isDesktopLayout(): boolean {
  return !window.matchMedia(
    "(max-width: 760px), (max-width: 960px) and (max-height: 520px)"
  ).matches
}

function readStoredRatio(key: string, fallback: number): number {
  try {
    return readPanelRatio(window.localStorage, key, fallback)
  } catch {
    return fallback
  }
}

function writeStoredRatio(key: string, ratio: number): void {
  try {
    window.localStorage.setItem(key, String(ratio))
  } catch {
    // Storage may be unavailable in hardened or private browser contexts.
  }
}
