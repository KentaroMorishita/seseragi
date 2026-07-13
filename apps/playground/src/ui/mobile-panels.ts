export function connectMobilePanels(workspace: HTMLElement): void {
  const tabs = document.querySelectorAll<HTMLButtonElement>(
    "[data-panel-target]"
  )
  for (const tab of tabs) {
    tab.addEventListener("click", () => {
      const target = tab.dataset.panelTarget
      if (target !== "code" && target !== "io") return
      workspace.dataset.mobilePanel = target
      for (const candidate of tabs) {
        candidate.setAttribute(
          "aria-pressed",
          String(candidate.dataset.panelTarget === target)
        )
      }
    })
  }
}
