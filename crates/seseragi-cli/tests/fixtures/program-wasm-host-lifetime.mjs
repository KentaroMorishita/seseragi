import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { pathToFileURL } from "node:url"

const site = process.argv[2]
const { loadProgram, ProgramWasmClosed } = await import(
  pathToFileURL(`${site}/programs/calculation/index.mjs`)
)
const { closePrograms } = await import(
  pathToFileURL(`${site}/programs/__lifecycle.mjs`)
)
const wasm = await readFile(`${site}/programs/calculation/program.wasm`)
const success = (value) => ({ tag: "Success", value })
const tick = () => new Promise((resolve) => setTimeout(resolve, 0))
function gate() {
  let resolve
  const promise = new Promise((complete) => {
    resolve = complete
  })
  return { promise, resolve }
}

const states = []
let closing
try {
  // Both instances join the generated page lifetime. A failing release must
  // remain observable after every owner has finished its asynchronous cleanup.
  for (const name of ["normal", "defect"]) {
    const state = {
      name,
      owner: await loadProgram(wasm),
      readEntered: gate(),
      readDone: gate(),
      releaseEntered: gate(),
      releaseDone: gate(),
      resources: new Set(),
      trace: [],
      aborted: 0,
    }
    states.push(state)
    const request = (action, { signal }) => {
      state.trace.push(action)
      if (action.startsWith("open:")) {
        state.resources.add(name)
        return success(name)
      }
      assert.equal(action, `read:${name}`)
      signal.addEventListener(
        "abort",
        () => {
          state.aborted++
        },
        { once: true }
      )
      state.readEntered.resolve()
      return state.readDone.promise
    }
    const release = async (resource, { signal }) => {
      assert.equal(resource, name)
      assert.equal(signal.aborted, false)
      state.trace.push(`release:start:${name}`)
      state.releaseEntered.resolve()
      await state.releaseDone.promise
      assert.equal(signal.aborted, false)
      assert(state.resources.delete(resource))
      state.trace.push(`release:end:${name}`)
      return name === "defect"
        ? { tag: "Defect", messages: ["page release defect"] }
        : success(undefined)
    }
    const first = state.owner.exports.transfer(request)
    const second = first(release)
    state.plan = second(name)
    first.dispose()
    second.dispose()
    assert.deepEqual(state.trace, [])
    state.execution = state.plan.run()
    await state.readEntered.promise
    assert.equal(state.resources.size, 1)
  }

  closing = closePrograms()
  assert.equal(closePrograms(), closing)
  const rejected = assert.rejects(
    closing,
    (error) =>
      error instanceof AggregateError &&
      error.errors.length === 1 &&
      error.errors[0] instanceof AggregateError &&
      error.errors[0].errors.length === 1 &&
      error.errors[0].errors[0].message === "page release defect"
  )
  let settled = false
  void closing.then(
    () => {
      settled = true
    },
    () => {
      settled = true
    }
  )
  await assert.rejects(loadProgram(wasm), ProgramWasmClosed)
  for (const state of states) {
    assert.equal(state.aborted, 1)
    assert.equal(state.owner.ownership().closing, true)
    assert.equal(state.owner.ownership().waitingHost, true)
    state.readDone.resolve(success("ignored after cancellation"))
    await state.releaseEntered.promise
    assert.equal(state.resources.size, 1)
  }
  await tick()
  assert.equal(settled, false)

  states[0].releaseDone.resolve()
  assert.deepEqual(await states[0].execution.result, { tag: "Cancelled" })
  await tick()
  assert.equal(settled, false)
  assert.equal(states[1].resources.size, 1)
  states[1].releaseDone.resolve()
  assert.deepEqual(await states[1].execution.result, {
    tag: "Defect",
    messages: ["page release defect"],
  })
  await rejected
  assert.equal(settled, true)
  for (const state of states) {
    assert.deepEqual(state.trace, [
      `open:${state.name}`,
      `read:${state.name}`,
      `release:start:${state.name}`,
      `release:end:${state.name}`,
    ])
    assert.equal(state.resources.size, 0)
    assert.equal(state.plan.disposed, true)
    assert.throws(() => state.plan.run(), ProgramWasmClosed)
    assert.throws(
      () => state.owner.exports.transfer(() => success("x")),
      ProgramWasmClosed
    )
    assert.deepEqual(state.owner.ownership(), {
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
  }
  console.log(
    "generated host page lifetime drained both owners and reported cleanup Defect"
  )
} finally {
  // Release test-owned gates even when an assertion fails. Each active WASM
  // run still acknowledges host completion and drains its real finalizers.
  for (const state of states) {
    state.readDone.resolve(success("cleanup"))
    state.releaseDone.resolve()
  }
  await Promise.allSettled(
    states.flatMap((state) => [state.execution?.result, state.owner.dispose()])
  )
  await Promise.allSettled([closing, closePrograms()])
}
