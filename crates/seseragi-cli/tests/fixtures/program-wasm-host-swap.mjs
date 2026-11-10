// Swaps a real host Effect program to its rebuilt module in place: the new
// module serves later calls, and the old one is released only after the
// callback and plan it already made are disposed.
import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"

const here = (path) => new URL(path, import.meta.url)
const raw = await import(here("./before/programs/calculation/program.mjs"))
const { createProgramLifetime } = await import(
  here("./before/programs/__lifecycle.mjs")
)
const oldBytes = await readFile(
  here("./before/programs/calculation/program.wasm")
)
const newBytes = await readFile(
  here("./after/programs/calculation/program.wasm")
)
// Only the module changed; the loader that instantiates it is the same.
assert.equal(
  await readFile(here("./before/programs/calculation/program.mjs"), "utf8"),
  await readFile(here("./after/programs/calculation/program.mjs"), "utf8")
)
assert.notDeepEqual(oldBytes, newBytes)
const success = (value) => ({ tag: "Success", value })
const request = (action) =>
  success(action.startsWith("open:") ? "record" : "result")
const release = () => success(undefined)
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms))

const lifetime = createProgramLifetime()
let old
const owner = await lifetime.load(
  async () => {
    old = await raw.loadProgram(oldBytes)
    return old
  },
  () => new raw.ProgramWasmClosed("closed"),
  {
    name: "calculation",
    path: "./programs/calculation/program.wasm",
    instantiate: raw.loadProgram,
  }
)
async function transfer(exports) {
  const first = exports.transfer(request)
  const second = first(release)
  const cold = second("value")
  first.dispose()
  second.dispose()
  try {
    return await cold.run().result
  } finally {
    cold.dispose()
  }
}
// The host keeps this destructured export across the swap.
const { transfer: saved } = owner.exports
assert.deepEqual(await transfer(owner.exports), success("processed:result"))
// The UI holds a callback and a cold plan the old module made.
const prepare = owner.exports.prepared(request)
const held = prepare(release)
prepare.dispose()
const plan = held("kept")

await lifetime.swap(["calculation"], "2", async (path, version) => {
  assert.equal(path, "./programs/calculation/program.wasm")
  assert.equal(version, "2")
  return newBytes
})
assert.deepEqual(await transfer(owner.exports), success("watched:result"))
assert.deepEqual(await transfer({ transfer: saved }), success("watched:result"))
// What the old module made keeps running there until it is released.
assert.deepEqual(await plan.run().result, success("processed:result"))
await wait(150)
assert.equal(old.ownership().alive, true)
plan.dispose()
await wait(150)
assert.equal(old.ownership().alive, true)
held.dispose()
await wait(150)
assert.equal(old.ownership().alive, false)
assert.equal(old.ownership().managedLiveAtClose, 0)
await lifetime.close()
assert.equal(owner.ownership().alive, false)
console.log("host swap: new module serves, old module drained and released")
