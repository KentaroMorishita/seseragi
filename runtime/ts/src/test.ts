import { fromUint8Array, type Bytes } from "./bytes"
import { createInstant, durationNanoseconds, type Duration } from "./clock-value"
import type { Clock } from "./clock"
import type { Console } from "./console-service"
import {
  createEffectExecution,
  type Effect,
  fail as effectFail,
  isEffectCancellation,
  run,
  type Unit,
  unit,
} from "./effect"
import type { LogEvent, Logger } from "./logger-service"
import { renderLogEvent } from "./logger-service"
import type { Random } from "./random"
import { EmptyRandomIntRange, InvalidProbability } from "./random"
import { serviceFailure, serviceSuccess } from "./service"
import type { Debug } from "./show"
import { renderDebug } from "./show"
import { Just, type Maybe, Nothing } from "./sum"

type Eq<Value> = (left: Value) => (right: Value) => boolean

export type TestFailure =
  | Readonly<{
      readonly tag: "AssertionFailed"
      readonly value: Readonly<{
        readonly message: string
        readonly expected: Maybe<string>
        readonly actual: Maybe<string>
      }>
    }>
  | Readonly<{ readonly tag: "ExpectedTypedFailure" }>
  | Readonly<{ readonly tag: "TypedFailureDidNotMatch"; readonly value: string }>
  | Readonly<{ readonly tag: "ExplicitTestFailure"; readonly value: string }>

export const AssertionFailed = (value: {
  readonly message: string
  readonly expected: Maybe<string>
  readonly actual: Maybe<string>
}): TestFailure => Object.freeze({ tag: "AssertionFailed", value })

export const ExpectedTypedFailure: TestFailure = Object.freeze({
  tag: "ExpectedTypedFailure",
})

export const TypedFailureDidNotMatch = (value: string): TestFailure =>
  Object.freeze({ tag: "TypedFailureDidNotMatch", value })

export const ExplicitTestFailure = (value: string): TestFailure =>
  Object.freeze({ tag: "ExplicitTestFailure", value })

type TestCase = Readonly<{
  readonly kind: "case"
  readonly name: string
  readonly body: Effect<TestEnvironment, TestFailure, Unit>
}>

type TestSuite = Readonly<{
  readonly kind: "suite"
  readonly name: string
  readonly children: ReadonlyArray<Test>
}>

type TestSkip = Readonly<{
  readonly kind: "skip"
  readonly reason: string
  readonly child: Test
}>

type TestTimeout = Readonly<{
  readonly kind: "timeout"
  readonly duration: Duration
  readonly child: Test
}>

export type Test = TestCase | TestSuite | TestSkip | TestTimeout

export type TestEnvironment = Readonly<{
  readonly clock: Clock
  readonly random: Random
  readonly console: Console
  readonly logger: Logger
}>

export function test(
  name: string,
  body: Effect<TestEnvironment, TestFailure, Unit>
): Test {
  return Object.freeze({ kind: "case", name, body })
}

export function suite(name: string, children: ReadonlyArray<Test>): Test {
  return Object.freeze({ kind: "suite", name, children: Object.freeze([...children]) })
}

export function skip(reason: string, child: Test): Test {
  return Object.freeze({ kind: "skip", reason, child })
}

export function timeout(duration: Duration, child: Test): Test {
  return Object.freeze({ kind: "timeout", duration, child })
}

export function equal<Value>(
  expected: Value,
  actual: Value,
  equality: Eq<Value>,
  debug: Debug<Value>
): Effect<unknown, TestFailure, Unit> {
  return equality(expected)(actual)
    ? () => unit
    : effectFail(
        AssertionFailed({
          message: "values are not equal",
          expected: Just(renderDebug(debug, expected)),
          actual: Just(renderDebug(debug, actual)),
        })
      )
}

export function notEqual<Value>(
  expected: Value,
  actual: Value,
  equality: Eq<Value>,
  debug: Debug<Value>
): Effect<unknown, TestFailure, Unit> {
  return !equality(expected)(actual)
    ? () => unit
    : effectFail(
        AssertionFailed({
          message: "values are equal",
          expected: Nothing,
          actual: Just(renderDebug(debug, actual)),
        })
      )
}

