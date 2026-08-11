import { clampPanelRatio, readPanelRatio } from "../ui/panel-layout"
import {
  beginExclusiveResize,
  finishExclusiveResize,
  ownsExclusiveResize,
} from "../ui/resize-coordinator"

const NAVIGATION_RATIO_KEY = "seseragi.tour.navigation-ratio"
const LESSON_RATIO_KEY = "seseragi.tour.lesson-ratio"
const CODE_RATIO_KEY = "seseragi.tour.code-ratio"

const DEFAULT_NAVIGATION_RATIO = 0.2
const DEFAULT_LESSON_RATIO = 0.43
const DEFAULT_CODE_RATIO = 0.64
const KEYBOARD_STEP = 0.025
const RESIZER_SIZE = 9

const NAVIGATION_MIN_WIDTH = 240
const LESSON_MIN_WIDTH = 300
const LAB_MIN_WIDTH = 460
const CODE_MIN_HEIGHT = 280
const OUTPUT_MIN_HEIGHT = 190

type TourPaneLayoutElements = Readonly<{
  workspace: HTMLElement
  navigationResizer: HTMLElement
  navigationToggle: HTMLButtonElement
  lessonResizer: HTMLElement
  lab: HTMLElement
  outputResizer: HTMLElement
  outputToggle: HTMLButtonElement
}>

type ResizerOptions = Readonly<{
  axis: "horizontal" | "vertical"
  defaultRatio: number
  getRatio: () => number
  getRatioFromPointer: (event: PointerEvent) => number
  onCommit: () => void
  resizer: HTMLElement
  setRatio: (ratio: number) => void
}>

export function connectTourPaneLayout(
  elements: TourPaneLayoutElements,
  onLayoutChange: () => void
): void {
  const desktopQuery = window.matchMedia("(min-width: 1181px)")
  let navigationRatio = readStoredRatio(
    NAVIGATION_RATIO_KEY,
    DEFAULT_NAVIGATION_RATIO
  )
  let lessonRatio = readStoredRatio(LESSON_RATIO_KEY, DEFAULT_LESSON_RATIO)
  let codeRatio = readStoredRatio(CODE_RATIO_KEY, DEFAULT_CODE_RATIO)
  let navigationWidth = 0
  let lessonWidth = 0
  let codeHeight = 0
  let navigationCollapsed = false
  let outputCollapsed = false

  const scheduleLayoutChange = (): void => {
    requestAnimationFrame(() => {
      onLayoutChange()
    })
  }

  const applyHorizontalRatios = (): void => {
    const width = elements.workspace.getBoundingClientRect().width
    if (width <= 0) return
    navigationRatio = clampPanelRatio(
      navigationRatio,
      width,
      NAVIGATION_MIN_WIDTH,
      LESSON_MIN_WIDTH + LAB_MIN_WIDTH + RESIZER_SIZE * 2
    )
    navigationWidth = navigationRatio * width
    const remaining = width - navigationWidth - RESIZER_SIZE
    lessonRatio = clampPanelRatio(
      lessonRatio,
      remaining,
      LESSON_MIN_WIDTH,
      LAB_MIN_WIDTH + RESIZER_SIZE
    )
    lessonWidth = lessonRatio * remaining
    elements.workspace.style.setProperty(
      "--tour-navigation-width",
      `${navigationWidth.toFixed(2)}px`
    )
    elements.workspace.style.setProperty(
      "--tour-lesson-width",
      `${lessonWidth.toFixed(2)}px`
    )
    elements.navigationResizer.setAttribute(
      "aria-valuenow",
      String(Math.round(navigationWidth))
    )
    elements.navigationResizer.setAttribute(
      "aria-valuemax",
      String(
        Math.round(
          width - (LESSON_MIN_WIDTH + LAB_MIN_WIDTH + RESIZER_SIZE * 2)
        )
      )
    )
    elements.lessonResizer.setAttribute(
      "aria-valuenow",
      String(Math.round(lessonWidth))
    )
    elements.lessonResizer.setAttribute(
      "aria-valuemax",
      String(Math.round(remaining - (LAB_MIN_WIDTH + RESIZER_SIZE)))
    )
    scheduleLayoutChange()
  }

  const applyCodeRatio = (): void => {
    const height = elements.lab.getBoundingClientRect().height
    if (height <= 0) return
    codeRatio = clampPanelRatio(
      codeRatio,
      height,
      CODE_MIN_HEIGHT,
      OUTPUT_MIN_HEIGHT + RESIZER_SIZE
    )
    codeHeight = codeRatio * height
    elements.lab.style.setProperty(
      "--tour-code-pane-height",
      `${codeHeight.toFixed(2)}px`
    )
    elements.outputResizer.setAttribute(
      "aria-valuenow",
      String(Math.round(codeHeight))
    )
    elements.outputResizer.setAttribute(
      "aria-valuemax",
      String(Math.round(height - (OUTPUT_MIN_HEIGHT + RESIZER_SIZE)))
    )
    scheduleLayoutChange()
  }

  const syncNavigationToggle = (): void => {
    elements.workspace.dataset.navigationCollapsed = String(navigationCollapsed)
    const label = navigationCollapsed
      ? "lesson一覧を開く"
      : "lesson一覧を閉じる"
    elements.navigationToggle.setAttribute(
      "aria-expanded",
      String(!navigationCollapsed)
    )
    elements.navigationToggle.setAttribute("aria-label", label)
    elements.navigationToggle.title = label
    scheduleLayoutChange()
  }

  const syncOutputToggle = (): void => {
    elements.lab.dataset.outputCollapsed = String(outputCollapsed)
    const label = outputCollapsed ? "Outputを開く" : "Outputを閉じる"
    elements.outputToggle.setAttribute(
      "aria-expanded",
      String(!outputCollapsed)
    )
    elements.outputToggle.setAttribute("aria-label", label)
    elements.outputToggle.title = label
    scheduleLayoutChange()
  }

  connectResizer(desktopQuery, {
    axis: "horizontal",
    defaultRatio: DEFAULT_NAVIGATION_RATIO,
    getRatio: () => navigationRatio,
    getRatioFromPointer: (event) => {
      const bounds = elements.workspace.getBoundingClientRect()
      return (event.clientX - bounds.left) / bounds.width
    },
    onCommit: () => writeStoredRatio(NAVIGATION_RATIO_KEY, navigationRatio),
    resizer: elements.navigationResizer,
    setRatio: (ratio) => {
      navigationRatio = ratio
      applyHorizontalRatios()
    },
  })

  connectResizer(desktopQuery, {
    axis: "horizontal",
    defaultRatio: DEFAULT_LESSON_RATIO,
    getRatio: () => lessonRatio,
    getRatioFromPointer: (event) => {
      const bounds = elements.workspace.getBoundingClientRect()
      const remaining = bounds.width - navigationWidth - RESIZER_SIZE
      const position =
        event.clientX - bounds.left - navigationWidth - RESIZER_SIZE
      return position / remaining
    },
    onCommit: () => writeStoredRatio(LESSON_RATIO_KEY, lessonRatio),
    resizer: elements.lessonResizer,
    setRatio: (ratio) => {
      lessonRatio = ratio
      applyHorizontalRatios()
    },
  })

  connectResizer(desktopQuery, {
    axis: "vertical",
    defaultRatio: DEFAULT_CODE_RATIO,
    getRatio: () => codeRatio,
    getRatioFromPointer: (event) => {
      const bounds = elements.lab.getBoundingClientRect()
      return (event.clientY - bounds.top) / bounds.height
    },
    onCommit: () => writeStoredRatio(CODE_RATIO_KEY, codeRatio),
    resizer: elements.outputResizer,
    setRatio: (ratio) => {
      codeRatio = ratio
      applyCodeRatio()
    },
  })

  elements.navigationToggle.addEventListener("click", () => {
    if (!desktopQuery.matches) return
    navigationCollapsed = !navigationCollapsed
    syncNavigationToggle()
  })
  elements.outputToggle.addEventListener("click", () => {
    if (!desktopQuery.matches) return
    outputCollapsed = !outputCollapsed
    syncOutputToggle()
  })
  window.addEventListener("resize", () => {
    applyHorizontalRatios()
    applyCodeRatio()
  })
  desktopQuery.addEventListener("change", () => {
    applyHorizontalRatios()
    applyCodeRatio()
    scheduleLayoutChange()
  })

  applyHorizontalRatios()
  applyCodeRatio()
  syncNavigationToggle()
  syncOutputToggle()
}

