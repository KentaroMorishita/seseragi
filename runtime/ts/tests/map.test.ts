import { afterEach, expect, test } from "bun:test"
import { arrayIterable } from "../src/array"
import { type Eq, intEq, stringEq } from "../src/equality"
import {
  type Hash,
  intHash,
  resetProcessHashSeedForTest,
  stringHash,
} from "../src/hash"
import * as maps from "../src/map"
import { Just, Nothing } from "../src/sum"

const host = globalThis as typeof globalThis & {
  __SESERAGI_HASH_SEED__?: number | bigint
}
const savedSeed = host.__SESERAGI_HASH_SEED__
afterEach(() => {
  if (savedSeed === undefined) delete host.__SESERAGI_HASH_SEED__
  else host.__SESERAGI_HASH_SEED__ = savedSeed
  resetProcessHashSeedForTest()
})

function stringMap(entries: ReadonlyArray<readonly [string, number]>) {
  return maps.fromEntries(arrayIterable, stringEq, stringHash, entries)
}

function reachable(root: unknown): Set<object> {
  const seen = new Set<object>()
  const visit = (value: unknown) => {
    if (value === null || typeof value !== "object" || seen.has(value)) return
    seen.add(value)
    for (const key of Reflect.ownKeys(value)) visit(Reflect.get(value, key))
  }
  visit(root)
  return seen
}

test("duplicate keys keep the first key and position but the last value", () => {
  const values = stringMap([
    ["b", 1],
    ["a", 2],
    ["b", 3],
  ])
  expect(maps.entries(values)).toEqual([
    ["b", 3],
    ["a", 2],
  ])
  expect(maps.get(stringEq, stringHash, "b", values)).toEqual(Just(3))
  expect(maps.get(stringEq, stringHash, "missing", values)).toBe(Nothing)
  expect(maps.size(values)).toBe(2)
  expect(maps.isEmpty(values)).toBe(false)
  expect(maps.isEmpty(maps.empty())).toBe(true)
  const changed = maps.insert(stringEq, stringHash, "b", 4, values)
  expect(maps.entries(changed)).toEqual([
    ["b", 4],
    ["a", 2],
  ])
  expect(maps.entries(values)).toEqual([
    ["b", 3],
    ["a", 2],
  ])
  expect(maps.remove(stringEq, stringHash, "missing", values)).toBe(values)
})

test("remove and reinsert appends even when the vacated internal slot is reused", () => {
  const initial = stringMap([
    ["a", 1],
    ["b", 2],
    ["c", 3],
  ])
  const removed = maps.remove(stringEq, stringHash, "b", initial)
  const added = maps.insert(stringEq, stringHash, "b", 4, removed)
  expect(maps.entries(added)).toEqual([
    ["a", 1],
    ["c", 3],
    ["b", 4],
  ])
  const noHead = maps.remove(stringEq, stringHash, "a", added)
  const noTail = maps.remove(stringEq, stringHash, "b", noHead)
  expect(maps.entries(noTail)).toEqual([["c", 3]])
  const empty = maps.remove(stringEq, stringHash, "c", noTail)
  expect(maps.entries(empty)).toEqual([])
  expect(
    maps.entries(maps.insert(stringEq, stringHash, "z", 9, empty))
  ).toEqual([["z", 9]])
  expect(maps.keys(initial)).toEqual(["a", "b", "c"])
})

test("upsert evaluates Hash once and update once for present and absent keys", () => {
  let hashes = 0
  const hash = {
    hash: (value: string) => {
      hashes += 1
      return stringHash.hash(value)
    },
  }
  const calls: unknown[] = []
  let values = stringMap([["a", 1]])
  for (const key of ["a", "b"]) {
    values = maps.upsert(
      stringEq,
      hash,
      key,
      (current) => {
        calls.push(current)
        return current.tag === "Nothing" ? 5 : current.value + 1
      },
      values
    )
  }
  expect(hashes).toBe(2)
  expect(calls).toEqual([Just(1), Nothing])
  expect(maps.entries(values)).toEqual([
    ["a", 2],
    ["b", 5],
  ])
})

