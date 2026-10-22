import { expect, test } from "bun:test"
import ts from "typescript"
import { checkpointTransformer } from "../src/runtime/execution-budget"

test("loop guards cover all loop forms and arrow callbacks without changing values", () => {
  const source = `let n = 0;
  for (let i = 0; i < 10; i++) n++;
  while (n < 20) n++;
  do n++; while (n < 30);
  for (const k in { a: 1, b: 2 }) n++;
  for (const v of [1, 2]) n++;
  const arrow = x => x + 1;
  function recurse(x) { return x ? recurse(x - 1) : arrow(n); }
  return recurse(4);`
  const tree = ts.createSourceFile(
    "program.js",
    source,
    ts.ScriptTarget.ES2022,
    true,
    ts.ScriptKind.JS
  )
  const result = ts.transform(tree, [checkpointTransformer(ts, "check")])
  try {
    const program = new Function(
      "check",
      ts.createPrinter().printFile(result.transformed[0]!)
    )
    let calls = 0
    expect(
      program(() => {
        calls++
      })
    ).toBe(35)
    expect(calls).toBe(40)
    calls = 0
    expect(() =>
      program(() => {
        if (++calls === 8) throw new Error("budget")
      })
    ).toThrow("budget")
  } finally {
    result.dispose()
  }
})
