import { expect, test } from "bun:test"
import {
  acquireRelease,
  attempt,
  die,
  fail,
  flatMap,
  mapError,
  recover,
  run,
  scoped,
  succeed,
} from "../src/effect"

test("die saves its argument once and creates a fresh cold Defect per run", async () => {
  let calls = 0
  const message = () => {
    calls++
    return "expected Defect"
  }
  const work = die(message())
  expect(calls).toBe(1)
  const defects: unknown[] = []
  for (let index = 0; index < 2; index++) {
    try {
      await run(work, {})
      throw new Error("die returned a result")
    } catch (error) {
      expect(error).toBeInstanceOf(Error)
      expect((error as Error).message).toBe("expected Defect")
      defects.push(error)
    }
  }
  expect(calls).toBe(1)
  expect(defects[0]).not.toBe(defects[1])
})

test("typed failure adapters do not catch or map die", async () => {
  let handled = 0
  let mapped = 0
  const work = die("uncatchable")
  await expect(
    run(
      recover(() => {
        handled++
        return succeed("recovered")
      }, work),
      {}
    )
  ).rejects.toThrow("uncatchable")
  await expect(
    run(
      mapError(() => {
        mapped++
        return "mapped"
      }, work),
      {}
    )
  ).rejects.toThrow("uncatchable")
  await expect(run(attempt(work), {})).rejects.toThrow("uncatchable")
  expect(handled).toBe(0)
  expect(mapped).toBe(0)
})

test("die waits for finalizers before the Defect leaves its scope", async () => {
  const events: string[] = []
  const work = scoped(
    flatMap(
      acquireRelease(succeed("resource"), () => () => {
        events.push("release")
      }),
      () => die("body")
    )
  )
  expect(events).toEqual([])
  await expect(run(work, {})).rejects.toThrow("body")
  expect(events).toEqual(["release"])
})

test("finalizer die preserves LIFO, attempts remaining cleanup and overrides failure", async () => {
  const events: string[] = []
  const resource = (name: string) =>
    acquireRelease(succeed(name), () =>
      flatMap(succeed(undefined), () => {
        events.push(name)
        return die(name)
      })
    )
  const work = scoped(
    flatMap(resource("outer"), () =>
      flatMap(resource("inner"), () => fail("typed failure"))
    )
  )
  let defect: unknown
  try {
    await run(work, {})
    throw new Error("cleanup Defect was lost")
  } catch (error) {
    defect = error
  }
  expect(events).toEqual(["inner", "outer"])
  expect(defect).toBeInstanceOf(Error)
  expect((defect as Error).message).toBe("inner")
  const suppressed = Object.getOwnPropertyDescriptor(
    defect,
    "suppressed"
  )?.value
  expect(suppressed).toHaveLength(1)
  expect(suppressed[0]).toBeInstanceOf(Error)
  expect(suppressed[0].message).toBe("outer")
})
