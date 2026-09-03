import { describe, expect, test } from "bun:test"
import * as arrays from "../src/array"
import { NonPositiveSize } from "../src/collection"
import {
  boolOrd,
  charOrd,
  intEq,
  intOrd,
  stringEq,
  stringOrd,
  unitOrd,
} from "../src/equality"
import { intHash, stringHash } from "../src/hash"
import * as lists from "../src/list"
import * as maps from "../src/map"
import type { Ord } from "../src/sequence"
import { Equal, Greater, Just, Left, Less, Nothing, Right } from "../src/sum"

const numberOrd: Ord<number> = {
  compare: (left) => (right) =>
    left < right ? Less : left > right ? Greater : Equal,
}

test("primitive Ord follows numeric, boolean and Unicode scalar ordering", () => {
  expect(
    arrays.sort(intOrd, [
      Number.MAX_SAFE_INTEGER,
      0,
      -1,
      Number.MIN_SAFE_INTEGER,
    ])
  ).toEqual([Number.MIN_SAFE_INTEGER, -1, 0, Number.MAX_SAFE_INTEGER])
  expect(arrays.sort(boolOrd, [true, false, true])).toEqual([false, true, true])
  expect(arrays.sort(charOrd, ["😀", "\ue000", "a"])).toEqual([
    "a",
    "\ue000",
    "😀",
  ])
  expect(
    arrays.sort(stringOrd, ["😀", "\ue000", "", "a😀", "a\ue000", "a"])
  ).toEqual(["", "a", "a\ue000", "a😀", "\ue000", "😀"])
  expect(unitOrd.compare(undefined)(undefined)).toEqual(Equal)
  expect(intOrd.eq(3)(3)).toBe(true)
  expect(boolOrd.eq(true)(false)).toBe(false)
  expect(charOrd.eq("😀")("😀")).toBe(true)
  expect(stringOrd.eq("a")("b")).toBe(false)
  expect(unitOrd.eq(undefined)(undefined)).toBe(true)
  for (const [a, b] of [
    ["a", "a"],
    ["😀x", "😀x"],
    ["", ""],
  ])
    expect(stringOrd.compare(a!)(b!)).toEqual(Equal)
})

function reachable(root: unknown): Set<object> {
  const found = new Set<object>()
  const visit = (value: unknown) => {
    if (value === null || typeof value !== "object" || found.has(value)) return
    found.add(value)
    for (const key of Reflect.ownKeys(value)) visit(Reflect.get(value, key))
  }
  visit(root)
  return found
}

