// Timing uses unmodified generated modules. Instrumentation is a separate run.
import { fullGC, heapStats } from "bun:jsc"
import { pathToFileURL } from "node:url"
import ts from "../../../../apps/playground/node_modules/typescript"
import { add } from "../../../../runtime/ts/src/int"

const [directory, processIndex = "0"] = process.argv.slice(2)
if (!directory) throw new Error("measurement output directory required")
if (Bun.version !== "1.3.11") throw new Error("Bun 1.3.11 required")

type Numeric = (seed: number) => number
type Factory = (seed: number) => (unit: undefined) => number
type Case = {
  name: string
  mode: "scalar" | "escaped"
  fn: Numeric | Factory
  instrumented?: Numeric | Factory
  expected: Numeric
}
const modules: Record<string, any> = {}
const instrumented: Record<string, any> = {}
const counts = { closure: 0, object: 0 }
;(globalThis as any).__ssrgPerfCounts = counts

function instrument(source: string): string {
  if (source.includes("__ssrgPerfTrack"))
    throw new Error("instrumentation name collision")
  const input = ts.createSourceFile(
    "generated.ts",
    source,
    ts.ScriptTarget.Latest,
    true
  )
  const transformed = ts.transform(input, [
    (context) => {
      const visit: ts.Visitor = (node) => {
        const visited = ts.visitEachChild(node, visit, context)
        const kind = ts.isArrowFunction(node)
          ? "closure"
          : ts.isObjectLiteralExpression(node)
            ? "object"
            : undefined
        return kind
          ? ts.factory.createCallExpression(
              ts.factory.createIdentifier("__ssrgPerfTrack"),
              undefined,
              [ts.factory.createStringLiteral(kind), visited as ts.Expression]
            )
          : visited
      }
      return (node) => ts.visitNode(node, visit) as ts.SourceFile
    },
  ])
  const printed = ts.createPrinter().printFile(transformed.transformed[0])
  transformed.dispose()
  return (
    "const __ssrgPerfTrack = (kind: string, value: any): any => { (globalThis as any).__ssrgPerfCounts[kind]++; return value; };\n" +
    printed
  )
}

for (const name of ["calls", "dictionaries"]) {
  const source = await Bun.file(`${directory}/${name}/release.ts`).text()
  const instrumentedPath = `${directory}/${name}/instrumented.ts`
  await Bun.write(instrumentedPath, instrument(source))
  modules[name] = await import(
    pathToFileURL(`${directory}/${name}/release.ts`).href
  )
  instrumented[name] = await import(pathToFileURL(instrumentedPath).href)
}

// Source arity identifies complete applications. Preserve the curried add for
// first-class/partial uses, redirecting only the private pure add(a)(b) site.
const callsCore = await Bun.file(`${directory}/calls/release.core.json`).json()
const addCore = callsCore.functions.find(
  (f: any) => f.symbol === "performance/main::add"
)
if (addCore?.parameters.length !== 2 || addCore.body.kind !== "binary")
  throw new Error("direct-call fixture changed")
const callsSource = await Bun.file(`${directory}/calls/release.ts`).text()
const callsInput = ts.createSourceFile(
  "calls.ts",
  callsSource,
  ts.ScriptTarget.Latest,
  true
)
let helper: ts.ArrowFunction | undefined
for (const statement of callsInput.statements) {
  if (!ts.isVariableStatement(statement)) continue
  for (const decl of statement.declarationList.declarations) {
    if (
      ts.isIdentifier(decl.name) &&
      decl.name.text === "add" &&
      decl.initializer &&
      ts.isArrowFunction(decl.initializer) &&
      ts.isArrowFunction(decl.initializer.body)
    ) {
      const outer = decl.initializer
      const inner = outer.body as ts.ArrowFunction
      helper = ts.factory.updateArrowFunction(
        outer,
        outer.modifiers,
        outer.typeParameters,
        [...outer.parameters, ...inner.parameters],
        inner.type,
        outer.equalsGreaterThanToken,
        inner.body
      )
    }
  }
}
if (!helper || callsSource.includes("__ssrgPerfDirectAdd"))
  throw new Error("direct helper unavailable")
