import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { pathToFileURL } from "node:url"

const directory = resolve(process.argv[2])
const packaged = process.argv[3] === "package"
const { loadProgram, ProgramWasmClosed } = await import(
  pathToFileURL(resolve(directory, packaged ? "index.mjs" : "program.mjs")).href
)
const bytes = await readFile(resolve(directory, "program.wasm"))
const success = (value) => ({ tag: "Success", value })
const turn = () => new Promise((resolve) => setTimeout(resolve, 0))
function gate() {
  let resolve
  const promise = new Promise((complete) => {
    resolve = complete
  })
  return { promise, resolve }
}
async function value(plan) {
  const result = await plan.run().result
  assert.equal(result.tag, "Success")
  return result.value
}
async function closed(owner) {
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
const markers = ":inner::outer:"
{
  const owner = await loadProgram(bytes)
  const trace = []
  const request = async (input, { signal }) => {
    trace.push(input)
    assert.equal(signal.aborted, false)
    if (input.startsWith(":")) return success("")
    if (input === "fail") return { tag: "Failure", error: "body failure" }
    if (input === "defect")
      return { tag: "Defect", messages: ["explicit host defect"] }
    return success(input)
  }
  try {
    const creation = owner.exports.create(request)
    assert.deepEqual(trace, [])
    assert.equal(owner.ownership().hostCallbacks, 1)
    const first = await value(creation)
    const second = await value(creation)
    creation.dispose()
    assert.deepEqual(trace, [])
    const retained = first("one")
    first.dispose()
    assert.throws(() => first("one"), ProgramWasmClosed)
    assert.equal(await value(retained), "seed:one")
    assert.equal(await value(retained), `seed:one${markers}one`)
    retained.dispose()
    assert.deepEqual(trace.splice(0), [
      "one",
      ":inner:seed:one",
      ":outer:seed:one:inner:",
      "one",
      ":inner:seed:one:inner::outer:one",
      ":outer:seed:one:inner::outer:one:inner:",
    ])
    const execute = async (input) => {
      const plan = second(input)
      try {
        return await plan.run().result
      } finally {
        plan.dispose()
      }
    }
    assert.deepEqual(await execute("other"), success("seed:other"))
    assert.deepEqual(await execute("fail"), {
      tag: "Failure",
      error: "body failure",
    })
    assert.deepEqual(
      await execute(""),
      success(`seed:other${markers.repeat(2)}`)
    )
    assert.deepEqual(await execute("defect"), {
      tag: "Defect",
      messages: ["explicit host defect"],
    })
    assert.deepEqual(
      await execute(""),
      success(`seed:other${markers.repeat(4)}`)
    )
    assert.equal(owner.ownership().retired, false)
    second.dispose()
    assert.equal(owner.ownership().hostCallbacks, 0)
    assert.equal(owner.ownership().plans, 0)
  } finally {
    await closed(owner)
  }
}

for (const mode of ["cancel", "dispose"]) {
  const owner = await loadProgram(bytes)
  const entered = gate(),
    body = gate(),
    innerEntered = gate(),
    inner = gate(),
    outerEntered = gate(),
    outer = gate()
  const trace = []
  let aborted = 0
  const request = (input, { signal }) => {
    trace.push(input)
    if (input === "slow") {
      signal.addEventListener(
        "abort",
        () => {
          aborted++
        },
        { once: true }
      )
      entered.resolve()
      return body.promise
    }
    assert.equal(signal.aborted, false)
    if (input.startsWith(":inner:")) {
      innerEntered.resolve(input)
      return inner.promise
    }
    if (input.startsWith(":outer:")) {
      outerEntered.resolve(input)
      return outer.promise
    }
    assert.equal(input, "")
    return success("")
  }
  let closing
  try {
    const creation = owner.exports.create(request)
    const callback = await value(creation)
    creation.dispose()
    const plan = callback("slow")
    const active = plan.run()
    plan.dispose()
    await entered.promise
    let terminal = false
    void active.result.then(() => {
      terminal = true
    })
    if (mode === "dispose") {
      callback.dispose()
      closing = owner.dispose()
      assert.equal(owner.ownership().closing, true)
      assert.throws(() => owner.exports.create(request), ProgramWasmClosed)
    } else {
      active.cancel()
      active.cancel()
    }
    await turn()
    assert.equal(aborted, 1)
    assert.equal(terminal, false)
    assert.equal(owner.ownership().waitingHost, true)
    body.resolve(success("late result must not commit"))
    assert.equal(await innerEntered.promise, ":inner:seed:")
    await turn()
    assert.equal(terminal, false)
    inner.resolve(success(""))
    assert.equal(await outerEntered.promise, ":outer:seed::inner:")
    await turn()
    assert.equal(terminal, false)
    outer.resolve(success(""))
    assert.deepEqual(await active.result, { tag: "Cancelled" })
    assert.deepEqual(trace.splice(0), [
      "slow",
      ":inner:seed:",
      ":outer:seed::inner:",
    ])
    if (mode === "cancel") {
      const inspect = callback("")
      callback.dispose()
      assert.equal(await value(inspect), `seed:${markers}`)
      inspect.dispose()
      assert.equal(owner.ownership().hostCallbacks, 0)
    } else {
      await closing
    }
  } finally {
    body.resolve(success(""))
    inner.resolve(success(""))
    outer.resolve(success(""))
    if (closing) await closing
    await closed(owner)
  }
}
console.log(
  "host private state: replay, detached leases, Failure/Defect, late-result cancellation, masked LIFO cleanup and roots released"
)
