import { describe, expect, test } from "bun:test"
import {
  arrayEq,
  boolEq,
  eitherEq,
  intEq,
  intOrd,
  listEq,
  maybeEq,
  recordEq,
  stringEq,
  stringOrd,
  tupleEq,
  tupleOrd,
  unitEq,
} from "../src/equality"
import { intHash, stringHash, tupleHash } from "../src/hash"
import { fromArray } from "../src/list"
import { Just, Left, Nothing, Right } from "../src/sum"

describe("standard Eq dictionaries", () => {
  test("use canonical primitive equality", () => {
    expect(intEq.eq(42)(42)).toBe(true)
    expect(boolEq.eq(true)(false)).toBe(false)
    expect(stringEq.eq("se")("seragi")).toBe(false)
    expect(unitEq.eq(undefined)(undefined)).toBe(true)
  })

  test("compose conditional Array and List evidence", () => {
    const arrays = arrayEq(intEq)
    expect(arrays.eq([1, 2])([1, 2])).toBe(true)
    expect(arrays.eq([1, 2])([2, 1])).toBe(false)

    const lists = listEq(intEq)
    expect(lists.eq(fromArray([1, 2]))(fromArray([1, 2]))).toBe(true)
    expect(lists.eq(fromArray([1, 2]))(fromArray([1, 3]))).toBe(false)
  })

  test("compose structural tuple and closed-record evidence", () => {
    const tuples = tupleEq<readonly [number, string]>(intEq, stringEq)
    expect(tuples.eq([1, "a"])([1, "a"])).toBe(true)
    expect(tuples.eq([1, "a"])([1, "b"])).toBe(false)

    type RecordValue = Readonly<{ label: string; note?: string }>
    const records = recordEq<RecordValue>(
      ["label", "note"],
      [false, true],
      stringEq,
      stringEq
    )
    expect(records.eq({ label: "ok" })({ label: "ok" })).toBe(true)
    expect(records.eq({ label: "ok" })({ label: "ok", note: "" })).toBe(false)
    expect(records.eq({} as RecordValue)({} as RecordValue)).toBe(false)
    expect(
      records.eq({ label: "ok", note: "a" })({ label: "ok", note: "a" })
    ).toBe(true)
  })
})

test("nested sum equality preserves tags and delegates to payload evidence", () => {
  const sum = maybeEq(eitherEq(stringEq, intEq))
  expect(sum.eq(Nothing)(Nothing)).toBe(true)
  expect(sum.eq(Nothing)(Just(Right(1)))).toBe(false)
  expect(sum.eq(Just(Right(1)))(Just(Right(1)))).toBe(true)
  expect(sum.eq(Just(Right(1)))(Just(Right(2)))).toBe(false)
  expect(sum.eq(Just(Left("1")))(Just(Right(1)))).toBe(false)
  expect(sum.eq(Just(Left("x")))(Just(Left("x")))).toBe(true)
  const parity = {
    eq: (left: number) => (right: number) => left % 2 === right % 2,
  }
  expect(maybeEq(parity).eq(Just(1))(Just(3))).toBe(true)
})

test("tuple ordering and hashing use positional evidence", () => {
  const order = tupleOrd<[number, string]>(intOrd, stringOrd)
  const hash = tupleHash<[number, string]>(intHash, stringHash)
  expect(order.compare([0, "z"])([1, "a"]).tag).toBe("Less")
  expect(order.compare([1, "b"])([1, "a"]).tag).toBe("Greater")
  expect(order.eq([1, "a"])([1, "a"])).toBe(true)
  expect(hash.hash([1, "a"])).toBe(hash.hash([1, "a"]))
  expect(tupleOrd<[]>().compare([])([]).tag).toBe("Equal")
})
