import { expect, test } from "bun:test"
import {
  acquireRelease,
  awaitEffectValue,
  awaitFiber,
  createEffectExecution,
  type Effect,
  fork,
  interrupt,
  scoped,
  succeed,
  yieldNow,
} from "../src/effect"

function gate<Value>() {
  let resolve = (_value: Value): void => {
    throw new Error("gate was not initialized")
  }
  const promise = new Promise<Value>((ready) => {
    resolve = ready
  })
  return { promise, resolve }
}

test("interrupt waits for a closing lexical scope and its masked finalizer", async () => {
  const execution = createEffectExecution()
  const acquired = gate<void>()
  const releaseStarted = gate<void>()
  const finish = gate<void>()
  const blocked = gate<void>()
  const trace: string[] = []
  const body: Effect<unknown, never, undefined> = async (
    environment,
    context
  ) => {
    await acquireRelease(succeed("resource"), () => async (_env, masked) => {
      expect(masked?.cancelled).toBe(false)
      trace.push("release-start")
      releaseStarted.resolve()
      await finish.promise
      trace.push("release-end")
    })(environment, context)
    acquired.resolve()
    await awaitEffectValue(blocked.promise, context ?? execution.context)
    return undefined
  }
  const fiber = await fork(scoped(body))({}, execution.context)
  await acquired.promise
  let returned = false
  const stopped = Promise.resolve(interrupt(fiber)({}, execution.context)).then(
    () => {
      returned = true
      trace.push("interrupted")
    }
  )
  try {
    await releaseStarted.promise
    await yieldNow()({}, execution.context)
    await yieldNow()({}, execution.context)
    expect(returned).toBe(false)
    finish.resolve()
    await stopped
    expect(trace).toEqual(["release-start", "release-end", "interrupted"])
    expect(await awaitFiber(fiber)({}, execution.context)).toEqual({
      tag: "FiberCancelled",
    })
    await execution.close()
  } finally {
    blocked.resolve()
    finish.resolve()
    await stopped.catch(() => undefined)
    await execution.close().catch(() => undefined)
  }
})

test("interrupt supervises an unavailable acquisition through late handoff and release", async () => {
  const execution = createEffectExecution()
  const acquireStarted = gate<void>()
  const acquired = gate<string>()
  const releaseStarted = gate<void>()
  const finish = gate<void>()
  let used = false
  let releases = 0
  const acquire: Effect<unknown, never, string> = () => {
    acquireStarted.resolve()
    return acquired.promise
  }
  const body: Effect<unknown, never, undefined> = async (
    environment,
    context
  ) => {
    await acquireRelease(acquire, (resource) => async (_env, masked) => {
      expect(resource).toBe("late-resource")
      expect(masked?.cancelled).toBe(false)
      releases += 1
      releaseStarted.resolve()
      await finish.promise
    })(environment, context)
    used = true
    return undefined
  }
  const fiber = await fork(scoped(body))({}, execution.context)
  await acquireStarted.promise
  let returned = false
  const stopped = Promise.resolve(interrupt(fiber)({}, execution.context)).then(
    () => {
      returned = true
    }
  )
  try {
    await yieldNow()({}, execution.context)
    await yieldNow()({}, execution.context)
    expect(returned).toBe(false)
    acquired.resolve("late-resource")
    await releaseStarted.promise
    expect(returned).toBe(false)
    finish.resolve()
    await stopped
    expect(used).toBe(false)
    expect(releases).toBe(1)
    await interrupt(fiber)({}, execution.context)
    expect(releases).toBe(1)
    await execution.close()
  } finally {
    acquired.resolve("late-resource")
    finish.resolve()
    await stopped.catch(() => undefined)
    await execution.close().catch(() => undefined)
  }
})

test("a finalizer defect is observed by interrupt after lexical cleanup", async () => {
  const execution = createEffectExecution()
  const acquired = gate<void>()
  const releaseStarted = gate<void>()
  const finish = gate<void>()
  const blocked = gate<void>()
  const defect = new Error("held cleanup defect")
  const body: Effect<unknown, never, undefined> = async (
    environment,
    context
  ) => {
    await acquireRelease(succeed("resource"), () => async () => {
      releaseStarted.resolve()
      await finish.promise
      throw defect
    })(environment, context)
    acquired.resolve()
    await awaitEffectValue(blocked.promise, context ?? execution.context)
    return undefined
  }
  const fiber = await fork(scoped(body))({}, execution.context)
  await acquired.promise
  const stopped = Promise.resolve(interrupt(fiber)({}, execution.context)).then(
    () => ({ kind: "success" as const }),
    (error: unknown) => ({ kind: "defect" as const, error })
  )
  try {
    await releaseStarted.promise
    finish.resolve()
    const result = await stopped
    expect(result.kind).toBe("defect")
    if (result.kind === "defect") expect(result.error).toBe(defect)
  } finally {
    blocked.resolve()
    finish.resolve()
    await stopped
    await execution.close().catch(() => undefined)
  }
})
