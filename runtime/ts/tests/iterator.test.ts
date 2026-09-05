import { expect, test } from "bun:test"
import { next, unfold } from "../src/iterator"
import { Just, Nothing } from "../src/sum"

test("unfold is lazy and next performs exactly one persistent pull", () => {
  const calls: number[] = []
  const values = unfold((state: number) => {
    calls.push(state)
    return Just([state, state + 1] as const)
  }, 0)
  expect(calls).toEqual([])
  const first = next(values)
  expect(calls).toEqual([0])
  const repeated = next(values)
  expect(calls).toEqual([0, 0])
  if (first.tag !== "Just" || repeated.tag !== "Just") {
    throw new Error("infinite unfold must yield")
  }
  expect(first.value[0]).toBe(0)
  expect(repeated.value[0]).toBe(0)
  const second = next(first.value[1])
  expect(calls).toEqual([0, 0, 1])
  expect(second.tag === "Just" && second.value[0]).toBe(1)
})

test("empty iterator terminates without mutating its state", () => {
  let calls = 0
  const values = unfold(() => {
    calls++
    return Nothing
  }, 0)
  expect(calls).toBe(0)
  expect(next(values)).toBe(Nothing)
  expect(next(values)).toBe(Nothing)
  expect(calls).toBe(2)
})
