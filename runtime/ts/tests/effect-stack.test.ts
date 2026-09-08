import { expect, test } from "bun:test"
import {
  acquireRelease,
  createEffectExecution,
  type Effect,
  EffectCancellation,
  fail,
  flatMap,
  run,
  scoped,
  succeed,
} from "../src/effect"

test("100,000 left-associated binds are cold, repeatable and stack safe", async () => {
  let calls = 0
  let work: Effect<unknown, never, number> = succeed(0)
  for (let index = 0; index < 100_000; index++) {
    work = flatMap(work, (value) => {
      calls++
      return succeed(value + 1)
    })
  }
  expect(calls).toBe(0)
  expect(await run(work, {})).toEqual({ kind: "success", value: 100_000 })
  expect(calls).toBe(100_000)
  expect(await run(work, {})).toEqual({ kind: "success", value: 100_000 })
  expect(calls).toBe(200_000)
})

test("100,000 right-associated continuations use constant stack", async () => {
  const loop = (remaining: number): Effect<unknown, never, number> =>
    flatMap(succeed(remaining), (value) =>
      value === 0 ? succeed(42) : loop(value - 1)
    )
  expect(await run(loop(100_000), {})).toEqual({ kind: "success", value: 42 })
})

test("bind plans preserve failure payloads, defects, cancellation and LIFO cleanup", async () => {
  for (const outcome of ["failure", "defect", "cancel"] as const) {
    const events: string[] = []
    const execution = createEffectExecution()
    const payload = { message: "expected" }
    const resource = (name: string) =>
      acquireRelease(succeed(name), () => () => {
        events.push(`release:${name}`)
      })
    const work = scoped(
      flatMap(resource("outer"), () =>
        flatMap(resource("inner"), () =>
          flatMap(succeed(1), () => {
            events.push("callback")
            if (outcome === "failure") return fail(payload)
            if (outcome === "defect") throw payload
            void execution.cancel()
            return succeed(42)
          })
        )
      )
    )
    const guarded = flatMap(work, () => {
      events.push("unexpected")
      return succeed(0)
    })
    if (outcome === "failure") {
      expect(await run(guarded, {}, execution.context)).toEqual({
        kind: "failure",
        error: payload,
      })
    } else if (outcome === "defect") {
      await expect(run(guarded, {}, execution.context)).rejects.toBe(payload)
    } else {
      await expect(run(guarded, {}, execution.context)).rejects.toBeInstanceOf(
        EffectCancellation
      )
      await execution.cancel()
    }
    await execution.close()
    expect(events).toEqual(["callback", "release:inner", "release:outer"])
  }
})
