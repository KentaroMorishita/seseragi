import { randomUUID } from "node:crypto"
import {
  closeSync,
  existsSync,
  fsyncSync,
  openSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs"
import { arch, cpus, platform } from "node:os"
import { resolve } from "node:path"
import {
  type BenchmarkComparison,
  compareBaseline,
  validateBaseline,
} from "./benchmark-baseline"
import {
  type BenchmarkConfig,
  BenchmarkDiscoveryError,
  type BenchmarkMetadata,
  type BenchmarkModule,
  type BenchmarkReport,
  runBenchmarks,
} from "./benchmark-runner"

export type BenchmarkHostOptions = {
  config: BenchmarkConfig
  filter?: string
  exact?: string
  baseline?: string
  saveBaseline?: string
  json: boolean
  version: string
}
export function saveBaseline(path: string, report: BenchmarkReport): void {
  validateBaseline(report)
  const destination = resolve(path)
  const temporary = `${destination}.${randomUUID()}.tmp`
  let descriptor: number | undefined
  try {
    descriptor = openSync(temporary, "wx", 0o600)
    writeFileSync(descriptor, `${JSON.stringify(report, null, 2)}\n`)
    fsyncSync(descriptor)
    closeSync(descriptor)
    descriptor = undefined
    renameSync(temporary, destination)
  } finally {
    if (descriptor !== undefined) closeSync(descriptor)
    if (existsSync(temporary)) unlinkSync(temporary)
  }
}
export async function runBenchmarkModules(
  modules: readonly BenchmarkModule[],
  options: BenchmarkHostOptions
): Promise<number> {
  const processors = cpus()
  const globals = globalThis as { gc?: unknown; Bun?: { gc?: unknown } }
  const metadata: BenchmarkMetadata = {
    languageVersion: options.version,
    compilerVersion: options.version,
    runtimeVersion: options.version,
    profile: "release",
    target: {
      identity: "seseragi/bun-process",
      version: process.versions.bun ?? process.version,
    },
    host: {
      os: platform(),
      architecture: arch(),
      cpu: processors[0]?.model ?? "unknown",
      logicalCores: Math.max(1, processors.length),
      gcAvailable:
        typeof globals.gc === "function" ||
        typeof globals.Bun?.gc === "function",
    },
    timerResolutionNs: 1,
  }
  let baseline: BenchmarkReport | undefined
  if (options.baseline !== undefined) {
    try {
      baseline = validateBaseline(
        JSON.parse(readFileSync(options.baseline, "utf8"))
      )
      compareBaseline(
        {
          schema: 1,
          kind: "benchmark-report",
          metadata,
          config: options.config,
          cases: [],
        },
        baseline
      )
    } catch (error) {
      process.stderr.write(`benchmark baseline: ${String(error)}\n`)
      return 1
    }
  }
  const cancellation = new AbortController()
  const cancel = () => cancellation.abort()
  process.on("SIGINT", cancel)
  process.on("SIGTERM", cancel)
  try {
    const origin = process.hrtime.bigint()
    const report = await runBenchmarks(
      modules,
      options.config,
      metadata,
      {
        nowNs: () => Number(process.hrtime.bigint() - origin),
        signal: cancellation.signal,
      },
      { filter: options.filter, exact: options.exact }
    )
    const comparison: BenchmarkComparison[] | undefined =
      baseline === undefined ? undefined : compareBaseline(report, baseline)
    const failed =
      report.cases.some((entry) => entry.status === "failed") ||
      comparison?.some((entry) =>
        ["regression", "incomparable", "failure"].includes(entry.status)
      ) === true
    if (
      options.saveBaseline !== undefined &&
      report.cases.every((entry) => entry.status === "passed")
    ) {
      try {
        saveBaseline(options.saveBaseline, report)
      } catch (error) {
        process.stderr.write(`benchmark baseline save: ${String(error)}\n`)
        return 1
      }
    }
    if (options.json)
      process.stdout.write(
        `${JSON.stringify({ ...report, ...(comparison === undefined ? {} : { comparison }) })}\n`
      )
    else {
      for (const entry of report.cases) {
        if (entry.status === "failed")
          process.stdout.write(
            `FAIL ${entry.name}: ${entry.failure?.kind}: ${entry.failure?.detail}\n`
          )
        else {
          const summary = entry.summary!
          process.stdout.write(
            `${entry.name}: median ${summary.median} ns; MAD ${summary.medianAbsoluteDeviation}; min ${summary.minimum}; max ${summary.maximum}; ${summary.sampleCount} samples x ${entry.iterations}; input ${entry.inputSize ?? "unspecified"}\n`
          )
        }
      }
      for (const entry of comparison ?? [])
        process.stdout.write(`${entry.status.toUpperCase()} ${entry.name}\n`)
    }
    return failed ? 1 : 0
  } catch (error) {
    process.stderr.write(`benchmark: ${String(error)}\n`)
    return error instanceof BenchmarkDiscoveryError ? 2 : 1
  } finally {
    process.off("SIGINT", cancel)
    process.off("SIGTERM", cancel)
  }
}
