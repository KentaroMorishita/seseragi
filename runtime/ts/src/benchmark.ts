import type { Console } from "./console-service"
import { type Effect, fail as effectFail, type Unit } from "./effect"
import type { Logger } from "./logger-service"
import type { Random } from "./random"
import { concat, indent, line, renderDocument, text } from "./show"

export type BenchmarkEnvironment = Readonly<{
  random: Random
  console: Console
  logger: Logger
}>
export type BenchmarkFailure = Readonly<{
  tag: "ExplicitBenchmarkFailure"
  value: string
}>
export const ExplicitBenchmarkFailure = (value: string): BenchmarkFailure =>
  Object.freeze({ tag: "ExplicitBenchmarkFailure", value })
export type Benchmark =
  | Readonly<{
      kind: "case"
      name: string
      body: Effect<BenchmarkEnvironment, BenchmarkFailure, Unit>
    }>
  | Readonly<{ kind: "suite"; name: string; children: readonly Benchmark[] }>
  | Readonly<{ kind: "input-size"; size: number; child: Benchmark }>

export function benchmark(
  name: string,
  body: Effect<BenchmarkEnvironment, BenchmarkFailure, Unit>
): Benchmark {
  return Object.freeze({ kind: "case", name, body })
}
export function suite(name: string, children: readonly Benchmark[]): Benchmark {
  return Object.freeze({
    kind: "suite",
    name,
    children: Object.freeze([...children]),
  })
}
export function inputSize(size: number, child: Benchmark): Benchmark {
  return Object.freeze({ kind: "input-size", size, child })
}
// This runtime feature is an optimization barrier. No observable allocation,
// global retention, identity conversion or evaluation is added.
export function blackBox<A>(value: A): A {
  return value
}
export function fail(message: string): Effect<unknown, BenchmarkFailure, Unit> {
  return effectFail(ExplicitBenchmarkFailure(message))
}

export const benchmarkFailureEq = /* @__PURE__ */ Object.freeze({
  eq: (left: BenchmarkFailure) => (right: BenchmarkFailure) =>
    left.value === right.value,
})
const failureDocument = (value: BenchmarkFailure) =>
  concat([
    text("ExplicitBenchmarkFailure"),
    indent(concat([line, text(value.value)])),
  ])
export const benchmarkFailureShow = /* @__PURE__ */ Object.freeze({
  show: (value: BenchmarkFailure) => renderDocument(failureDocument(value)),
  document: failureDocument,
})
