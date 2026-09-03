import { expect, test } from "bun:test"
import { arrayIterable } from "../src/array"
import { intEq } from "../src/equality"
import { intHash } from "../src/hash"
import { fromArray } from "../src/list"
import * as sets from "../src/set"
import { Nothing } from "../src/sum"

const from = (values: ReadonlyArray<number>) =>
  sets.fromIterable(arrayIterable, intEq, intHash, values)

test("Set deduplicates in first occurrence order and reinsert moves to the end", () => {
  const values = from([3, 1, 3, 2])
  expect(sets.toArray(values)).toEqual([3, 1, 2])
  expect(sets.toArray(sets.insert(intEq, intHash, 1, values))).toEqual([
    3, 1, 2,
  ])
  const removed = sets.remove(intEq, intHash, 1, values)
  expect(sets.toArray(sets.insert(intEq, intHash, 1, removed))).toEqual([
    3, 2, 1,
  ])
  expect(sets.toArray(values)).toEqual([3, 1, 2])
  expect(sets.size(values)).toBe(3)
  expect(sets.isEmpty(sets.empty())).toBe(true)
  expect(sets.toList(values)).toEqual(fromArray([3, 1, 2]))
  expect(sets.contains(intEq, intHash, 1, removed)).toBe(false)
})

test("union, intersection and difference follow data-last order", () => {
  const left = from([3, 1, 2])
  const right = from([2, 4, 1, 5])
  expect(sets.toArray(sets.union(intEq, intHash, right, left))).toEqual([
    3, 1, 2, 4, 5,
  ])
  expect(sets.toArray(sets.intersection(intEq, intHash, right, left))).toEqual([
    1, 2,
  ])
  expect(sets.toArray(sets.difference(intEq, intHash, right, left))).toEqual([
    3,
  ])
  expect(sets.isSubsetOf(intEq, intHash, left, from([1, 2]))).toBe(true)
  expect(sets.isSubsetOf(intEq, intHash, left, right)).toBe(false)
  expect(sets.isSubsetOf(intEq, intHash, sets.empty(), sets.empty())).toBe(true)
  expect(sets.setEq(intEq, intHash).eq(left)(from([2, 1, 3]))).toBe(true)
})

test("Set.map is explicit and collapses outputs while evaluating each input once", () => {
  const calls: number[] = []
  const mapped = sets.map(
    intEq,
    intHash,
    (value: number) => {
      calls.push(value)
      return value % 2
    },
    from([3, 1, 4, 2])
  )
  expect(calls).toEqual([3, 1, 4, 2])
  expect(sets.toArray(mapped)).toEqual([1, 0])
  expect(Object.hasOwn(sets, "setFunctor")).toBe(false)
  calls.length = 0
  expect(
    sets.toArray(
      sets.filter(
        (value: number) => {
          calls.push(value)
          return value > 2
        },
        from([4, 1, 3])
      )
    )
  ).toEqual([4, 3])
  expect(calls).toEqual([4, 1, 3])
})

test("Set Iterable/Reducible use insertion order and do not mutate the source", () => {
  const values = from([3, 1, 2])
  expect(
    sets.setReducible.reduce(0)(
      (total: number) => (value: number) => total * 10 + value
    )(values)
  ).toBe(312)
  let iterator = sets.setIterable.iterate(values)
  for (const expected of [3, 1, 2]) {
    const step = iterator.next()
    expect(step.tag).toBe("Just")
    if (step.tag === "Nothing") throw new Error("missing entry")
    expect(step.value[0]).toBe(expected)
    iterator = step.value[1]
  }
  expect(iterator.next()).toBe(Nothing)
  expect(sets.toArray(values)).toEqual([3, 1, 2])
})

test("full hash collision buckets still use Eq and retain first representatives", () => {
  const collision = { hash: (_value: number) => 0 }
  const values = sets.fromIterable(
    arrayIterable,
    intEq,
    collision,
    [2, 1, 2, 3]
  )
  expect(sets.toArray(values)).toEqual([2, 1, 3])
  expect(sets.toArray(sets.remove(intEq, collision, 1, values))).toEqual([2, 3])
  expect(sets.contains(intEq, collision, 4, values)).toBe(false)
})
