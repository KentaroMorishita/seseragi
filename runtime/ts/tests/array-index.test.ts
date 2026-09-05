import { expect, test } from "bun:test"
import { get, index } from "../src/array"
import { Just, Nothing } from "../src/sum"

test("postfix ABI shares safe get boundaries and receiver-first evaluation", () => {
  const values = [10, 20, 30]
  for (const offset of [-1, 0, 1, 2, 3, 99]) {
    expect(index(values, offset)).toEqual(get(offset, values))
  }
  expect(index([], 0)).toEqual(Nothing)
  const events: string[] = []
  const receiver = () => {
    events.push("receiver")
    return values
  }
  const offset = () => {
    events.push("index")
    return 1
  }
  expect(index(receiver(), offset())).toEqual(Just(20))
  expect(events).toEqual(["receiver", "index"])
})
