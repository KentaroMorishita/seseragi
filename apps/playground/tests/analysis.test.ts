import { describe, expect, test } from "bun:test"
import { createLiveAnalysis } from "../src/analysis/live-analysis"
import type { AnalysisDocument } from "../src/compiler/types"

function document(source: string): AnalysisDocument {
  return {
    schema: 1,
    source,
    module: "playground/main",
    diagnostics: { diagnostics: [] },
    symbols: [],
    symbolOccurrences: [],
    typeOccurrences: [],
    callableOccurrences: [],
    standardLibrary: [],
  }
}

describe("live analysis scheduling", () => {
  test("debounces rapid edits and analyzes only the newest source", async () => {
    const analyzed: string[] = []
    const applied: string[] = []
    const controller = createLiveAnalysis({
      delayMs: 10,
      analyze: async (source) => {
        analyzed.push(source)
        return document(source)
      },
      apply: (_analysis, source) => applied.push(source),
    })

    controller.schedule("first")
    controller.schedule("second")
    controller.schedule("latest")
    await Bun.sleep(25)

    expect(analyzed).toEqual(["latest"])
    expect(applied).toEqual(["latest"])
  })

  test("drops an older asynchronous result after a newer revision wins", async () => {
    const resolvers = new Map<string, (analysis: AnalysisDocument) => void>()
    const applied: string[] = []
    const controller = createLiveAnalysis({
      delayMs: 0,
      analyze: (source) =>
        new Promise((resolve) => {
          resolvers.set(source, resolve)
        }),
      apply: (_analysis, source) => applied.push(source),
    })

    controller.schedule("old")
    await Bun.sleep(5)
    controller.schedule("new")
    await Bun.sleep(5)
    resolvers.get("new")?.(document("new"))
    await Bun.sleep(1)
    resolvers.get("old")?.(document("old"))
    await Bun.sleep(1)

    expect(applied).toEqual(["new"])
  })

  test("drops an in-flight result after cancellation", async () => {
    let resolveAnalysis: ((analysis: AnalysisDocument) => void) | undefined
    const applied: string[] = []
    const controller = createLiveAnalysis({
      delayMs: 0,
      analyze: () =>
        new Promise((resolve) => {
          resolveAnalysis = resolve
        }),
      apply: (_analysis, source) => applied.push(source),
    })

    controller.schedule("same revision")
    await Bun.sleep(5)
    controller.cancel()
    resolveAnalysis?.(document("same revision"))
    await Bun.sleep(1)

    expect(applied).toEqual([])
  })

  test("reports adapter failures instead of leaving analysis pending", async () => {
    const errors: unknown[] = []
    const controller = createLiveAnalysis({
      delayMs: 0,
      analyze: async () => document("source"),
      apply: () => {
        throw new Error("broken adapter")
      },
      onError: (error) => errors.push(error),
    })

    controller.schedule("source")
    await Bun.sleep(5)

    expect(errors).toHaveLength(1)
    expect(errors[0]).toBeInstanceOf(Error)
    expect((errors[0] as Error).message).toBe("broken adapter")
  })

  test("keeps identical source requests separated by file identity", async () => {
    const resolvers: ((analysis: AnalysisDocument) => void)[] = []
    const applied: (string | undefined)[] = []
    const controller = createLiveAnalysis({
      delayMs: 0,
      analyze: () =>
        new Promise((resolve) => {
          resolvers.push(resolve)
        }),
      apply: (_analysis, _source, identity) => applied.push(identity),
    })

    controller.schedule("same source", "one.ssrg")
    await Bun.sleep(5)
    controller.schedule("same source", "two.ssrg")
    await Bun.sleep(5)
    resolvers[1]?.(document("same source"))
    await Bun.sleep(1)
    resolvers[0]?.(document("same source"))
    await Bun.sleep(1)

    expect(applied).toEqual(["two.ssrg"])
  })
})
