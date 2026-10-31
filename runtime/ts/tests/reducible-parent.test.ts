import { expect, test } from "bun:test"
import { arrayIterable, arrayReducible } from "../src/array"
import {
  type Iterable,
  Next,
  type Reducible,
  reduceUntil,
} from "../src/collection"
import { intEq } from "../src/equality"
import { intHash } from "../src/hash"
import {
  fromArray,
  listIterable,
  listReducible,
  NonEmptyList,
  nonEmptyListIterable,
  nonEmptyListReducible,
} from "../src/list"
import * as maps from "../src/map"
import { inclusive, rangeIterable, rangeReducible } from "../src/range"
import * as sets from "../src/set"

function checkParent<C, A>(
  dictionary: Reducible<C, A>,
  parent: Iterable<C, A>,
  values: C,
  expected: ReadonlyArray<A>
) {
  expect(Object.isFrozen(dictionary)).toBe(true)
  expect(dictionary.iterate).toBe(parent.iterate)
  const append = (acc: ReadonlyArray<A>) => (value: A) => [...acc, value]
  expect(dictionary.reduce<ReadonlyArray<A>>([])(append)(values)).toEqual(
    expected
  )
  expect(
    reduceUntil(
      dictionary,
      [] as ReadonlyArray<A>,
      (acc) => (value: A) => Next(append(acc)(value)),
      values
    )
  ).toEqual(expected)
}

test("all standard Reducible dictionaries expose their selected Iterable parent", () => {
  checkParent(arrayReducible, arrayIterable, [3, 1, 2], [3, 1, 2])
  checkParent(arrayReducible, arrayIterable, [], [])
  checkParent(listReducible, listIterable, fromArray([3, 1, 2]), [3, 1, 2])
  checkParent(listReducible, listIterable, fromArray([]), [])
  checkParent(
    nonEmptyListReducible,
    nonEmptyListIterable,
    NonEmptyList(3, fromArray([1, 2])),
    [3, 1, 2]
  )
  checkParent(rangeReducible, rangeIterable, inclusive(1, 3), [1, 2, 3])
  checkParent(rangeReducible, rangeIterable, inclusive(3, 1), [])
  const entries: ReadonlyArray<readonly [number, number]> = [
    [2, 20],
    [1, 10],
    [2, 22],
  ]
  checkParent(
    maps.mapReducible,
    maps.mapIterable,
    maps.fromEntries(arrayIterable, intEq, intHash, entries),
    [
      [2, 22],
      [1, 10],
    ]
  )
  checkParent(
    maps.mapReducible,
    maps.mapIterable,
    maps.empty<number, number>(),
    []
  )
  checkParent(
    sets.setReducible,
    sets.setIterable,
    sets.fromIterable(arrayIterable, intEq, intHash, [3, 1, 3, 2]),
    [3, 1, 2]
  )
  checkParent(sets.setReducible, sets.setIterable, sets.empty<number>(), [])
})
