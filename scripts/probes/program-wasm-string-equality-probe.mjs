// Use the normal build's program entry and public ownership API in fresh engines.
import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { pathToFileURL } from "node:url"

const directory = resolve(process.argv[2])
const { loadProgram, ProgramWasmClosed } = await import(
  pathToFileURL(resolve(directory, "program.mjs"))
)
const bytes = await readFile(resolve(directory, "program.wasm"))
const program = await loadProgram(bytes)
const values = ["", "ready\n", "雪🌊\0", "é", "é", "other"]
let calls = 0
try {
  for (const left of values) {
    const same = program.exports.same(left)
    const different = program.exports.different(left)
    const saved = program.exports.saved(left)
    try {
      for (const right of values) {
        assert.equal(same(right), left === right)
        assert.equal(different(right), left !== right)
        assert.equal(saved(right), left === right)
        calls += 3
      }
      assert.throws(() => same("\ud800"), TypeError)
      assert.equal(saved(left), true)
    } finally {
      same.dispose()
      different.dispose()
      saved.dispose()
    }
    assert.throws(() => saved(left), ProgramWasmClosed)
  }
  const first = program.exports.saved("first")
  const second = program.exports.saved("second")
  try {
    first.dispose()
    assert.equal(second("second"), true)
    assert.equal(second("first"), false)
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
assert.throws(() => program.exports.saved("closed"), ProgramWasmClosed)
console.log(
  JSON.stringify({
    calls,
    sameAndDifferentAndSaved: true,
    invalidInputRecovery: true,
    independentCallbacks: true,
    callbacks: 0,
    managedLiveAtClose: 0,
  })
)