function assertNumbers<C>(api: {
  name: string
  from: (values: readonly number[]) => C
  to: (values: C) => readonly number[]
  reduceRight: (
    initial: string,
    step: (value: number) => (acc: string) => string,
    values: C
  ) => string
  findIndex: (
    predicate: (value: number) => boolean,
    values: C
  ) => ReturnType<typeof arrays.findIndex>
  takeWhile: (predicate: (value: number) => boolean, values: C) => C
  dropWhile: (predicate: (value: number) => boolean, values: C) => C
  zipWith: (
    f: (left: number) => (right: number) => number,
    right: C,
    left: C
  ) => C
  last: (values: C) => ReturnType<typeof arrays.last<number>>
  init: (values: C) => { tag: "Nothing" } | { tag: "Just"; value: C }
  sort: (ord: Ord<number>, values: C) => C
  chunksOf: (
    size: number,
    values: C
  ) => { tag: "Left"; value: unknown } | { tag: "Right"; value: readonly C[] }
  windows: (
    size: number,
    values: C
  ) => { tag: "Left"; value: unknown } | { tag: "Right"; value: readonly C[] }
}) {
  describe(api.name, () => {
    test("empty, singleton, multi-element and right-to-left callback order", () => {
      for (const source of [[], [7], [1, 2, 3]]) {
        const input = api.from(source)
        const calls: number[] = []
        expect(
          api.reduceRight(
            "end",
            (value) => (acc) => {
              calls.push(value)
              return `${value}:${acc}`
            },
            input
          )
        ).toBe([...source.map(String), "end"].join(":"))
        expect(calls).toEqual(source.slice().reverse())
        expect(api.last(input)).toEqual(
          source.length ? Just(source.at(-1)!) : Nothing
        )
        const initial = api.init(input)
        expect(initial.tag).toBe(source.length ? "Just" : "Nothing")
        if (initial.tag === "Just")
          expect(api.to(initial.value)).toEqual(source.slice(0, -1))
        expect(api.to(input)).toEqual(source)
      }
    })

    test("prefix predicates stop at the first decisive item", () => {
      for (const source of [[], [1], [1, 2, 0, 3], [0, 1], [1, 2]]) {
        const calls: number[] = []
        const predicate = (value: number) => {
          calls.push(value)
          return value > 0
        }
        const index = source.findIndex((value) => value <= 0)
        const count = index < 0 ? source.length : index
        const visited = source.slice(0, index < 0 ? source.length : index + 1)
        const input = api.from(source)
        expect(api.to(api.takeWhile(predicate, input))).toEqual(
          source.slice(0, count)
        )
        expect(calls).toEqual(visited)
        calls.length = 0
        expect(api.to(api.dropWhile(predicate, input))).toEqual(
          source.slice(count)
        )
        expect(calls).toEqual(visited)
        calls.length = 0
        expect(api.findIndex((value) => !predicate(value), input)).toEqual(
          index < 0 ? Nothing : Just(index)
        )
        expect(calls).toEqual(visited)
      }
    })

    test("zipWith preserves argument and callback order, stops at shorter input", () => {
      for (const [left, right] of [
        [[], [10]],
        [[1], []],
        [[1, 2, 3], [10]],
        [[1], [10, 20, 30]],
        [
          [1, 2],
          [10, 20],
        ],
      ]) {
        const calls: number[][] = []
        const result = api.zipWith(
          (a) => (b) => {
            calls.push([a, b])
            return a - b
          },
          api.from(right!),
          api.from(left!)
        )
        const pairs = left!
          .slice(0, right!.length)
          .map((a, i) => [a, right![i]!])
        expect(calls).toEqual(pairs)
        expect(api.to(result)).toEqual(pairs.map(([a, b]) => a! - b!))
      }
    })

    test("size failures and every chunk/window boundary preserve exact size payload", () => {
      for (const source of [[], [1], [1, 2, 3, 4, 5]]) {
        for (const size of [-3, 0, 1, 2, 5, 6, Number.MAX_SAFE_INTEGER]) {
          for (const operation of ["chunksOf", "windows"] as const) {
            const result = api[operation](size, api.from(source))
            if (size <= 0) {
              expect(result).toEqual(Left(NonPositiveSize(size)))
              continue
            }
            expect(result.tag).toBe("Right")
            if (result.tag !== "Right") throw new Error("expected success")
            const expected: number[][] = []
            if (operation === "chunksOf") {
              for (let i = 0; i < source.length; i += size)
                expected.push(source.slice(i, i + size))
            } else {
              for (let i = 0; i <= source.length - size; i += 1)
                expected.push(source.slice(i, i + size))
            }
            expect(result.value.map(api.to)).toEqual(expected)
          }
        }
      }
    })

    test("sort is non-mutating with a logarithmic comparison bound", () => {
      const values = Array.from({ length: 4096 }, (_, i) => (i * 7919) % 4096)
      let comparisons = 0
      const ord: Ord<number> = {
        compare: (a) => (b) => {
          comparisons++
          return numberOrd.compare(a)(b)
        },
      }
      const input = api.from(values)
      expect(api.to(api.sort(ord, input))).toEqual(
        values.slice().sort((a, b) => a - b)
      )
      expect(comparisons).toBeLessThanOrEqual(values.length * 12)
      expect(api.to(input)).toEqual(values)
      expect(api.to(api.sort(ord, api.from([])))).toEqual([])
    })
  })
}

assertNumbers<ReadonlyArray<number>>({
  ...arrays,
  name: "Array",
  from: (values) => values,
  to: (values) => values,
})
assertNumbers<lists.List<number>>({
  ...lists,
  name: "List",
  from: lists.fromArray,
  to: lists.toArray,
  chunksOf: (size, values) => {
    const result = lists.chunksOf(size, values)
    return result.tag === "Left" ? result : Right(lists.toArray(result.value))
  },
  windows: (size, values) => {
    const result = lists.windows(size, values)
    return result.tag === "Left" ? result : Right(lists.toArray(result.value))
  },
})

test("sortBy caches each generic key once, source order, and is stable", () => {
  const source = [
    { key: 2, id: "a" },
    { key: 1, id: "b" },
    { key: 2, id: "c" },
    { key: 1, id: "d" },
  ]
  for (const kind of ["array", "list"] as const) {
    const calls: string[] = []
    const key = (value: (typeof source)[number]) => {
      calls.push(value.id)
      return value.key
    }
    const sorted =
      kind === "array"
        ? arrays.sortBy(numberOrd, key, source)
        : lists.toArray(lists.sortBy(numberOrd, key, lists.fromArray(source)))
    expect(calls).toEqual(["a", "b", "c", "d"])
    expect(sorted.map((value) => value.id)).toEqual(["b", "d", "a", "c"])
    expect(source.map((value) => value.id)).toEqual(["a", "b", "c", "d"])
  }
})

