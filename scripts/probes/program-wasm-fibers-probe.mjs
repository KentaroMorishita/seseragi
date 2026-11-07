import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { pathToFileURL } from "node:url"

const directory = resolve(process.argv[2])
const { loadProgram } = await import(
  pathToFileURL(resolve(directory, "program.mjs")).href
)
const owner = await loadProgram(
  await readFile(resolve(directory, "program.wasm"))
)
const run = async (name) => {
  const plan = owner.exports[name](undefined)
  try {
    return await plan.run().result
  } finally {
    plan.dispose()
  }
}
const lines = []
for (const name of ["order", "cleanup"]) {
  const outcome = await run(name)
  assert.equal(outcome.tag, "Success", name)
  lines.push(outcome.value)
}
const failed = await run("failing")
assert.equal(failed.tag, "Failure")
assert.equal(failed.error, "child failed")
lines.push("-1")
const first = await run("first")
assert.equal(first.tag, "Success")
lines.push(first.value)

// Every Fiber waits and nothing can wake one, so the run stays pending
// without polling until the host cancels it.
const plan = owner.exports.stuck(undefined)
const execution = plan.run()
plan.dispose()
let settled = false
execution.result.then(() => {
  settled = true
})
await new Promise((resolve) => setTimeout(resolve, 20))
assert.equal(settled, false)
assert.equal(owner.ownership().running, true)
execution.cancel()
assert.deepEqual(await execution.result, { tag: "Cancelled" })

// A stuck run also ends when the instance is disposed.
const again = owner.exports.stuck(undefined)
again.run()
again.dispose()
await new Promise((resolve) => setTimeout(resolve, 20))
await owner.dispose()
assert.equal(owner.ownership().managedLiveAtClose, 0)
console.log(lines.join("\n"))
