import assert from "node:assert/strict"
import path from "node:path"
import { pathToFileURL } from "node:url"
import ts from "typescript"

// Execution probe for the canonical compiler fixture. Instrument copies only;
// product shape checks below inspect the original emitted TypeScript.
const input = process.argv[2]
const globalMode = process.argv[3] === "globals"
assert(input, "expected an emitted TypeScript module")
const source = await Bun.file(input).text()
const ast = ts.createSourceFile(input, source, ts.ScriptTarget.Latest, true)
const sum = ast.statements.flatMap((statement) =>
  ts.isVariableStatement(statement)
    ? statement.declarationList.declarations.filter(
        (declaration) =>
          ts.isIdentifier(declaration.name) && declaration.name.text === "sum"
      )
    : []
)
assert.equal(sum.length, 1)
let loops = 0
function checkShape(node: ts.Node, insideLoop = false) {
  if (ts.isTypeNode(node)) return
  const looping = insideLoop || ts.isWhileStatement(node)
  if (ts.isWhileStatement(node)) loops++
  if (looping) {
    assert(
      !ts.isArrayLiteralExpression(node),
      "loop allocates an argument array"
    )
    assert(!ts.isObjectLiteralExpression(node), "loop allocates a tail marker")
    assert(!ts.isNewExpression(node), "loop allocates a constructed value")
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      assert.notEqual(node.expression.text, "sum", "self recursion remains")
    }
  }
  ts.forEachChild(node, (child) => checkShape(child, looping))
}
checkShape(sum[0])
assert.equal(loops, 1)
if (globalMode) {
  let reads = 0
  function checkRead(node: ts.Node, insideLoop = false) {
    const looping = insideLoop || ts.isWhileStatement(node)
    if (ts.isIdentifier(node) && node.text === "increment") {
      assert(looping, "global read moved outside its loop branch")
      reads++
    }
    ts.forEachChild(node, (child) => checkRead(child, looping))
  }
  checkRead(sum[0])
  assert.equal(reads, 1, "original global read was inlined or duplicated")
}

const program = ts.createProgram([input], {
  noEmit: true,
  strict: true,
  skipLibCheck: true,
  target: ts.ScriptTarget.ES2022,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
})
const diagnostics = ts.getPreEmitDiagnostics(program)
assert.equal(
  diagnostics.length,
  0,
  ts.formatDiagnosticsWithColorAndContext(diagnostics, {
    getCanonicalFileName: (name) => name,
    getCurrentDirectory: () => process.cwd(),
    getNewLine: () => "\n",
  })
)

