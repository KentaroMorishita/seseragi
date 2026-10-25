import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { readFileSync, writeFileSync } from "node:fs"
import { dirname, join } from "node:path"
import ts from "typescript"

const [sourcePath, bun, node] = process.argv.slice(2)
const original = readFileSync(sourcePath, "utf8")
const source = ts.createSourceFile(
  sourcePath,
  original,
  ts.ScriptTarget.Latest,
  true
)
const names = ["condition", "calleeValue", "argument", "beforeReuse"]
const edits = []
const found = []
function visit(value) {
  if (
    ts.isVariableDeclaration(value) &&
    names.includes(value.name.getText(source))
  ) {
    const name = value.name.getText(source)
    assert(ts.isArrowFunction(value.initializer), name)
    const body = value.initializer.body
    assert(!ts.isBlock(body), name)
    found.push(name)
    edits.push({
      start: body.getStart(source),
      end: body.end,
      text: `(__trace.push(${JSON.stringify(name)}), (${body.getText(source)}))`,
    })
  }
  ts.forEachChild(value, visit)
}
visit(source)
assert.deepEqual(found.sort(), [...names].sort())
// Only these four test helper bodies are observed. The computed call and both
// uses of the saved partial remain exactly as generated.
let instrumented = original
for (const edit of edits.sort((a, b) => b.start - a.start)) {
  instrumented =
    instrumented.slice(0, edit.start) + edit.text + instrumented.slice(edit.end)
}
instrumented = `const __trace: Array<string> = []\n${instrumented}
const value = probe(undefined)
const expected = ${JSON.stringify(names)}
if (JSON.stringify(value) !== "[42,43]" || JSON.stringify(__trace) !== JSON.stringify(expected)) {
  throw new Error(JSON.stringify({ value, trace: __trace, expected }))
}
console.log(JSON.stringify({ value, trace: __trace }))
`
const folder = dirname(sourcePath)
const input = join(folder, "trace-observed.ts")
const output = join(folder, "trace-observed.mjs")
writeFileSync(input, instrumented)
function run(command, args) {
  const result = spawnSync(command, args, { encoding: "utf8", timeout: 60000 })
  assert.equal(result.error, undefined)
  assert.equal(result.status, 0, result.stderr)
  assert.equal(result.signal, null)
  assert.equal(result.stderr, "")
  return result.stdout
}
run(bun, ["build", input, "--target=node", "--outfile", output])
const expected = { value: [42, 43], trace: names }
for (const engine of [bun, node]) {
  assert.deepEqual(JSON.parse(run(engine, [output])), expected)
}
