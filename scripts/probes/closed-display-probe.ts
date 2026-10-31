// Component observation only; timing always uses the ordinary generated module.
import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import path from "node:path"
import { pathToFileURL } from "node:url"
import ts from "typescript"

const [input] = process.argv.slice(2)
assert(input, "expected the original emitted tuple Show module")
const source = await Bun.file(input).text()
let factories = 0
const tree = ts.createSourceFile(input, source, ts.ScriptTarget.Latest, true)
const transformed = ts.transform(tree, [
  (context) => {
    const visit: ts.Visitor = (node) => {
      if (
        ts.isVariableDeclaration(node) &&
        ts.isIdentifier(node.name) &&
        node.name.text === "create" &&
        node.initializer &&
        ts.isArrowFunction(node.initializer) &&
        !ts.isBlock(node.initializer.body)
      ) {
        factories++
        const arrow = node.initializer
        return ts.factory.updateVariableDeclaration(
          node,
          node.name,
          node.exclamationToken,
          node.type,
          ts.factory.updateArrowFunction(
            arrow,
            arrow.modifiers,
            arrow.typeParameters,
            arrow.parameters,
            arrow.type,
            arrow.equalsGreaterThanToken,
            ts.factory.createCallExpression(
              ts.factory.createIdentifier("__ssrgTrackFactory"),
              undefined,
              [arrow.body as ts.Expression]
            )
          )
        )
      }
      return ts.visitEachChild(node, visit, context)
    }
    return (node) => ts.visitNode(node, visit) as ts.SourceFile
  },
])
assert.equal(factories, 1, "one proved tuple factory in the original fixture")
const directory = path.dirname(input)
const observed = path.join(directory, "observed.ts")
await Bun.write(
  observed,
  "const __ssrgTrackFactory = <Value>(value: Value): Value => { (globalThis as any).__ssrgFactoryReads++; return value; };\n" +
    ts.createPrinter().printFile(transformed.transformed[0])
)
transformed.dispose()
for (const [file, name] of [
  [input, "normal"],
  [observed, "observed"],
]) {
  const build = await Bun.build({
    entrypoints: [file],
    target: "node",
    format: "esm",
  })
  assert(build.success, build.logs.map(String).join("\n"))
  assert.equal(build.outputs.length, 1)
  await Bun.write(path.join(directory, `${name}.mjs`), build.outputs[0])
}
await Bun.write(
  path.join(directory, "observed-second.mjs"),
  Bun.file(path.join(directory, "observed.mjs"))
)
const runner = path.join(directory, "observe-cache.mjs")
await Bun.write(
  runner,
  `
import assert from "node:assert/strict";
globalThis.__ssrgFactoryReads=0;
const normal=await import(process.argv[2]);
const first=await import(process.argv[3]);
assert.equal(globalThis.__ssrgFactoryReads,0,"zero-step import remains cold");
for(let value=41;value<1041;value++) {
  const expected="("+value+", "+(value+1)+", True)";
  assert.equal(normal.kernel(value),expected);
  assert.equal(first.kernel(value),expected);
  assert.equal(globalThis.__ssrgFactoryReads,1,"one complete factory, no input captured");
}
const second=await import(process.argv[4]);
assert.equal(globalThis.__ssrgFactoryReads,1,"a new module also stays cold");
assert.equal(second.kernel(99),"(99, 100, True)");
assert.equal(globalThis.__ssrgFactoryReads,2,"each finite module owns its storage");
assert.equal(first.kernel(7),"(7, 8, True)");
assert.equal(globalThis.__ssrgFactoryReads,2);
console.log(JSON.stringify({factoryReads:2,moduleInstances:2,zeroStepCold:true,changingInputs:1000,normalBody:true}));
`
)
for (const engine of [process.execPath, "node"]) {
  const result = spawnSync(
    engine,
    [
      runner,
      ...["normal", "observed", "observed-second"].map(
        (name) => pathToFileURL(path.join(directory, `${name}.mjs`)).href
      ),
    ],
    { timeout: 60_000, encoding: "utf8" }
  )
  assert.equal(result.status, 0, `${engine}: ${result.stderr}`)
  const report = JSON.parse(result.stdout)
  assert.equal(report.factoryReads, 2)
  assert.equal(report.zeroStepCold, true)
  console.log(`${engine}: ${JSON.stringify(report)}`)
}
