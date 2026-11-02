import { describe, expect, test } from "bun:test"
import { intEq, intOrd, nonEmptyListEq } from "../src/equality"
import { intHash, nonEmptyListHash } from "../src/hash"
import {
  fromArray,
  NonEmptyList,
  nonEmptyListApplicative,
  nonEmptyListFunctor,
  nonEmptyListIterable,
  nonEmptyListMonad,
  nonEmptyListOrd,
  nonEmptyListReducible,
  nonEmptyListSemigroup,
  toArray,
  toListNonEmpty,
} from "../src/list"
import {
  intDebug,
  intShow,
  nonEmptyListDebug,
  nonEmptyListShow,
} from "../src/show"
import { Equal, Greater, Less } from "../src/sum"

const values = (head: number, ...tail: number[]) => ({
  tag: "NonEmpty" as const,
  head,
  tail: fromArray(tail),
})

const toArrayNonEmpty = <Value>(value: NonEmptyList<Value>) =>
  toArray(toListNonEmpty(value))

describe("NonEmptyList standard dictionaries", () => {
  test("compose Eq, Ord, Hash, Show, and Debug element evidence", () => {
    const eq = nonEmptyListEq(intEq)
    expect(eq.eq(values(1, 2))(values(1, 2))).toBe(true)
    expect(eq.eq(values(1, 2))(values(1, 3))).toBe(false)

    const ord = nonEmptyListOrd(intOrd)
    expect(ord.compare(values(1, 2))(values(1, 3))).toBe(Less)
    expect(ord.compare(values(1, 2))(values(1, 2, 0))).toBe(Less)
    expect(ord.compare(values(2))(values(1, 9))).toBe(Greater)

    const hash = nonEmptyListHash(intHash)
    expect(hash.hash(values(1, 2))).toBe(hash.hash(values(1, 2)))
    expect(hash.hash(values(1, 2))).not.toBe(hash.hash(values(2, 1)))
    expect(nonEmptyListShow(intShow).show(values(1, 2))).toBe("`[1, 2]")
    expect(nonEmptyListDebug(intDebug).debug(values(1, 2))).toBe("`[1, 2]")
  })

  test("Ord includes structural Eq for singletons, heads, tails, and lengths", () => {
    const ord = nonEmptyListOrd(intOrd)
    expect(Object.isFrozen(ord)).toBe(true)
    expect(ord.eq(values(1))(values(1))).toBe(true)
    expect(ord.eq(values(1))(values(2))).toBe(false)
    expect(ord.eq(values(1, 2))(values(1, 2))).toBe(true)
    expect(ord.eq(values(1, 2))(values(1, 3))).toBe(false)
    expect(ord.eq(values(1, 2))(values(2, 2))).toBe(false)
    expect(ord.eq(values(1))(values(1, 2))).toBe(false)
    expect(ord.eq(values(1, 2))(values(1))).toBe(false)
    expect(ord.compare(values(1, 2))(values(1, 2))).toBe(Equal)
  })

  test("Ord composes the selected element Eq and stops at the first mismatch", () => {
    const calls: [number, number][] = []
    const ord = nonEmptyListOrd({
      eq: (left: number) => (right: number) => {
        calls.push([left, right])
        return left % 10 === right % 10
      },
      compare: (left: number) => (right: number) =>
        intOrd.compare(left % 10)(right % 10),
    })
    expect(ord.eq(values(1, 2))(values(11, 12))).toBe(true)
    expect(calls).toEqual([
      [1, 11],
      [2, 12],
    ])
    calls.length = 0
    expect(ord.eq(values(1, 2))(values(3, 12))).toBe(false)
    expect(calls).toEqual([[1, 3]])
    calls.length = 0
    expect(ord.eq(values(1, 2, 3))(values(11, 14, 13))).toBe(false)
    expect(calls).toEqual([
      [1, 11],
      [2, 14],
    ])
  })

  test("Ord retains Eq when NonEmptyList dictionaries are nested", () => {
    const ord = nonEmptyListOrd(nonEmptyListOrd(intOrd))
    const left = NonEmptyList(values(1), fromArray([values(2, 3)]))
    const equal = NonEmptyList(values(1), fromArray([values(2, 3)]))
    const different = NonEmptyList(values(1), fromArray([values(2, 4)]))
    expect(ord.eq(left)(equal)).toBe(true)
    expect(ord.eq(left)(different)).toBe(false)
    expect(ord.compare(left)(equal)).toBe(Equal)
    expect(ord.compare(left)(different)).toBe(Less)
  })

  test("preserve source order and non-emptiness across algebraic instances", () => {
    const appended = nonEmptyListSemigroup.append(values(1, 2))(values(3, 4))
    expect(toArrayNonEmpty(appended)).toEqual([1, 2, 3, 4])

    const mapped = nonEmptyListFunctor.map((value: number) => value * 2)(
      values(1, 2, 3)
    )
    expect(toArrayNonEmpty(mapped)).toEqual([2, 4, 6])

    const functions = values<(value: number) => number>(
      (value) => value + 10,
      (value) => value * 2
    )
    const applied = nonEmptyListApplicative.apply(functions)(values(1, 2))
    expect(toArrayNonEmpty(applied)).toEqual([11, 12, 2, 4])

    const flattened = nonEmptyListMonad.flatMap((value: number) =>
      values(value, value + 10)
    )(values(1, 2))
    expect(toArrayNonEmpty(flattened)).toEqual([1, 11, 2, 12])
    expect(toArrayNonEmpty(nonEmptyListApplicative.pure(7))).toEqual([7])
  })

  test("iterate and reduce in source order", () => {
    const iterator = nonEmptyListIterable.iterate(values(1, 2, 3))
    const first = iterator.next()
    expect(first.tag).toBe("Just")
    if (first.tag === "Just") {
      expect(first.value[0]).toBe(1)
      const second = first.value[1].next()
      expect(second.tag === "Just" ? second.value[0] : undefined).toBe(2)
    }

    const reduced = nonEmptyListReducible.reduce(0)(
      (accumulator: number) => (value: number) => accumulator * 10 + value
    )(values(1, 2, 3))
    expect(reduced).toBe(123)
  })
})
