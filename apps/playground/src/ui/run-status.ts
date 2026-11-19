// 出力のカードの右上に出す、実行の状態のチップ（DESIGN.md「5. コンポーネントの決めごと › 出力」）。
// 色だけに頼らず、✓ や件数の文字でも結果を伝える。文言はラベルなので英語。
// 完了したときの数字は main の実行時間。押すと区間ごとの内訳を開く（run-timing-panel.ts）。

import { formatChipMilliseconds, type RunTiming } from "../runtime/run-timing"

export type RunOutcome =
  | Readonly<{ kind: "idle" }>
  | Readonly<{ kind: "running" }>
  | Readonly<{ kind: "completed"; timing?: RunTiming }>
  | Readonly<{ kind: "interactive" }>
  | Readonly<{ kind: "diagnostics"; count: number }>
  | Readonly<{ kind: "failed" }>

export type RunStatusView = Readonly<{
  hidden: boolean
  text: string
  tone: "neutral" | "ok" | "error"
  /** 押すと実行時間の内訳を開けるか */
  details: boolean
}>

export function runStatusView(outcome: RunOutcome): RunStatusView {
  switch (outcome.kind) {
    case "idle":
      return { hidden: true, text: "", tone: "neutral", details: false }
    case "running":
      return {
        hidden: false,
        text: "Running…",
        tone: "neutral",
        details: false,
      }
    case "completed":
      // 時間が分からないときは、数字を出さずに完了だけを伝える
      return outcome.timing === undefined
        ? { hidden: false, text: "✓ Completed", tone: "ok", details: false }
        : {
            hidden: false,
            text: `✓ ${formatChipMilliseconds(outcome.timing.mainMilliseconds)}`,
            tone: "ok",
            details: true,
          }
    case "interactive":
      return {
        hidden: false,
        text: "✓ Interactive",
        tone: "ok",
        details: false,
      }
    case "diagnostics":
      return {
        hidden: false,
        text: `${outcome.count} ${outcome.count === 1 ? "error" : "errors"}`,
        tone: "error",
        details: false,
      }
    case "failed":
      return { hidden: false, text: "Failed", tone: "error", details: false }
  }
}

export function renderRunStatus(
  element: HTMLElement,
  outcome: RunOutcome
): void {
  const view = runStatusView(outcome)
  element.hidden = view.hidden
  if (view.details) {
    // 状態の読み上げ（role="status"）は残し、中のボタンで内訳を開く
    const button = element.ownerDocument.createElement("button")
    button.type = "button"
    button.className = "ui-status-button"
    button.textContent = view.text
    button.title = "Run timing"
    button.setAttribute("aria-haspopup", "dialog")
    button.setAttribute("aria-expanded", "false")
    button.setAttribute("aria-controls", "run-timing")
    element.replaceChildren(button)
  } else {
    element.textContent = view.text
  }
  element.classList.toggle("ui-status--action", view.details)
  element.classList.toggle("ui-status--ok", view.tone === "ok")
  element.classList.toggle("ui-status--error", view.tone === "error")
}
