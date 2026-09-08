import { benchmark, inputSize, suite } from "../../src/benchmark"
import { runBenchmarks } from "../../src/benchmark-runner"
import { createBrowserDom } from "../../src/browser/dom"
import {
  bindRegion,
  bindText,
  content,
  createDomTarget,
  defaultOptions,
  mountContent,
  unmount,
} from "../../src/dom"
import { type Effect, run, unit } from "../../src/effect"
import { div, span } from "../../src/html"
import {
  combine,
  distinct,
  make,
  planSet,
  transaction,
  update,
} from "../../src/signal"

async function value<A>(
  work: Effect<Record<string, never>, unknown, A>
): Promise<A> {
  const result = await run(work, {})
  if (result.kind !== "success")
    throw new Error("benchmark setup/update failed")
  return result.value
}
function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}
const root = document.createElement("main")
document.body.append(root)
const leaf = await value(make("0"))
const left = await value(make(0))
const right = await value(make(0))
const repeated = await value(make("same"))
const region = await value(make(content(span({ children: "0" }), [])))
const combined = combine((a: number) => (b: number) => `${a}:${b}`, left, right)
const dom = createBrowserDom(document, () => undefined)
const mounted = await run(
  mountContent(
    defaultOptions(unit),
    createDomTarget(root),
    () => () => unit,
    content(
      div({
        children: [
          span({ id: "static", children: "static" }),
          span({ id: "leaf", children: "0" }),
          span({ id: "transaction", children: "0:0" }),
          span({ id: "distinct", children: "same" }),
          div({ id: "region", children: "0" }),
        ],
      }),
      [
        bindText("#leaf", leaf),
        bindText("#transaction", combined),
        bindText(
          "#distinct",
          distinct((a: string) => (b: string) => a === b, repeated)
        ),
        bindRegion("#region", region),
      ]
    )
  ),
  { dom: dom.service }
)
assert(mounted.kind === "success", "mount failed")
const staticNode = root.querySelector("#static")
const leafNode = root.querySelector("#leaf")
const regionNode = root.querySelector("#region")
let writes = 0
const observer = new MutationObserver((records) => {
  writes += records.length
})
observer.observe(root.querySelector("#distinct")!, {
  subtree: true,
  childList: true,
  characterData: true,
})
let count = 0
try {
  const report = await runBenchmarks(
    [
      {
        name: "browser",
        benchmarks: suite("reactive-dom", [
          inputSize(
            1,
            benchmark("leaf", async () => {
              count++
              await value(update(() => String(count), leaf))
              assert(
                root.querySelector("#leaf") === leafNode &&
                  leafNode?.textContent === String(count),
                "leaf replacement"
              )
            })
          ),
          inputSize(
            1,
            benchmark("region", async () => {
              count++
              await value(
                update(
                  () => content(span({ children: String(count) }), []),
                  region
                )
              )
              assert(
                root.querySelector("#region") === regionNode &&
                  regionNode?.textContent === String(count),
                "region boundary replacement"
              )
            })
          ),
          inputSize(
            2,
            benchmark("transaction", async () => {
              count++
              await value(
                transaction([planSet(count, left), planSet(count, right)])
              )
              assert(
                root.querySelector("#transaction")?.textContent ===
                  `${count}:${count}`,
                "torn transaction"
              )
            })
          ),
          inputSize(
            1,
            benchmark("distinct", async () => {
              await value(update(() => "same", repeated))
              assert(writes === 0, "distinct produced DOM writes")
            })
          ),
        ]),
      },
    ],
    {
      warmup: 1,
      samples: 3,
      minimumSampleMs: 1,
      regressionThresholdPercent: 5,
      seed: "0",
      timeoutMs: 10000,
      cleanupGraceMs: 100,
    },
    {
      languageVersion: "0.61.6",
      compilerVersion: "0.61.6",
      runtimeVersion: "0.61.6",
      profile: "release",
      target: {
        identity: "seseragi/browser-performance",
        version: navigator.userAgent,
      },
      host: {
        os: navigator.platform,
        architecture: "browser-unavailable",
        cpu: "browser-unavailable",
        logicalCores: navigator.hardwareConcurrency || 1,
        gcAvailable: false,
      },
      timerResolutionNs: 100_000,
    },
    { nowNs: () => performance.now() * 1e6 }
  )
  assert(
    root.querySelector("#static") === staticNode,
    "unrelated static subtree replaced"
  )
  assert(
    report.cases.every((entry) => entry.status === "passed"),
    JSON.stringify(report.cases)
  )
  ;(
    globalThis as typeof globalThis & { benchmarkReport?: unknown }
  ).benchmarkReport = report
} finally {
  observer.disconnect()
  await value(unmount(mounted.value))
  assert(root.childNodes.length === 0, "benchmark teardown leaked DOM")
  root.remove()
}
document.documentElement.dataset.benchmark = "complete"
