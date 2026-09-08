import { expect, test } from "bun:test"
import { benchmark, blackBox, fail, inputSize, suite } from "../src/benchmark"
import { compareBaseline, validateBaseline } from "../src/benchmark-baseline"
import {
  type BenchmarkConfig,
  type BenchmarkMetadata,
  discoverBenchmarks,
  runBenchmarks,
  summarize,
} from "../src/benchmark-runner"
import { acquireRelease, flatMap, succeed } from "../src/effect"
import { Empty } from "../src/list"

const config: BenchmarkConfig = {
  warmup: 2,
  samples: 3,
  minimumSampleMs: 1,
  regressionThresholdPercent: 5,
  seed: "42",
  timeoutMs: 1000,
  cleanupGraceMs: 10,
}
const metadata: BenchmarkMetadata = {
  languageVersion: "0.61.6",
  compilerVersion: "0.61.6",
  runtimeVersion: "0.61.6",
  profile: "release",
  target: { identity: "fake-clock", version: "1" },
  host: {
    os: "test",
    architecture: "test",
    cpu: "test",
    logicalCores: 1,
    gcAvailable: false,
  },
  timerResolutionNs: 1,
}

test("ordinary values discover in canonical order without running bodies", () => {
  let calls = 0
  const body = () => {
    calls++
  }
  const first = inputSize(
    10,
    suite("suite", [
      inputSize(20, benchmark("first", body)),
      benchmark("second", body),
    ])
  )
  const entries = discoverBenchmarks([
    { name: "z", benchmarks: benchmark("last", body) },
    { name: "a", benchmarks: first },
  ])
  expect(entries.map((entry) => [entry.name, entry.inputSize])).toEqual([
    ["a::suite::first", 20],
    ["a::suite::second", 10],
    ["z::last", null],
  ])
  expect(calls).toBe(0)
  const identity = { n: 1 }
  expect(blackBox(identity)).toBe(identity)
  expect(() =>
    discoverBenchmarks([
      {
        name: "a",
        benchmarks: suite("suite", [
          benchmark("same", body),
          benchmark("same", body),
        ]),
      },
    ])
  ).toThrow("duplicate")
  expect(() =>
    discoverBenchmarks([
      { name: "a", benchmarks: inputSize(0, benchmark("case", body)) },
    ])
  ).toThrow("positive")
})

test("warmup and calibration are excluded and statistics describe fixed-count samples", async () => {
  let time = 0
  let calls = 0
  const result = await runBenchmarks(
    [
      {
        name: "main",
        benchmarks: inputSize(
          5,
          benchmark("case", () => {
            calls++
            time += 1_000_000
          })
        ),
      },
    ],
    config,
    metadata,
    { nowNs: () => time }
  )
  expect(calls).toBe(6) // two warmup, one calibration, three reported samples
  expect(result.cases[0]).toMatchObject({
    status: "passed",
    inputSize: 5,
    iterations: 1,
    samples: [1_000_000, 1_000_000, 1_000_000],
    summary: {
      median: 1_000_000,
      medianAbsoluteDeviation: 0,
      minimum: 1_000_000,
      maximum: 1_000_000,
      sampleCount: 3,
    },
  })
  expect(validateBaseline(JSON.parse(JSON.stringify(result)))).toEqual(result)
  const baseline = structuredClone(result)
  const current = structuredClone(result)
  current.cases[0]!.summary!.median = 1_050_000
  expect(compareBaseline(current, baseline)[0]!.status).toBe("unchanged")
  current.cases[0]!.summary!.median++
  expect(compareBaseline(current, baseline)[0]!.status).toBe("regression")
  current.cases[0]!.inputSize = 6
  expect(compareBaseline(current, baseline)[0]!.status).toBe("incomparable")
  current.cases[0]!.name = "main::added"
  expect(
    compareBaseline(current, baseline).map((entry) => entry.status)
  ).toEqual(["new", "missing"])
  baseline.metadata.host.cpu = "other"
  expect(() => compareBaseline(current, baseline)).toThrow("incompatible")
})

test("Random is case-name seeded, captures are private, and application Clock is absent", async () => {
  const seen: Record<string, number[]> = {}
  const make = (name: string) =>
    benchmark(name, async (environment) => {
      expect("clock" in environment).toBe(false)
      seen[name] ??= []
      seen[name].push(await environment.random.nextInt(undefined as never))
      await environment.console.println(name, undefined as never)
      await environment.logger.log({
        level: { tag: "LogInfo" },
        message: `log-${name}`,
        fields: Empty,
      })(undefined as never)
    })
  let clock = 0
  const modules = [
    { name: "main", benchmarks: suite("suite", [make("a"), make("b")]) },
  ]
  const first = await runBenchmarks(modules, config, metadata, {
    nowNs: () => {
      clock += 1_000_000
      return clock
    },
  })
  const values = [...seen.b!]
  seen.b = []
  const second = await runBenchmarks(
    modules,
    config,
    metadata,
    {
      nowNs: () => {
        clock += 1_000_000
        return clock
      },
    },
    { exact: "main::suite::b" }
  )
  expect(seen.b).toEqual(values)
  expect(first.cases[0]!.stdout).not.toContain("b")
  expect(first.cases[0]!.stderr).toContain("log-a")
  expect(first.cases[0]!.stderr).not.toContain("log-b")
  expect(second.cases[0]!.stderr).toBe(first.cases[1]!.stderr)
  expect(second.cases[0]!.stdout).toBe(first.cases[1]!.stdout)
})

