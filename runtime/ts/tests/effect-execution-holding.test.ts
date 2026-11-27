import { expect, test } from "bun:test"
import {
  createEffectExecution,
  handOverExecution,
  registerResourceFinalizer,
} from "../src/effect"

test("cancel absorbs cleanup failures unless asked to report them", async () => {
  const absorbing = createEffectExecution()
  absorbing.context.onCancel(() => {
    throw new Error("cleanup failed")
  })
  await absorbing.cancel()

  const reporting = createEffectExecution(undefined, true)
  reporting.context.onCancel(() => {
    throw new Error("cleanup failed")
  })
  await expect(reporting.cancel()).rejects.toThrow("cleanup failed")
})

test("a reported cleanup failure comes with the release failures", async () => {
  const execution = createEffectExecution(undefined, true)
  registerResourceFinalizer(execution.context, () => {
    throw new Error("release failed")
  })
  execution.context.onCancel(() => {
    throw new Error("cleanup failed")
  })
  const failure = await execution.cancel().then(
    () => undefined,
    (error: unknown) => error
  )
  expect(failure).toBeInstanceOf(Error)
  expect((failure as Error).message).toBe("cleanup failed")
  const suppressed = (failure as { suppressed?: unknown[] }).suppressed ?? []
  expect(suppressed.map((error) => (error as Error).message)).toEqual([
    "release failed",
  ])
})

test("an execution holding nothing is left to its caller", async () => {
  const owner = createEffectExecution()
  const execution = createEffectExecution(owner.context)
  // A resource released early leaves an inactive entry, as `scoped` does.
  registerResourceFinalizer(execution.context, () => undefined).unregister()
  expect(handOverExecution(execution, owner.context)).toBe(false)
  await execution.close()
  await owner.close()
})

test("a resource an execution holds moves to its owner", async () => {
  const log: string[] = []
  const owner = createEffectExecution()
  registerResourceFinalizer(owner.context, () => {
    log.push("owner")
  })
  const execution = createEffectExecution(owner.context)
  registerResourceFinalizer(execution.context, () => {
    log.push("moved")
  })
  const released = registerResourceFinalizer(execution.context, () => {
    log.push("unregistered")
  })
  expect(handOverExecution(execution, owner.context)).toBe(true)
  // A moved resource can still be unregistered.
  released.unregister()
  expect(log).toEqual([])
  await owner.close()
  expect(log).toEqual(["moved", "owner"])
})

test("an execution holding a cleanup is kept until its owner closes", async () => {
  const owner = createEffectExecution()
  const execution = createEffectExecution(owner.context)
  let cancelled = false
  execution.context.onCancel(() => {
    cancelled = true
  })
  expect(handOverExecution(execution, owner.context)).toBe(true)
  await owner.cancel()
  expect(cancelled).toBe(true)
})

test("a moved finalizer may still register in its own scope", async () => {
  const log: string[] = []
  const owner = createEffectExecution()
  registerResourceFinalizer(owner.context, () => {
    log.push("owner")
  })
  const execution = createEffectExecution(owner.context)
  registerResourceFinalizer(execution.context, () => {
    log.push("release")
    registerResourceFinalizer(execution.context, () => {
      log.push("acquired-while-releasing")
    })
    log.push("used")
  })
  expect(handOverExecution(execution, owner.context)).toBe(true)
  await owner.close()
  expect(log).toEqual(["release", "used", "acquired-while-releasing", "owner"])
})