let directCallCount = 0
const callsTransformed = ts.transform(callsInput, [
  (context) => {
    const visit: ts.Visitor = (node) => {
      const visited = ts.visitEachChild(node, visit, context)
      if (
        ts.isCallExpression(visited) &&
        visited.arguments.length === 1 &&
        ts.isCallExpression(visited.expression) &&
        visited.expression.arguments.length === 1 &&
        ts.isIdentifier(visited.expression.expression) &&
        visited.expression.expression.text === "add"
      ) {
        directCallCount++
        return ts.factory.createCallExpression(
          ts.factory.createIdentifier("__ssrgPerfDirectAdd"),
          undefined,
          [...visited.expression.arguments, ...visited.arguments]
        )
      }
      return visited
    }
    return (node) => ts.visitNode(node, visit) as ts.SourceFile
  },
])
const directPrinter = ts.createPrinter()
const directSource = `${directPrinter.printFile(callsTransformed.transformed[0])}\nconst __ssrgPerfDirectAdd = ${directPrinter.printNode(ts.EmitHint.Expression, helper, callsInput)};`
callsTransformed.dispose()
await Bun.write(`${directory}/calls/direct.ts`, directSource)
await Bun.write(
  `${directory}/calls/direct-instrumented.ts`,
  instrument(directSource)
)
modules.direct = await import(
  pathToFileURL(`${directory}/calls/direct.ts`).href
)
instrumented.direct = await import(
  pathToFileURL(`${directory}/calls/direct-instrumented.ts`).href
)

// Counterfactual only: share closed local dictionary factories. The CoreIr
// identifies eligible user instances; AST checks prove their factory body only
// constructs method closures and an object. Parameters remain unhoistable.
const core = await Bun.file(
  `${directory}/dictionaries/release.core.json`
).json()
const metadata = await Bun.file(
  `${directory}/dictionaries/release.metadata.json`
).json()
const eligible = new Set(
  core.instances
    .filter(
      (instance: any) =>
        instance.implementation.kind === "user-defined" &&
        !(instance.supertraitCount ?? 0) &&
        instance.implementation.methods.every(
          (method: any) => !method.constraints?.length
        )
    )
    .map((i: any) => i.identity)
)
const dictionaryNames = new Set<string>(
  metadata.instances
    .filter((i: any) => eligible.has(i.identity))
    .map((i: any) => i.dictionaryExport)
)
function unwrap(node: ts.Expression): ts.Expression {
  while (ts.isParenthesizedExpression(node) || ts.isAsExpression(node))
    node = node.expression
  return node
}
const source = await Bun.file(`${directory}/dictionaries/release.ts`).text()
const input = ts.createSourceFile(
  "generated.ts",
  source,
  ts.ScriptTarget.Latest,
  true
)
const staticDictionaries = new Set<string>()
const factoryNames = new Set<string>()
for (const statement of input.statements) {
  if (!ts.isVariableStatement(statement)) continue
  for (const decl of statement.declarationList.declarations) {
    if (
      !ts.isIdentifier(decl.name) ||
      !dictionaryNames.has(decl.name.text) ||
      !decl.initializer
    )
      continue
    const initializer = unwrap(decl.initializer)
    if (ts.isObjectLiteralExpression(initializer))
      staticDictionaries.add(decl.name.text)
    if (ts.isArrowFunction(initializer) && !ts.isBlock(initializer.body)) {
      const body = unwrap(initializer.body)
      if (
        ts.isObjectLiteralExpression(body) &&
        body.properties.every(
          (p) =>
            ts.isPropertyAssignment(p) &&
            ts.isArrowFunction(unwrap(p.initializer))
        )
      )
        factoryNames.add(decl.name.text)
    }
  }
}
const printer = ts.createPrinter()
const hoisted = new Map<
  string,
  { name: string; expression: ts.CallExpression }
>()
const generatedNames = new Set<string>()
const transformed = ts.transform(input, [
  (context) => {
    const visit: ts.Visitor = (node) => {
      const visited = ts.visitEachChild(node, visit, context)
      if (
        !ts.isCallExpression(visited) ||
        !ts.isIdentifier(visited.expression) ||
        !factoryNames.has(visited.expression.text)
      )
        return visited
      if (
        !visited.arguments.every(
          (argument) =>
            ts.isIdentifier(argument) &&
            (staticDictionaries.has(argument.text) ||
              generatedNames.has(argument.text))
        )
      )
        return visited
      const key = printer.printNode(ts.EmitHint.Expression, visited, input)
      let entry = hoisted.get(key)
      if (!entry) {
        const name = `__ssrgPerfDictionary${hoisted.size}`
        if (source.includes(name))
          throw new Error("hoisted dictionary name collision")
        entry = { name, expression: visited }
        hoisted.set(key, entry)
        generatedNames.add(name)
      }
      return ts.factory.createIdentifier(entry.name)
    }
    return (node) => ts.visitNode(node, visit) as ts.SourceFile
  },
])
const sharedSource =
  printer.printFile(transformed.transformed[0]) +
  "\n" +
  [...hoisted.values()]
    .map(
      (entry) =>
        `const ${entry.name} = ${printer.printNode(ts.EmitHint.Expression, entry.expression, input)};`
    )
    .join("\n")
