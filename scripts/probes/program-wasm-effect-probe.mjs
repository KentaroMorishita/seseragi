import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { pathToFileURL } from "node:url"

const directory = resolve(process.argv[2])
const { loadProgram, ProgramWasmClosed, ProgramWasmDefect } = await import(
  pathToFileURL(resolve(directory, "program.mjs")).href
)
const bytes = await readFile(resolve(directory, "program.wasm"))
const receipt = JSON.parse(
  await readFile(resolve(directory, "program-wasm.json"), "utf8")
)
assert.equal(receipt.abi, "seseragi-program-wasm-effect-v1")
const taskTurn = () => new Promise((resolve) => setTimeout(resolve, 0))
const program = await loadProgram(bytes)
let cancelled = 0
let callbacks = 0
try {
  const cold = program.exports.value("雪😀")
  assert.equal(program.ownership().running, false)
  assert.equal(program.ownership().plans, 1)
  const first = cold.run()
  assert.throws(() => cold.run(), ProgramWasmClosed)
  assert.deepEqual(await first.result, { tag: "Success", value: "雪😀" })
  const replay = cold.run()
  first.cancel()
  assert.deepEqual(await replay.result, { tag: "Success", value: "雪😀" })
  cold.dispose()
  assert.throws(() => cold.run(), ProgramWasmClosed)

  const failure = program.exports.rejection("typed failure")
  assert.deepEqual(await failure.run().result, {
    tag: "Failure",
    error: "typed failure",
  })
  failure.dispose()
  const defect = program.exports.defect("explicit Defect")
  assert.equal(program.ownership().running, false)
  assert.deepEqual(await defect.run().result, {
    tag: "Defect",
    messages: ["explicit Defect"],
  })
  assert.equal(program.ownership().retired, false)
  defect.dispose()

  for (const name of ["callback", "saved", "staged"]) {
    let prefix
    let plan
    if (name === "staged") {
      prefix = program.exports.staged("保存:")
      plan = prefix(undefined)
    } else {
      plan = program.exports[name]("保存:")
    }
    prefix?.dispose()
    const outcome = await plan.run().result
    assert.equal(outcome.tag, "Success")
    const callback = outcome.value
    plan.dispose()
    assert.equal(callback("一"), "保存:一")
    assert.equal(callback("二"), "保存:二")
    callback.dispose()
    assert.throws(() => callback("終了"), ProgramWasmClosed)
    callbacks++
  }

  for (const fail of [false, true]) {
    for (const tuple of [
      ["雪", { tag: "Just", value: 42 }, { tag: "Left", value: "domain" }],
      ["空", { tag: "Nothing" }, { tag: "Right", value: false }],
    ]) {
      const partial = program.exports.transport(fail)
      const plan = partial(tuple)
      partial.dispose()
      assert.deepEqual(
        await plan.run().result,
        fail
          ? { tag: "Failure", error: tuple }
          : { tag: "Success", value: tuple }
      )
      plan.dispose()
    }
  }
  const partial = program.exports.transport(false)
  assert.throws(
    () =>
      partial([
        "bad",
        { tag: "Just", value: 9007199254740992 },
        { tag: "Right", value: true },
      ]),
    TypeError
  )
  partial.dispose()
  assert.equal(program.ownership().retired, false)

  const detached = program.exports.value("execution owns its plan")
  const active = detached.run()
  detached.dispose()
  assert.deepEqual(await active.result, {
    tag: "Success",
    value: "execution owns its plan",
  })

  const cleanup = program.exports.cleanup("body failure")
  assert.deepEqual(await cleanup.run().result, {
    tag: "Defect",
    messages: ["inner released", "outer released"],
  })
  cleanup.dispose()

  for (const name of ["pending", "cancelCleanup"]) {
    const plan = program.exports[name]("held resource")
    const execution = plan.run()
    // The pump's first bounded turn precedes this task. cancelCleanup's
    // Defect independently witnesses that acquisition registered its release.
    await taskTurn()
    execution.cancel()
    execution.cancel()
    assert.deepEqual(
      await execution.result,
      name === "pending"
        ? { tag: "Cancelled" }
        : { tag: "Defect", messages: ["cancel released"] }
    )
    plan.dispose()
    cancelled++
  }

  const pending = program.exports.cancelCleanup("dispose resource")
  const execution = pending.run()
  await taskTurn()
  const closing = assert.rejects(
    program.dispose(),
    (error) =>
      error instanceof AggregateError &&
      error.errors.length === 1 &&
      error.errors[0].message === "cancel released"
  )
  assert.equal(program.ownership().closing, true)
  assert.throws(() => program.exports.value("late"), ProgramWasmClosed)
  assert.deepEqual(await execution.result, {
    tag: "Defect",
    messages: ["cancel released"],
  })
  await closing
  assert.equal(pending.disposed, true)
  assert.throws(() => pending.run(), ProgramWasmClosed)
  assert.deepEqual(program.ownership(), {
    alive: false,
    retired: false,
    callbacks: 0,
    plans: 0,
    running: false,
    closing: true,
    managedLiveAtClose: 0,
  })
} finally {
  if (program.ownership().alive) await program.dispose()
}

if (process.argv[3]) {
  const former = resolve(process.argv[3])
  const { loadProgram } = await import(
    pathToFileURL(resolve(former, "program.mjs")).href
  )
  const owner = await loadProgram(
    await readFile(resolve(former, "program.wasm"))
  )
  try {
    const plan = owner.exports.selected(42)
    assert.deepEqual(await plan.run().result, { tag: "Success", value: 42 })
    plan.dispose()
  } finally {
    await owner.dispose()
  }
  assert.equal(owner.ownership().managedLiveAtClose, 0)
}

// Controlled negative images keep the checked contract/data and replace only
// ssrg_poll. This tests retirement after an unexpected engine/protocol failure;
// it is not evidence that arbitrary supplied WASM is safe or can unwind Rust.
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
  [[0, 0, 11], ProgramWasmDefect], // no locals; unreachable; end
  [[0, 65, 0, 11], TypeError], // no locals; i32.const 0; end, but no outcome
]) {
  const owner = await loadProgram(replacePoll(bytes, body))
  const plan = owner.exports.value("negative image")
  await assert.rejects(plan.run().result, error)
  assert.equal(owner.ownership().alive, false)
  assert.equal(owner.ownership().retired, true)
  assert.equal(owner.ownership().running, false)
  assert.equal(owner.ownership().callbacks, 0)
  assert.equal(owner.ownership().plans, 0)
  assert.equal(owner.ownership().managedLiveAtClose, null)
  assert.throws(() => plan.run(), ProgramWasmClosed)
  await owner.dispose()
}

console.log(
  JSON.stringify({
    abi: receipt.abi,
    rustc: receipt.rustc,
    callbacks,
    cancelled,
    explicitDefect: true,
    lifoCleanup: true,
    disposeDefectObservable: true,
    unexpectedTrapAndProtocolRetired: true,
    normalManagedLiveAtClose: program.ownership().managedLiveAtClose,
  })
)
