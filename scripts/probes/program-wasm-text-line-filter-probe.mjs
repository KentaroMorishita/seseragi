import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { pathToFileURL } from "node:url"

const directory = resolve(process.argv[2])
const { loadProgram, ProgramWasmClosed } = await import(
  pathToFileURL(resolve(directory, "program.mjs"))
)
const program = await loadProgram(
  await readFile(resolve(directory, "program.wasm"))
)
const cases = [
  ["", [], ""],
  ["\n", [""], ""],
  ["\r\n", [""], ""],
  ["\n\n", ["", ""], ""],
  ["a\n", ["a"], "a\n"],
  ["a\r\n\r\nb\r", ["a", "", "b"], "a\nb\n"],
  ["a\n\rb", ["a", "", "b"], "a\nb\n"],
  [
    "雫🙂\u0085e\u0301\u2028\0\u2029終",
    ["雫🙂", "e\u0301", "\0", "終"],
    "雫🙂\ne\u0301\n\0\n終\n",
  ],
  [
    "  kept \0🙂\ne\u0301\n",
    ["  kept \0🙂", "e\u0301"],
    "  kept \0🙂\ne\u0301\n",
  ],
]
let calls = 0
try {
  for (const [input, _lines, expected] of cases) {
    assert.equal(program.exports.clean(input), expected)
    const callback = program.exports.factory("")
    try {
      assert.equal(callback(input), expected)
      calls += 2
      assert.throws(() => callback("\ud800"), TypeError)
      assert.equal(callback(input), expected)
    } finally {
      callback.dispose()
    }
    assert.throws(() => callback(input), ProgramWasmClosed)
  }
  assert.throws(() => program.exports.clean("\udfff"), TypeError)
  assert.equal(program.exports.clean("a\n\nb"), "a\nb\n")
  const first = program.exports.factory("first:")
  const second = program.exports.factory("second:")
  try {
    assert.equal(first("a\n"), "first:a\n")
    first.dispose()
    assert.equal(second("b\n\n"), "second:b\n")
  } finally {
    first.dispose()
    second.dispose()
  }
} finally {
  program.dispose()
}
assert.deepEqual(program.ownership(), {
  alive: false,
  retired: false,
  callbacks: 0,
  managedLiveAtClose: 0,
})
assert.throws(() => program.exports.factory("closed"), ProgramWasmClosed)
console.log(
  JSON.stringify({
    calls,
    unicodeLinesAndKeptContents: true,
    invalidInputRecovery: true,
    independentCallbacks: true,
    callbacks: 0,
    managedLiveAtClose: 0,
  })
)