export function isTrue(value: boolean): Effect<unknown, TestFailure, Unit> {
  return value
    ? () => unit
    : effectFail(
        AssertionFailed({
          message: "expected true",
          expected: Just("true"),
          actual: Just("false"),
        })
      )
}

export function isFalse(value: boolean): Effect<unknown, TestFailure, Unit> {
  return !value
    ? () => unit
    : effectFail(
        AssertionFailed({
          message: "expected false",
          expected: Just("false"),
          actual: Just("true"),
        })
      )
}

export function fail(message: string): Effect<unknown, TestFailure, never> {
  return effectFail(ExplicitTestFailure(message))
}

export function expectFailure<Environment, Failure, Success>(
  predicate: (failure: Failure) => boolean,
  source: Effect<Environment, Failure, Success>,
  debug: Debug<Failure>
): Effect<Environment, TestFailure, Unit> {
  return async (environment, context) => {
    const result = await run(source, environment, context)
    if (result.kind === "success") {
      return effectFail(ExpectedTypedFailure)(environment, context)
    }
    if (!predicate(result.error)) {
      return effectFail(
        TypedFailureDidNotMatch(renderDebug(debug, result.error))
      )(environment, context)
    }
    return unit
  }
}

export type TestModule = Readonly<{ readonly name: string; readonly tests: Test }>

export type TestRunOptions = Readonly<{
  readonly filter?: string
  readonly exact?: string
  readonly jobs: number
  readonly timeoutMs: number
  readonly cleanupGraceMs: number
  readonly seed: number
}>

type FlatCase = Readonly<{
  readonly index: number
  readonly name: string
  readonly body: Effect<TestEnvironment, TestFailure, Unit>
  readonly skipReason?: string
  readonly timeoutMs?: number
}>

type CaseResult = Readonly<{
  readonly status: "passed" | "failed" | "skipped"
  readonly name: string
  readonly reason?: string
  readonly detail?: string
  readonly stdout: string
  readonly stderr: string
}>

export async function runTestModules(
  modules: ReadonlyArray<TestModule>,
  options: TestRunOptions
): Promise<number> {
  let cases: FlatCase[]
  try {
    cases = discover(modules)
  } catch (error) {
    process.stderr.write(`seseragi: ${messageOf(error)}\n`)
    return 2
  }
  cases = cases.filter((entry) =>
    options.exact !== undefined
      ? entry.name === options.exact
      : options.filter === undefined || entry.name.includes(options.filter)
  )
  if (cases.length === 0) {
    process.stderr.write("seseragi: test selection matched zero cases\n")
    return 2
  }

  const results = new Array<CaseResult>(cases.length)
  let next = 0
  const workers = Array.from(
    { length: Math.min(options.jobs, cases.length) },
    async () => {
      while (next < cases.length) {
        const position = next
        next += 1
        const entry = cases[position]
        if (entry !== undefined) {
          results[position] = await runCase(entry, options)
        }
      }
    }
  )
  await Promise.all(workers)

  let passed = 0
  let failed = 0
  let skipped = 0
  for (const result of results) {
    if (result.status === "passed") {
      passed += 1
      process.stdout.write(`PASS ${result.name}\n`)
    } else if (result.status === "skipped") {
      skipped += 1
      process.stdout.write(`SKIP ${result.name} -- ${result.reason ?? "skipped"}\n`)
    } else {
      failed += 1
      process.stdout.write(`FAIL ${result.name}\n`)
      if (result.detail !== undefined) {
        process.stderr.write(`${result.name}: ${result.detail}\n`)
      }
    }
    if (result.stdout.length > 0) {
      process.stderr.write(`${result.name} stdout:\n${result.stdout}`)
    }
    if (result.stderr.length > 0) {
      process.stderr.write(`${result.name} stderr:\n${result.stderr}`)
    }
  }
  process.stdout.write(`${passed} passed; ${failed} failed; ${skipped} skipped\n`)
  return failed === 0 ? 0 : 1
}

