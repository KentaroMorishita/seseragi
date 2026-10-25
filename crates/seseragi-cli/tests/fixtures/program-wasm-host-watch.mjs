import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"

const { cleanupDefect } = JSON.parse(
  await readFile(new URL("./watch-case.json", import.meta.url), "utf8")
)
const oldModule = await import(
  new URL("./before/programs/calculation/index.mjs", import.meta.url)
)
const lifecycle = await import(
  new URL("./before/programs/__lifecycle.mjs", import.meta.url)
)
const oldBytes = await readFile(
  new URL("./before/programs/calculation/program.wasm", import.meta.url)
)
const success = (value) => ({ tag: "Success", value })
const tick = () => new Promise((resolve) => setTimeout(resolve, 0))
function gate() {
  let resolve
  const promise = new Promise((complete) => {
    resolve = complete
  })
  return { promise, resolve }
}
async function within(promise, label) {
  let timer
  try {
    return await Promise.race([
      promise,
      new Promise((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(`${label} timed out`)), 2000)
      }),
    ])
  } finally {
    clearTimeout(timer)
  }
}
async function knownResult(owner, expected) {
  const first = owner.exports.transfer((action) =>
    success(action.startsWith("open:") ? "record" : "result")
  )
  const second = first(() => success(undefined))
  const cold = second("value")
  first.dispose()
  second.dispose()
  try {
    assert.deepEqual(await within(cold.run().result, "known result"), {
      tag: "Success",
      value: expected,
    })
  } finally {
    cold.dispose()
  }
  assert.equal(owner.ownership().hostCallbacks, 0)
}
function messages(error) {
  return [error?.message, ...(error?.errors ?? []).flatMap(messages)]
}

const readEntered = gate()
const readDone = gate()
const aborted = gate()
const releaseEntered = gate()
const releaseDone = gate()
const uiEntered = gate()
const uiDone = gate()
const appStarted = gate()
const appStopped = gate()
const navigated = gate()
const reported = gate()
const resources = new Set()
const trace = []
const errors = []
const originalError = console.error
let oldOwner
let newOwner
let saved
let cold
let running
let applicationResult
let readSignal
let releaseSignal
let shutdowns = 0
let reloads = 0

