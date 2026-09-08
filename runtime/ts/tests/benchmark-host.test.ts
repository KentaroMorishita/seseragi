import { expect, test } from "bun:test"
import { mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { saveBaseline } from "../src/benchmark-host"
import type { BenchmarkReport } from "../src/benchmark-runner"

test("baseline atomic replace preserves previous data on validation failure and leaves no temporary files", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-benchmark-baseline-"))
  try {
    const report: BenchmarkReport = {
      schema: 1,
      kind: "benchmark-report",
      config: {
        warmup: 0,
        samples: 3,
        minimumSampleMs: 1,
        regressionThresholdPercent: 5,
        seed: "0",
        timeoutMs: 1000,
        cleanupGraceMs: 10,
      },
      metadata: {
        languageVersion: "0.61.6",
        compilerVersion: "0.61.6",
        runtimeVersion: "0.61.6",
        profile: "release",
        target: { identity: "test", version: "1" },
        host: {
          os: "test",
          architecture: "test",
          cpu: "test",
          logicalCores: 1,
          gcAvailable: false,
        },
        timerResolutionNs: 1,
      },
      cases: [
        {
          name: "main::case",
          inputSize: 1,
          status: "passed",
          iterations: 1,
          samples: [1e6, 1e6, 1e6],
          summary: {
            median: 1e6,
            medianAbsoluteDeviation: 0,
            minimum: 1e6,
            maximum: 1e6,
            sampleCount: 3,
          },
          stdout: "",
          stderr: "",
        },
      ],
    }
    const path = join(directory, "baseline.json")
    saveBaseline(path, report)
    const first = readFileSync(path, "utf8")
    report.cases[0]!.samples![0] = Infinity
    expect(() => saveBaseline(path, report)).toThrow()
    expect(readFileSync(path, "utf8")).toBe(first)
    report.cases[0]!.samples![0] = 1e6
    report.cases[0]!.inputSize = 2
    saveBaseline(path, report)
    expect(JSON.parse(readFileSync(path, "utf8")).cases[0].inputSize).toBe(2)
    expect(() => saveBaseline(directory, report)).toThrow()
    expect(readdirSync(directory)).toEqual(["baseline.json"])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
