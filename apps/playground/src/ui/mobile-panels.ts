// 狭い画面では、出力のカードを下から出すシートにする（DESIGN.md「3. 画面の骨格 › モバイル」）。
// シートは三段で止まる。
// - "closed": 持ち手と出力の見出しだけを残して畳み、エディターを広く使う。
// - "code": 196px だけ顔を出す（開いたときの初期状態）。
// - "io": 画面の 7 割まで引き上げる（Run のあと）。
// [data-panel-target] はその状態にし、[data-panel-toggle]（シートの持ち手）は
// "io" と "code" を行き来する。畳んでいるときは "code" に戻す。
export type MobilePanel = "closed" | "code" | "io"

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
  const current = (): MobilePanel => {
    const panel = workspace.dataset.mobilePanel
    return panel === "io" || panel === "closed" ? panel : "code"
  }
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
      if (panel !== "code" && panel !== "io" && panel !== "closed") return
      show(panel)
    })
  }
  for (const toggle of toggles) {
    toggle.addEventListener("click", () =>
      show(current() === "code" ? "io" : "code")
    )
  }
  show(current())
  return { show, current }
}