function connectResizer(
  desktopQuery: MediaQueryList,
  options: ResizerOptions
): void {
  options.resizer.addEventListener("pointerdown", (event) => {
    if (!desktopQuery.matches) return
    if (!beginExclusiveResize(options.resizer, event.pointerId)) return
    event.preventDefault()
  })
  options.resizer.addEventListener("pointermove", (event) => {
    if (!ownsExclusiveResize(options.resizer, event.pointerId)) return
    options.setRatio(options.getRatioFromPointer(event))
  })
  const finishPointerResize = (event: PointerEvent): void => {
    if (!finishExclusiveResize(options.resizer, event.pointerId)) return
    options.onCommit()
  }
  options.resizer.addEventListener("pointerup", finishPointerResize)
  options.resizer.addEventListener("pointercancel", finishPointerResize)
  options.resizer.addEventListener("lostpointercapture", finishPointerResize)
  options.resizer.addEventListener("keydown", (event) => {
    if (!desktopQuery.matches) return
    const decreaseKey = options.axis === "horizontal" ? "ArrowLeft" : "ArrowUp"
    const increaseKey =
      options.axis === "horizontal" ? "ArrowRight" : "ArrowDown"
    let ratio: number | undefined
    if (event.key === decreaseKey) ratio = options.getRatio() - KEYBOARD_STEP
    if (event.key === increaseKey) ratio = options.getRatio() + KEYBOARD_STEP
    if (event.key === "Home") ratio = 0
    if (event.key === "End") ratio = 1
    if (event.key === "Enter") ratio = options.defaultRatio
    if (ratio === undefined) return
    event.preventDefault()
    options.setRatio(ratio)
    options.onCommit()
  })
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
    // Private or hardened browsing may deny storage without disabling resizing.
  }
}
