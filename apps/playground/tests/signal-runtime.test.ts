import { describe, expect, test } from "bun:test"
import {
  combine,
  make,
  map,
  planSet,
  planUpdate,
  read,
  transaction,
} from "../../../runtime/ts/src/signal"

describe("Signal browser runtime", () => {
  test("publishes ordered transaction changes atomically", async () => {
    const source = await make(1)({})
    const doubled = map((value: number) => value * 2, source)
    const total = combine(
      (left: number) => (right: number) => left + right,
      doubled,
      source
    )

    const setThree = planSet(3)
    const increment = planUpdate((value: number) => value + 1)
    await transaction([setThree(source), increment(source)])({})

    expect(await read(source)({})).toBe(4)
    expect(await read(total)({})).toBe(12)
  })

  test("does not commit any staged value when planning fails", async () => {
    const left = await make(1)({})
    const right = await make(2)({})
    const broken = planUpdate<number>(() => {
      throw new Error("broken update")
    }, right)

    expect(() => transaction([planSet(10, left), broken])({})).toThrow(
      "broken update"
    )
    expect(await read(left)({})).toBe(1)
    expect(await read(right)({})).toBe(2)
  })
})
