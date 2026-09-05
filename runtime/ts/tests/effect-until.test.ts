import { expect, test } from "bun:test"
import { arrayIterable } from "../src/array"
import type { Iterable } from "../src/collection"
import {
  acquireRelease,
  Break,
  Continue,
  createEffectExecution,
  type Effect,
  EffectCancellation,
  fail,
  flatMap,
  forEachUntil,
  type LoopControl,
  run,
  scoped,
  succeed,
} from "../src/effect"
import { iteratorIterable, unfold } from "../src/iterator"
import { Cons, Empty, listIterable } from "../src/list"
import { inclusive, rangeIterable } from "../src/range"
import { Just, Nothing } from "../src/sum"

test("Array, List, Range and custom Iterable preserve order and stop at Break", async () => {
  async function check<C>(values: C, dictionary: Iterable<C, number>) {
    const calls: number[] = []
    const work = forEachUntil(
      (value: number) => () => {
        calls.push(value)
        return value === 3 ? Break : Continue
      },
      values,
      dictionary
    )
    expect(calls).toEqual([])
    expect(await run(work, {})).toEqual({ kind: "success", value: undefined })
    expect(calls).toEqual([1, 2, 3])
    await run(work, {})
    expect(calls).toEqual([1, 2, 3, 1, 2, 3])
  }
  await check([1, 2, 3, 4], arrayIterable)
  await check(Cons(1, Cons(2, Cons(3, Cons(4, Empty)))), listIterable)
  await check(inclusive(1, 4), rangeIterable)
  await check(
    { start: 1 },
    {
      iterate: ({ start }) =>
        unfold((n: number) => Just([n, n + 1] as const), start),
    }
  )
})

test("iteration itself is cold and an infinite input is never pulled past Break", async () => {
  const events: string[] = []
  const values = unfold((n: number) => {
    events.push(`pull:${n}`)
    if (n > 3) throw new Error("pulled past Break")
    return Just([n, n + 1] as const)
  }, 1)
  const dictionary = {
    iterate: () => {
      events.push("iterate")
      return values
    },
  }
  const work = forEachUntil(
    (n: number) => {
      events.push(`construct:${n}`)
      return async () => {
        events.push(`start:${n}`)
        await Promise.resolve()
        events.push(`end:${n}`)
        return n === 3 ? Break : Continue
      }
    },
    undefined,
    dictionary
  )
  expect(events).toEqual([])
  await run(work, {})
  expect(events).toEqual([
    "iterate",
    "pull:1",
    "construct:1",
    "start:1",
    "end:1",
    "pull:2",
    "construct:2",
    "start:2",
    "end:2",
    "pull:3",
    "construct:3",
    "start:3",
    "end:3",
  ])
})

test("empty input calls no action and Continue reaches exhaustion", async () => {
  let called = 0
  await run(
    forEachUntil(
      () => {
        called++
        return succeed(Continue)
      },
      unfold(() => Nothing, 0),
      iteratorIterable
    ),
    {}
  )
  expect(called).toBe(0)
  await run(
    forEachUntil(
      () => {
        called++
        return succeed(Continue)
      },
      [1, 2],
      arrayIterable
    ),
    {}
  )
  expect(called).toBe(2)
})

test("typed failure and defect propagate without starting later actions", async () => {
  for (const defect of [false, true]) {
    const calls: number[] = []
    const error = new Error("host defect")
    const action = (n: number): Effect<unknown, string, LoopControl> => {
      calls.push(n)
      if (n === 2)
        return defect
          ? () => {
              throw error
            }
          : fail("typed")
      return succeed(Continue)
    }
    const pending = run(forEachUntil(action, [1, 2, 3], arrayIterable), {})
    if (defect) await expect(pending).rejects.toBe(error)
    else expect(await pending).toEqual({ kind: "failure", error: "typed" })
    expect(calls).toEqual([1, 2])
  }
})

test("cancellation stops pending action and finalizes its acquired resource once", async () => {
  const execution = createEffectExecution()
  const events: string[] = []
  let started!: () => void
  const ready = new Promise<void>((resolve) => {
    started = resolve
  })
  const work = scoped(
    forEachUntil(
      (n: number) =>
        flatMap(
          acquireRelease(
            () => {
              events.push(`acquire:${n}`)
              return n
            },
            (value) => () => {
              events.push(`release:${value}`)
              return undefined
            }
          ),
          () => () => {
            started()
            return new Promise<LoopControl>(() => {})
          }
        ),
      [1, 2],
      arrayIterable
    )
  )
  const result = run(work, {}, execution.context).catch(
    (error: unknown) => error
  )
  await ready
  await execution.cancel()
  expect(await result).toBeInstanceOf(EffectCancellation)
  await execution.close()
  expect(events).toEqual(["acquire:1", "release:1"])
})

test("Break and failure release the containing scope, and pre-cancel avoids iterator access", async () => {
  for (const shouldFail of [false, true]) {
    const events: string[] = []
    const work = scoped(
      forEachUntil(
        (n: number) =>
          flatMap(
            acquireRelease(
              () => n,
              (value) => () => {
                events.push(`release:${value}`)
                return undefined
              }
            ),
            () => (shouldFail ? fail("stop") : succeed(Break))
          ),
        [1, 2],
        arrayIterable
      )
    )
    expect((await run(work, {})).kind).toBe(shouldFail ? "failure" : "success")
    expect(events).toEqual(["release:1"])
  }
  const execution = createEffectExecution()
  await execution.cancel()
  const work = forEachUntil(() => succeed(Continue), undefined, {
    iterate: () => {
      throw new Error("must not iterate")
    },
  })
  await expect(work({}, execution.context)).rejects.toBeInstanceOf(
    EffectCancellation
  )
  await execution.close()
})

test("run reports pre-cancel and synchronous callback cancellation only once", async () => {
  for (const before of [true, false]) {
    const execution = createEffectExecution()
    if (before) await execution.cancel()
    let called = 0
    const work = forEachUntil(
      () => async () => {
        called++
        await execution.cancel()
        return Continue
      },
      [1, 2],
      arrayIterable
    )
    await expect(run(work, {}, execution.context)).rejects.toBeInstanceOf(
      EffectCancellation
    )
    expect(called).toBe(before ? 0 : 1)
    await execution.close()
  }
})

test("pre-cancel starts no host effect and callback rejection is consumed during cancel", async () => {
  const before = createEffectExecution()
  await before.cancel()
  let calls = 0
  await expect(
    run(
      async () => {
        calls++
        return Continue
      },
      {},
      before.context
    )
  ).rejects.toBeInstanceOf(EffectCancellation)
  expect(calls).toBe(0)
  await before.close()

  const during = createEffectExecution()
  const work = forEachUntil(
    () => () => {
      void during.cancel()
      return Promise.reject(
        new Error("late rejection after synchronous cancel")
      )
    },
    [1, 2],
    arrayIterable
  )
  await expect(run(work, {}, during.context)).rejects.toBeInstanceOf(
    EffectCancellation
  )
  await during.close()
})
