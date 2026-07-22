export type MobilePanel = "code" | "io"

export function connectMobilePanels(workspace: HTMLElement): {
  readonly show: (panel: MobilePanel) => void
} {
  const tabs = document.querySelectorAll<HTMLButtonElement>(
    "[data-panel-target]"
  )
  const show = (target: MobilePanel): void => {
    workspace.dataset.mobilePanel = target
    for (const candidate of tabs) {
      candidate.setAttribute(
        "aria-pressed",
        String(candidate.dataset.panelTarget === target)
      )
    }
  }
  for (const tab of tabs) {
    tab.addEventListener("click", () => {
      const target = tab.dataset.panelTarget
      if (target !== "code" && target !== "io") return
      show(target)
    })
  }
  return { show }
}
