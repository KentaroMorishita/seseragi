// 実行時間の内訳。Output の状態のチップ（✓ 1 ms）を押すと開く。
// 区間の名前は UI のラベルなので英語にする（DESIGN.md「画面の言葉」）。
//
// 出力のカードは中身をはみ出させず、モバイルでは動くシートにもなるので、
// 内訳は body の直下に置き、開いている間はチップの位置について行く。

import { formatPhaseMilliseconds, type RunTiming } from "../runtime/run-timing"

export type RunTimingPanel = Readonly<{
  /** 新しい実行の結果に切り替える。開いていれば閉じる */
  update: (timing: RunTiming | undefined) => void
  close: () => void
}>

const viewportMargin = 16

export function connectRunTimingPanel(
  status: HTMLElement,
  panel: HTMLElement
): RunTimingPanel {
  const ownerDocument = panel.ownerDocument
  const view = ownerDocument.defaultView!
  let timing: RunTiming | undefined
  let following = 0

  const toggleButton = () =>
    status.querySelector<HTMLButtonElement>(".ui-status-button")

  const close = (restoreFocus = false): void => {
    if (panel.hidden) return
    panel.hidden = true
    view.cancelAnimationFrame(following)
    const button = toggleButton()
    button?.setAttribute("aria-expanded", "false")
    if (restoreFocus) button?.focus()
  }

  const place = (button: HTMLElement): void => {
    const anchor = button.getBoundingClientRect()
    const width = Math.min(
      panel.offsetWidth,
      view.innerWidth - viewportMargin * 2
    )
    const left = Math.min(
      Math.max(viewportMargin, anchor.right - width),
      view.innerWidth - viewportMargin - width
    )
    // 下に入りきらなければ、チップの上に開く
    const below = anchor.bottom + 8
    const top =
      below + panel.offsetHeight <= view.innerHeight - viewportMargin
        ? below
        : Math.max(viewportMargin, anchor.top - 8 - panel.offsetHeight)
    panel.style.left = `${left}px`
    panel.style.top = `${top}px`
  }

  // Run のあとにモバイルのシートが上がる途中でも、チップから離れないようにする
  const follow = (button: HTMLElement): void => {
    let last = ""
    const tick = () => {
      if (panel.hidden || !button.isConnected) return
      const { top, right } = button.getBoundingClientRect()
      const key = `${top},${right},${view.innerWidth},${view.innerHeight}`
      if (key !== last) {
        last = key
        place(button)
      }
      following = view.requestAnimationFrame(tick)
    }
    tick()
  }

  const open = (button: HTMLElement): void => {
    if (timing === undefined) return
    render(panel, timing)
    panel.hidden = false
    follow(button)
    button.setAttribute("aria-expanded", "true")
  }

  status.addEventListener("click", (event) => {
    const button = (event.target as Element).closest<HTMLElement>(
      ".ui-status-button"
    )
    if (button === null) return
    if (panel.hidden) open(button)
    else close()
  })
  ownerDocument.addEventListener("pointerdown", (event) => {
    const target = event.target as Node
    if (!panel.hidden && !panel.contains(target) && !status.contains(target))
      close()
  })
  ownerDocument.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || panel.hidden) return
    event.preventDefault()
    close(true)
  })

  return {
    update: (next) => {
      close()
      timing = next
    },
    close: () => close(),
  }
}

function render(panel: HTMLElement, timing: RunTiming): void {
  const ownerDocument = panel.ownerDocument
  const element = <K extends keyof HTMLElementTagNameMap>(
    tag: K,
    className: string,
    text?: string
  ) => {
    const node = ownerDocument.createElement(tag)
    node.className = className
    if (text !== undefined) node.textContent = text
    return node
  }

  const head = element("div", "pg-run-timing-head")
  head.append(
    element("h2", "pg-run-timing-title", "Run timing"),
    element(
      "span",
      "pg-run-timing-total",
      `Total ${formatPhaseMilliseconds(timing.totalMilliseconds)}`
    )
  )

  const list = element("dl", "pg-run-timing-list")
  const largest = Math.max(...timing.phases.map((phase) => phase.milliseconds))
  for (const phase of timing.phases) {
    const row = element("div", "pg-run-timing-row")
    row.dataset.phase = phase.id
    const bar = element("span", "pg-run-timing-bar")
    bar.setAttribute("aria-hidden", "true")
    // 一番長い区間を 100% にして、長さを比べられるようにする
    bar.style.setProperty(
      "--run-timing-share",
      `${largest > 0 ? (phase.milliseconds / largest) * 100 : 0}%`
    )
    row.append(
      element("dt", "", phase.label),
      element("dd", "", formatPhaseMilliseconds(phase.milliseconds)),
      bar
    )
    list.append(row)
  }
  panel.replaceChildren(head, list)
}