test("structural Eq handles complete hash collisions without host identity", () => {
  type Key = { id: number }
  const eq: Eq<Key> = { eq: (left) => (right) => left.id === right.id }
  const hash: Hash<Key> = { hash: () => 0 }
  const first = { id: 1 }
  let values = maps.singleton(eq, hash, first, "first")
  values = maps.insert(eq, hash, { id: 2 }, "second", values)
  values = maps.insert(eq, hash, { id: 1 }, "updated", values)
  expect(maps.keys(values)[0]).toBe(first)
  expect(maps.values(values)).toEqual(["updated", "second"])
  expect(maps.get(eq, hash, { id: 2 }, values)).toEqual(Just("second"))
  expect(maps.values(maps.remove(eq, hash, { id: 1 }, values))).toEqual([
    "second",
  ])
})

test("transform callbacks follow insertion order and resolve current then incoming", () => {
  const source = stringMap([
    ["b", 2],
    ["a", 1],
    ["c", 3],
  ])
  const calls: unknown[] = []
  expect(
    maps.entries(
      maps.filter(
        (key) => (value) => {
          calls.push([key, value])
          return key !== "a"
        },
        source
      )
    )
  ).toEqual([
    ["b", 2],
    ["c", 3],
  ])
  expect(calls).toEqual([
    ["b", 2],
    ["a", 1],
    ["c", 3],
  ])
  calls.length = 0
  expect(
    maps.values(
      maps.mapValues((value) => {
        calls.push(value)
        return value * 10
      }, source)
    )
  ).toEqual([20, 10, 30])
  expect(calls).toEqual([2, 1, 3])
  calls.length = 0
  const collapsed = maps.mapKeysWith(
    stringEq,
    stringHash,
    (current) => (incoming) => {
      calls.push([current, incoming])
      return current * 10 + incoming
    },
    (key) => {
      calls.push(key)
      return key === "a" ? "a" : "same"
    },
    source
  )
  expect(calls).toEqual(["b", "a", "c", [2, 3]])
  expect(maps.entries(collapsed)).toEqual([
    ["same", 23],
    ["a", 1],
  ])
})

test("different process seeds preserve observations and combination uses the left seed", () => {
  host.__SESERAGI_HASH_SEED__ = 11
  resetProcessHashSeedForTest()
  const left = stringMap([
    ["b", 2],
    ["a", 1],
  ])
  host.__SESERAGI_HASH_SEED__ = 29
  resetProcessHashSeedForTest()
  const equal = stringMap([
    ["a", 1],
    ["b", 2],
  ])
  const right = stringMap([
    ["a", 3],
    ["c", 4],
  ])
  expect(maps.mapEq(stringEq, stringHash, intEq).eq(left)(equal)).toBe(true)
  const merged = maps.mergeWith(
    stringEq,
    stringHash,
    (a) => (b) => a * 10 + b,
    right,
    left
  )
  expect(maps.entries(merged)).toEqual([
    ["b", 2],
    ["a", 13],
    ["c", 4],
  ])
  expect(maps.get(stringEq, stringHash, "c", merged)).toEqual(Just(4))
  const state = (map: unknown) =>
    Reflect.get(map as object, Reflect.ownKeys(map as object)[0] as symbol)
  expect(state(merged).seed).toBe(11)
  expect(state(maps.filter(() => () => true, left)).seed).toBe(11)
  expect(state(maps.mapValues((value) => value + 1, left)).seed).toBe(11)
  expect(
    state(
      maps.mapKeysWith(
        stringEq,
        stringHash,
        (a) => () => a,
        (key) => key,
        left
      )
    ).seed
  ).toBe(11)
  expect(state(right).seed).toBe(29)
})