transformed.dispose()
await Bun.write(`${directory}/dictionaries/shared.ts`, sharedSource)
await Bun.write(
  `${directory}/dictionaries/shared-instrumented.ts`,
  instrument(sharedSource)
)
modules.shared = await import(
  pathToFileURL(`${directory}/dictionaries/shared.ts`).href
)
instrumented.shared = await import(
  pathToFileURL(`${directory}/dictionaries/shared-instrumented.ts`).href
)

const expected = (seed: number) => seed + 7
const cases: Case[] = [
  { name: "host-bare", mode: "scalar", fn: expected, expected },
  {
    name: "host-checked",
    mode: "scalar",
    fn: (seed) => add(seed, 7),
    expected,
  },
  ...["saturated", "partial", "immediateLambda"].map(
    (name): Case => ({
      name,
      mode: "scalar",
      fn: modules.calls[name],
      instrumented: instrumented.calls[name],
      expected,
    })
  ),
  ...["saturated", "partial"].map(
    (name): Case => ({
      name: `${name}-direct`,
      mode: "scalar",
      fn: modules.direct[name],
      instrumented: instrumented.direct[name],
      expected,
    })
  ),
  ...["concrete", "nested"].map(
    (name): Case => ({
      name,
      mode: "scalar",
      fn: modules.dictionaries[name],
      instrumented: instrumented.dictionaries[name],
      expected,
    })
  ),
  {
    name: "host-checked-twice",
    mode: "scalar",
    fn: (seed) => add(add(seed, 7), add(seed, 7)),
    expected: (seed) => 2 * expected(seed),
  },
  {
    name: "returned",
    mode: "scalar",
    fn: modules.dictionaries.returned,
    instrumented: instrumented.dictionaries.returned,
    expected: (seed) => 2 * expected(seed),
  },
  ...["nestedEscaping", "specializedEscaping"].map(
    (name): Case => ({
      name,
      mode: "escaped",
      fn: modules.dictionaries[name],
      instrumented: instrumented.dictionaries[name],
      expected,
    })
  ),
  ...["nested", "returned", "nestedEscaping"].map(
    (name): Case => ({
      name: `${name}-shared`,
      mode: name === "nestedEscaping" ? "escaped" : "scalar",
      fn: modules.shared[name],
      instrumented: instrumented.shared[name],
      expected: name === "returned" ? (seed) => 2 * expected(seed) : expected,
    })
  ),
  {
    name: "parameterized",
    mode: "scalar",
    fn: (seed) =>
      modules.dictionaries.parameterized(seed)(
        modules.dictionaries.__ssrg$instance$Read$0
      ),
    instrumented: (seed) =>
      instrumented.dictionaries.parameterized(seed)(
        instrumented.dictionaries.__ssrg$instance$Read$0
      ),
    expected,
  },
  {
    name: "other",
    mode: "scalar",
    fn: modules.dictionaries.other,
    instrumented: instrumented.dictionaries.other,
    expected: (seed) => seed + 17,
  },
  {
    name: "other-shared",
    mode: "scalar",
    fn: modules.shared.other,
    instrumented: instrumented.shared.other,
    expected: (seed) => seed + 17,
  },
]
const inputs = new Int32Array(8192)
let state = 0x512739
for (let i = 0; i < inputs.length; i++) {
  state = (Math.imul(state, 1664525) + 1013904223) >>> 0
  inputs[i] = state & 4095
}
let blackhole = 0

function exercise(test: Case, iterations: number, fn = test.fn): number {
  let checksum = 0
  if (test.mode === "scalar") {
    const operation = fn as Numeric
    for (let i = 0; i < iterations; i++)
      checksum = (checksum + operation(inputs[i & 8191])) >>> 0
  } else {
    const factory = fn as Factory
    const escaped: ((unit: undefined) => number)[] = new Array(64)
    for (let start = 0; start < iterations; start += 64) {
      const length = Math.min(64, iterations - start)
      for (let i = 0; i < length; i++)
        escaped[i] = factory(inputs[(start + i) & 8191])
      for (let i = 0; i < length; i++)
        checksum = (checksum + escaped[i](undefined)) >>> 0
    }
  }
  blackhole = (blackhole ^ checksum) >>> 0
  return checksum
}

const cycleSums = new Map<Case, number>()
function expectedChecksum(test: Case, iterations: number): number {
  let cycle = cycleSums.get(test)
  if (cycle === undefined) {
    cycle = 0
    for (const input of inputs) cycle += test.expected(input)
    cycleSums.set(test, cycle)
  }
  let checksum = cycle * Math.floor(iterations / inputs.length)
  for (let i = 0; i < iterations % inputs.length; i++)
    checksum += test.expected(inputs[i])
  return checksum >>> 0
}