test("typed failure, defect and cleanup leak never become samples", async () => {
  const released: string[] = []
  const resource = acquireRelease(succeed(1), () => () => {
    released.push("released")
  })
  const leak = acquireRelease(
    succeed(1),
    () => () => new Promise<void>(() => {})
  )
  let now = 0
  const report = await runBenchmarks(
    [
      {
        name: "main",
        benchmarks: suite("suite", [
          benchmark(
            "failure",
            flatMap(resource, () => fail("expected"))
          ),
          benchmark("defect", () => {
            throw new Error("broken")
          }),
          benchmark(
            "leak",
            flatMap(leak, () => succeed(undefined))
          ),
        ]),
      },
    ],
    config,
    metadata,
    {
      nowNs: () => {
        now += 1_000_000
        return now
      },
    }
  )
  expect(report.cases.map((entry) => entry.failure?.kind)).toEqual([
    "typed-failure",
    "defect",
    "resource-leak",
  ])
  expect(report.cases.every((entry) => entry.samples === undefined)).toBe(true)
  expect(released).toEqual(["released"])
  expect(() => validateBaseline(report)).toThrow("failed")
})

test("timeout, cancellation and finalizer defects are bounded case failures", async () => {
  const controller = new AbortController()
  const modules = [
    {
      name: "main",
      benchmarks: suite("failure", [
        benchmark("timeout", () => new Promise<void>(() => {})),
        benchmark(
          "cleanup-defect",
          flatMap(
            acquireRelease(succeed(1), () => () => {
              throw new Error("finalizer")
            }),
            () => succeed(undefined)
          )
        ),
        benchmark("cancel", () => {
          controller.abort()
        }),
        benchmark("after-cancel", () => {
          throw new Error("must not execute")
        }),
      ]),
    },
  ]
  const report = await runBenchmarks(
    modules,
    { ...config, timeoutMs: 15 },
    metadata,
    { nowNs: () => 0, signal: controller.signal }
  )
  expect(report.cases.map((entry) => entry.failure?.kind)).toEqual([
    "timeout",
    "defect",
    "cancelled",
    "cancelled",
  ])
  expect(report.cases.every((entry) => entry.samples === undefined)).toBe(true)
})

test("invalid baseline values and duplicate names fail closed", async () => {
  let clock = 0
  const report = await runBenchmarks(
    [{ name: "main", benchmarks: benchmark("case", () => {}) }],
    config,
    metadata,
    {
      nowNs: () => {
        clock += 1_000_000
        return clock
      },
    }
  )
  for (const mutate of [
    (value: typeof report) => {
      value.cases.push({ ...value.cases[0]! })
    },
    (value: typeof report) => {
      value.cases[0]!.samples![0] = Infinity
    },
    (value: typeof report) => {
      value.cases[0]!.summary!.median = NaN
    },
    (value: typeof report) => {
      value.cases[0]!.summary!.maximum++
    },
    (value: typeof report) => {
      value.metadata.timerResolutionNs = 0
    },
  ]) {
    const invalid = structuredClone(report)
    mutate(invalid)
    expect(() => validateBaseline(invalid)).toThrow()
  }
  expect(() => validateBaseline({ ...report, schema: 2 })).toThrow()
  expect(() =>
    discoverBenchmarks([
      { name: "main", benchmarks: benchmark("bad::name", () => {}) },
    ])
  ).toThrow()
  await expect(
    runBenchmarks(
      [{ name: "main", benchmarks: suite("empty", []) }],
      config,
      metadata,
      { nowNs: () => 0 }
    )
  ).rejects.toThrow("zero")
})

test("a faster post-calibration body restarts sampling at a common larger iteration count", async () => {
  let clock = 0
  let calls = 0
  const report = await runBenchmarks(
    [
      {
        name: "main",
        benchmarks: benchmark("speedup", () => {
          calls++
          clock += calls <= 2 ? 1_000_000 : 100_000
        }),
      },
    ],
    { ...config, warmup: 0 },
    metadata,
    { nowNs: () => clock }
  )
  expect(report.cases[0]!.iterations).toBe(11)
  expect(report.cases[0]!.samples).toEqual([100_000, 100_000, 100_000])
  expect(calls).toBe(47) // calibration, discarded first sample, short sample, new calibration and three samples
})

test("every warmup, calibration and sample invocation owns a fresh resource scope", async () => {
  let acquired = 0
  let released = 0
  let clock = 0
  const resource = acquireRelease(
    () => {
      acquired++
      return acquired
    },
    () => () => {
      released++
      return undefined
    }
  )
  const report = await runBenchmarks(
    [
      {
        name: "main",
        benchmarks: benchmark(
          "resource",
          flatMap(resource, () => succeed(undefined))
        ),
      },
    ],
    config,
    metadata,
    {
      nowNs: () => {
        clock += 1_000_000
        return clock
      },
    }
  )
  expect(report.cases[0]!.status).toBe("passed")
  expect(acquired).toBe(6)
  expect(released).toBe(acquired)
})

test("finite large samples do not overflow median arithmetic", () => {
  expect(summarize(Array(4).fill(Number.MAX_VALUE))).toEqual({
    median: Number.MAX_VALUE,
    medianAbsoluteDeviation: 0,
    minimum: Number.MAX_VALUE,
    maximum: Number.MAX_VALUE,
    sampleCount: 4,
  })
})
