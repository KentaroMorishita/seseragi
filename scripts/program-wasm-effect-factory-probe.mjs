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
const owner = await loadProgram(bytes)
const exportedCallbacks = Object.keys(owner.exports).length
assert.equal(owner.ownership().callbacks, exportedCallbacks)
const taskTurn = () => new Promise((resolve) => setTimeout(resolve, 0))
const success = async (plan) => {
  const outcome = await plan.run().result
  assert.equal(outcome.tag, "Success")
  return outcome.value
}
const released = ["inner released", "outer released"]
try {
  const creation = owner.exports.create("seed:")
  assert.equal(owner.ownership().running, false)
  assert.equal(owner.ownership().callbacks, exportedCallbacks)
  const first = await success(creation)
  const second = await success(creation)
  assert.notEqual(first, second)
  assert.equal(owner.ownership().callbacks, exportedCallbacks + 2)
  creation.dispose()
  assert.throws(() => creation.run(), ProgramWasmClosed)
  const saved = first("one")
  assert.equal(owner.ownership().running, false)
  first.dispose()
  assert.throws(() => first("one"), ProgramWasmClosed)
  assert.equal(await success(saved), "seed:one")
  assert.equal(await success(saved), "seed:one")
  saved.dispose()
  assert.throws(() => saved.run(), ProgramWasmClosed)
  assert.throws(() => second(42), TypeError)
  const next = second("two")
  assert.equal(await success(next), "seed:two")
  next.dispose()
  const defectCreation = owner.exports.createDefect("factory defect")
  const crash = await success(defectCreation)
  defectCreation.dispose()
  const failed = crash(undefined)
  crash.dispose()
  assert.deepEqual(await failed.run().result, {
    tag: "Defect",
    messages: ["factory defect"],
  })
  assert.equal(owner.ownership().retired, false)
  failed.dispose()
  second.dispose()

  const outer = owner.exports.nested("nested")
  const inner = await success(outer)
  outer.dispose()
  assert.equal(await success(inner), "nested")
  assert.equal(await success(inner), "nested")
  inner.dispose()

  const checkedCreation = owner.exports.createChecked("typed failure")
  const checked = await success(checkedCreation)
  checkedCreation.dispose()
  const failure = checked(true)
  const normal = checked(false)
  checked.dispose()
  assert.deepEqual(await failure.run().result, {
    tag: "Failure",
    error: "typed failure",
  })
  assert.equal(await success(normal), "typed failure")
  failure.dispose()
  normal.dispose()

  const cleanupCreation = owner.exports.cleanupFactory(undefined)
  const cleanup = await success(cleanupCreation)
  cleanupCreation.dispose()
  const finishing = cleanup(undefined)
  cleanup.dispose()
  assert.deepEqual(await finishing.run().result, {
    tag: "Defect",
    messages: released,
  })
  finishing.dispose()

  const pendingCreation = owner.exports.pendingFactory(undefined)
  const pendingCallback = await success(pendingCreation)
  pendingCreation.dispose()
  const pending = pendingCallback(undefined)
  pendingCallback.dispose()
  const execution = pending.run()
  pending.dispose()
  await taskTurn()
  execution.cancel()
  execution.cancel()
  assert.deepEqual(await execution.result, {
    tag: "Defect",
    messages: released,
  })
  assert.equal(pending.disposed, true)
  assert.equal(owner.ownership().retired, false)
  assert.equal(owner.ownership().callbacks, exportedCallbacks)
  assert.equal(owner.ownership().plans, 0)

  const closingCreation = owner.exports.pendingFactory(undefined)
  const closingCallback = await success(closingCreation)
  closingCreation.dispose()
  const closingPlan = closingCallback(undefined)
  closingCallback.dispose()
  const active = closingPlan.run()
  await taskTurn()
  const closing = assert.rejects(
    owner.dispose(),
    (error) =>
      error instanceof AggregateError &&
      error.errors.length === 2 &&
      error.errors.every((value, index) => value.message === released[index])
  )
  assert.equal(owner.ownership().closing, true)
  assert.throws(() => owner.exports.create("late"), ProgramWasmClosed)
  assert.deepEqual(await active.result, {
    tag: "Defect",
    messages: released,
  })
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
  assert.throws(() => closingPlan.run(), ProgramWasmClosed)
} finally {
  if (owner.ownership().alive) await owner.dispose()
}

const fresh = await loadProgram(bytes)
try {
  const creation = fresh.exports.create("fresh:")
  const callback = await success(creation)
  creation.dispose()
  const plan = callback("two")
  callback.dispose()
  assert.equal(await success(plan), "fresh:two")
  plan.dispose()
} finally {
  await fresh.dispose()
}
assert.equal(fresh.ownership().managedLiveAtClose, 0)
console.log(
  JSON.stringify({
    abi: "seseragi-program-wasm-effect-v1",
    factoryReplay: true,
    detachedCallbacksAndPlans: true,
    nestedEffect: true,
    typedFailureAndLanguageDefect: true,
    maskedLifoCancellationAndDispose: true,
    normalManagedLiveAtClose: 0,
    freshInstance: true,
  })
)
