import { describe, expect, test } from "bun:test"
import { createEffectExecution, isEffectCancellation, run } from "../src/effect"
import {
  annotateForeignTask,
  createForeignTaskModule,
  invokeForeignPure,
  invokeForeignTask,
  renderJsErrorDiagnostic,
} from "../src/foreign"

describe("foreign TypeScript boundary", () => {
  test("copies readonly collections and Bytes in both directions", () => {
    const hostArray = [1, 2]
    const hostBytes = new Uint8Array([3, 4])
    const namespace = {
      snapshot: (values: number[], bytes: Uint8Array) => {
        values.push(99)
        bytes[0] = 99
        return { values, bytes }
      },
    }
    const sourceArray = [1, 2]
    const sourceBytes = new Uint8Array([3, 4])
    const result = invokeForeignPure(
      namespace,
      "snapshot",
      "function",
      [sourceArray, sourceBytes],
      [{ array: "int" }, "bytes"],
      { record: { values: { array: "int" }, bytes: "bytes" } }
    ) as { values: number[]; bytes: Uint8Array }

    expect(sourceArray).toEqual(hostArray)
    expect(sourceBytes).toEqual(hostBytes)
    expect(result.values).toEqual([1, 2, 99])
    expect(result.bytes).toEqual(new Uint8Array([99, 4]))
    namespace.snapshot([], new Uint8Array())
    expect(result.values).toEqual([1, 2, 99])
  })

  test("adapts a curried synchronous callback to one host call", () => {
    const result = invokeForeignPure(
      {
        call: (callback: (left: string, right: string) => string) =>
          callback("a", "b"),
      },
      "call",
      "function",
      [(left: string) => (right: string) => `${left}:${right}`],
      [
        {
          callback: {
            parameters: ["string", "string"],
            result: "string",
          },
        },
      ],
      "string"
    )
    expect(result).toBe("a:b")
  })

  test("constructs opaque handles and invokes methods and properties", () => {
    class Counter {
      constructor(readonly value: number) {}
      add(delta: number): number {
        return this.value + delta
      }
    }
    const counter = invokeForeignPure(
      { Counter },
      "Counter",
      "constructor",
      [40],
      ["int"],
      "opaque"
    )
    expect(
      invokeForeignPure(
        {},
        "add",
        "method",
        [counter, 2],
        ["opaque", "int"],
        "int"
      )
    ).toBe(42)
    expect(
      invokeForeignPure({}, "value", "property", [counter], ["opaque"], "int")
    ).toBe(40)
  })

  test("memoizes task module success and failure and preserves phases", async () => {
    let loads = 0
    const module = createForeignTaskModule(async () => {
      loads += 1
      return {
        sync: () => 1,
        promise: async () => 2,
        throws: () => {
          throw new Error("sync boom")
        },
        rejects: async () => {
          throw new Error("async boom")
        },
      }
    })
    const call = (member: string) =>
      run(invokeForeignTask(module, member, "function", [], [], "int"), {})

    expect(await Promise.all([call("sync"), call("promise")])).toEqual([
      { kind: "success", value: 1 },
      { kind: "success", value: 2 },
    ])
    expect(loads).toBe(1)
    expect(await call("missing")).toMatchObject({
      kind: "failure",
      error: { phase: "BindingLookup" },
    })
    expect(
      await run(
        invokeForeignTask(module, "sync", "function", [1.5], ["int"], "int"),
        {}
      )
    ).toMatchObject({
      kind: "failure",
      error: { phase: "SynchronousThrow" },
    })
    expect(await call("throws")).toMatchObject({
      kind: "failure",
      error: { phase: "SynchronousThrow", message: "sync boom" },
    })
    const rejection = await call("rejects")
    expect(rejection).toMatchObject({
      kind: "failure",
      error: {
        phase: "PromiseRejection",
        message: "async boom",
        hostStack: expect.stringContaining("rejects"),
        observedStack: expect.stringContaining("foreign task rejects observed"),
        adapterFrame: { language: "interop", generated: true },
      },
    })

    let failedLoads = 0
    const failed = createForeignTaskModule(async () => {
      failedLoads += 1
      throw new Error("load boom")
    })
    for (let attempt = 0; attempt < 2; attempt += 1) {
      expect(
        await run(
          invokeForeignTask(failed, "run", "function", [], [], "unit"),
          {}
        )
      ).toMatchObject({
        kind: "failure",
        error: { phase: "ModuleLoad", message: "load boom" },
      })
    }
    expect(failedLoads).toBe(1)
  })

  test("shares one task load across aliases of the exact host identity", async () => {
    let loads = 0
    const load = async () => {
      loads += 1
      return { value: () => 42 }
    }
    const first = createForeignTaskModule(load, "file:///host/value.mjs")
    const alias = createForeignTaskModule(load, "file:///host/value.mjs")

    const results = await Promise.all(
      [first, alias].map((module) =>
        run(invokeForeignTask(module, "value", "function", [], [], "int"), {})
      )
    )

    expect(results).toEqual([
      { kind: "success", value: 42 },
      { kind: "success", value: 42 },
    ])
    expect(loads).toBe(1)
  })

  test("cancels a caller waiting on a host Promise without retrying the load", async () => {
    let resolveHost: ((value: number) => void) | undefined
    const module = createForeignTaskModule(async () => ({
      wait: () =>
        new Promise<number>((resolve) => {
          resolveHost = resolve
        }),
    }))
    const execution = createEffectExecution()
    const pending = run(
      invokeForeignTask(module, "wait", "function", [], [], "int"),
      {},
      execution.context
    )
    const observed = pending.then(
      () => undefined,
      (error: unknown) => error
    )
    while (resolveHost === undefined) await Promise.resolve()
    await execution.cancel()
    expect(isEffectCancellation(await observed)).toBe(true)
    resolveHost(1)
    await execution.close()
  })

  test("validates explicit JS boundary types without weakening unsupported values", () => {
    const object = { value: 1 }
    const mutable = [1, 2]
    expect(
      invokeForeignPure(
        { same: (value: unknown) => value },
        "same",
        "function",
        [object],
        ["js-object"],
        "js-object"
      )
    ).toBe(object)
    expect(
      invokeForeignPure(
        { same: (value: unknown) => value },
        "same",
        "function",
        [mutable],
        [{ mutableArray: "int" }],
        { mutableArray: "int" }
      )
    ).toBe(mutable)
    expect(
      invokeForeignPure(
        { same: (value: unknown) => value },
        "same",
        "function",
        [null],
        [{ nullable: "string" }],
        { nullable: "string" }
      )
    ).toBeNull()
    expect(() =>
      invokeForeignPure(
        { same: (value: unknown) => value },
        "same",
        "function",
        [{}],
        ["unsupported"],
        "unsupported"
      )
    ).toThrow("unsupported foreign boundary type")
  })

  test("keeps explicit raw callbacks stable across retention and reentrancy", () => {
    let retained: ((value: string) => string) | undefined
    const namespace = {
      retain(callback: (value: string) => string) {
        retained = callback
      },
      invoke(value: string) {
        return retained?.(value)
      },
    }
    const callback = (value: string): string =>
      value === "outer"
        ? `outer:${invokeForeignPure(
            namespace,
            "invoke",
            "function",
            ["inner"],
            ["string"],
            "string"
          )}`
        : value

    invokeForeignPure(
      namespace,
      "retain",
      "function",
      [callback],
      [{ rawCallback: true }],
      "unit"
    )
    expect(retained).toBe(callback)
    expect(
      invokeForeignPure(
        namespace,
        "invoke",
        "function",
        ["outer"],
        ["string"],
        "string"
      )
    ).toBe("outer:inner")
  })

  test("preserves optional record presence and rejects lone surrogate chars", () => {
    const codec = { record: { name: "string", note: { optional: "string" } } } as const
    const missing = invokeForeignPure(
      { same: (value: unknown) => value },
      "same",
      "function",
      [{ name: "Mio" }],
      [codec],
      codec
    ) as Record<string, unknown>
    const present = invokeForeignPure(
      { same: (value: unknown) => value },
      "same",
      "function",
      [{ name: "Mio", note: "ready" }],
      [codec],
      codec
    ) as Record<string, unknown>

    expect(Object.hasOwn(missing, "note")).toBe(false)
    expect(present.note).toBe("ready")
    expect(() =>
      invokeForeignPure(
        { same: (value: unknown) => value },
        "same",
        "function",
        ["\ud800"],
        ["char"],
        "char"
      )
    ).toThrow("one Unicode scalar")
    expect(
      invokeForeignPure(
        { same: (value: unknown) => value },
        "same",
        "function",
        ["𠮷"],
        ["char"],
        "char"
      )
    ).toBe("𠮷")
  })

  test("renders causal JSON frames without losing source byte ranges", async () => {
    const effect = annotateForeignTask(
      invokeForeignTask(
        createForeignTaskModule(async () => ({
          failNow: async () => {
            throw new Error("boom")
          },
        })),
        "failNow",
        "function",
        [],
        [],
        "unit"
      ),
      {
        language: "seseragi",
        function: "main",
        uri: "seseragi://fixture/source-map-rejection/main",
        range: { start: 134, end: 144 },
        generated: false,
      }
    )
    const exit = await run(effect, {})
    expect(exit.kind).toBe("failure")
    if (exit.kind !== "failure") return
    const diagnostic = renderJsErrorDiagnostic(exit.error, () => undefined)
    expect(JSON.parse(diagnostic ?? "null")).toMatchObject({
      schema: 1,
      kind: "TypedFailure",
      phase: "PromiseRejection",
      message: "boom",
      groups: [
        { role: "Thrown", frames: [] },
        {
          role: "Observed",
          frames: [
            {
              language: "seseragi",
              function: "main",
              range: { start: 134, end: 144 },
            },
            { language: "interop", function: "failNow", generated: true },
          ],
        },
      ],
    })
  })
})
