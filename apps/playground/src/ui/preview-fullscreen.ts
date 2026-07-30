export type PreviewFullscreenController = Readonly<{
  toggle: () => Promise<void>
  exit: () => Promise<void>
  mode: () => "native" | "fallback" | undefined
}>

const fallbackBodyClass = "preview-fullscreen-fallback-open"

export function connectPreviewFullscreen(
  surface: HTMLElement,
  button: HTMLButtonElement
): PreviewFullscreenController {
  const ownerDocument = surface.ownerDocument
  let activeMode: "native" | "fallback" | undefined

  surface.classList.add("preview-fullscreen-surface")

  const syncButton = (): void => {
    const active = activeMode !== undefined
    button.setAttribute("aria-pressed", String(active))
    button.setAttribute(
      "aria-label",
      active ? "Close full screen preview" : "Open full screen preview"
    )
    button.textContent = active ? "Close" : "Full screen"
  }

  const setMode = (
    mode: "native" | "fallback" | undefined,
    fallbackReason?: "unsupported" | "rejected"
  ): void => {
    activeMode = mode
    if (mode === undefined) {
      delete surface.dataset.previewFullscreen
      delete surface.dataset.previewFullscreenFallback
      delete surface.dataset.previewFullscreenExit
      button.removeAttribute("title")
    } else {
      surface.dataset.previewFullscreen = mode
      if (mode === "fallback" && fallbackReason !== undefined) {
        surface.dataset.previewFullscreenFallback = fallbackReason
      } else {
        delete surface.dataset.previewFullscreenFallback
      }
    }
    ownerDocument.body.classList.toggle(fallbackBodyClass, mode === "fallback")
    syncButton()
  }

  const useFallback = (
    reason: "unsupported" | "rejected" = "rejected"
  ): void => {
    setMode("fallback", reason)
  }

  const enter = async (): Promise<void> => {
    const requestFullscreen = surface.requestFullscreen
    if (typeof requestFullscreen !== "function") {
      useFallback("unsupported")
      return
    }

    try {
      await requestFullscreen.call(surface)
      if (ownerDocument.fullscreenElement === surface) {
        setMode("native")
      } else {
        useFallback("rejected")
      }
    } catch {
      useFallback("rejected")
    }
  }

  const exit = async (): Promise<void> => {
    if (activeMode === "fallback") {
      setMode(undefined)
      return
    }
    if (ownerDocument.fullscreenElement !== surface) {
      setMode(undefined)
      return
    }

    try {
      await ownerDocument.exitFullscreen()
    } catch {
      surface.dataset.previewFullscreenExit = "failed"
      button.title =
        "全画面を閉じられませんでした。browserの全画面解除操作を使用してください。"
      syncButton()
      return
    }
    setMode(undefined)
  }

  const toggle = async (): Promise<void> => {
    if (
      activeMode !== undefined ||
      ownerDocument.fullscreenElement === surface
    ) {
      await exit()
    } else {
      await enter()
    }
  }

  const syncNativeState = (): void => {
    if (ownerDocument.fullscreenElement === surface) {
      setMode("native")
    } else if (activeMode === "native") {
      setMode(undefined)
    }
  }

  button.addEventListener("click", () => void toggle())
  ownerDocument.addEventListener("fullscreenchange", syncNativeState)
  surface.addEventListener("fullscreenerror", () => {
    if (activeMode !== "native") useFallback("rejected")
  })
  ownerDocument.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || activeMode !== "fallback") return
    event.preventDefault()
    void exit()
  })
  syncButton()
  return {
    toggle,
    exit,
    mode: () => activeMode,
  }
}
