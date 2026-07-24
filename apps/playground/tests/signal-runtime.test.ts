import { describe, expect, test } from "bun:test"
import {
  combine,
  constant,
  make,
  map,
  planSet,
  planUpdate,
  read,
  set,
  signalApplicative,
  signalFunctor,
  subscribe,
  switchMap,
  transaction,
  unsubscribe,
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

  test("notifies derived subscribers once after an atomic transaction", async () => {
    const left = await make(1)({})
    const right = await make(2)({})
    const total = combine(
      (left: number) => (right: number) => left + right,
      left,
      right
    )
    const observed: string[] = []
    const first = await subscribe(
      (value: number) => () => {
        observed.push(`first:${value}`)
        return undefined
      },
      total
    )({})
    const second = await subscribe(
      (value: number) => () => {
        observed.push(`second:${value}`)
        return undefined
      },
      total
    )({})

    await transaction([planSet(10, left), planSet(20, right)])({})

    expect(observed).toEqual(["first:3", "second:3", "first:30", "second:30"])
    await unsubscribe(first)({})
    await unsubscribe(second)({})
  })

  test("queues observer updates and unsubscribes idempotently", async () => {
    const source = await make(0)({})
    const observed: number[] = []
    const subscription = await subscribe(
      (value: number) => (environment) => {
        observed.push(value)
        return value === 1 ? set(2, source)(environment) : undefined
      },
      source
    )({})

    await set(1, source)({})
    expect(await read(source)({})).toBe(2)
    expect(observed).toEqual([0, 1, 2])

    await unsubscribe(subscription)({})
    await unsubscribe(subscription)({})
    await set(3, source)({})
    expect(observed).toEqual([0, 1, 2])
  })

  test("stops a defective observer after notifying the others", async () => {
    const source = await make(0)({})
    let defects = 0
    const healthyValues: number[] = []
    await subscribe(
      (value: number) => () => {
        if (value > 0) {
          defects += 1
          throw new Error("observer defect")
        }
        return undefined
      },
      source
    )({})
    const healthy = await subscribe(
      (value: number) => () => {
        healthyValues.push(value)
        return undefined
      },
      source
    )({})

    await expect(set(1, source)({})).rejects.toThrow("observer defect")
    await set(2, source)({})

    expect(defects).toBe(1)
    expect(healthyValues).toEqual([0, 1, 2])
    await unsubscribe(healthy)({})
  })

  test("switches dynamic dependencies without observing the old branch", async () => {
    const chooseLeft = await make(true)({})
    const left = await make(10)({})
    const right = await make(20)({})
    let selections = 0
    const selected = switchMap((useLeft: boolean) => {
      selections += 1
      return useLeft ? left : right
    }, chooseLeft)
    const observed: number[] = []
    const subscription = await subscribe(
      (value: number) => () => {
        observed.push(value)
        return undefined
      },
      selected
    )({})

    await set(21, right)({})
    await set(false, chooseLeft)({})
    await set(11, left)({})
    await set(22, right)({})

    expect(selections).toBe(2)
    expect(observed).toEqual([10, 21, 22])
    await unsubscribe(subscription)({})
  })

  test("retains local feature state while its branch is hidden", async () => {
    const visible = await make(true)({})
    const child = await make(0)({})
    const selected = switchMap(
      (show: boolean) => (show ? child : constant(-1)),
      visible
    )
    const observed: number[] = []
    const subscription = await subscribe(
      (value: number) => () => {
        observed.push(value)
        return undefined
      },
      selected
    )({})

    await set(1, child)({})
    await set(false, visible)({})
    await set(2, child)({})
    await set(true, visible)({})

    expect(observed).toEqual([0, 1, -1, 2])
    await unsubscribe(subscription)({})
  })

  test("keeps feature state attached to constructor bindings when order changes", async () => {
    const reversed = await make(false)({})
    const first = await make(0)({})
    const second = await make(10)({})
    const pair = combine(
      (firstValue: number) => (secondValue: number) => ({
        first: firstValue,
        second: secondValue,
      }),
      first,
      second
    )
    const ordered = combine(
      (isReversed: boolean) =>
        (values: { readonly first: number; readonly second: number }) =>
          isReversed
            ? `B:${values.second}|A:${values.first}`
            : `A:${values.first}|B:${values.second}`,
      reversed,
      pair
    )
    const observed: string[] = []
    const subscription = await subscribe(
      (value: string) => () => {
        observed.push(value)
        return undefined
      },
      ordered
    )({})

    await set(1, first)({})
    await set(true, reversed)({})
    await set(11, second)({})

    expect(observed).toEqual(["A:0|B:10", "A:1|B:10", "B:10|A:1", "B:11|A:1"])
    await unsubscribe(subscription)({})
  })

  test("publishes nested switches when dependency revisions collide", async () => {
    const chooseLeft = await make(true)({})
    const left = await make(10)({})
    const right = await make(20)({})
    await set(11, left)({})

    const selected = switchMap(
      (useLeft: boolean) => (useLeft ? left : right),
      chooseLeft
    )
    const nested = switchMap((value: number) => constant(value * 2), selected)
    const observed: number[] = []
    const subscription = await subscribe(
      (value: number) => () => {
        observed.push(value)
        return undefined
      },
      nested
    )({})

    await set(false, chooseLeft)({})

    expect(observed).toEqual([22, 40])
    await unsubscribe(subscription)({})
  })

  test("provides Functor and Applicative dictionaries", async () => {
    const source = await make(20)({})
    const doubled = signalFunctor.map((value: number) => value * 2)(source)
    const functions = constant((value: number) => value + 2)
    const answer = signalApplicative.apply(functions)(doubled)

    expect(await read(answer)({})).toBe(42)
  })
})
