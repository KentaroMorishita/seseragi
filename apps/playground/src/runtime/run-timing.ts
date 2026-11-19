// 実行時間の内訳。Output の状態のチップに main の実行時間を出し、
// チップを押すと区間ごとの時間を見せる。
//
// 時刻は、シェル（エディター）、実行用のフレーム、Worker の三か所で記録する。
// 別々の実行環境の時刻を比べるため、`performance.timeOrigin` を足した値を使う。
// フレームと Worker から届く値はプログラムが書き換えられるので、
// 表示に使う前に数として確かめ、負の区間は 0 にそろえる。

export function timingClock(): number {
  return performance.timeOrigin + performance.now()
}

/** Worker（DOM を使うプログラムではフレーム）が記録する */
export type ExecutionTimingMarks = {
  /** Worker のスクリプトが動き始めた時刻。読み込みの終わり */
  workerScriptStartedAt?: number
  evaluateStartedAt?: number
  /** TypeScript コンパイラの初期化にかかった時間 */
  compilerInitMilliseconds: number
  /** 生成された TypeScript を JavaScript に変換した時間 */
  transpileMilliseconds: number
  evaluatedAt?: number
  mainStartedAt?: number
  mainEndedAt?: number
  cleanedUpAt?: number
}

export function createExecutionTimingMarks(): ExecutionTimingMarks {
  return { compilerInitMilliseconds: 0, transpileMilliseconds: 0 }
}

/** フレームが Worker を作るときに記録する */
export type FrameTimingMarks = {
  runReceivedAt: number
  workerCreatedAt: number
}

/** シェルが記録し、フレームと Worker から届いた値とまとめて返す */
export type ExecutionTimingReport = Readonly<{
  frameStartedAt: number
  runSentAt: number
  completedAt: number
  frame?: unknown
  worker?: unknown
}>

export type RunTimingPhaseId =
  | "compile"
  | "frame"
  | "worker-start"
  | "worker-load"
  | "compiler-init"
  | "transpile"
  | "evaluate"
  | "environment"
  | "main"
  | "cleanup"
  | "messaging"

export type RunTimingPhase = Readonly<{
  id: RunTimingPhaseId
  label: string
  milliseconds: number
}>

export type RunTiming = Readonly<{
  mainMilliseconds: number
  totalMilliseconds: number
  phases: readonly RunTimingPhase[]
}>

// 区間の名前は UI のラベルなので英語にする（DESIGN.md「画面の言葉」）
const phaseLabels: Readonly<Record<RunTimingPhaseId, string>> = {
  compile: "Compile",
  frame: "Sandbox start",
  "worker-start": "Worker start",
  "worker-load": "Worker load",
  "compiler-init": "TypeScript init",
  transpile: "Transpile",
  evaluate: "Evaluate",
  environment: "Environment",
  main: "main",
  cleanup: "Cleanup",
  messaging: "Messaging",
}

// 届いた時刻を受け入れる幅。10 秒の上限を大きく超える区間は壊れた値として扱う
const maxSpanMilliseconds = 60_000

function finite(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined
}

function span(from: number | undefined, to: number | undefined) {
  if (from === undefined || to === undefined) return undefined
  const milliseconds = to - from
  if (milliseconds > maxSpanMilliseconds) return undefined
  return Math.max(0, milliseconds)
}

function duration(value: unknown): number | undefined {
  const milliseconds = finite(value)
  if (milliseconds === undefined || milliseconds < 0) return undefined
  return milliseconds > maxSpanMilliseconds ? undefined : milliseconds
}

function field(source: unknown, name: string): unknown {
  return source !== null && typeof source === "object"
    ? (source as Record<string, unknown>)[name]
    : undefined
}

/**
 * 区間ごとの時間にまとめる。main の時間が分からないときは undefined を返し、
 * チップは数字を出さない。区間の和が全体と合うよう、残りを Messaging に入れる。
 */
export function summarizeRunTiming(
  report: ExecutionTimingReport,
  compileMilliseconds?: number
): RunTiming | undefined {
  const at = (source: unknown, name: string) => finite(field(source, name))
  const { worker, frame } = report
  const main = span(at(worker, "mainStartedAt"), at(worker, "mainEndedAt"))
  const sandbox = span(report.frameStartedAt, report.runSentAt)
  const afterSend = span(report.runSentAt, report.completedAt)
  if (main === undefined || sandbox === undefined || afterSend === undefined)
    return undefined

  const compilerInit = duration(field(worker, "compilerInitMilliseconds")) ?? 0
  const transpile = duration(field(worker, "transpileMilliseconds")) ?? 0
  const evaluateTotal = span(
    at(worker, "evaluateStartedAt"),
    at(worker, "evaluatedAt")
  )
  const measured: [RunTimingPhaseId, number | undefined][] = [
    ["compile", duration(compileMilliseconds)],
    ["frame", sandbox],
    // DOM を使うプログラムはフレームで動くので、Worker の区間はない
    [
      "worker-start",
      span(at(frame, "runReceivedAt"), at(frame, "workerCreatedAt")),
    ],
    [
      "worker-load",
      span(at(frame, "workerCreatedAt"), at(worker, "workerScriptStartedAt")),
    ],
    ["compiler-init", compilerInit],
    ["transpile", transpile],
    [
      "evaluate",
      evaluateTotal === undefined
        ? undefined
        : Math.max(0, evaluateTotal - compilerInit - transpile),
    ],
    [
      "environment",
      span(at(worker, "evaluatedAt"), at(worker, "mainStartedAt")),
    ],
    ["main", main],
    ["cleanup", span(at(worker, "mainEndedAt"), at(worker, "cleanedUpAt"))],
  ]
  const phases: RunTimingPhase[] = []
  for (const [id, milliseconds] of measured) {
    if (milliseconds === undefined) continue
    phases.push({ id, label: phaseLabels[id], milliseconds })
  }
  // Run を送ってから完了が届くまでのうち、どの区間にも入らない時間（受け渡し）
  const insideRun = phases
    .filter(({ id }) => id !== "compile" && id !== "frame")
    .reduce((sum, phase) => sum + phase.milliseconds, 0)
  phases.push({
    id: "messaging",
    label: phaseLabels.messaging,
    milliseconds: Math.max(0, afterSend - insideRun),
  })
  return {
    mainMilliseconds: main,
    totalMilliseconds: phases.reduce(
      (sum, phase) => sum + phase.milliseconds,
      0
    ),
    phases,
  }
}

/** チップの数字。1 ms に満たないときは 0 ms と出さず、<1 ms と出す */
export function formatChipMilliseconds(milliseconds: number): string {
  if (milliseconds < 1) return "<1 ms"
  return `${Math.round(milliseconds)} ms`
}

/** 内訳の数字。小さい区間も比べられるよう、10 ms 未満は小数一桁まで出す */
export function formatPhaseMilliseconds(milliseconds: number): string {
  if (milliseconds < 10) return `${milliseconds.toFixed(1)} ms`
  return `${Math.round(milliseconds)} ms`
}