const samples: Record<string, number[]> = {}
const iterations: Record<string, number> = {}
const constructors: Record<string, { closure: number; object: number }> = {}
for (const test of cases) {
  for (const seed of [0, 1, 42, 4095]) {
    const value =
      test.mode === "scalar"
        ? (test.fn as Numeric)(seed)
        : (test.fn as Factory)(seed)(undefined)
    if (value !== test.expected(seed))
      throw new Error(`${test.name}: incorrect result`)
  }
  if (exercise(test, 10_000) !== expectedChecksum(test, 10_000))
    throw new Error("warmup checksum mismatch")
  // Calibrate independently, excluding compilation/module loading from timing.
  let count = 32768
  while (true) {
    const started = performance.now()
    exercise(test, count)
    if (performance.now() - started >= 20 || count >= 8_388_608) break
    count *= 2
  }
  iterations[test.name] = count
  samples[test.name] = []
  if (test.instrumented) {
    counts.closure = 0
    counts.object = 0
    if (
      exercise(test, 4096, test.instrumented) !== expectedChecksum(test, 4096)
    )
      throw new Error("instrumented result mismatch")
    constructors[test.name] = {
      closure: counts.closure / 4096,
      object: counts.object / 4096,
    }
  }
}

// Rotate order to reduce a fixed ordering advantage. Each case retains raw samples.
for (let round = 0; round < 9; round++) {
  for (let offset = 0; offset < cases.length; offset++) {
    const test = cases[(offset + round + Number(processIndex)) % cases.length]
    const count = iterations[test.name]
    const started = performance.now()
    const actual = exercise(test, count)
    const elapsed = performance.now() - started
    if (actual !== expectedChecksum(test, count))
      throw new Error(`${test.name}: measured checksum mismatch`)
    samples[test.name].push((elapsed * 1e6) / count)
  }
}

const tick = () => new Promise<void>((resolve) => setTimeout(resolve, 0))
async function snapshot() {
  await tick()
  fullGC()
  fullGC()
  const stats = heapStats()
  return {
    heapSize: stats.heapSize,
    objectCount: stats.objectCount,
    functions: stats.objectTypeCounts.Function ?? 0,
    environments: stats.objectTypeCounts.JSLexicalEnvironment ?? 0,
    arrays: stats.objectTypeCounts.Array ?? 0,
  }
}
const memory: Record<string, any> = {}
for (const name of [
  "specializedEscaping",
  "nestedEscaping",
  "nestedEscaping-shared",
  "captureOne-unused",
]) {
  let held: ((unit: undefined) => number)[] = []
  const before = await snapshot()
  const factory =
    name === "captureOne-unused"
      ? (seed: number) =>
          modules.calls.captureOne(seed)(new Array(512).fill(seed))
      : name.endsWith("-shared")
        ? modules.shared.nestedEscaping
        : modules.dictionaries[name]
  for (let i = 0; i < 20_000; i++) held.push(factory(inputs[i & 8191]))
  const retained = await snapshot()
  let checksum = 0
  for (let i = 0; i < held.length; i++) checksum += held[i](undefined)
  if (
    checksum !==
    Array.from({ length: 20_000 }, (_, i) =>
      name === "captureOne-unused"
        ? inputs[i & 8191]
        : expected(inputs[i & 8191])
    ).reduce((a, b) => a + b, 0)
  )
    throw new Error("retained checksum mismatch")
  held.length = 0
  held = []
  const released = await snapshot()
  memory[name] = { heldCount: 20_000, before, retained, released }
}

console.log(
  JSON.stringify({
    schema: 1,
    processIndex: Number(processIndex),
    bun: Bun.version,
    samples_ns_per_op: samples,
    iterations_per_sample: iterations,
    emitted_constructions_per_op: constructors,
    retained_heap: memory,
    blackhole,
    saturated_direct_call: {
      redirectedCallSites: directCallCount,
      sourceArity: addCore.parameters.length,
      scope:
        "counterfactual for private pure add; partial application unchanged",
    },
    dictionary_sharing: {
      eligibleCoreIdentities: [...eligible],
      factoryNames: [...factoryNames],
      declarations: [...hoisted.values()].map((entry) => ({
        name: entry.name,
        initializer: printer.printNode(
          ts.EmitHint.Expression,
          entry.expression,
          input
        ),
      })),
      scope:
        "counterfactual local user factory sharing; production output unchanged",
    },
  })
)
