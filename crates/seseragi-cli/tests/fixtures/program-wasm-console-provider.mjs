import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"

// Runs Console provider Effects of program-wasm-console-provider.ssrg.
const { loadProgram } = await import(
  new URL("./console/program.mjs", import.meta.url).href
)
const bytes = await readFile(new URL("./console/program.wasm", import.meta.url))
const owner = await loadProgram(bytes)
const { greet, guarded, quiet } = owner.exports
const log = []
const recording = {
  print: (text) => {
    log.push(["print", text])
    return { kind: "success", value: undefined }
  },
  println: async (text) => {
    log.push(["println", text])
    return { kind: "success", value: undefined }
  },
  error: (text) => {
    log.push(["error", text])
    return { kind: "success", value: undefined }
  },
  errorLine: (text) => {
    log.push(["errorLine", text])
    return { kind: "success", value: undefined }
  },
  flush: () => {
    log.push(["flush"])
    return { kind: "success", value: undefined }
  },
}
const plan = greet("雪🙂")
assert.deepEqual(await plan.run({ console: recording }).result, {
  tag: "Success",
  value: undefined,
})
assert.deepEqual(log, [
  ["print", "hello "],
  ["println", "雪🙂"],
  ["error", "to "],
  ["errorLine", "stderr"],
  ["flush"],
])
// The run requires its providers; a providerless Effect takes none.
assert.throws(() => plan.run(), TypeError)
assert.throws(() => plan.run({ console: {} }), TypeError)
assert.throws(() => quiet("x").run({ console: recording }), TypeError)
assert.deepEqual(await quiet("x").run().result, { tag: "Success", value: "x" })
// A provider failure is the typed ConsoleError.
const failing = {
  ...recording,
  println: () => ({
    kind: "failure",
    error: { kind: "console-error", message: "closed" },
  }),
}
log.length = 0
const failed = await plan.run({ console: failing }).result
assert.equal(failed.tag, "Failure")
assert.deepEqual(failed.error, { kind: "console-error", message: "closed" })
// A throwing provider is a Defect.
const throwing = {
  ...recording,
  print: () => {
    throw new Error("boom")
  },
}
const defect = await plan.run({ console: throwing }).result
assert.equal(defect.tag, "Defect")
assert.match(defect.messages[0], /Console provider threw: boom/)
plan.dispose()
// Cancelling during acquisition still releases through the provider, and
// the release's Console call is not interrupted.
log.length = 0
let started
const opened = new Promise((resolve) => {
  started = resolve
})
const request = (text, { signal }) =>
  new Promise((resolve, reject) => {
    if (text.startsWith("open:")) {
      started()
      setTimeout(() => resolve({ tag: "Success", value: text.slice(5) }), 20)
      return
    }
    signal.addEventListener("abort", () => reject(signal.reason))
  })
const slowRelease = {
  ...recording,
  println: (text) =>
    new Promise((resolve) => {
      log.push(["println", text])
      setTimeout(() => resolve({ kind: "success", value: undefined }), 20)
    }),
}
const execution = guarded(request)("r").run({ console: slowRelease })
await opened
execution.cancel()
assert.deepEqual(await execution.result, { tag: "Cancelled" })
assert.deepEqual(log, [
  ["println", "acquire:r"],
  ["println", "release:r"],
])
// A normal run uses and releases its resource.
log.length = 0
const ok = guarded(async (text) => ({
  tag: "Success",
  value: text.replace("open:", "").replace("read:", "data-"),
}))("s")
assert.deepEqual(await ok.run({ console: recording }).result, {
  tag: "Success",
  value: "data-s",
})
assert.deepEqual(log, [
  ["println", "acquire:s"],
  ["println", "using:s"],
  ["println", "release:s"],
])
await owner.dispose()
assert.equal(owner.ownership().managedLiveAtClose, 0)
console.log("console provider")
