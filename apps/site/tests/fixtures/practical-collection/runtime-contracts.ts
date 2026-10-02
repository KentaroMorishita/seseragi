import assert from "node:assert/strict"
import * as arrays from "../../../../../runtime/ts/src/array.ts"
import { intEq, stringEq } from "../../../../../runtime/ts/src/equality.ts"
import { intHash, stringHash } from "../../../../../runtime/ts/src/hash.ts"
import * as lists from "../../../../../runtime/ts/src/list.ts"
import * as maps from "../../../../../runtime/ts/src/map.ts"
import * as sets from "../../../../../runtime/ts/src/set.ts"
import { Just, Nothing } from "../../../../../runtime/ts/src/sum.ts"

const results: Record<string, unknown> = {}
const values = [0, 2, 0, 3]
const linked = lists.fromArray(values)
for (const index of [-1, 0, 1, 3, 4, 99]) {
  const expected =
    index < 0 || index >= values.length ? Nothing : Just(values[index])
  assert.deepEqual(arrays.get(index, values), expected)
  assert.deepEqual(lists.get(index, linked), expected)
}
for (const count of [-1, 0, 1, 4, 5]) {
  assert.deepEqual(
    arrays.take(count, values),
    values.slice(0, Math.max(0, count))
  )
  assert.deepEqual(
    lists.toArray(lists.take(count, linked)),
    values.slice(0, Math.max(0, count))
  )
  assert.deepEqual(arrays.drop(count, values), values.slice(Math.max(0, count)))
  assert.deepEqual(
    lists.toArray(lists.drop(count, linked)),
    values.slice(Math.max(0, count))
  )
}
const arrayFindCalls: number[] = []
assert.deepEqual(
  arrays.find((value) => {
    arrayFindCalls.push(value)
    return value === 2
  }, values),
  Just(2)
)
assert.deepEqual(arrayFindCalls, [0, 2])
const listFindCalls: number[] = []
assert.deepEqual(
  lists.find((value) => {
    listFindCalls.push(value)
    return value === 2
  }, linked),
  Just(2)
)
assert.deepEqual(listFindCalls, [0, 2])
const arrayFilterCalls: number[] = []
assert.deepEqual(
  arrays.filter((value) => {
    arrayFilterCalls.push(value)
    return value === 0
  }, values),
  [0, 0]
)
assert.deepEqual(arrayFilterCalls, values)
const listFilterCalls: number[] = []
assert.deepEqual(
  lists.toArray(
    lists.filter((value) => {
      listFilterCalls.push(value)
      return value === 0
    }, linked)
  ),
  [0, 0]
)
assert.deepEqual(listFilterCalls, values)
assert.deepEqual(values, [0, 2, 0, 3])
assert.deepEqual(lists.toArray(linked), values)
if (linked.tag !== "Cons") throw new Error("Expected nonempty fixture")
assert.equal(lists.drop(1, linked), linked.tail)
assert.equal(lists.drop(0, linked), linked)
assert.notEqual(arrays.drop(0, values), values)
results.sequence = {
  getOffsets: [-1, 0, 1, 3, 4, 99],
  counts: [-1, 0, 1, 4, 5],
  arrayFindCalls,
  listFindCalls,
  arrayFilterCalls,
  listFilterCalls,
  originalPreserved: true,
  listDropSharesTail: true,
  arrayDropCopies: true,
}
const stock = maps.fromEntries<
  ReadonlyArray<readonly [string, number]>,
  string,
  number
>(arrays.arrayIterable, stringEq, stringHash, [
  ["tea", 2],
  ["hold", 5],
  ["coffee", 3],
  ["empty", 0],
] as const)
const mapCalls: Array<[string, number]> = []
const selected = maps.filter(
  (key: string) => (value: number) => {
    mapCalls.push([key, value])
    return key !== "hold" && value > 0
  },
  stock
)
assert.deepEqual(mapCalls, [
  ["tea", 2],
  ["hold", 5],
  ["coffee", 3],
  ["empty", 0],
])
assert.deepEqual(maps.entries(selected), [
  ["tea", 2],
  ["coffee", 3],
])
assert.deepEqual(maps.entries(stock), mapCalls)
assert.equal(maps.isEmpty(maps.filter(() => () => false, stock)), true)
const ids = sets.fromIterable(
  arrays.arrayIterable,
  intEq,
  intHash,
  [4, 1, 4, 3, 0]
)
const setCalls: number[] = []
assert.deepEqual(
  sets.toArray(
    sets.filter((value: number) => {
      setCalls.push(value)
      return value >= 3
    }, ids)
  ),
  [4, 3]
)
assert.deepEqual(setCalls, [4, 1, 3, 0])
assert.deepEqual(sets.toArray(ids), [4, 1, 3, 0])
results.mapSet = {
  mapCalls,
  filteredEntries: maps.entries(selected),
  setCalls,
  originalPreserved: true,
}
let emptyCalls = 0
arrays.find(() => {
  emptyCalls++
  return true
}, [])
lists.find(() => {
  emptyCalls++
  return true
}, lists.fromArray([]))
arrays.filter(() => {
  emptyCalls++
  return true
}, [])
lists.filter(() => {
  emptyCalls++
  return true
}, lists.fromArray([]))
maps.filter(
  () => () => {
    emptyCalls++
    return true
  },
  maps.empty()
)
sets.filter(() => {
  emptyCalls++
  return true
}, sets.empty())
assert.equal(emptyCalls, 0)
results.emptyCallbacks = emptyCalls
results.scope =
  "Direct current TypeScript runtime assertions under Bun; representation/iteration evidence, not timing or a browser-host test"
console.log(JSON.stringify(results, null, 2))
