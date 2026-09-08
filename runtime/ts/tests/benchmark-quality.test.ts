import { expect, test } from "bun:test"
import { portableBenchmarks } from "../fixtures/benchmark-quality/portable"
import { runBenchmarks } from "../src/benchmark-runner"

test("minimum runtime quality suite measures actual stack, backpressure, copy, SSR and Signal boundaries", async () => {
  const report = await runBenchmarks(
    [{ name: "quality", benchmarks: portableBenchmarks }],
    {
      warmup: 0,
      samples: 3,
      minimumSampleMs: 1,
      regressionThresholdPercent: 5,
      seed: "0",
      timeoutMs: 30000,
      cleanupGraceMs: 100,
    },
    {
      languageVersion: "0.61.6",
      compilerVersion: "0.61.6",
      runtimeVersion: "0.61.6",
      profile: "release",
      target: { identity: "bun-runtime-test", version: Bun.version },
      host: {
        os: process.platform,
        architecture: process.arch,
        cpu: "test",
        logicalCores: 1,
        gcAvailable: true,
      },
      timerResolutionNs: 1,
    },
    { nowNs: () => Number(process.hrtime.bigint()) }
  )
  expect(report.cases).toHaveLength(5)
  for (const entry of report.cases) {
    expect(entry.failure).toBeUndefined()
    expect(entry.status).toBe("passed")
    expect(entry.samples).toHaveLength(3)
    expect(
      entry.samples!.every((sample) => sample * entry.iterations! >= 1_000_000)
    ).toBe(true)
  }
}, 30000)
