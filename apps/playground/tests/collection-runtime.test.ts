import { describe, expect, test } from "bun:test"
import {
  arrayIterable,
  arrayMonoid,
  arrayReducible,
  get as getArray,
  head as headArray,
  isEmpty as isEmptyArray,
  length as lengthArray,
  tail as tailArray,
} from "../../../runtime/ts/src/array"
import {
  all,
  any,
  combine,
  forEach,
  product,
} from "../../../runtime/ts/src/collection"
import { intMul, intOne } from "../../../runtime/ts/src/int64"
import {
  Empty,
  fromArray,
  get as getList,
  head as headList,
  isEmpty as isEmptyList,
  length as lengthList,
  type List,
  listMonoid,
  listReducible,
  reduce as reduceList,
  tail as tailList,
} from "../../../runtime/ts/src/list"
import { stringMonoid } from "../../../runtime/ts/src/string"
import { Just, Nothing } from "../../../runtime/ts/src/sum"

describe("Collection runtime", () => {
  test("observes Array values without leaking invalid indexes", () => {
    const values = [10n, 20n]

    expect(lengthArray(values)).toBe(2n)
    expect(isEmptyArray(values)).toBe(false)
    expect(isEmptyArray([])).toBe(true)
    expect(getArray(-1n, values)).toBe(Nothing)
    expect(getArray(2n, values)).toBe(Nothing)
    expect(getArray(1n, values)).toEqual(Just(20n))
    expect(headArray([])).toBe(Nothing)
    expect(headArray(values)).toEqual(Just(10n))
    expect(tailArray([])).toBe(Nothing)
    expect(tailArray([10n])).toEqual(Just([]))
    expect(tailArray(values)).toEqual(Just([20n]))
  })

  test("observes persistent List values without leaking invalid indexes", () => {
    const values = fromArray([10n, 20n])

    expect(lengthList(values)).toBe(2n)
    expect(isEmptyList(values)).toBe(false)
    expect(isEmptyList(Empty)).toBe(true)
    expect(getList(-1n, values)).toBe(Nothing)
    expect(getList(2n, values)).toBe(Nothing)
    expect(getList(1n, values)).toEqual(Just(20n))
    expect(headList(Empty)).toBe(Nothing)
    expect(headList(values)).toEqual(Just(10n))
    expect(tailList(Empty)).toBe(Nothing)
    expect(tailList(fromArray([10n]))).toEqual(Just(Empty))
    expect(tailList(values)).toEqual(Just(fromArray([20n])))
  })

  test("combines String, Array, and List values in source order", () => {
    expect(
      combine<ReadonlyArray<string>, string>(arrayReducible, stringMonoid, [
        "Sese",
        "ragi",
      ])
    ).toBe("Seseragi")
    expect(
      combine<ReadonlyArray<ReadonlyArray<string>>, ReadonlyArray<string>>(
        arrayReducible,
        arrayMonoid,
        [["a", "b"], ["c"]]
      )
    ).toEqual(["a", "b", "c"])

    const combined = combine<ReadonlyArray<List<string>>, List<string>>(
      arrayReducible,
      listMonoid,
      [fromArray(["l1", "l2"]), fromArray(["l3"])]
    )
    expect(
      reduceList<string, string[]>(
        [],
        (values) => (value) => {
          values.push(value)
          return values
        },
        combined
      )
    ).toEqual(["l1", "l2", "l3"])
  })

  test("uses the selected Monoid empty value for empty collections", () => {
    expect(
      combine<ReadonlyArray<string>, string>(arrayReducible, stringMonoid, [])
    ).toBe("")
    expect(
      combine<ReadonlyArray<ReadonlyArray<never>>, ReadonlyArray<never>>(
        arrayReducible,
        arrayMonoid,
        []
      )
    ).toEqual([])
    expect(
      combine<List<never>, List<never>>(listReducible, listMonoid, Empty)
    ).toBe(Empty)
  })

  test("multiplies from one and preserves the empty product", () => {
    expect(
      product<ReadonlyArray<bigint>, bigint>(arrayReducible, intOne, intMul, [
        2n,
        3n,
        4n,
      ])
    ).toBe(24n)
    expect(
      product<ReadonlyArray<bigint>, bigint>(arrayReducible, intOne, intMul, [])
    ).toBe(1n)
  })

  test("short-circuits any and all with their documented empty results", () => {
    const anyObserved: number[] = []
    expect(
      any(
        arrayIterable,
        (value: number) => {
          anyObserved.push(value)
          return value === 2
        },
        [1, 2, 3]
      )
    ).toBe(true)
    expect(anyObserved).toEqual([1, 2])

    const allObserved: number[] = []
    expect(
      all(
        arrayIterable,
        (value: number) => {
          allObserved.push(value)
          return value < 2
        },
        [1, 2, 3]
      )
    ).toBe(false)
    expect(allObserved).toEqual([1, 2])
    expect(any(arrayIterable, () => true, [])).toBe(false)
    expect(all(arrayIterable, () => false, [])).toBe(true)
  })

  test("runs effectful traversal cold and in source order", async () => {
    const observed: number[] = []
    const traversal = forEach(
      arrayIterable,
      (value: number) => async () => {
        observed.push(value)
      },
      [1, 2, 3]
    )

    expect(observed).toEqual([])
    await traversal({})
    expect(observed).toEqual([1, 2, 3])
  })

  test("stops at the first typed effect failure", async () => {
    const observed: number[] = []
    const traversal = forEach(
      arrayIterable,
      (value: number) => async () => {
        observed.push(value)
        if (value === 2) throw new Error("stop")
      },
      [1, 2, 3]
    )

    expect(traversal({})).rejects.toThrow("stop")
    expect(observed).toEqual([1, 2])
  })
})
