import { type BenchmarkReport, summarize } from "./benchmark-runner"

export type BenchmarkComparison = {
  name: string
  status:
    | "new"
    | "missing"
    | "regression"
    | "unchanged"
    | "incomparable"
    | "failure"
  baselineMedian?: number
  currentMedian?: number
}
function object(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value))
    throw new Error("invalid benchmark baseline object")
  return value as Record<string, unknown>
}
function text(value: unknown): string {
  if (typeof value !== "string" || !value.length)
    throw new Error("invalid baseline identity")
  return value
}
function finite(value: unknown, minimum = 0): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value < minimum)
    throw new Error("invalid baseline number")
  return value
}
function integer(value: unknown, minimum = 0): number {
  const result = finite(value, minimum)
  if (!Number.isSafeInteger(result)) throw new Error("invalid baseline integer")
  return result
}
export function validateBaseline(value: unknown): BenchmarkReport {
  const report = object(value)
  if (report.schema !== 1 || report.kind !== "benchmark-report")
    throw new Error("unsupported benchmark baseline schema")
  const metadata = object(report.metadata)
  for (const key of ["languageVersion", "compilerVersion", "runtimeVersion"]) {
    if (!/^\d+\.\d+\.\d+(?:[-+].*)?$/u.test(text(metadata[key])))
      throw new Error("invalid baseline toolchain version")
  }
  if (metadata.profile !== "release")
    throw new Error("benchmark baseline requires release profile")
  const target = object(metadata.target)
  text(target.identity)
  text(target.version)
  const host = object(metadata.host)
  for (const key of ["os", "architecture", "cpu"]) text(host[key])
  integer(host.logicalCores, 1)
  if (typeof host.gcAvailable !== "boolean")
    throw new Error("invalid GC availability")
  if (finite(metadata.timerResolutionNs) <= 0)
    throw new Error("invalid timer resolution")
  const config = object(report.config)
  integer(config.warmup)
  integer(config.samples, 3)
  integer(config.minimumSampleMs, 1)
  finite(config.regressionThresholdPercent)
  integer(config.timeoutMs, 1)
  integer(config.cleanupGraceMs)
  if (!/^-?\d+$/u.test(text(config.seed))) throw new Error("invalid seed")
  const seed = BigInt(config.seed as string)
  if (seed < -(1n << 63n) || seed >= 1n << 63n)
    throw new Error("seed is outside Int64")
  if (!Array.isArray(report.cases) || report.cases.length === 0)
    throw new Error("baseline requires cases")
  const names = new Set<string>()
  for (const item of report.cases) {
    const entry = object(item)
    const name = text(entry.name)
    if (names.has(name)) throw new Error(`duplicate baseline case ${name}`)
    names.add(name)
    if (entry.inputSize !== null) integer(entry.inputSize, 1)
    if (entry.status !== "passed")
      throw new Error("baseline cannot contain failed measurements")
    integer(entry.iterations, 1)
    if (
      !Array.isArray(entry.samples) ||
      entry.samples.length !== config.samples
    )
      throw new Error("baseline sample count mismatch")
    entry.samples.forEach((sample) => finite(sample))
    const computed = summarize(entry.samples as number[])
    const summary = object(entry.summary)
    for (const [key, expected] of Object.entries(computed)) {
      if (summary[key] !== expected)
        throw new Error(`invalid baseline summary ${name}:${key}`)
    }
    if (typeof entry.stdout !== "string" || typeof entry.stderr !== "string")
      throw new Error("invalid capture metadata")
  }
  return value as BenchmarkReport
}
export function compareBaseline(
  current: BenchmarkReport,
  input: unknown
): BenchmarkComparison[] {
  const baseline = validateBaseline(input)
  const a = current.metadata
  const b = baseline.metadata
  for (const key of [
    "languageVersion",
    "compilerVersion",
    "runtimeVersion",
  ] as const) {
    if (a[key].split(".")[0] !== b[key].split(".")[0])
      throw new Error(`incompatible baseline ${key}`)
  }
  if (
    a.profile !== b.profile ||
    a.target.identity !== b.target.identity ||
    a.target.version !== b.target.version ||
    a.host.os !== b.host.os ||
    a.host.architecture !== b.host.architecture ||
    a.host.cpu !== b.host.cpu ||
    a.timerResolutionNs !== b.timerResolutionNs
  )
    throw new Error("incompatible benchmark baseline host/profile/target/timer")
  const previous = new Map(baseline.cases.map((entry) => [entry.name, entry]))
  const comparison: BenchmarkComparison[] = []
  for (const entry of current.cases) {
    const old = previous.get(entry.name)
    previous.delete(entry.name)
    if (entry.status === "failed")
      comparison.push({ name: entry.name, status: "failure" })
    else if (!old) comparison.push({ name: entry.name, status: "new" })
    else if (entry.inputSize !== old.inputSize)
      comparison.push({ name: entry.name, status: "incomparable" })
    else {
      const baselineMedian = old.summary!.median
      const currentMedian = entry.summary!.median
      comparison.push({
        name: entry.name,
        status:
          currentMedian >
          baselineMedian * (1 + current.config.regressionThresholdPercent / 100)
            ? "regression"
            : "unchanged",
        baselineMedian,
        currentMedian,
      })
    }
  }
  for (const entry of previous.values())
    comparison.push({ name: entry.name, status: "missing" })
  return comparison
}
