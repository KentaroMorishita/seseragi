import { describe, expect, test } from "bun:test"
import { arrayIterable } from "../../../runtime/ts/src/array"
import { forEach } from "../../../runtime/ts/src/collection"

describe("Collection runtime", () => {
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
