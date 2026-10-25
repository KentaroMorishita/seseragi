import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { pathToFileURL } from "node:url"

const directory = process.argv[2]
const { loadProgram, ProgramWasmClosed, ProgramWasmDefect } = await import(
  pathToFileURL(`${directory}/program.mjs`)
)
const wasm = await readFile(`${directory}/program.wasm`)
const success = (value) => ({ tag: "Success", value })
const failure = (error) => ({ tag: "Failure", error })
const tick = () => new Promise((resolve) => setTimeout(resolve, 0))
function gate() {
  let resolve
  const promise = new Promise((done) => {
    resolve = done
  })
  return { promise, resolve }
}
function plan(owner, request, release, input = "snow", name = "transfer") {
  const first = owner.exports[name](request)
  try {
    const second = first(release)
    try {
      return second(input)
    } finally {
      second.dispose()
    }
  } finally {
    first.dispose()
  }
}
function host(trace, mode = "success") {
  const open = new Set()
  const request = async (input) => {
    trace.push(input)
    if (input.startsWith("open:")) {
      const resource = input.slice(5)
      open.add(resource)
      return success(resource)
    }
    if (mode === "failure") return failure("missing")
    if (mode === "throw") throw new Error("host exploded")
    if (mode === "wrong-value") return success(123)
    if (mode === "large-defect")
      return { tag: "Defect", messages: ["x".repeat(16 * 1024 * 1024 + 1)] }
    if (mode === "accessor")
      return {
        get tag() {
          throw new Error("tag accessor")
        },
        value: "x",
      }
    if (mode === "thenable")
      return {
        // biome-ignore lint/suspicious/noThenProperty: exercise a throwing host thenable and verify finalizer cleanup.
        get then() {
          throw new Error("then accessor")
        },
      }
    return success("data")
  }
  const release = async (resource) => {
    trace.push(`release:${resource}`)
    assert(open.delete(resource))
    return success(undefined)
  }
  return { request, release, open }
}
async function close(owner) {
  await owner.dispose()
  assert.deepEqual(owner.ownership(), {
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

let cases = 0
for (const name of ["transfer", "prepared"]) {
  const owner = await loadProgram(wasm)
  try {
    const trace = []
    const hooks = host(trace)
    const cold = plan(owner, hooks.request, hooks.release, "snow", name)
    assert.deepEqual(trace, [])
    assert.equal(owner.ownership().hostCallbacks, 2)
    for (let replay = 0; replay < 2; replay++) {
      assert.deepEqual(await cold.run().result, success("processed:data"))
      assert.equal(hooks.open.size, 0)
    }
    assert.deepEqual(trace, [
      "open:snow",
      "read:snow",
      "release:snow",
      "open:snow",
      "read:snow",
      "release:snow",
    ])
    cold.dispose()
    assert.equal(owner.ownership().hostCallbacks, 0)
  } finally {
    await close(owner)
  }
  cases++
}
for (const mode of [
  "failure",
  "throw",
  "wrong-value",
  "accessor",
  "thenable",
  "large-defect",
]) {
  const owner = await loadProgram(wasm)
  try {
    const trace = []
    const hooks = host(trace, mode)
    const cold = plan(owner, hooks.request, hooks.release)
    const outcome = await cold.run().result
    if (mode === "failure") assert.deepEqual(outcome, failure("missing"))
    else {
      assert.equal(outcome.tag, "Defect")
      if (mode === "large-defect")
        assert.deepEqual(outcome.messages, [
          "Host callback returned an invalid typed response",
        ])
    }
    assert.deepEqual(trace, ["open:snow", "read:snow", "release:snow"])
    assert.equal(hooks.open.size, 0)
    assert.equal(owner.ownership().retired, false)
    cold.dispose()
    assert.equal(owner.ownership().hostCallbacks, 0)
  } finally {
    await close(owner)
  }
  cases++
}
{
  const owner = await loadProgram(wasm)
  try {
    const trace = []
    const hooks = host(trace)
    const cold = plan(
      owner,
      (input) =>
        input.startsWith("open:")
          ? hooks.request(input)
          : { tag: "Defect", messages: ["explicit host Defect"] },
      hooks.release
    )
    assert.deepEqual(await cold.run().result, {
      tag: "Defect",
      messages: ["explicit host Defect"],
    })
    assert.deepEqual(trace, ["open:snow", "release:snow"])
    assert.equal(hooks.open.size, 0)
    cold.dispose()
  } finally {
    await close(owner)
  }
  cases++
}
for (const invalid of [
  failure("not Never"),
  success("not Unit"),
  { tag: "Cancelled" },
]) {
  const owner = await loadProgram(wasm)
  try {
    const trace = []
    const hooks = host(trace)
    const cold = plan(owner, hooks.request, async (resource) => {
      await hooks.release(resource)
      return invalid
    })
    assert.deepEqual(await cold.run().result, {
      tag: "Defect",
      messages: ["Host callback returned an invalid typed response"],
    })
    assert.deepEqual(trace, ["open:snow", "read:snow", "release:snow"])
    assert.equal(hooks.open.size, 0)
    assert.equal(owner.ownership().retired, false)
    cold.dispose()
  } finally {
    await close(owner)
  }
  cases++
}
{
  const owner = await loadProgram(wasm)
  try {
    const trace = []
    const hooks = host(trace)
    const release = async (resource) => {
      await hooks.release(resource)
      throw new Error(`closed ${resource}`)
    }
    const cold = plan(owner, hooks.request, release, "snow", "nested")
    const result = await cold.run().result
    assert.equal(result.tag, "Defect")
    assert.deepEqual(trace, [
      "open:outer:snow",
      "open:inner:snow",
      "read:inner:snow",
      "release:inner:snow",
      "release:outer:snow",
    ])
    assert.deepEqual(result.messages, [
      "Host callback threw: closed inner:snow",
      "Host callback threw: closed outer:snow",
    ])
    assert.equal(hooks.open.size, 0)
    cold.dispose()
  } finally {
    await close(owner)
  }
  cases++
}
{
  const owner = await loadProgram(wasm)
  try {
    const acquired = gate()
    const enter = gate()
    const trace = []
    let signal
    const cold = plan(
      owner,
      (input, context) => {
        trace.push(input)
        signal = context.signal
        enter.resolve()
        return acquired.promise
      },
      async (resource) => {
        trace.push(`release:${resource}`)
        return success(undefined)
      }
    )
    const running = cold.run()
    await enter.promise
    running.cancel()
    assert.equal(signal.aborted, false)
    assert.equal(owner.ownership().waitingHost, true)
    acquired.resolve(success("resource"))
    assert.deepEqual(await running.result, { tag: "Cancelled" })
    assert.deepEqual(trace, ["open:snow", "release:resource"])
    cold.dispose()
    assert.equal(owner.ownership().hostCallbacks, 0)
  } finally {
    await close(owner)
  }
  cases++
}
for (const dispose of [false, true]) {
  const owner = await loadProgram(wasm)
  let closing
  try {
    const read = gate()
    const entered = gate()
    const releasing = gate()
    const releaseDone = gate()
    let readSignal
    let releaseSignal
    let aborted = 0
    const cold = plan(
      owner,
      (input, { signal }) => {
        if (input.startsWith("open:")) return success("resource")
        readSignal = signal
        signal.addEventListener("abort", () => {
          aborted++
        })
        entered.resolve()
        return read.promise
      },
      (resource, { signal }) => {
        assert.equal(resource, "resource")
        releaseSignal = signal
        releasing.resolve()
        return releaseDone.promise
      }
    )
    const running = cold.run()
    let ended = false
    void running.result.then(() => {
      ended = true
    })
    await entered.promise
    assert.throws(
      () => owner.exports.transfer(() => success("x")),
      ProgramWasmClosed
    )
    if (dispose) closing = owner.dispose()
    else running.cancel()
    assert.equal(readSignal.aborted, true)
    assert.equal(aborted, 1)
    await tick()
    assert.equal(ended, false)
    read.resolve(success("data"))
    await releasing.promise
    assert.equal(releaseSignal.aborted, false)
    running.cancel()
    assert.equal(releaseSignal.aborted, false)
    await tick()
    assert.equal(ended, false)
    releaseDone.resolve(success(undefined))
    assert.deepEqual(await running.result, { tag: "Cancelled" })
    if (dispose) await closing
    else cold.dispose()
    assert.equal(owner.ownership().hostCallbacks, 0)
  } finally {
    await close(owner)
  }
  cases++
}
{
  const owner = await loadProgram(wasm)
  try {
    const entered = gate()
    let closed = 0
    const cold = plan(
      owner,
      (input, { signal }) => {
        if (input.startsWith("open:")) return success("resource")
        entered.resolve()
        return new Promise((_resolve, reject) =>
          signal.addEventListener("abort", () => reject(signal.reason), {
            once: true,
          })
        )
      },
      () => {
        closed++
        return success(undefined)
      }
    )
    const running = cold.run()
    await entered.promise
    running.cancel()
    assert.deepEqual(await running.result, { tag: "Cancelled" })
    assert.equal(closed, 1)
    assert.equal(owner.ownership().retired, false)
    cold.dispose()
  } finally {
    await close(owner)
  }
  cases++
}
{
  const owner = await loadProgram(wasm)
  try {
    const entered = gate()
    const response = gate()
    const hooks = host([])
    const cold = plan(
      owner,
      async (input) => {
        if (input.startsWith("open:")) return hooks.request(input)
        entered.resolve()
        return response.promise
      },
      hooks.release
    )
    const running = cold.run()
    await entered.promise
    cold.dispose()
    assert.equal(owner.ownership().plans, 0)
    assert.equal(owner.ownership().hostCallbacks, 2)
    response.resolve(success("data"))
    assert.deepEqual(await running.result, success("processed:data"))
    assert.equal(hooks.open.size, 0)
    assert.equal(owner.ownership().hostCallbacks, 0)
  } finally {
    await close(owner)
  }
  cases++
}
{
  const owner = await loadProgram(wasm)
  const entered = gate()
  const finished = gate()
  let closed = 0
  const cold = plan(
    owner,
    (input) => {
      if (input.startsWith("open:")) return success("resource")
      entered.resolve()
      return finished.promise
    },
    () => {
      closed++
      return { tag: "Defect", messages: ["release rejected"] }
    }
  )
  const running = cold.run()
  await entered.promise
  const closing = owner.dispose()
  const rejection = assert.rejects(
    closing,
    (error) =>
      error instanceof AggregateError &&
      error.errors.length === 1 &&
      error.errors[0].message === "release rejected"
  )
  finished.resolve(success("data"))
  assert.deepEqual(await running.result, {
    tag: "Defect",
    messages: ["release rejected"],
  })
  await rejection
  assert.equal(closed, 1)
  assert.equal(cold.disposed, true)
  assert.deepEqual(owner.ownership(), {
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
  await assert.rejects(owner.dispose(), AggregateError)
  const regenerated = await loadProgram(wasm)
  try {
    const hooks = host([])
    const fresh = plan(regenerated, hooks.request, hooks.release)
    assert.deepEqual(await fresh.run().result, success("processed:data"))
    fresh.dispose()
  } finally {
    await close(regenerated)
  }
  cases++
}
{
  const owner = await loadProgram(wasm)
  try {
    const partial = owner.exports.transfer(async () => success("resource"))
    assert.equal(owner.ownership().hostCallbacks, 1)
    assert.throws(() => partial(42), TypeError)
    assert.equal(owner.ownership().hostCallbacks, 1)
    partial.dispose()
    assert.equal(owner.ownership().hostCallbacks, 0)
  } finally {
    await close(owner)
  }
  cases++
}
{
  const first = await loadProgram(wasm)
  const second = await loadProgram(wasm)
  try {
    const a = host([])
    const b = host([])
    const left = plan(first, a.request, a.release)
    const right = plan(second, b.request, b.release)
    assert.deepEqual(
      await Promise.all([left.run().result, right.run().result]),
      [success("processed:data"), success("processed:data")]
    )
    left.dispose()
    assert.equal(first.ownership().hostCallbacks, 0)
    assert.equal(second.ownership().hostCallbacks, 2)
    right.dispose()
    assert.equal(second.ownership().hostCallbacks, 0)
  } finally {
    await Promise.all([close(first), close(second)])
  }
  cases++
}

// Controlled negative images replace only ssrg_poll. They verify host-root
// retirement, not execution safety for arbitrary WASM or Rust unwinding.
function replacePoll(original, replacement) {
  function input(bytes) {
    return {
      bytes,
      offset: 0,
      u32() {
        let value = 0
        let scale = 1
        for (let count = 0; count < 5; count++) {
          const byte = this.bytes[this.offset++]
          assert.notEqual(byte, undefined)
          value += (byte & 127) * scale
          if (byte < 128) return value
          scale *= 128
        }
        assert.fail("invalid test image length")
      },
      take(length) {
        const start = this.offset
        this.offset += length
        assert(this.offset <= this.bytes.length)
        return this.bytes.subarray(start, this.offset)
      },
    }
  }
  function u32(value) {
    const result = []
    do {
      const byte = value % 128
      value = Math.floor(value / 128)
      result.push(byte | (value ? 128 : 0))
    } while (value)
    return Buffer.from(result)
  }
  const source = input(original)
  const header = source.take(8)
  const sections = []
  let poll
  let replaced = false
  while (source.offset < original.length) {
    const kind = source.take(1)[0]
    let payload = source.take(source.u32())
    if (kind === 7) {
      const exports = input(payload)
      const count = exports.u32()
      for (let index = 0; index < count; index++) {
        const name = new TextDecoder().decode(exports.take(exports.u32()))
        const kind = exports.take(1)[0]
        const target = exports.u32()
        if (name === "ssrg_poll") {
          assert.equal(kind, 0)
          poll = target
        }
      }
    }
    if (kind === 10) {
      assert.notEqual(poll, undefined)
      const code = input(payload)
      const count = code.u32()
      assert(poll < count)
      const bodies = [u32(count)]
      for (let index = 0; index < count; index++) {
        let body = code.take(code.u32())
        if (index === poll) {
          body = Buffer.from(replacement)
          replaced = true
        }
        bodies.push(u32(body.length), body)
      }
      payload = Buffer.concat(bodies)
    }
    sections.push(Buffer.from([kind]), u32(payload.length), payload)
  }
  assert(replaced)
  const result = Buffer.concat([header, ...sections])
  assert(WebAssembly.validate(result))
  return result
}

for (const [body, error] of [
  [[0, 0, 11], ProgramWasmDefect],
  [[0, 65, 0, 11], TypeError],
]) {
  const owner = await loadProgram(replacePoll(wasm, body))
  const trace = []
  const hooks = host(trace)
  const cold = plan(owner, hooks.request, hooks.release)
  assert.equal(owner.ownership().hostCallbacks, 2)
  await assert.rejects(cold.run().result, error)
  assert.deepEqual(trace, [])
  assert.equal(owner.ownership().alive, false)
  assert.equal(owner.ownership().retired, true)
  assert.equal(owner.ownership().callbacks, 0)
  assert.equal(owner.ownership().plans, 0)
  assert.equal(owner.ownership().hostCallbacks, 0)
  assert.equal(owner.ownership().running, false)
  assert.equal(owner.ownership().waitingHost, false)
  assert.equal(owner.ownership().managedLiveAtClose, null)
  assert.equal(cold.disposed, true)
  assert.throws(() => cold.run(), ProgramWasmClosed)
  await owner.dispose()
  cases++
}

console.log(`host Effect ${cases} cases passed`)
