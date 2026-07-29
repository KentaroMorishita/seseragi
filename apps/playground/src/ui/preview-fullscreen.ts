export type PreviewFullscreenController = Readonly<{
  enter: () => Promise<void>
}>

export function connectPreviewFullscreen(
  surface: HTMLElement,
  button: HTMLButtonElement
): PreviewFullscreenController {
  const sync = (): void => {
    const active = document.fullscreenElement === surface
    button.setAttribute("aria-pressed", String(active))
    button.textContent = active ? "Exit full screen" : "Full screen"
  }
  const enter = async (): Promise<void> => {
    if (document.fullscreenElement === surface) {
      await document.exitFullscreen()
    } else {
      await surface.requestFullscreen()
    }
  }
  button.addEventListener("click", () => void enter().catch(() => undefined))
  document.addEventListener("fullscreenchange", sync)
  sync()
  return { enter }
}
