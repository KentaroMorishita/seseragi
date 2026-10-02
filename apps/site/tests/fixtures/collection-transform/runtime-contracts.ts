import assert from "node:assert/strict"
import * as arrays from "../../../../../runtime/ts/src/array.ts"
import * as lists from "../../../../../runtime/ts/src/list.ts"
import { Just, type Maybe, Nothing } from "../../../../../runtime/ts/src/sum.ts"

// Instrumentation observes the runtime directly. Seseragi callbacks remain pure.
const receipts: unknown[] = []
for (const input of [[], [0], [2, 1, 0, 3]]) {
  for (const family of ["array", "list"] as const) {
    const visits: number[] = []
    const callback = (n: number): Maybe<number> => {
      visits.push(n)
      return n % 2 === 0 ? Just(n) : Nothing
    }
    const frozen = Object.freeze([...input])
    const sourceList = lists.fromArray(frozen)
    const output =
      family === "array"
        ? arrays.filterMap(callback, frozen)
        : lists.toArray(lists.filterMap(callback, sourceList))
    assert.deepEqual(visits, input)
    assert.deepEqual(
      output,
      input.filter((n) => n % 2 === 0)
    )
    assert.deepEqual(frozen, input)
    assert.deepEqual(lists.toArray(sourceList), input)
    receipts.push({
      family,
      operation: "filterMap",
      input,
      visits: [...visits],
      callbackCount: visits.length,
      output,
    })
    visits.length = 0
    const flatOutput =
      family === "array"
        ? arrays.flatMap((n: number) => {
            visits.push(n)
            return n === 0 ? [] : [[n], [n + 10]]
          }, frozen)
        : lists.toArray(
            lists.flatMap((n: number) => {
              visits.push(n)
              return lists.fromArray(n === 0 ? [] : [[n], [n + 10]])
            }, sourceList)
          )
    assert.deepEqual(visits, input)
    assert.deepEqual(
      flatOutput,
      input.flatMap((n) => (n === 0 ? [] : [[n], [n + 10]]))
    )
    assert.deepEqual(frozen, input)
    assert.deepEqual(lists.toArray(sourceList), input)
    receipts.push({
      family,
      operation: "flatMap",
      input,
      visits: [...visits],
      callbackCount: visits.length,
      output: flatOutput,
    })
  }
}
console.log(JSON.stringify(receipts))
