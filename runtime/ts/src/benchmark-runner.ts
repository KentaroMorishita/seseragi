import type { Benchmark, BenchmarkEnvironment } from "./benchmark"
import { createEffectExecution, isEffectCancellation, run } from "./effect"
import { testEnvironment, validateName as validateTestName } from "./test"

export type BenchmarkModule = {
  readonly name: string
  readonly benchmarks: Benchmark
}
export type BenchmarkConfig = {
  warmup: number
  samples: number
  minimumSampleMs: number
  regressionThresholdPercent: number
  seed: string
  timeoutMs: number
  cleanupGraceMs: number
}
export type BenchmarkMetadata = {
  languageVersion: string
  compilerVersion: string
  runtimeVersion: string
  profile: string
  target: { identity: string; version: string }
  host: {
    os: string
    architecture: string
    cpu: string
    logicalCores: number
    gcAvailable: boolean
  }
  timerResolutionNs: number
}
export type Summary = {
  median: number
  medianAbsoluteDeviation: number
  minimum: number
  maximum: number
  sampleCount: number
}
export type BenchmarkCaseResult = {
  name: string
  inputSize: number | null
  status: "passed" | "failed"
  iterations?: number
  samples?: number[]
  summary?: Summary
  failure?: {
    kind: "typed-failure" | "defect" | "cancelled" | "timeout" | "resource-leak"
    detail: string
  }
  stdout: string
  stderr: string
}
export type BenchmarkReport = {
  schema: 1
  kind: "benchmark-report"
  metadata: BenchmarkMetadata
  config: BenchmarkConfig
  cases: BenchmarkCaseResult[]
}
export type BenchmarkHost = { nowNs: () => number; signal?: AbortSignal }
type FlatCase = {
  name: string
  inputSize: number | null
  body: Extract<Benchmark, { kind: "case" }>["body"]
}
export class BenchmarkDiscoveryError extends Error {}

function validateName(kind: string, name: string): void {
  try {
    validateTestName(kind, name)
  } catch (error) {
    throw new BenchmarkDiscoveryError(String(error))
  }
}

export function discoverBenchmarks(
  modules: readonly BenchmarkModule[]
): FlatCase[] {
  const cases: FlatCase[] = []
  const names = new Set<string>()
  const moduleNames = new Set<string>()
  for (const module of [...modules].sort((a, b) =>
    a.name < b.name ? -1 : a.name > b.name ? 1 : 0
  )) {
    validateName("module", module.name)
    if (moduleNames.has(module.name))
      throw new BenchmarkDiscoveryError(`duplicate module ${module.name}`)
    moduleNames.add(module.name)
    const pending = [
      {
        tree: module.benchmarks,
        parents: [module.name],
        inputSize: null as number | null,
      },
    ]
    while (pending.length) {
      const entry = pending.pop()!
      const tree = entry.tree
      if (tree === null || typeof tree !== "object")
        throw new BenchmarkDiscoveryError("invalid Benchmark value")
      if (tree.kind === "input-size") {
        if (!Number.isSafeInteger(tree.size) || tree.size <= 0)
          throw new BenchmarkDiscoveryError("inputSize must be a positive Int")
        pending.push({ ...entry, tree: tree.child, inputSize: tree.size })
        continue
      }
      validateName(tree.kind, tree.name)
      const parents = [...entry.parents, tree.name]
      if (tree.kind === "suite") {
        if (!Array.isArray(tree.children))
          throw new BenchmarkDiscoveryError("invalid Benchmark children")
        for (const child of [...tree.children].reverse())
          pending.push({ tree: child, parents, inputSize: entry.inputSize })
        continue
      }
      if (tree.kind !== "case" || typeof tree.body !== "function")
        throw new BenchmarkDiscoveryError("invalid Benchmark value")
      const name = parents.join("::")
      if (names.has(name))
        throw new BenchmarkDiscoveryError(`duplicate benchmark name ${name}`)
      names.add(name)
      cases.push({ name, inputSize: entry.inputSize, body: tree.body })
    }
  }
  return cases
}

