import { expect, test } from "bun:test"
import {
  type Index,
  indexGet,
  indexRemove,
  indexSet,
  indexVacant,
} from "../src/persistent-index"

function nodes<Value>(index: Index<Value>): Set<object> {
  const result = new Set<object>()
  const visit = (node: Index<Value>) => {
    if (node === undefined) return
    result.add(node)
    expect(Object.isFrozen(node)).toBe(true)
    if (node.tag === "Branch") {
      expect(node.left.max).toBeLessThan(node.right.min)
      expect(node.size).toBe(node.left.size + node.right.size)
      visit(node.left)
      visit(node.right)
    }
  }
  visit(index)
  return result
}

test("safe-integer addresses use all 53 bits, not JavaScript's 32-bit coercion", () => {
  const keys = [
    0,
    1,
    2 ** 31,
    2 ** 32,
    2 ** 32 + 1,
    2 ** 51,
    Number.MAX_SAFE_INTEGER,
  ]
  let index: Index<string>
  for (const key of keys) index = indexSet(index, key, `value:${key}`)
  for (const key of keys) expect(indexGet(index, key)).toBe(`value:${key}`)
  for (const key of [2, 2 ** 32 - 1, 2 ** 40, Number.MAX_SAFE_INTEGER - 1]) {
    expect(indexGet(index, key)).toBeUndefined()
    expect(indexRemove(index, key)).toBe(index)
  }
  expect(nodes(index).size).toBe(2 * keys.length - 1)
  for (const key of keys) index = indexRemove(index, key)
  expect(index).toBeUndefined()
})

test("persistent edits share untouched paths and bound live storage", () => {
  let index: Index<object>
  const removed = { removed: true }
  for (let key = 0; key < 4096; key += 1) {
    index = indexSet(index, key, key === 2000 ? removed : { key })
  }
  const old = index
  const oldNodes = nodes(old)
  index = indexSet(index, 2048, { replacement: true })
  expect(
    [...nodes(index)].filter((node) => !oldNodes.has(node)).length
  ).toBeLessThanOrEqual(54)
  expect(indexGet(old, 2048)).toEqual({ key: 2048 })
  index = indexRemove(index, 2000)
  expect(indexGet(old, 2000)).toBe(removed)
  expect(indexGet(index, 2000)).toBeUndefined()
  expect(nodes(index).size).toBe(2 * 4095 - 1)
  for (const node of nodes(index)) {
    expect(Object.values(node)).not.toContain(removed)
  }
})

test("vacant addresses are reused without tombstones or growing history", () => {
  let index: Index<number>
  const model = new Map<number, number>()
  let random = 42
  for (let step = 0; step < 5000; step += 1) {
    random = (Math.imul(random, 1664525) + 1013904223) >>> 0
    const key = random % 257
    if (random % 3 === 0) {
      index = indexRemove(index, key)
      model.delete(key)
    } else {
      index = indexSet(index, key, step)
      model.set(key, step)
    }
    let vacant = 0
    while (model.has(vacant)) vacant += 1
    expect(indexVacant(index)).toBe(vacant)
    expect(index?.size ?? 0).toBe(model.size)
  }
  expect(nodes(index).size).toBe(model.size * 2 - 1)
  for (const [key, value] of model) expect(indexGet(index, key)).toBe(value)
  for (const key of model.keys()) index = indexRemove(index, key)
  expect(index).toBeUndefined()
  expect(indexVacant(index)).toBe(0)
})

test("the entire index rejects non-address values", () => {
  for (const key of [-1, 0.5, NaN, Infinity, 2 ** 53]) {
    expect(() => indexSet(undefined, key, true)).toThrow(RangeError)
  }
})