try {
  oldOwner = await oldModule.loadProgram(oldBytes)
  await knownResult(oldOwner, "processed:result")
  const first = oldOwner.exports.transfer((action, { signal }) => {
    trace.push(action)
    if (action.startsWith("open:")) {
      resources.add("record")
      return success("record")
    }
    assert.equal(action, "read:record")
    readSignal = signal
    signal.addEventListener("abort", () => aborted.resolve(), { once: true })
    readEntered.resolve()
    return readDone.promise
  })
  saved = first(async (record, { signal }) => {
    assert.equal(record, "record")
    releaseSignal = signal
    assert.equal(signal.aborted, false)
    trace.push("release:start")
    releaseEntered.resolve()
    await releaseDone.promise
    assert.equal(signal.aborted, false)
    assert(resources.delete(record))
    trace.push("release:end")
    return cleanupDefect
      ? { tag: "Defect", messages: ["watch release defect"] }
      : success(undefined)
  })
  first.dispose()
  cold = saved("watch")
  assert.deepEqual(trace, [])
  running = cold.run()
  await within(readEntered.promise, "old host read")

  applicationResult = lifecycle
    .runProgramApplication(async (frame) => {
      frame.onShutdown(() => {
        shutdowns++
        trace.push("application:stop")
        appStopped.resolve()
      })
      frame.cleanup(async () => {
        trace.push("ui:start")
        // UI cleanup runs before the program lifetime closes. Same-instance
        // reentry while a host Promise is active remains outside this ABI.
        const before = trace.length
        assert.throws(
          () => saved("UI finalizer"),
          (error) =>
            error instanceof oldModule.ProgramWasmClosed &&
            error.message ===
              "Same-instance host callback reentry is unsupported"
        )
        assert.equal(trace.length, before)
        uiEntered.resolve()
        await uiDone.promise
        trace.push("ui:end")
      })
      appStarted.resolve()
      await appStopped.promise
    })
    .then(
      () => ({ ok: true }),
      (error) => ({ ok: false, error })
    )
  await within(appStarted.promise, "old application")
  console.error = (...arguments_) => {
    errors.push(arguments_)
    reported.resolve()
  }
  const host = new EventTarget()
  lifecycle.installProgramReload(host, () => {
    reloads++
    trace.push("navigate")
    navigated.resolve()
  })
  for (let i = 0; i < 2; i++) {
    const event = new Event("seseragi:before-reload", { cancelable: true })
    assert.equal(host.dispatchEvent(event), false)
  }
  await within(uiEntered.promise, "UI finalizer")
  assert.equal(shutdowns, 1)
  assert.equal(readSignal.aborted, false)
  assert.equal(oldOwner.ownership().closing, false)
  assert.equal(reloads, 0)
  uiDone.resolve()
  await within(aborted.promise, "host read cancellation")
  assert.equal(oldOwner.ownership().closing, true)
  assert.equal(oldOwner.ownership().waitingHost, true)
  assert.equal(reloads, 0)
  await tick()
  assert(!trace.includes("release:start"))
  // Abort acknowledgement precedes release; release is masked until it ends.
  readDone.resolve(success("late host acknowledgement"))
  await within(releaseEntered.promise, "masked host release")
  assert.equal(releaseSignal.aborted, false)
  assert.equal(resources.size, 1)
  assert.equal(reloads, 0)
  await tick()
  assert.equal(reloads, 0)
  releaseDone.resolve()
  assert.deepEqual(
    await within(running.result, "cancelled host result"),
    cleanupDefect
      ? { tag: "Defect", messages: ["watch release defect"] }
      : { tag: "Cancelled" }
  )
  const application = await within(applicationResult, "application cleanup")
  if (cleanupDefect) {
    await within(reported.promise, "reload cleanup error")
    assert.equal(application.ok, false)
    assert(messages(application.error).includes("watch release defect"))
    assert.equal(errors.length, 1)
    assert.equal(errors[0][0], "seseragi: program cleanup failed before reload")
    assert(messages(errors[0][1]).includes("watch release defect"))
    assert.equal(reloads, 0)
  } else {
    await within(navigated.promise, "completed navigation")
    assert.equal(application.ok, true)
    assert.equal(reloads, 1)
    assert.deepEqual(errors, [])
  }
  assert.deepEqual(trace, [
    "open:watch",
    "read:record",
    "application:stop",
    "ui:start",
    "ui:end",
    "release:start",
    "release:end",
    ...(!cleanupDefect ? ["navigate"] : []),
  ])
  assert.equal(resources.size, 0)
  assert.throws(() => saved("stale"), oldModule.ProgramWasmClosed)
  assert.throws(() => cold.run(), oldModule.ProgramWasmClosed)
  assert.deepEqual(oldOwner.ownership(), {
    alive: false,
    retired: false,
    callbacks: 0,
    plans: 0,
    running: false,
    closing: true,
    managedLiveAtClose: 0,
    hostCallbacks: 0,
    waitingHost: false,
  })

  // The next completed dev image has an independent page/program lifetime.
  const next = await import(
    new URL("./after/programs/calculation/index.mjs", import.meta.url)
  )
  newOwner = await next.loadProgram(
    await readFile(
      new URL("./after/programs/calculation/program.wasm", import.meta.url)
    )
  )
  await knownResult(newOwner, "watched:result")
  await newOwner.dispose()
  assert.equal(newOwner.ownership().managedLiveAtClose, 0)
  assert.equal(newOwner.ownership().hostCallbacks, 0)
  console.log("host watch event: old cleanup drained; next image verified")
} finally {
  // Gates and tracked program owners are released even after a failed assert.
  uiDone.resolve()
  appStopped.resolve()
  readDone.resolve(success("cleanup"))
  releaseDone.resolve()
  try {
    await within(
      Promise.allSettled([
        applicationResult,
        lifecycle.closeProgramApplication(),
        oldOwner?.dispose(),
        newOwner?.dispose(),
      ]),
      "owned watch fixture cleanup"
    )
  } finally {
    console.error = originalError
  }
}