test("iteration is persistent and reduction and Functor use insertion order", () => {
  const values = stringMap([
    ["b", 2],
    ["a", 1],
  ])
  const iterator = maps.mapIterable.iterate(values)
  const first = iterator.next()
  const again = iterator.next()
  expect(first.tag).toBe("Just")
  expect(again.tag).toBe("Just")
  if (first.tag !== "Just" || again.tag !== "Just")
    throw new Error("missing entry")
  expect(first.value[0]).toEqual(["b", 2])
  expect(again.value[0]).toEqual(first.value[0])
  const rest = first.value[1].next()
  expect(rest.tag).toBe("Just")
  if (rest.tag !== "Just") throw new Error("missing second entry")
  expect(rest.value[0]).toEqual(["a", 1])
  expect(rest.value[1].next()).toBe(Nothing)
  expect(iterator.next().tag).toBe("Just")
  expect(
    maps.mapReducible.reduce(0)(
      (total: number) =>
        ([, value]: readonly [string, number]) =>
          total * 10 + value
    )(values)
  ).toBe(21)
  expect(
    maps.entries(maps.mapFunctor.map((value: number) => value + 1)(values))
  ).toEqual([
    ["b", 3],
    ["a", 2],
  ])
})

test("signed 64-bit seeds remain internal across lookup, update and removal", () => {
  for (const seed of [-(1n << 63n), (1n << 63n) - 1n]) {
    host.__SESERAGI_HASH_SEED__ = seed
    resetProcessHashSeedForTest()
    const original = stringMap([
      ["first", 1],
      ["second", 2],
    ])
    const updated = maps.insert(stringEq, stringHash, "first", 3, original)
    expect(maps.get(stringEq, stringHash, "first", updated)).toEqual(Just(3))
    expect(maps.entries(original)).toEqual([
      ["first", 1],
      ["second", 2],
    ])
    expect(
      maps.keys(maps.remove(stringEq, stringHash, "first", updated))
    ).toEqual(["second"])
  }
})

test("point updates share storage; deletion and mapValues retain no removed user objects", () => {
  const removed = { secret: "removed" }
  let values = maps.empty<number, object>()
  for (let key = 0; key < 1024; key += 1)
    values = maps.insert(
      intEq,
      intHash,
      key,
      key === 500 ? removed : { key },
      values
    )
  const previousNodes = reachable(values)
  const deleted = maps.remove(intEq, intHash, 500, values)
  const newNodes = reachable(deleted)
  expect(newNodes.has(removed)).toBe(false)
  expect(previousNodes.has(removed)).toBe(true)
  expect(
    [...newNodes].filter((node) => !previousNodes.has(node)).length
  ).toBeLessThan(220)
  const mapped = maps.mapValues(() => 1, values)
  expect(reachable(mapped).has(removed)).toBe(false)
  const rejected = maps.filter((key) => () => key !== 500, values)
  expect(reachable(rejected).has(removed)).toBe(false)
  let tiny = values
  for (let key = 0; key < 1023; key += 1)
    tiny = maps.remove(intEq, intHash, key, tiny)
  expect(maps.size(tiny)).toBe(1)
  expect(reachable(tiny).size).toBeLessThan(10)
})

test("mixed operation history agrees with an insertion-order reference model", () => {
  let values = maps.empty<number, number>()
  const reference = new Map<number, number>()
  let random = 1
  for (let step = 0; step < 2000; step += 1) {
    random = (Math.imul(random, 1664525) + 1013904223) >>> 0
    const key = random % 127
    if (random % 3 === 0) {
      values = maps.remove(intEq, intHash, key, values)
      reference.delete(key)
    } else {
      values = maps.insert(intEq, intHash, key, step, values)
      reference.set(key, step)
    }
    expect(maps.entries(values)).toEqual([...reference.entries()])
    expect(maps.size(values)).toBe(reference.size)
  }
})
