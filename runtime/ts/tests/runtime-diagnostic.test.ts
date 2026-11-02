import { expect, test } from "bun:test"
import { defectWithSuppressed } from "../src/cleanup-defect"
import {
  acquireRelease,
  atSource,
  createEffectExecution,
  defer,
  die,
  type Effect,
  EffectCancellation,
  fail,
  flatMap,
  run,
  scoped,
  succeed,
} from "../src/effect"
import {
  renderRuntimeDefectDiagnostic,
  renderRuntimeDefectText,
} from "../src/foreign"
import { add, divide, multiply, power, remainder, subtract } from "../src/int"
import type { RuntimeSource } from "../src/runtime-diagnostic"

const inner: RuntimeSource = {
  file: "例-🙂.ssrg",
  line: 3,
  column: 25,
  uri: "seseragi://fixture/inner",
  start: 42,
  end: 47,
}
const outer: RuntimeSource = { ...inner, line: 9, column: 7 }

test("integer arithmetic attaches its own source without a surrounding call", () => {
  for (const [operation, left, right, message] of [
    [add, Number.MAX_SAFE_INTEGER, 1, "overflow"],
    [subtract, Number.MIN_SAFE_INTEGER, 1, "overflow"],
    [multiply, Number.MAX_SAFE_INTEGER, 2, "overflow"],
    [divide, 1, 0, "division by zero"],
    [remainder, 1, 0, "remainder by zero"],
    [power, 1, -1, "negative exponent"],
  ] as const) {
    let observed: unknown
    try {
      operation(left, right, inner)
    } catch (error) {
      observed = error
    }
    expect(renderRuntimeDefectText(observed)).toBe(
      `例-🙂.ssrg:3:25: Seseragi Defect: Seseragi Int ${message}\n`
    )
  }
})

test("suppressed cleanup defects retain their individual source positions", () => {
  const located = (source: RuntimeSource, message: string): unknown => {
    try {
      atSource(
        source,
        () => {
          throw Object.freeze(new Error(message))
        },
        false
      )
    } catch (error) {
      return error
    }
    throw new Error("expected a defect")
  }
  const first = located(inner, "body")
  const second = located(outer, "finalizer")
  expect(renderRuntimeDefectText(defectWithSuppressed([first, second]))).toBe(
    "例-🙂.ssrg:3:25: Seseragi Defect: body\n" +
      "例-🙂.ssrg:9:7: Seseragi suppressed Defect: finalizer\n"
  )
})

test("reports the innermost source without changing a thrown error's identity", () => {
  const error = Object.freeze(new RangeError("division by zero"))
  let observed: unknown
  try {
    atSource(
      outer,
      () =>
        atSource(
          inner,
          () => {
            throw error
          },
          false
        ),
      false
    )
  } catch (failure) {
    observed = failure
  }
  expect(observed).toBe(error)
  expect(renderRuntimeDefectText(observed)).toBe(
    "例-🙂.ssrg:3:25: Seseragi Defect: division by zero\n"
  )
  expect(
    JSON.parse(renderRuntimeDefectDiagnostic(observed, () => undefined))
  ).toEqual({
    schema: 1,
    kind: "Defect",
    phase: null,
    message: "division by zero",
    groups: [
      {
        role: "Thrown",
        frames: [
          {
            language: "seseragi",
            function: "<expression>",
            uri: "seseragi://fixture/inner",
            range: { start: 42, end: 47 },
            generated: false,
          },
        ],
      },
    ],
  })
})

test("keeps primitive defects readable and handles missing source information", () => {
  for (const value of ["message", undefined, null, 0]) {
    try {
      atSource(
        inner,
        () => {
          throw value
        },
        false
      )
      throw new Error("expected a primitive defect")
    } catch (error) {
      expect(renderRuntimeDefectText(error)).toBe(
        `例-🙂.ssrg:3:25: Seseragi Defect: ${String(value)}\n`
      )
    }
  }
  expect(renderRuntimeDefectText(new Error("host defect"))).toBe(
    "Seseragi Defect: host defect\n"
  )
})