test("groupBy uses Eq/Hash including collisions, first key representative and source order", () => {
  const source = [
    { key: { id: 2 }, value: "a" },
    { key: { id: 1 }, value: "b" },
    { key: { id: 2 }, value: "c" },
  ]
  const eq = { eq: (a: { id: number }) => (b: { id: number }) => a.id === b.id }
  const hash = { hash: (_key: { id: number }) => 0 }
  for (const kind of ["array", "list"] as const) {
    const calls: string[] = []
    const key = (value: (typeof source)[number]) => {
      calls.push(value.value)
      return value.key
    }
    const grouped =
      kind === "array"
        ? arrays.groupBy(eq, hash, key, source)
        : maps.mapValues(
            lists.toArray<(typeof source)[number]>,
            lists.groupBy(eq, hash, key, lists.fromArray(source))
          )
    expect(calls).toEqual(["a", "b", "c"])
    expect(
      maps
        .entries(grouped)
        .map(([key, group]) => [key.id, group.map((value) => value.value)])
    ).toEqual([
      [2, ["a", "c"]],
      [1, ["b"]],
    ])
    expect(maps.keys(grouped)[0]).toBe(source[0]!.key)
  }
  expect(
    maps.size(arrays.groupBy(stringEq, stringHash, (s: string) => s, []))
  ).toBe(0)
  expect(
    maps.size(
      lists.groupBy(stringEq, stringHash, (s: string) => s, lists.Empty)
    )
  ).toBe(0)
})

test("one large group does not repeatedly copy an immutable prefix", () => {
  const source = Array.from({ length: 30_000 }, (_, i) => i)
  expect(maps.values(arrays.groupBy(intEq, intHash, () => 0, source))).toEqual([
    source,
  ])
  const grouped = lists.groupBy(
    intEq,
    intHash,
    () => 0,
    lists.fromArray(source)
  )
  expect(lists.toArray(maps.values(grouped)[0]!)).toEqual(source)
  expect(
    lists.reduceRight(
      0,
      (a: number) => (b: number) => a + b,
      lists.fromArray(source)
    )
  ).toBe(449985000)
})

test("zip/unzip, constructors and custom Iterable preserve generic values", () => {
  const left = [{ id: 1 }, { id: 2 }]
  const right = ["a", "b", "extra"]
  expect(arrays.unzip(arrays.zip(right, left))).toEqual([left, ["a", "b"]])
  const [a, b] = lists.unzip(
    lists.zip(lists.fromArray(right), lists.fromArray(left))
  )
  expect(lists.toArray(a)).toEqual(left)
  expect(lists.toArray(b)).toEqual(["a", "b"])
  expect(arrays.empty()).toEqual([])
  expect(lists.empty()).toBe(lists.Empty)
  expect(arrays.singleton(left[0]!)).toEqual([left[0]!])
  expect(lists.toArray(lists.singletonList(left[0]!))).toEqual([left[0]!])
  const source = { values: left }
  let nextCalls = 0
  const iterable = {
    iterate: ({ values }: typeof source) => {
      const at = (
        index: number
      ): import("../src/iterator").Iterator<(typeof left)[number]> => ({
        next: () => {
          nextCalls++
          return index === values.length
            ? Nothing
            : Just([values[index]!, at(index + 1)] as const)
        },
      })
      return at(0)
    },
  }
  expect(arrays.fromIterable(iterable, source)).toEqual(left)
  expect(nextCalls).toBe(3)
  nextCalls = 0
  expect(lists.toArray(lists.fromIterable(iterable, source))).toEqual(left)
  expect(nextCalls).toBe(3)
})

test("small sequence results do not retain excluded input elements", () => {
  const source = [{ id: 1 }, { id: 2 }, { id: 3 }, { id: 4 }]
  const list = lists.fromArray(source)
  const initialArray = arrays.init(source)
  const initialList = lists.init(list)
  if (initialArray.tag !== "Just" || initialList.tag !== "Just")
    throw new Error("expected init")
  for (const result of [
    initialArray.value,
    initialList.value,
    arrays.takeWhile((v) => v.id < 3, source),
    lists.takeWhile((v) => v.id < 3, list),
  ]) {
    expect(reachable(result).has(source[3]!)).toBe(false)
  }
  for (const result of [
    arrays.dropWhile((v) => v.id < 3, source),
    lists.dropWhile((v) => v.id < 3, list),
  ]) {
    expect(reachable(result).has(source[0]!)).toBe(false)
  }
  const arrayWindows = arrays.windows(2, source)
  const listWindows = lists.windows(2, list)
  if (arrayWindows.tag !== "Right" || listWindows.tag !== "Right")
    throw new Error("expected windows")
  const firstList = lists.head(listWindows.value)
  if (firstList.tag !== "Just") throw new Error("expected first window")
  for (const result of [arrayWindows.value[0], firstList.value]) {
    expect(reachable(result).has(source[2]!)).toBe(false)
    expect(reachable(result).has(source[3]!)).toBe(false)
  }
})
