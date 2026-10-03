import { expect, test } from "bun:test"
import { flatMap, run, succeed } from "../src/effect"
import {
  combine,
  make,
  planSet,
  read,
  subscribe,
  transaction,
  unsubscribe,
} from "../src/signal"

async function sources() {
  const left = await make(0)({})
  const right = await make(0)({})
  const snapshot = combine(
    (a: number) => (b: number) => ({ a, b }),
    left,
    right
  )
  const update = (value: number) =>
    transaction([planSet(value, left), planSet(value, right)])
  return { left, right, snapshot, update }
}

test("separate Effect reads can observe different committed publications", async () => {
  const { left, right, update } = await sources()
  const separate = flatMap(read(left), (a) =>
    flatMap(read(right), (b) => succeed({ a, b }))
  )
  const pending = run(separate, {})
  // The first read has run, but the Effect bind resumes in a microtask.
  // Commit both sources together before the second read starts.
  await update(1)({})
  expect(await pending).toEqual({
    kind: "success",
    value: { a: 0, b: 1 },
  })
})

test("one combined read returns one committed snapshot and retains its value", async () => {
  const { snapshot, update } = await sources()
  const pending = run(read(snapshot), {})
  await update(1)({})
  expect(await pending).toEqual({
    kind: "success",
    value: { a: 0, b: 0 },
  })
  expect(await run(read(snapshot), {})).toEqual({
    kind: "success",
    value: { a: 1, b: 1 },
  })
})

test("constructing a derived snapshot neither samples nor subscribes", async () => {
  const { left, right, update } = await sources()
  let evaluations = 0
  const snapshot = combine(
    (a: number) => (b: number) => {
      evaluations++
      return { a, b }
    },
    left,
    right
  )
  const sample = read(snapshot)
  expect(evaluations).toBe(0)
  await update(1)({})
  expect(evaluations).toBe(0)
  expect(await sample({})).toEqual({ a: 1, b: 1 })
  expect(evaluations).toBe(1)
  expect(await sample({})).toEqual({ a: 1, b: 1 })
  expect(evaluations).toBe(1)
  await update(2)({})
  expect(evaluations).toBe(1)
  expect(await sample({})).toEqual({ a: 2, b: 2 })
  expect(evaluations).toBe(2)
})

test("nested publication exposes whole snapshots in publication order", async () => {
  const { snapshot, update } = await sources()
  const seen: { a: number; b: number }[] = []
  const subscription = await subscribe((value: { a: number; b: number }) => {
    return async () => {
      seen.push(value)
      if (value.a === 1) await update(2)({})
    }
  }, snapshot)({})
  try {
    await update(1)({})
    expect(seen).toEqual([
      { a: 0, b: 0 },
      { a: 1, b: 1 },
      { a: 2, b: 2 },
    ])
  } finally {
    await unsubscribe(subscription)({})
  }
})
