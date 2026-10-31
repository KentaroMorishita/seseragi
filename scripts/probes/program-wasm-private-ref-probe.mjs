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
const success = async (plan) => {
  const outcome = await plan.run().result
  assert.equal(outcome.tag, "Success")
  return outcome.value
}
const turn = () => new Promise((resolve) => setTimeout(resolve, 0))
const owner = await loadProgram(bytes)
try {
  const baseline = Object.keys(owner.exports).length
  const creation = owner.exports.create("seed:")
  const first = await success(creation)
  const second = await success(creation)
  creation.dispose()
  const retained = first("one")
  first.dispose()
  assert.throws(() => first("one"), ProgramWasmClosed)
  assert.equal(await success(retained), "seed:one")
  assert.equal(await success(retained), "seed:oneone")
  retained.dispose()
  const independent = second("two")
  assert.equal(await success(independent), "seed:two")
  independent.dispose()
  second.dispose()

  const readerCreation = owner.exports.reader("雪🙂")
  const reader = await success(readerCreation)
  readerCreation.dispose()
  const read = reader(undefined)
  reader.dispose()
  assert.equal(await success(read), "雪🙂")
  assert.equal(await success(read), "雪🙂")
  read.dispose()
  const unit = owner.exports.unit(undefined)
  assert.equal(await success(unit), undefined)
  unit.dispose()
  const snapshot = owner.exports.snapshot(42)
  assert.equal(await success(snapshot), 42)
  snapshot.dispose()

  const start = owner.exports.session("state")
  const session = await success(start)
  start.dispose()
  assert.throws(() => session(42), TypeError)
  const bind = ([suffix, failing, defective, pending]) => {
    const stage2 = session(suffix)
    const stage3 = stage2(failing)
    stage2.dispose()
    const stage4 = stage3(defective)
    stage3.dispose()
    const plan = stage4(pending)
    stage4.dispose()
    return plan
  }
  const execute = async (request) => {
    const plan = bind(request)
    try {
      return await plan.run().result
    } finally {
      plan.dispose()
    }
  }
  assert.deepEqual(await execute([":one", false, false, false]), {
    tag: "Success",
    value: "state:one",
  })
  assert.deepEqual(await execute([":bad", true, false, false]), {
    tag: "Failure",
    error: "typed failure",
  })
  assert.deepEqual(await execute(["", false, false, false]), {
    tag: "Success",
    value: "state:one:inner:outer:bad:inner:outer",
  })
  assert.deepEqual(await execute([":die", false, true, false]), {
    tag: "Defect",
    messages: ["explicit defect"],
  })
  assert.deepEqual(await execute(["", false, false, false]), {
    tag: "Success",
    value: "state:one:inner:outer:bad:inner:outer:inner:outer:die:inner:outer",
  })
  const pending = bind([":wait", false, false, true])
  const execution = pending.run()
  pending.dispose()
  await turn()
  execution.cancel()
  execution.cancel()
  assert.deepEqual(await execution.result, { tag: "Cancelled" })
  assert.deepEqual(await execute(["", false, false, false]), {
    tag: "Success",
    value:
      "state:one:inner:outer:bad:inner:outer:inner:outer:die:inner:outer:inner:outer:wait:inner:outer",
  })
  assert.equal(owner.ownership().retired, false)
  assert.equal(owner.ownership().callbacks, baseline + 1)
  assert.equal(owner.ownership().plans, 0)

  const closingPlan = bind([":close", false, false, true])
  session.dispose()
  const active = closingPlan.run()
  closingPlan.dispose()
  await turn()
  const closing = owner.dispose()
  assert.equal(owner.ownership().closing, true)
  assert.deepEqual(await active.result, { tag: "Cancelled" })
  await closing
  assert.deepEqual(owner.ownership(), {
    alive: false,
    retired: false,
    callbacks: 0,
    plans: 0,
    running: false,
    closing: true,
    managedLiveAtClose: 0,
  })
} finally {
  if (owner.ownership().alive) await owner.dispose()
}

const fresh = await loadProgram(bytes)
try {
  const creation = fresh.exports.create("fresh:")
  const callback = await success(creation)
  creation.dispose()
  const plan = callback("one")
  callback.dispose()
  assert.equal(await success(plan), "fresh:one")
  plan.dispose()
} finally {
  await fresh.dispose()
}
assert.equal(fresh.ownership().managedLiveAtClose, 0)
console.log(
  "private cells: cold independent state, retained plans, Failure/Defect/cancel, LIFO cleanup and roots released"
)