function median(values: readonly number[]): number {
  const sorted = [...values].sort((a, b) => a - b)
  const middle = Math.floor(sorted.length / 2)
  return sorted.length % 2
    ? sorted[middle]!
    : sorted[middle - 1]! + (sorted[middle]! - sorted[middle - 1]!) / 2
}
export function summarize(samples: readonly number[]): Summary {
  if (
    samples.length < 3 ||
    samples.some((value) => !Number.isFinite(value) || value < 0)
  )
    throw new Error("invalid benchmark samples")
  const center = median(samples)
  return {
    median: center,
    medianAbsoluteDeviation: median(
      samples.map((value) => Math.abs(value - center))
    ),
    minimum: samples.reduce((left, right) => Math.min(left, right)),
    maximum: samples.reduce((left, right) => Math.max(left, right)),
    sampleCount: samples.length,
  }
}
function caseSeed(name: string): string {
  let value = 0xcbf29ce484222325n
  for (const byte of new TextEncoder().encode(name))
    value = BigInt.asUintN(64, (value ^ BigInt(byte)) * 0x100000001b3n)
  return value.toString()
}
class CaseFailure extends Error {
  constructor(readonly failure: NonNullable<BenchmarkCaseResult["failure"]>) {
    super(failure.detail)
  }
}
function detail(error: unknown): string {
  if (error instanceof Error) return error.message
  try {
    return JSON.stringify(error) ?? String(error)
  } catch {
    return String(error)
  }
}
async function cleanup(
  work: Promise<void>,
  milliseconds: number
): Promise<"done" | "timeout" | { error: unknown }> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      work.then(
        () => "done" as const,
        (error: unknown) => ({ error })
      ),
      new Promise<"timeout">((resolve) => {
        timer = setTimeout(() => resolve("timeout"), milliseconds)
      }),
    ])
  } finally {
    if (timer !== undefined) clearTimeout(timer)
  }
}
async function invoke(
  entry: FlatCase,
  environment: BenchmarkEnvironment,
  config: BenchmarkConfig,
  host: BenchmarkHost
): Promise<void> {
  const execution = createEffectExecution()
  const cancel = () => {
    void execution.cancel()
  }
  host.signal?.addEventListener("abort", cancel, { once: true })
  if (host.signal?.aborted) cancel()
  let timer: ReturnType<typeof setTimeout> | undefined
  let failure: NonNullable<BenchmarkCaseResult["failure"]> | undefined
  try {
    const outcome = await Promise.race([
      run(entry.body, environment, execution.context).then(
        (result) => ({ kind: "result" as const, result }),
        (error: unknown) => ({ kind: "error" as const, error })
      ),
      new Promise<{ kind: "timeout" }>((resolve) => {
        timer = setTimeout(() => resolve({ kind: "timeout" }), config.timeoutMs)
      }),
    ])
    if (outcome.kind === "timeout")
      failure = { kind: "timeout", detail: "benchmark body timed out" }
    else if (outcome.kind === "error")
      failure = {
        kind: isEffectCancellation(outcome.error) ? "cancelled" : "defect",
        detail: detail(outcome.error),
      }
    else if (outcome.result.kind === "failure")
      failure = { kind: "typed-failure", detail: detail(outcome.result.error) }
  } finally {
    if (timer !== undefined) clearTimeout(timer)
    const clean = await cleanup(
      failure && failure.kind !== "typed-failure"
        ? execution.cancel()
        : execution.close(),
      config.cleanupGraceMs
    )
    host.signal?.removeEventListener("abort", cancel)
    if (clean === "timeout")
      failure = {
        kind: "resource-leak",
        detail: "benchmark resource cleanup did not complete",
      }
    else if (clean !== "done")
      failure = { kind: "defect", detail: detail(clean.error) }
  }
  if (failure) throw new CaseFailure(failure)
}

