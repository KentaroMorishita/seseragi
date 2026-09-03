import { expect, test } from "bun:test"
import { Done, type Iterable, Next, reduceUntil } from "../src/collection"
import { type Iterator, unfold } from "../src/iterator"
import { Just, Nothing } from "../src/sum"

const identity: Iterable<Iterator<number>, number> = {
  iterate: (value) => value,
}

test("Done stops both callback invocation and iterator pulls on infinite input", () => {
  const events: string[] = []
  const values = unfold((n: number) => {
    events.push(`pull:${n}`)
    if (n > 3) throw new Error("pulled beyond Done")
    return Just([n, n + 1] as const)
  }, 1)
  const step = (acc: string) => (n: number) => {
    events.push(`step:${n}`)
    return n === 3 ? Done(`${acc}!`) : Next(`${acc}${n}`)
  }
  expect(reduceUntil(identity, "", step, values)).toBe("12!")
  expect(events).toEqual([
    "pull:1",
    "step:1",
    "pull:2",
    "step:2",
    "pull:3",
    "step:3",
  ])
  // The original persistent iterator remains usable.
  expect(reduceUntil(identity, "", step, values)).toBe("12!")
})

test("empty input returns the initial value without a callback", () => {
  const initial = Object.freeze({ untouched: true })
  let pulls = 0
  const values: Iterator<number> = {
    next: () => {
      pulls++
      return Nothing
    },
  }
  expect(
    reduceUntil(
      identity,
      initial,
      () => () => {
        throw new Error("callback")
      },
      values
    )
  ).toBe(initial)
  expect(pulls).toBe(1)
})

test("Next preserves source order and finite exhaustion returns the last accumulator", () => {
  const values = unfold(
    (n: number) => (n <= 4 ? Just([n, n + 1] as const) : Nothing),
    1
  )
  expect(
    reduceUntil(identity, "", (acc) => (n) => Next(`${acc}${n}`), values)
  ).toBe("1234")
})

test("first-element Done returns its payload and never evaluates the rest", () => {
  const values: Iterator<number> = {
    next: () =>
      Just([
        1,
        {
          next: () => {
            throw new Error("rest")
          },
        },
      ] as const),
  }
  const result = { selected: true }
  expect(reduceUntil(identity, {}, () => () => Done(result), values)).toBe(
    result
  )
})

test("constructors are immutable and callable accumulators are values", () => {
  expect(Object.isFrozen(Next(1))).toBe(true)
  expect(Object.isFrozen(Done(1))).toBe(true)
  const values = unfold((n: number) => Just([n, n + 1] as const), 1)
  const result = reduceUntil(
    identity,
    (n: number) => n,
    (acc) => (n) =>
      n === 3 ? Done(acc) : Next((value: number) => acc(value + n)),
    values
  )
  expect(result(10)).toBe(13)
})
