import { expect, test } from "bun:test"
import { arrayTraversable } from "../src/array"
import { fromArray, listTraversable, toArray } from "../src/list"
import {
  bimap,
  eitherApplicative,
  eitherSequence,
  eitherTraverse,
  fold,
  Just,
  Left,
  mapLeft,
  mapRight,
  maybeApplicative,
  maybeMonoid,
  maybeSemigroup,
  maybeSequence,
  maybeTraverse,
  Nothing,
  orElse,
  Right,
  swap,
  withDefault,
} from "../src/sum"
import type { RuntimeDictionary } from "../src/traversable"

test("fallback selects existing values without invoking callable payloads", () => {
  const value = Just({ selected: true })
  const fallback = Just({ selected: false })
  expect(orElse(fallback, value)).toBe(value)
  expect(orElse(fallback, Nothing)).toBe(fallback)
  const callable = () => {
    throw new Error("payload must not be invoked")
  }
  expect(withDefault(callable, Nothing)).toBe(callable)
  expect(withDefault(callable, Just(callable))).toBe(callable)
  expect(withDefault(9, Just(3))).toBe(3)
})

test("Either helpers invoke only the selected branch, once", () => {
  const events: string[] = []
  const left = (value: string) => {
    events.push(`left:${value}`)
    return value.length
  }
  const right = (value: number) => {
    events.push(`right:${value}`)
    return value + 1
  }
  expect(bimap(left, right, Left("bad"))).toEqual(Left(3))
  expect(bimap(left, right, Right(4))).toEqual(Right(5))
  expect(fold(left, right, Left("no"))).toBe(2)
  expect(fold(left, right, Right(6))).toBe(7)
  expect(events).toEqual(["left:bad", "right:4", "left:no", "right:6"])
  const originalLeft = Left("same")
  const originalRight = Right(9)
  expect(mapLeft(left, originalRight)).toBe(originalRight)
  expect(mapRight(right, originalLeft)).toBe(originalLeft)
  expect(swap(originalLeft)).toEqual(Right("same"))
  expect(swap(originalRight)).toEqual(Left(9))
})

test("fold returns callable results as values", () => {
  const callback = () => 42
  expect(
    fold(
      () => callback,
      () => callback,
      Right(1)
    )
  ).toBe(callback)
})

test("module traversal uses canonical strict source order and first failure", () => {
  const events: number[] = []
  const result = eitherTraverse(
    arrayTraversable,
    (n: number) => {
      events.push(n)
      return n > 1 ? Left(`error:${n}`) : Right(n)
    },
    [1, 2, 3]
  )
  expect(result).toEqual(Left("error:2"))
  expect(events).toEqual([1, 2, 3])
  expect(maybeSequence(arrayTraversable, [Just(1), Nothing, Just(3)])).toBe(
    Nothing
  )
  expect(eitherSequence(arrayTraversable, [])).toEqual(Right([]))
  expect(maybeSequence(arrayTraversable, [])).toEqual(Just([]))
  const list = eitherTraverse(
    listTraversable,
    (n: number) => Right(n + 1),
    fromArray([1, 2])
  )
  expect(toArray(list.value)).toEqual([2, 3])
})

test("custom Traversable receives the existing target dictionary and owns shape", () => {
  const dictionaries: RuntimeDictionary[] = []
  const custom: RuntimeDictionary = {
    traverse: (f) => (value) => (dictionary: RuntimeDictionary) => {
      dictionaries.push(dictionary)
      return dictionary.map((item: unknown) => ({ item }))(f(value.item))
    },
  }
  expect(
    maybeTraverse(custom, (n: number) => Just(n + 1), { item: 4 })
  ).toEqual(Just({ item: 5 }))
  expect(eitherSequence(custom, { item: Left("bad") })).toEqual(Left("bad"))
  expect(dictionaries).toEqual([maybeApplicative, eitherApplicative])
})

test("Maybe Monoid needs only the element Semigroup and preserves append order", () => {
  const events: string[] = []
  const element = {
    append: (left: string) => (right: string) => {
      events.push(`${left}:${right}`)
      return left + right
    },
  }
  const dictionary = maybeMonoid(element)
  const value = Just("a")
  expect(dictionary.empty(undefined)).toBe(Nothing)
  expect(dictionary.append(Nothing)(value)).toBe(value)
  expect(dictionary.append(value)(Nothing)).toBe(value)
  expect(dictionary.append(value)(Just("b"))).toEqual(Just("ab"))
  expect(events).toEqual(["a:b"])
  const semigroup = maybeSemigroup(element)
  expect(
    semigroup.append(semigroup.append(Just("a"))(Just("b")))(Just("c"))
  ).toEqual(semigroup.append(Just("a"))(semigroup.append(Just("b"))(Just("c"))))
  expect(Object.isFrozen(dictionary)).toBe(true)
})
