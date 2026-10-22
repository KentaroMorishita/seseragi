import { expect, test } from "bun:test"
import { createProgramLifetime } from "../src/browser/program-lifecycle.js"

const closed = () => new Error("closed lifetime")

test("controlled reload waits for Effect cancellation and finalizers", async () => {
  const lifetime = createProgramLifetime()
  const finish = Promise.withResolvers<void>()
  const trace: string[] = []
  await lifetime.load(
    async () => ({
      exports: {},
      dispose() {
        trace.push("cancel")
        return finish.promise.then(() => {
          trace.push("released")
        })
      },
      ownership: () => ({}),
    }),
    closed
  )
  let completed = false
  const closing = lifetime.close().then(() => {
    completed = true
  })
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(trace).toEqual(["cancel"])
  expect(completed).toBe(false)
  finish.resolve()
  await closing
  expect(trace).toEqual(["cancel", "released"])
  expect(completed).toBe(true)
})

test("an Effect instance created during close drains before load and close settle", async () => {
  const lifetime = createProgramLifetime()
  const create = Promise.withResolvers<{
    exports: object
    dispose(): Promise<void>
    ownership(): object
  }>()
  const finish = Promise.withResolvers<void>()
  const loading = lifetime.load(() => create.promise, closed)
  // Attach the rejection observer before allowing asynchronous construction.
  let loadSettled = false
  const observed = loading.catch((error: Error) => {
    loadSettled = true
    expect(error.message).toBe("closed lifetime")
  })
  let closeSettled = false
  const closing = lifetime.close().then(() => {
    closeSettled = true
  })
  create.resolve({
    exports: {},
    dispose: () => finish.promise,
    ownership: () => ({}),
  })
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(loadSettled).toBe(false)
  expect(closeSettled).toBe(false)
  finish.resolve()
  await Promise.all([observed, closing])
  expect(loadSettled).toBe(true)
  expect(closeSettled).toBe(true)
})

test("manual asynchronous disposal remains owned until release finishes", async () => {
  const lifetime = createProgramLifetime()
  const finish = Promise.withResolvers<void>()
  let calls = 0
  const owner = await lifetime.load(
    async () => ({
      exports: {},
      dispose() {
        calls++
        return finish.promise
      },
      ownership: () => ({}),
    }),
    closed
  )
  const manual = owner.dispose()
  expect(owner.dispose()).toBe(manual)
  let finished = false
  const closing = lifetime.close().then(() => {
    finished = true
  })
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(finished).toBe(false)
  finish.resolve()
  await Promise.all([manual, closing])
  expect(calls).toBe(1)
})

test("one asynchronous cleanup failure waits for the other owner", async () => {
  const lifetime = createProgramLifetime()
  const finish = Promise.withResolvers<void>()
  await lifetime.load(
    async () => ({
      exports: {},
      dispose: () => Promise.reject(new Error("release defect")),
      ownership: () => ({}),
    }),
    closed
  )
  await lifetime.load(
    async () => ({
      exports: {},
      dispose: () => finish.promise,
      ownership: () => ({}),
    }),
    closed
  )
  let settled = false
  const observed = lifetime.close().catch((error: AggregateError) => {
    settled = true
    expect(error.errors.map((error: Error) => error.message)).toEqual([
      "release defect",
    ])
  })
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(settled).toBe(false)
  finish.resolve()
  await observed
  expect(settled).toBe(true)
})
