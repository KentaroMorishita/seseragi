// Run against ordinary CLI output in a fresh Bun or Node process.
import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { pathToFileURL } from "node:url"

const directory = resolve(process.argv[2])
const bytes = await readFile(resolve(directory, "program.wasm"))
const { loadProgram, ProgramWasmClosed, ProgramWasmDefect } = await import(
  pathToFileURL(resolve(directory, "program.mjs"))
)
const metadata = JSON.parse(
  await readFile(resolve(directory, "program-wasm.json"), "utf8")
)
assert.match(metadata.rustc.version, /^rustc /)
assert.equal(metadata.rustc.target, "wasm32-unknown-unknown")
assert.deepEqual(
  WebAssembly.Module.imports(await WebAssembly.compile(bytes)),
  []
)
const program = await loadProgram(bytes)
const exports = program.exports
const baseline = program.ownership().callbacks
const prefix = exports.label("合計: 雪🌊 ")
const callback = prefix(40)
prefix.dispose()
assert.equal(prefix.disposed, true)
assert.throws(() => prefix(0), ProgramWasmClosed)
for (let i = -500; i < 500; i++) {
  assert.equal(callback(i), `合計: 雪🌊 ${40 + i}`)
}
assert.deepEqual(exports.validate(-1), { tag: "Left", value: "negative" })
assert.deepEqual(exports.validate(2), { tag: "Right", value: 3 })
assert.deepEqual(exports.enabled(false), [false, undefined])
assert.deepEqual(exports.optional(-1), { tag: "Nothing" })
assert.deepEqual(exports.optional(7), { tag: "Just", value: 7 })
for (const value of [
  ["", { tag: "Nothing" }, { tag: "Right", value: -9007199254740991 }],
  [
    "雪🌊\u0000",
    { tag: "Just", value: 9007199254740991 },
    { tag: "Left", value: "失敗" },
  ],
]) {
  assert.deepEqual(exports.transport(value), value)
}
const ready = exports.ready(undefined)
assert.equal(ready(undefined), true)
ready.dispose()
let getterCalls = 0
const getterTuple = ["safe", { tag: "Nothing" }, { tag: "Right", value: 0 }]
Object.defineProperty(getterTuple, 0, {
  get() {
    getterCalls++
    return "unsafe"
  },
})
const invalid = [
  () => exports.label("\ud800"),
  () => exports.label("\udc00"),
  () => callback(1.5),
  () => callback(Infinity),
  () => callback(9007199254740992),
  () => exports.enabled(1),
  () => exports.ready(null),
  () => exports.transport(getterTuple),
  () => exports.transport(["x", { tag: "Just" }, { tag: "Right", value: 1 }]),
  () =>
    exports.transport([
      "x",
      { tag: "Nothing", value: 1 },
      { tag: "Left", value: "x" },
    ]),
  () =>
    exports.transport(["x", { tag: "Nothing" }, { tag: "Other", value: 1 }]),
]
for (const reject of invalid) assert.throws(reject, TypeError)
assert.equal(getterCalls, 0)
assert.equal(callback(2), "合計: 雪🌊 42")
callback.dispose()
callback.dispose()
assert.throws(() => callback(2), ProgramWasmClosed)
assert.equal(program.ownership().callbacks, baseline)
program.dispose()
program.dispose()
assert.deepEqual(program.ownership(), {
  alive: false,
  retired: false,
  callbacks: 0,
  managedLiveAtClose: 0,
})
assert.throws(() => exports.validate(2), ProgramWasmClosed)

const trapped = await loadProgram(bytes)
const trappedBase = trapped.exports.label("retired:")
const trappedCallback = trappedBase(40)
trappedBase.dispose()
assert.throws(
  () => trapped.exports.staged(1),
  (error) => {
    assert.ok(error instanceof ProgramWasmDefect)
    assert.match(error.message, /::staged/)
    assert.ok(error.origin)
    // A copied cause cannot hold a native RuntimeError or WASM instance alive.
    assert.equal(Object.getPrototypeOf(error.cause), Object.prototype)
    assert.equal(typeof error.cause.message, "string")
    return true
  }
)
assert.equal(trappedCallback.disposed, true)
assert.throws(() => trappedCallback(2), ProgramWasmClosed)
assert.deepEqual(trapped.ownership(), {
  alive: false,
  retired: true,
  callbacks: 0,
  managedLiveAtClose: null,
})
trapped.dispose()
const recovered = await loadProgram(bytes)
assert.deepEqual(recovered.exports.validate(41), { tag: "Right", value: 42 })
recovered.dispose()
assert.equal(recovered.ownership().managedLiveAtClose, 0)
// Source recursion can exhaust the engine stack independently of Int Defect.
const stack = await loadProgram(bytes)
assert.throws(() => stack.exports.deep(1000000), ProgramWasmDefect)
assert.equal(stack.ownership().retired, true)
assert.equal(stack.ownership().callbacks, 0)
assert.equal(stack.ownership().managedLiveAtClose, null)
stack.dispose()
if (process.argv[3]) {
  await assert.rejects(
    loadProgram(await readFile(resolve(process.argv[3], "program.wasm"))),
    /checked contract/
  )
}
console.log(
  JSON.stringify({
    runtime: process.versions.bun
      ? `Bun ${process.versions.bun}`
      : `Node ${process.versions.node}`,
    calls: 1000,
    invalidInputs: invalid.length,
    typedEither: true,
    callbackDetach: true,
    normalManagedLiveAtClose: 0,
    sourceDefect: "retired",
    engineStackTrap: "retired",
    recovery: true,
    compiler: metadata.rustc,
  })
)