function discover(modules: ReadonlyArray<TestModule>): FlatCase[] {
  const cases: FlatCase[] = []
  for (const module of modules) {
    validateName("module", module.name)
    flatten(module.tests, [module.name], {}, cases)
  }
  const names = new Set<string>()
  for (const entry of cases) {
    if (names.has(entry.name)) throw new Error(`duplicate test name ${entry.name}`)
    names.add(entry.name)
  }
  return cases
}

function flatten(
  tree: Test,
  parents: ReadonlyArray<string>,
  inherited: Readonly<{ skipReason?: string; timeoutMs?: number }>,
  cases: FlatCase[]
): void {
  if (tree.kind === "skip") {
    validateName("skip reason", tree.reason)
    flatten(tree.child, parents, { ...inherited, skipReason: tree.reason }, cases)
    return
  }
  if (tree.kind === "timeout") {
    const timeoutMs = Number(durationNanoseconds(tree.duration) / 1_000_000n)
    flatten(tree.child, parents, { ...inherited, timeoutMs }, cases)
    return
  }
  validateName(tree.kind, tree.name)
  if (tree.kind === "suite") {
    for (const child of tree.children) {
      flatten(child, [...parents, tree.name], inherited, cases)
    }
    return
  }
  cases.push({
    index: cases.length,
    name: [...parents, tree.name].join("::"),
    body: tree.body,
    ...inherited,
  })
}

function validateName(kind: string, value: string): void {
  if (value.length === 0 || value.includes("::") || /[\r\n]/u.test(value)) {
    throw new Error(`${kind} name is not canonical: ${JSON.stringify(value)}`)
  }
}

async function runCase(
  entry: FlatCase,
  options: TestRunOptions
): Promise<CaseResult> {
  if (entry.skipReason !== undefined) {
    return {
      status: "skipped",
      name: entry.name,
      reason: entry.skipReason,
      stdout: "",
      stderr: "",
    }
  }
  const output = { stdout: "", stderr: "" }
  const execution = createEffectExecution()
  const environment = testEnvironment(options.seed, entry.index, output)
  const timeoutMs = entry.timeoutMs ?? options.timeoutMs
  let timer: ReturnType<typeof setTimeout> | undefined
  const timeoutSignal = new Promise<"timeout">((resolve) => {
    timer = setTimeout(() => resolve("timeout"), timeoutMs)
  })
  const body = run(entry.body, environment, execution.context)
    .then((result) => ({ kind: "result" as const, result }))
    .catch((error: unknown) => ({ kind: "defect" as const, error }))
  const outcome = await Promise.race([body, timeoutSignal])
  if (timer !== undefined) clearTimeout(timer)

  if (outcome === "timeout") {
    const clean = await within(execution.cancel(), options.cleanupGraceMs)
    return {
      status: "failed",
      name: entry.name,
      detail: clean ? `timed out after ${timeoutMs} ms` : "resource leak after timeout",
      ...output,
    }
  }
  if (outcome.kind === "defect") {
    const clean = await within(execution.cancel(), options.cleanupGraceMs)
    return {
      status: "failed",
      name: entry.name,
      detail: !clean
        ? "resource leak after defect"
        : isEffectCancellation(outcome.error)
          ? "cancelled"
          : `defect: ${messageOf(outcome.error)}`,
      ...output,
    }
  }
  const clean = await within(execution.close(), options.cleanupGraceMs)
  if (!clean) {
    return {
      status: "failed",
      name: entry.name,
      detail: "resource leak during cleanup",
      ...output,
    }
  }
  return outcome.result.kind === "success"
    ? { status: "passed", name: entry.name, ...output }
    : {
        status: "failed",
        name: entry.name,
        detail: renderFailure(outcome.result.error),
        ...output,
      }
}

