import { describe, expect, test } from "bun:test"
import {
  formatChipMilliseconds,
  formatPhaseMilliseconds,
  summarizeRunTiming,
} from "../src/runtime/run-timing"

// シェル、フレーム、Worker が記録する時刻の例。FizzBuzz を測ったときの形に近い
const report = {
  frameStartedAt: 1_000,
  runSentAt: 1_040,
  completedAt: 1_330,
  frame: { runReceivedAt: 1_040.2, workerCreatedAt: 1_047.2 },
  worker: {
    workerScriptStartedAt: 1_175.2,
    evaluateStartedAt: 1_180,
    compilerInitMilliseconds: 96,
    transpileMilliseconds: 50,
    evaluatedAt: 1_326.4,
    mainStartedAt: 1_326.8,
    mainEndedAt: 1_327.5,
    cleanedUpAt: 1_327.7,
  },
}

describe("run timing", () => {
  test("puts main in its own phase and the rest in named phases", () => {
    const timing = summarizeRunTiming(report, 12)!
    expect(timing.mainMilliseconds).toBeCloseTo(0.7)
    const phases = Object.fromEntries(
      timing.phases.map((phase) => [phase.id, phase.milliseconds])
    )
    expect(Object.keys(phases)).toEqual([
      "compile",
      "frame",
      "worker-start",
      "worker-load",
      "compiler-init",
      "transpile",
      "evaluate",
      "environment",
      "main",
      "cleanup",
      "messaging",
    ])
    expect(phases.compile).toBe(12)
    expect(phases.frame).toBe(40)
    expect(phases["worker-start"]).toBeCloseTo(7)
    expect(phases["worker-load"]).toBeCloseTo(128)
    expect(phases["compiler-init"]).toBe(96)
    expect(phases.transpile).toBe(50)
    // 評価の区間から、コンパイラの初期化と変換を除いた残り
    expect(phases.evaluate).toBeCloseTo(0.4)
    expect(phases.cleanup).toBeCloseTo(0.2)
  })

  test("adds up to the whole wait, with hand-offs in Messaging", () => {
    const timing = summarizeRunTiming(report, 12)!
    const sum = timing.phases.reduce(
      (total, phase) => total + phase.milliseconds,
      0
    )
    expect(timing.totalMilliseconds).toBeCloseTo(sum)
    // Compile ＋ Run を送る前 ＋ 送ってから完了が届くまで
    expect(timing.totalMilliseconds).toBeCloseTo(12 + 40 + 290)
    expect(timing.phases.map((phase) => phase.label)).toEqual([
      "Compile",
      "Sandbox start",
      "Worker start",
      "Worker load",
      "TypeScript init",
      "Transpile",
      "Evaluate",
      "Environment",
      "main",
      "Cleanup",
      "Messaging",
    ])
  })

  test("leaves out Worker phases for programs that run in the frame", () => {
    const timing = summarizeRunTiming({ ...report, frame: undefined }, 12)!
    const ids = timing.phases.map((phase) => phase.id)
    expect(ids).not.toContain("worker-start")
    expect(ids).not.toContain("worker-load")
    expect(timing.mainMilliseconds).toBeCloseTo(0.7)
  })

  test("does not trust values the program could have changed", () => {
    // main の時刻がなければ、数字を出さない
    expect(
      summarizeRunTiming({
        ...report,
        worker: { ...report.worker, mainEndedAt: "soon" },
      })
    ).toBeUndefined()
    expect(summarizeRunTiming({ ...report, worker: null })).toBeUndefined()
    // 逆向きの区間は 0、ありえない長さの区間は捨てる
    const timing = summarizeRunTiming({
      ...report,
      frame: { runReceivedAt: 1_050, workerCreatedAt: 1_047 },
      worker: {
        ...report.worker,
        compilerInitMilliseconds: -5,
        transpileMilliseconds: Number.POSITIVE_INFINITY,
        cleanedUpAt: 1e12,
      },
    })!
    const phases = Object.fromEntries(
      timing.phases.map((phase) => [phase.id, phase.milliseconds])
    )
    expect(phases["worker-start"]).toBe(0)
    expect(phases["compiler-init"]).toBe(0)
    expect(phases.transpile).toBe(0)
    expect(phases.cleanup).toBeUndefined()
    for (const phase of timing.phases)
      expect(Number.isFinite(phase.milliseconds)).toBe(true)
  })

  test("formats the chip and the breakdown", () => {
    expect(formatChipMilliseconds(0)).toBe("<1 ms")
    expect(formatChipMilliseconds(0.98)).toBe("<1 ms")
    expect(formatChipMilliseconds(1)).toBe("1 ms")
    expect(formatChipMilliseconds(17.6)).toBe("18 ms")
    expect(formatPhaseMilliseconds(0.04)).toBe("0.0 ms")
    expect(formatPhaseMilliseconds(7.25)).toBe("7.3 ms")
    expect(formatPhaseMilliseconds(128.4)).toBe("128 ms")
  })
})
