import { describe, expect, test } from "bun:test"
import {
  arrayEq,
  boolEq,
  intEq,
  listEq,
  recordEq,
  stringEq,
  tupleEq,
  unitEq,
} from "../src/equality"
import { fromArray } from "../src/list"

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