function testEnvironment(
  seed: number,
  index: number,
  output: { stdout: string; stderr: string }
): TestEnvironment {
  let instant = 0n
  let state = (BigInt(seed) ^ (BigInt(index) + 0x9e3779b97f4a7c15n)) & 0xffff_ffff_ffff_ffffn
  const next = (): bigint => {
    state ^= state << 13n
    state ^= state >> 7n
    state ^= state << 17n
    state &= 0xffff_ffff_ffff_ffffn
    return state
  }
  const clock: Clock = Object.freeze({
    async now() {
      return createInstant(instant)
    },
    async sleep(duration, context) {
      if (context.cancelled) return unit
      instant += durationNanoseconds(duration)
      return unit
    },
  })
  const random: Random = Object.freeze({
    async algorithmId() {
      return "seseragi-test-xorshift64"
    },
    async nextBool() {
      return (next() & 1n) === 1n
    },
    async nextInt() {
      return Number(BigInt.asIntN(32, next()))
    },
    async intBetween(lower, upperExclusive) {
      if (upperExclusive <= lower) {
        return serviceFailure(EmptyRandomIntRange({ lower, upperExclusive }))
      }
      const width = BigInt(upperExclusive - lower)
      return serviceSuccess(lower + Number(next() % width))
    },
    async unitFloat() {
      return Number(next() >> 11n) / 9_007_199_254_740_992
    },
    async chance(probability) {
      if (probability < 0 || probability > 1 || !Number.isFinite(probability)) {
        return serviceFailure(InvalidProbability(probability))
      }
      return serviceSuccess(Number(next() >> 11n) / 9_007_199_254_740_992 < probability)
    },
    async randomBytes(size) {
      const bytes = new Uint8Array(size)
      for (let cursor = 0; cursor < size; cursor += 1) {
        bytes[cursor] = Number(next() & 0xffn)
      }
      return fromUint8Array(bytes) as Bytes
    },
    async chooseIndex(length) {
      return length === 0 ? 0 : Number(next() % BigInt(length))
    },
    async shuffleIndices(length) {
      const values = Array.from({ length }, (_, index_) => index_)
      for (let cursor = length - 1; cursor > 0; cursor -= 1) {
        const other = Number(next() % BigInt(cursor + 1))
        const value = values[cursor]
        values[cursor] = values[other] as number
        values[other] = value as number
      }
      return values
    },
  })
  const console: Console = Object.freeze({
    print(value) {
      output.stdout += value
      return serviceSuccess(unit)
    },
    println(value) {
      output.stdout += `${value}\n`
      return serviceSuccess(unit)
    },
    error(value) {
      output.stderr += value
      return serviceSuccess(unit)
    },
    errorLine(value) {
      output.stderr += `${value}\n`
      return serviceSuccess(unit)
    },
    flush() {
      return serviceSuccess(unit)
    },
  })
  const logger: Logger = Object.freeze({
    log(event: LogEvent) {
      output.stderr += `${renderLogEvent(event)}\n`
      return serviceSuccess(unit)
    },
  })
  return Object.freeze({ clock, random, console, logger })
}

async function within(source: Promise<void>, timeoutMs: number): Promise<boolean> {
  let timer: ReturnType<typeof setTimeout> | undefined
  const expired = new Promise<false>((resolve) => {
    timer = setTimeout(() => resolve(false), timeoutMs)
  })
  const result = await Promise.race([source.then(() => true), expired])
  if (timer !== undefined) clearTimeout(timer)
  return result
}

function renderFailure(failure: TestFailure): string {
  if (failure.tag === "ExpectedTypedFailure") return "expected a typed failure"
  if (failure.tag === "ExplicitTestFailure") return failure.value
  if (failure.tag === "TypedFailureDidNotMatch") {
    return `typed failure did not match: ${failure.value}`
  }
  const expected = failure.value.expected.tag === "Just" ? `; expected ${failure.value.expected.value}` : ""
  const actual = failure.value.actual.tag === "Just" ? `; actual ${failure.value.actual.value}` : ""
  return `${failure.value.message}${expected}${actual}`
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
