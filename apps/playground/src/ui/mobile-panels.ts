// 狭い画面では、出力のカードを下から顔を出すシートにする（DESIGN.md「3. 画面の骨格 › モバイル」）。
// "code" はシートが顔を出しているだけの状態、"io" はシートを引き上げた状態。
// - [data-panel-target="code" | "io"] は、その状態にする。
// - [data-panel-toggle] は、二つの状態を行き来する（シートの持ち手）。
export type MobilePanel = "code" | "io"

export function connectMobilePanels(workspace: HTMLElement): {
  readonly show: (panel: MobilePanel) => void
  readonly current: () => MobilePanel
} {
  const targets = workspace.ownerDocument.querySelectorAll<HTMLButtonElement>(
    "[data-panel-target]"
  )
  const toggles = workspace.ownerDocument.querySelectorAll<HTMLButtonElement>(
    "[data-panel-toggle]"
  )
  const current = (): MobilePanel =>
    workspace.dataset.mobilePanel === "io" ? "io" : "code"
  const show = (target: MobilePanel): void => {
    workspace.dataset.mobilePanel = target
    for (const candidate of targets) {
      candidate.setAttribute(
        "aria-pressed",
        String(candidate.dataset.panelTarget === target)
      )
    }
    for (const toggle of toggles) {
      toggle.setAttribute("aria-expanded", String(target === "io"))
    }
  }
  for (const target of targets) {
    target.addEventListener("click", () => {
      const panel = target.dataset.panelTarget
      if (panel !== "code" && panel !== "io") return
      show(panel)
    })
  }
  for (const toggle of toggles) {
    toggle.addEventListener("click", () =>
      show(current() === "io" ? "code" : "io")
    )
  }
  show(current())
  return { show, current }
}
