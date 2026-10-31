import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { pathToFileURL } from "node:url"

const directory = resolve(process.argv[2])
const cleanupDefect = process.argv[3] === "defect"
assert(["normal", "defect"].includes(process.argv[3]))
const { loadProgram, ProgramWasmClosed } = await import(
  pathToFileURL(resolve(directory, "index.mjs")).href
)
const { closePrograms } = await import(
  pathToFileURL(resolve(directory, "../__lifecycle.mjs")).href
)
const bytes = await readFile(resolve(directory, "program.wasm"))
const first = await loadProgram(bytes)
const second = await loadProgram(bytes)
try {
  const callbackPlan = first.exports.callback("retained:")
  const result = await callbackPlan.run().result
  assert.equal(result.tag, "Success")
  const callback = result.value
  callbackPlan.dispose()
  assert.equal(callback("before close"), "retained:before close")

  const firstPlan = cleanupDefect
    ? first.exports.cancelCleanup("first resource")
    : first.exports.pending("first resource")
  const secondPlan = second.exports.pending("second resource")
  const firstRun = firstPlan.run()
  const secondRun = secondPlan.run()
  await new Promise((resolve) => setTimeout(resolve, 0))
  assert.equal(first.ownership().running, true)
  assert.equal(second.ownership().running, true)
  // Reserve a factory before close. It must be reclaimed before close settles,
  // and the caller must never receive a usable late instance.
  const late = loadProgram(bytes).then(
    () => assert.fail("a late instance escaped its closing lifetime"),
    (error) => assert(error instanceof ProgramWasmClosed)
  )
  const closing = closePrograms()
  assert.equal(closing, closePrograms())
  assert.equal(first.ownership().closing, true)
  assert.equal(second.ownership().closing, true)
  if (cleanupDefect) {
    await assert.rejects(
      closing,
      (error) =>
        error instanceof AggregateError &&
        error.errors.length === 1 &&
        error.errors[0] instanceof AggregateError &&
        error.errors[0].errors[0].message === "cancel released"
    )
    assert.deepEqual(await firstRun.result, {
      tag: "Defect",
      messages: ["cancel released"],
    })
  } else {
    await closing
    assert.deepEqual(await firstRun.result, { tag: "Cancelled" })
  }
  await late
  assert.deepEqual(await secondRun.result, { tag: "Cancelled" })
  assert.throws(() => callback("closed"), ProgramWasmClosed)
  assert.throws(() => firstPlan.run(), ProgramWasmClosed)
  for (const owner of [first, second]) {
    assert.deepEqual(owner.ownership(), {
      alive: false,
      retired: false,
      callbacks: 0,
      plans: 0,
      running: false,
      closing: true,
      managedLiveAtClose: 0,
    })
  }
  console.log(
    JSON.stringify({
      roots: 2,
      normalManagedLiveAtClose: 0,
      cleanupDefect,
      lateFactoryClosed: true,
    })
  )
} finally {
  await Promise.allSettled([first.dispose(), second.dispose()])
}