test("located Effects stay cold and retain their source on every execution", async () => {
  let calls = 0
  const work = atSource(
    outer,
    () =>
      defer(() => {
        calls++
        return atSource(inner, () => die("cold defect"), true)
      }),
    true
  )
  expect(calls).toBe(0)
  for (let index = 0; index < 2; index++) {
    let defect: unknown
    try {
      await run(work, {})
    } catch (error) {
      defect = error
    }
    expect(renderRuntimeDefectText(defect)).toBe(
      "例-🙂.ssrg:3:25: Seseragi Defect: cold defect\n"
    )
  }
  expect(calls).toBe(2)
})

test("located bind plans remain stack safe across 100,000 stages", async () => {
  let work: Effect<unknown, never, number> = succeed(0)
  for (let index = 0; index < 100_000; index++) {
    const previous = work
    work = atSource(
      inner,
      () => flatMap(previous, (value) => succeed(value + 1)),
      true
    )
  }
  expect(await run(work, {})).toEqual({ kind: "success", value: 100_000 })
})

test("a continuation's defect uses its own position after a located bind source", async () => {
  const work = atSource(
    outer,
    () =>
      flatMap(
        atSource(inner, () => succeed(1), true),
        () => () => {
          throw new Error("continuation defect")
        }
      ),
    true
  )
  try {
    await run(work, {})
    throw new Error("expected a defect")
  } catch (error) {
    expect(renderRuntimeDefectText(error)).toBe(
      "例-🙂.ssrg:9:7: Seseragi Defect: continuation defect\n"
    )
  }
})

test("source tracking preserves typed failure, cancellation and LIFO finalizers", async () => {
  for (const outcome of ["failure", "defect", "cancel"] as const) {
    const execution = createEffectExecution()
    const events: string[] = []
    const payload = { message: "expected" }
    const resource = (name: string) =>
      atSource(
        inner,
        () =>
          acquireRelease(succeed(name), () => () => {
            events.push(name)
          }),
        true
      )
    const work = atSource(
      outer,
      () =>
        scoped(
          flatMap(resource("outer"), () =>
            flatMap(resource("inner"), () => {
              if (outcome === "failure") return fail(payload)
              if (outcome === "defect")
                return atSource(inner, () => die("defect"), true)
              void execution.cancel()
              return succeed(42)
            })
          )
        ),
      true
    )
    try {
      if (outcome === "failure") {
        expect(await run(work, {}, execution.context)).toEqual({
          kind: "failure",
          error: payload,
        })
      } else if (outcome === "defect") {
        await expect(run(work, {}, execution.context)).rejects.toThrow("defect")
      } else {
        await expect(run(work, {}, execution.context)).rejects.toBeInstanceOf(
          EffectCancellation
        )
      }
    } finally {
      await execution.close()
    }
    expect(events).toEqual(["inner", "outer"])
  }
})

test("Decimal host magnitude limits have a stable located diagnostic", async () => {
  const decimal = await import("../src/decimal")
  const number = await import("../src/number")
  const huge = decimal.parse("1e100000000000000000000000")
  expect(huge.tag).toBe("Right")
  if (huge.tag !== "Right") throw new Error("valid decimal")
  const precision = decimal.context(Number.MAX_SAFE_INTEGER, number.HalfEven)
  expect(precision.tag).toBe("Right")
  if (precision.tag !== "Right") throw new Error("valid precision")
  for (const action of [
    () => decimal.add(huge.value, decimal.fromInt(2)),
    () =>
      decimal.divide(precision.value, decimal.fromInt(1), decimal.fromInt(7)),
  ]) {
    expect(() => atSource(inner, action, false)).toThrow(
      "Decimal canonical spelling exceeds host limits"
    )
    try {
      atSource(inner, action, false)
    } catch (error) {
      expect(renderRuntimeDefectText(error)).toBe(
        "例-🙂.ssrg:3:25: Seseragi Defect: Decimal canonical spelling exceeds host limits\n"
      )
    }
  }
})