const edits: { start: number; end: number; text: string }[] = []
const wrappers: string[] = []
if (globalMode) {
  // Instrument copies of the two bodies only. A trap lets us compare an
  // untaken branch and a failed read without changing the product source.
  for (const statement of ast.statements) {
    if (!ts.isVariableStatement(statement)) continue
    for (const declaration of statement.declarationList.declarations) {
      if (
        !ts.isIdentifier(declaration.name) ||
        !["sum", "retained"].includes(declaration.name.text)
      )
        continue
      function replaceRead(node: ts.Node) {
        if (ts.isIdentifier(node) && node.text === "increment") {
          edits.push({
            start: node.getStart(ast),
            end: node.end,
            text: "$probeReadGlobal()",
          })
        }
        ts.forEachChild(node, replaceRead)
      }
      replaceRead(declaration)
    }
  }
  wrappers.push(
    "let $probeReady = true; export const $probeSetReady = (value: boolean) => { $probeReady = value; }; const $probeReadGlobal = (): number => { $probeTrace.push(['global',0,0]); if (!$probeReady) throw new Error('uninitialized global'); return increment; };"
  )
}
let importEnd = 0
for (const statement of ast.statements) {
  if (!ts.isImportDeclaration(statement)) continue
  importEnd = statement.end
  const bindings = statement.importClause?.namedBindings
  if (!bindings || !ts.isNamedImports(bindings)) continue
  for (const binding of bindings.elements) {
    const exported = binding.propertyName?.text ?? binding.name.text
    const specifier = (statement.moduleSpecifier as ts.StringLiteral).text
    let wrapper: string | undefined
    const name = binding.name.text
    const original = `${name}$raw`
    if (
      specifier === "@seseragi/runtime/int" &&
      ["add", "subtract"].includes(exported)
    ) {
      const operator = exported === "add" ? "+" : "-"
      wrapper = `const ${name} = (a: number, b: number): number => { $probeTrace.push(["${operator}",a,b]); return ${original}(a,b); };`
    }
    if (specifier === "@seseragi/runtime/equality" && exported === "intEq") {
      wrapper = `const ${name} = { ...${original}, eq: (a: number) => (b: number): boolean => { $probeTrace.push(["==",a,b]); return ${original}.eq(a)(b); } };`
    }
    if (!wrapper) continue
    edits.push({
      start: binding.name.getStart(ast),
      end: binding.name.end,
      text: original,
    })
    wrappers.push(wrapper)
  }
}
assert.equal(
  wrappers.length,
  globalMode ? 4 : 3,
  "expected the three canonical primitive imports"
)
edits.push({
  start: importEnd,
  end: importEnd,
  text: `\nconst $probeTrace: [string,number,number][] = [];\nexport const $probeTakeTrace = () => { const value = [...$probeTrace]; $probeTrace.length = 0; return value; };\n${wrappers.join("\n")}\n`,
})
let instrumented = source
for (const edit of edits.sort((a, b) => b.start - a.start)) {
  instrumented =
    instrumented.slice(0, edit.start) + edit.text + instrumented.slice(edit.end)
}
const directory = path.dirname(input)
const copy = path.join(directory, "instrumented.ts")
const module = path.join(directory, "instrumented.mjs")
const harness = path.join(directory, "scalar-check.mjs")
await Bun.write(copy, instrumented)
const built = await Bun.build({
  entrypoints: [copy],
  target: "node",
  format: "esm",
})
assert(built.success, built.logs.map(String).join("\n"))
assert.equal(built.outputs.length, 1)
await Bun.write(module, built.outputs[0])
await Bun.write(
  harness,
  `
import assert from "node:assert/strict";
import { sum, retained, probe, $probeTakeTrace as trace ${globalMode ? ", $probeSetReady as ready" : ""} } from ${JSON.stringify(pathToFileURL(module).href)};
const MAX = 9007199254740991;
function observe(fn,n,seed) {
 trace(); let outcome;
 try { outcome = { value: fn(n)(seed) }; }
 catch (error) { outcome = { defect: error.name, message: error.message }; }
 return { outcome, trace: trace() };
}
const cases = [];
${
  globalMode
    ? `ready(false);
for (const n of [0,1]) {
 const actual = observe(sum,n,42), ordinary = observe(retained,n,42);
 assert.deepEqual(actual,ordinary);
 if(n===0) { assert.deepEqual(actual.outcome,{value:42}); assert(!actual.trace.some(x=>x[0]==='global')); }
 else { assert(actual.outcome.defect); assert.equal(actual.trace.filter(x=>x[0]==='global').length,1); }
}
ready(true);`
    : ""
}
for (let n=0;n<=10;n++) for(const seed of [0,1,MAX,MAX-1,-MAX,-MAX+1]) cases.push([n,seed]);
cases.push([-MAX,0],[-MAX,-MAX],[-MAX,MAX]);
for (const [n,seed] of cases) {
 const actual = observe(sum,n,seed), ordinary = observe(retained,n,seed);
 assert.deepEqual(actual,ordinary,JSON.stringify({n,seed}));
 if(n>=0) {
  const exact = BigInt(seed)+${globalMode ? "BigInt(n)*3n" : "BigInt(n)*BigInt(n+1)/2n"};
  if(exact>=-BigInt(MAX)&&exact<=BigInt(MAX)) assert.deepEqual(actual.outcome,{value:Number(exact)});
  else assert(actual.outcome.defect);
 } else {
  assert(actual.outcome.defect); assert.deepEqual(actual.trace.at(-1),["-",-MAX,1]);
 }
}
const partial=sum(40); assert.equal(partial(1),${globalMode ? 121 : 821}); assert.equal(partial(2),${globalMode ? 122 : 822});
assert.deepEqual(probe(undefined),${globalMode ? "[6,7,4,22,true]" : "[821,822,821,true,35,42,3628800]"});
console.log(JSON.stringify({cases:cases.length,primitiveTraceAgrees:true,partialAndProbeAgree:true}));
`
)
for (const runtime of [process.execPath, "node"]) {
  const result = Bun.spawnSync([runtime, harness], {
    stdout: "pipe",
    stderr: "pipe",
  })
  assert(result.success, result.stderr.toString())
  console.log(`${runtime}: ${result.stdout.toString().trim()}`)
}
