// 出力のカードの右上に出す、実行の状態のチップ（DESIGN.md「5. コンポーネントの決めごと › 出力」）。
// 色だけに頼らず、✓ や件数の文字でも結果を伝える。文言はラベルなので英語。

export type RunOutcome =
  | Readonly<{ kind: "idle" }>
  | Readonly<{ kind: "running" }>
  | Readonly<{ kind: "completed"; milliseconds: number }>
  | Readonly<{ kind: "interactive" }>
  | Readonly<{ kind: "diagnostics"; count: number }>
  | Readonly<{ kind: "failed" }>

export type RunStatusView = Readonly<{
  hidden: boolean
  text: string
  tone: "neutral" | "ok" | "error"
}>

export function runStatusView(outcome: RunOutcome): RunStatusView {
  switch (outcome.kind) {
    case "idle":
      return { hidden: true, text: "", tone: "neutral" }
    case "running":
      return { hidden: false, text: "Running…", tone: "neutral" }
    case "completed":
      return {
        hidden: false,
        text: `✓ ${Math.max(0, Math.round(outcome.milliseconds))} ms`,
        tone: "ok",
      }
    case "interactive":
      return { hidden: false, text: "✓ Interactive", tone: "ok" }
    case "diagnostics":
      return {
        hidden: false,
        text: `${outcome.count} ${outcome.count === 1 ? "error" : "errors"}`,
        tone: "error",
      }
    case "failed":
      return { hidden: false, text: "Failed", tone: "error" }
  }
}

export function renderRunStatus(element: HTMLElement, outcome: RunOutcome): void {
  const view = runStatusView(outcome)
  element.hidden = view.hidden
  element.textContent = view.text
  element.classList.toggle("ui-status--ok", view.tone === "ok")
  element.classList.toggle("ui-status--error", view.tone === "error")
}
