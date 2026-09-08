import { expect, test } from "bun:test"
import { Cons, Empty, fromArray } from "../src/list"

test("cons shares an untouched tail in constant time", () => {
  const tail = new Proxy(fromArray([2, 3]), {
    get() {
      throw new Error("cons must not traverse or inspect its tail")
    },
  })
  const value = Cons(1, tail)
  expect(value.tag).toBe("Cons")
  if (value.tag !== "Cons") throw new Error("expected Cons")
  expect(value.tail).toBe(tail)
  expect(Object.isFrozen(value)).toBe(true)
  expect(Cons(undefined, Empty)).toEqual({
    tag: "Cons",
    head: undefined,
    tail: Empty,
  })
})