export async function runBenchmarks(
  modules: readonly BenchmarkModule[],
  config: BenchmarkConfig,
  metadata: BenchmarkMetadata,
  host: BenchmarkHost,
  selection: { filter?: string; exact?: string } = {}
): Promise<BenchmarkReport> {
  if (
    !Number.isSafeInteger(config.warmup) ||
    config.warmup < 0 ||
    !Number.isSafeInteger(config.samples) ||
    config.samples < 3 ||
    !Number.isSafeInteger(config.minimumSampleMs) ||
    config.minimumSampleMs <= 0 ||
    !Number.isFinite(config.regressionThresholdPercent) ||
    config.regressionThresholdPercent < 0 ||
    !Number.isSafeInteger(config.timeoutMs) ||
    config.timeoutMs <= 0 ||
    !Number.isSafeInteger(config.cleanupGraceMs) ||
    config.cleanupGraceMs < 0
  )
    throw new BenchmarkDiscoveryError("invalid benchmark configuration")
  if (
    !/^-?\d+$/u.test(config.seed) ||
    BigInt(config.seed) < -(1n << 63n) ||
    BigInt(config.seed) >= 1n << 63n
  )
    throw new BenchmarkDiscoveryError("seed must be an Int64")
  if (selection.filter !== undefined && selection.exact !== undefined)
    throw new BenchmarkDiscoveryError("filter and exact are mutually exclusive")
  // Discovery and duplicate checking finish before any body runs, including
  // excluded cases. Filter selection cannot hide a malformed tree.
  const discovered = discoverBenchmarks(modules)
  const selected = discovered.filter((entry) =>
    selection.exact !== undefined
      ? entry.name === selection.exact
      : selection.filter === undefined || entry.name.includes(selection.filter)
  )
  if (!selected.length)
    throw new BenchmarkDiscoveryError("benchmark discovery selected zero cases")
  const report: BenchmarkReport = {
    schema: 1,
    kind: "benchmark-report",
    metadata,
    config,
    cases: [],
  }
  for (const entry of selected) {
    const output = { stdout: "", stderr: "" }
    const { random, console, logger } = testEnvironment(
      config.seed,
      caseSeed(entry.name),
      output
    )
    const environment = Object.freeze({ random, console, logger })
    const result: BenchmarkCaseResult = {
      name: entry.name,
      inputSize: entry.inputSize,
      status: "failed",
      ...output,
    }
    try {
      if (host.signal?.aborted)
        throw new CaseFailure({
          kind: "cancelled",
          detail: "benchmark run cancelled",
        })
      for (let warmup = 0; warmup < config.warmup; warmup++)
        await invoke(entry, environment, config, host)
      const deadline = performance.now() + config.timeoutMs
      const batch = async (iterations: number) => {
        const start = host.nowNs()
        for (let iteration = 0; iteration < iterations; iteration++) {
          if (iteration % 256 === 0 && performance.now() > deadline)
            throw new CaseFailure({
              kind: "timeout",
              detail: "benchmark measurement timed out",
            })
          await invoke(entry, environment, config, host)
        }
        const elapsed = host.nowNs() - start
        if (!Number.isFinite(elapsed) || elapsed < 0)
          throw new CaseFailure({
            kind: "defect",
            detail: "invalid monotonic measurement clock",
          })
        return elapsed
      }
      let iterations = 1
      const minimum = config.minimumSampleMs * 1_000_000
      let samples: number[] = []
      let calibrated = false
      // Recalibrate and discard short samples if JIT/host changes make the
      // calibrated batch too short. Every reported sample uses one fixed count.
      while (samples.length < config.samples) {
        const elapsed = await batch(iterations)
        if (elapsed < minimum) {
          const next =
            elapsed === 0
              ? iterations * 10
              : Math.max(
                  iterations + 1,
                  Math.ceil(((iterations * minimum) / elapsed) * 1.1)
                )
          if (!Number.isSafeInteger(next) || next > 1_000_000_000)
            throw new CaseFailure({
              kind: "defect",
              detail: "measurement clock could not calibrate a bounded batch",
            })
          iterations = next
          samples = []
          calibrated = false
          continue
        }
        if (!calibrated) {
          calibrated = true
          continue
        }
        samples.push(elapsed / iterations)
      }
      result.status = "passed"
      result.iterations = iterations
      result.samples = samples
      result.summary = summarize(samples)
    } catch (error) {
      result.failure =
        error instanceof CaseFailure
          ? error.failure
          : { kind: "defect", detail: detail(error) }
    }
    Object.assign(result, output)
    report.cases.push(result)
  }
  return report
}
