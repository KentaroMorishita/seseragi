import { expect, test } from "bun:test"
import {
  createProgramApplication,
  createProgramLifetime,
  installProgramReload,
  loadOwnedProgram,
  runProgramApplication,
} from "../src/browser/program-lifecycle.js"

function program(failClose = false) {
  let closed = false
  let closes = 0
  return {
    exports: {
      call: () => {
        if (closed) throw new Error("closed callback")
        return 42
      },
    },
    dispose() {
      closed = true
      closes++
      if (failClose) throw new Error("close defect")
    },
    ownership: () => ({ alive: !closed, closes }),
  }
}

const closedError = () => new Error("closed lifetime")

test("reload closes every owner once and invalidates saved callbacks", async () => {
  const lifetime = createProgramLifetime()
  const first = await lifetime.load(
    () => Promise.resolve(program()),
    closedError
  )
  const second = await lifetime.load(
    () => Promise.resolve(program()),
    closedError
  )
  const saved = second.exports.call
  expect(saved()).toBe(42)
  first.dispose()
  const closing = lifetime.close()
  expect(first.ownership()).toEqual({ alive: false, closes: 1 })
  expect(second.ownership()).toEqual({ alive: false, closes: 1 })
  expect(saved).toThrow("closed callback")
  expect(lifetime.close()).toBe(closing)
  await closing
  second.dispose()
  expect(second.ownership().closes).toBe(1)
})

test("reload waits for an in-flight instance and closes it before settling", async () => {
  const lifetime = createProgramLifetime()
  const created = program()
  const result = Promise.withResolvers<ReturnType<typeof program>>()
  let started = false
  const loading = lifetime.load(() => {
    started = true
    return result.promise
  }, closedError)
  await Promise.resolve()
  expect(started).toBe(true)
  let finished = false
  const closing = lifetime.close().then(() => {
    finished = true
  })
  await Promise.resolve()
  expect(finished).toBe(false)
  result.resolve(created)
  await expect(loading).rejects.toThrow("closed lifetime")
  await closing
  expect(created.ownership()).toEqual({ alive: false, closes: 1 })
})

test("closed lifetime refuses the factory before instantiation", async () => {
  const lifetime = createProgramLifetime()
  await lifetime.close()
  let calls = 0
  await expect(
    lifetime.load(() => {
      calls++
      return Promise.resolve(program())
    }, closedError)
  ).rejects.toThrow("closed lifetime")
  expect(calls).toBe(0)
})

test("the factory starts once at the call before its asynchronous result", async () => {
  const lifetime = createProgramLifetime()
  let calls = 0
  const created = program()
  const loading = lifetime.load(() => {
    calls++
    return Promise.resolve(created)
  }, closedError)
  expect(calls).toBe(1)
  const closing = lifetime.close()
  await expect(loading).rejects.toThrow("closed lifetime")
  await closing
  expect(calls).toBe(1)
  expect(created.ownership()).toEqual({ alive: false, closes: 1 })
})

test("one disposer defect does not abandon another owner", async () => {
  const lifetime = createProgramLifetime()
  const failed = await lifetime.load(
    () => Promise.resolve(program(true)),
    closedError
  )
  const other = await lifetime.load(
    () => Promise.resolve(program()),
    closedError
  )
  await expect(lifetime.close()).rejects.toThrow("program disposal failed")
  expect(failed.ownership()).toEqual({ alive: false, closes: 1 })
  expect(other.ownership()).toEqual({ alive: false, closes: 1 })
})

test("an initialization rejection does not poison closing another instance", async () => {
  const lifetime = createProgramLifetime()
  const instance = await lifetime.load(
    () => Promise.resolve(program()),
    closedError
  )
  await expect(
    lifetime.load(() => Promise.reject(new Error("init defect")), closedError)
  ).rejects.toThrow("init defect")
  await lifetime.close()
  expect(instance.ownership()).toEqual({ alive: false, closes: 1 })
})

