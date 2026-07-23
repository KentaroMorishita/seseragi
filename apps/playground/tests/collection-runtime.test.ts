import { describe, expect, test } from "bun:test"
import {
  append as appendArray,
  arrayIterable,
  arrayMonoid,
  arrayReducible,
  concat as concatArray,
  drop as dropArray,
  filter as filterArray,
  filterMap as filterMapArray,
  find as findArray,
  flatMap as flatMapArray,
  get as getArray,
  head as headArray,
  isEmpty as isEmptyArray,
  length as lengthArray,
  reverse as reverseArray,
  take as takeArray,
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
  append as appendList,
  concat as concatList,
  Empty,
  drop as dropList,
  filter as filterList,
  filterMap as filterMapList,
  find as findList,
  flatMap as flatMapList,
  fromArray,
  get as getList,
  head as headList,
  isEmpty as isEmptyList,
  length as lengthList,
  type List,
  listMonoid,
  listReducible,
  reduce as reduceList,
  reverse as reverseList,
  take as takeList,
  tail as tailList,
} from "../../../runtime/ts/src/list"
import { stringMonoid } from "../../../runtime/ts/src/string"
import { Just, Nothing } from "../../../runtime/ts/src/sum"

describe("Collection runtime", () => {
  test("appends, concatenates, and reverses Array values without mutation", () => {
    const values = [1, 2]
    const suffix = [3, 4]
    expect(appendArray(suffix, values)).toEqual([1, 2, 3, 4])
    expect(values).toEqual([1, 2])
    expect(suffix).toEqual([3, 4])
    expect(concatArray([[1, 2], [], [3]])).toEqual([1, 2, 3])
    expect(concatArray([])).toEqual([])
    expect(reverseArray([1, 2, 3])).toEqual([3, 2, 1])
    expect(reverseArray([])).toEqual([])
  })

  test("appends, concatenates, and reverses persistent List values", () => {
    const values = fromArray([1, 2])
    const suffix = fromArray([3, 4])
    const appended = appendList(suffix, values)
    expect(appended).toEqual(fromArray([1, 2, 3, 4]))
    expect(dropList(2n, appended)).toBe(suffix)
    expect(
      concatList(fromArray([fromArray([1, 2]), Empty, fromArray([3])]))
    ).toEqual(fromArray([1, 2, 3]))
    expect(concatList(Empty)).toBe(Empty)
    expect(reverseList(fromArray([1, 2, 3]))).toEqual(fromArray([3, 2, 1]))
    expect(reverseList(Empty)).toBe(Empty)
  })

  test("finds and slices Array values with documented count boundaries", () => {
    const observed: number[] = []
    expect(
      findArray(
        (value: number) => {
          observed.push(value)
          return value === 2
        },
        [1, 2, 3]
      )
    ).toEqual(Just(2))
    expect(observed).toEqual([1, 2])
    expect(findArray(() => true, [])).toBe(Nothing)

    expect(takeArray(-1n, [1, 2, 3])).toEqual([])
    expect(takeArray(0n, [1, 2, 3])).toEqual([])
    expect(takeArray(2n, [1, 2, 3])).toEqual([1, 2])
    expect(takeArray(20n, [1, 2, 3])).toEqual([1, 2, 3])
    expect(dropArray(-1n, [1, 2, 3])).toEqual([1, 2, 3])
    expect(dropArray(0n, [1, 2, 3])).toEqual([1, 2, 3])
    expect(dropArray(2n, [1, 2, 3])).toEqual([3])
    expect(dropArray(20n, [1, 2, 3])).toEqual([])
  })

  test("finds and slices persistent List values with shared suffixes", () => {
    const values = fromArray([1, 2, 3])
    expect(findList((value) => value === 2, values)).toEqual(Just(2))
    expect(findList(() => true, Empty)).toBe(Nothing)

    expect(takeList(-1n, values)).toBe(Empty)
    expect(takeList(0n, values)).toBe(Empty)
    expect(takeList(2n, values)).toEqual(fromArray([1, 2]))
    expect(takeList(20n, values)).toEqual(values)
    expect(dropList(-1n, values)).toBe(values)
    expect(dropList(0n, values)).toBe(values)
    expect(dropList(2n, values)).toBe(
      values.tag === "Cons" && values.tail.tag === "Cons"
        ? values.tail.tail
        : Empty
    )
    expect(dropList(20n, values)).toBe(Empty)
  })

  test("transforms Array values in source order", () => {
    const observed: number[] = []
    expect(
      filterArray(
        (value) => {
          observed.push(value)
          return value % 2 === 0
        },
        [1, 2, 3, 4]
      )
    ).toEqual([2, 4])
    expect(observed).toEqual([1, 2, 3, 4])
    expect(filterArray(() => true, [])).toEqual([])
    expect(
      filterMapArray(
        (value: number) => (value % 2 === 0 ? Just(`#${value}`) : Nothing),
        [1, 2, 3]
      )
    ).toEqual(["#2"])
    expect(flatMapArray((value: number) => [value, -value], [1, 2])).toEqual([
      1, -1, 2, -2,
    ])
  })

  test("transforms persistent List values in source order", () => {
    const values = fromArray([1, 2, 3, 4])
    expect(filterList((value) => value % 2 === 0, values)).toEqual(
      fromArray([2, 4])
    )
    expect(filterList(() => true, Empty)).toBe(Empty)
    expect(
      filterMapList(
        (value: number) => (value % 2 === 0 ? Just(`#${value}`) : Nothing),
        values
      )
    ).toEqual(fromArray(["#2", "#4"]))
    expect(
      flatMapList(
        (value: number) => fromArray([value, -value]),
        fromArray([1, 2])
      )
    ).toEqual(fromArray([1, -1, 2, -2]))
  })

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