test("application cancellation drains UI finalizers before closing its programs", async () => {
  const lifetime = createProgramLifetime()
  const application = createProgramApplication(lifetime)
  const instance = await lifetime.load(
    () => Promise.resolve(program()),
    closedError
  )
  const finalizer = Promise.withResolvers<void>()
  const body = Promise.withResolvers<void>()
  const events: string[] = []
  const running = application.run(async (frame) => {
    frame.onShutdown(async () => {
      events.push("cancel")
      body.resolve()
      await finalizer.promise
      expect(instance.exports.call()).toBe(42)
      events.push("finalizer")
    })
    frame.cleanup(() => {
      expect(instance.exports.call()).toBe(42)
      events.push("execution")
    })
    frame.cleanup(() => {
      events.push("dom")
    })
    frame.cleanup(() => {
      events.push("providers")
    })
    await body.promise
  })
  const closing = application.close()
  expect(application.close()).toBe(closing)
  await Promise.resolve()
  expect(events).toEqual(["cancel"])
  expect(instance.ownership().alive).toBe(true)
  finalizer.resolve()
  await Promise.all([running, closing])
  expect(events).toEqual([
    "cancel",
    "finalizer",
    "execution",
    "dom",
    "providers",
  ])
  expect(instance.ownership()).toEqual({ alive: false, closes: 1 })
})

test("a reload during startup waits for initialized services and cancels a late execution", async () => {
  const lifetime = createProgramLifetime()
  const application = createProgramApplication(lifetime)
  const imported = Promise.withResolvers<void>()
  const events: string[] = []
  const running = application.run(async (frame) => {
    await imported.promise
    frame.onShutdown(() => {
      events.push("cancel")
    })
    frame.cleanup(() => {
      events.push("execution")
    })
    expect(frame.closing).toBe(true)
    if (frame.closing) return
    events.push("main")
  })
  let finished = false
  const closing = application.close().then(() => {
    finished = true
  })
  await Promise.resolve()
  expect(finished).toBe(false)
  imported.resolve()
  await Promise.all([running, closing])
  expect(events).toEqual(["cancel", "execution"])
})

test("startup failure and cleanup defects remain observable after every cleanup", async () => {
  const lifetime = createProgramLifetime()
  const application = createProgramApplication(lifetime)
  const instance = await lifetime.load(
    () => Promise.resolve(program()),
    closedError
  )
  const primary = new Error("startup failure")
  const cleanup = new Error("execution close defect")
  let cleaned = 0
  const running = application.run(async (frame) => {
    frame.cleanup(() => {
      throw cleanup
    })
    frame.cleanup(() => {
      cleaned++
    })
    throw primary
  })
  const error = await running.catch((failure: unknown) => failure)
  expect(error).toBeInstanceOf(AggregateError)
  expect((error as AggregateError).errors).toEqual([primary, cleanup])
  expect(cleaned).toBe(1)
  expect(instance.ownership()).toEqual({ alive: false, closes: 1 })
  const closeError = await application
    .close()
    .catch((failure: unknown) => failure)
  expect(closeError).toBeInstanceOf(AggregateError)
  expect((closeError as AggregateError).errors).toEqual([cleanup])
})

test("the dev event waits for UI finalizers and late programs before one navigation", async () => {
  const current = await loadOwnedProgram(
    () => Promise.resolve(program()),
    closedError
  )
  const created = program()
  const result = Promise.withResolvers<ReturnType<typeof program>>()
  const started = Promise.withResolvers<void>()
  const loading = loadOwnedProgram(() => {
    started.resolve()
    return result.promise
  }, closedError)
  await started.promise
  const body = Promise.withResolvers<void>()
  const finalizer = Promise.withResolvers<void>()
  let finalized = false
  const running = runProgramApplication(async (frame) => {
    frame.onShutdown(async () => {
      await finalizer.promise
      expect(current.exports.call()).toBe(42)
      finalized = true
      body.resolve()
    })
    await body.promise
  })
  let reloads = 0
  class Host extends EventTarget {
    location = {
      reload() {
        expect(created.ownership()).toEqual({ alive: false, closes: 1 })
        expect(current.ownership()).toEqual({ alive: false, closes: 1 })
        expect(finalized).toBe(true)
        reloads++
      },
    }
  }
  const host = new Host()
  installProgramReload(host)
  installProgramReload(host)
  expect(
    host.dispatchEvent(
      new Event("seseragi:before-reload", { cancelable: true })
    )
  ).toBe(false)
  expect(
    host.dispatchEvent(
      new Event("seseragi:before-reload", { cancelable: true })
    )
  ).toBe(false)
  expect(reloads).toBe(0)
  finalizer.resolve()
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(reloads).toBe(0)
  result.resolve(created)
  await expect(loading).rejects.toThrow("closed lifetime")
  await running
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(reloads).toBe(1)
  host.dispatchEvent(new Event("pagehide"))
  await Promise.resolve()
  expect(created.ownership().closes).toBe(1)
})
