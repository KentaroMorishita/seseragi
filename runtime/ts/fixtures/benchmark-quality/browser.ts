import { benchmark, inputSize, suite } from "../../src/benchmark"
import { runBenchmarks } from "../../src/benchmark-runner"
import {
  type BrowserDomTraceEvent,
  createBrowserDom,
} from "../../src/browser/dom"
import {
  bind,
  bindChecked,
  bindRegion,
  bindStyle,
  bindText,
  content,
  createDomTarget,
  defaultOptions,
  mountContent,
  numberAttributeTarget,
  unmount,
} from "../../src/dom"
import { type Effect, run, unit } from "../../src/effect"
import { div, elementRef, fragment, input, span } from "../../src/html"
import {
  combine,
  distinct,
  make,
  planSet,
  transaction,
  update,
} from "../../src/signal"
import { Just } from "../../src/sum"

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
const checked = await value(make(false))
const opacity = await value(make(Just("0")))
const region = await value(make(content(span({ children: "0" }), [])))
const keyed = await value(
  make(
    content(
      fragment([
        span({ key: "a", id: "key-a", children: "a" }),
        span({ key: "b", id: "key-b", children: "b" }),
      ]),
      []
    )
  )
)
const pointerLike = await value(make(0))
const pointerRef = elementRef("pointer-like")
const combined = combine((a: number) => (b: number) => `${a}:${b}`, left, right)
const trace: BrowserDomTraceEvent[] = []
const dom = createBrowserDom(document, () => undefined, {
  trace(event) {
    trace.push(event)
    if (event.sequence === 0) throw new Error("ignored trace sink defect")
  },
})
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
          input({ id: "property", type: "checkbox", checked: false }),
          span({ id: "styled", children: "styled" }),
          div({ id: "region", children: "0" }),
          div({
            id: "keyed",
            children: [
              span({ key: "a", id: "key-a", children: "a" }),
              span({ key: "b", id: "key-b", children: "b" }),
            ],
          }),
          span({
            elementRef: pointerRef,
            id: "pointer-like",
          }),
        ],
      }),
      [
        bindText("#leaf", leaf),
        bindText("#transaction", combined),
        bindText(
          "#distinct",
          distinct((a: string) => (b: string) => a === b, repeated)
        ),
        bindChecked("#property", checked),
        bindStyle("#styled", "opacity", opacity),
        bindRegion("#region", region),
        bindRegion("#keyed", keyed),
        bind(numberAttributeTarget(pointerRef, "data-x"), pointerLike),
      ]
    )
  ),
  { dom: dom.service }
)
assert(mounted.kind === "success", "mount failed")
const staticNode = root.querySelector("#static")
const leafNode = root.querySelector("#leaf")
const propertyNode = root.querySelector("#property")
const styledNode = root.querySelector("#styled")
const regionNode = root.querySelector("#region")
const keyedA = root.querySelector("#key-a")
const keyedB = root.querySelector("#key-b")
const pointerNode = root.querySelector("#pointer-like")
await value(update(() => "private-binding-value", leaf))
assert(
  !JSON.stringify(trace).includes("private-binding-value"),
  "binding trace leaked an application value"
)
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
let keyedExpanded = false
try {
  const report = await runBenchmarks(
    [
      {
        name: "browser",
        benchmarks: suite("reactive-dom", [
          inputSize(
            3,
            benchmark("leaf", async () => {
              const traceStart = trace.length
              count++
              await value(
                transaction([
                  planSet(String(count), leaf),
                  planSet(count % 2 === 1, checked),
                  planSet(Just(String(count)), opacity),
                ])
              )
              assert(
                root.querySelector("#leaf") === leafNode &&
                  leafNode?.textContent === String(count) &&
                  root.querySelector("#property") === propertyNode &&
                  (propertyNode as HTMLInputElement).checked ===
                    (count % 2 === 1) &&
                  root.querySelector("#styled") === styledNode &&
                  (styledNode as HTMLElement).style.opacity === String(count),
                "leaf replacement"
              )
              const updates = trace
                .slice(traceStart)
                .filter(
                  (event) =>
                    event.type === "binding-update" &&
                    ["#leaf", "#property", "#styled"].includes(
                      event.logicalTarget.value
                    )
                )
              assert(
                updates.length === 3 &&
                  updates.every((event) => event.outcome === "write") &&
                  updates.some((event) => event.mutations.text === 1) &&
                  updates.some((event) => event.mutations.property === 1) &&
                  updates.some((event) => event.mutations.style === 1) &&
                  new Set(updates.map((event) => event.transactionId)).size ===
                    1,
                `leaf trace ${JSON.stringify(updates.at(-1))}`
              )
            })
          ),
          inputSize(
            1,
            benchmark("region", async () => {
              const traceStart = trace.length
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
              const updates = trace
                .slice(traceStart)
                .filter(
                  (event) =>
                    event.type === "binding-update" &&
                    event.bindingKind === "region" &&
                    event.logicalTarget.value === "#region"
                )
              assert(
                updates.length === 1 &&
                  updates[0]?.outcome === "write" &&
                  updates[0].mutations.replaced === 1,
                `structural trace ${JSON.stringify(updates.at(-1))}`
              )
            })
          ),
          inputSize(
            3,
            benchmark("keyed", async () => {
              const traceStart = trace.length
              keyedExpanded = !keyedExpanded
              await value(
                update(
                  () =>
                    content(
                      keyedExpanded
                        ? fragment([
                            span({ key: "b", id: "key-b", children: "b" }),
                            span({ key: "c", id: "key-c", children: "c" }),
                            span({ key: "a", id: "key-a", children: "a" }),
                          ])
                        : fragment([
                            span({ key: "a", id: "key-a", children: "a" }),
                            span({ key: "b", id: "key-b", children: "b" }),
                          ]),
                      []
                    ),
                  keyed
                )
              )
              assert(
                root.querySelector("#key-a") === keyedA &&
                  root.querySelector("#key-b") === keyedB,
                "keyed identity"
              )
              const updates = trace
                .slice(traceStart)
                .filter(
                  (event) =>
                    event.type === "binding-update" &&
                    event.bindingKind === "region" &&
                    event.logicalTarget.value === "#keyed"
                )
              assert(
                updates.length === 1 &&
                  updates[0]?.mutations.replaced === 0 &&
                  updates[0].outcome === "write" &&
                  updates[0].mutations.moved +
                    updates[0].mutations.inserted +
                    updates[0].mutations.removed >
                    0,
                `keyed trace ${JSON.stringify(updates.at(-1))}`
              )
            })
          ),
          inputSize(
            16,
            benchmark("pointer-like", async () => {
              const traceStart = trace.length
              for (let index = 0; index < 16; index += 1) {
                count++
                await value(update(() => count, pointerLike))
              }
              assert(
                root.querySelector("#pointer-like") === pointerNode &&
                  pointerNode?.getAttribute("data-x") === String(count),
                "pointer-like node replacement"
              )
              const updates = trace
                .slice(traceStart)
                .filter(
                  (event) =>
                    event.type === "binding-update" &&
                    event.bindingKind === "number-attribute" &&
                    event.logicalTarget.value === "pointer-like"
                )
              assert(
                updates.length === 16 &&
                  updates.every(
                    (event) =>
                      event.outcome === "write" &&
                      event.mutations.attribute === 1 &&
                      event.mutations.replaced === 0
                  ),
                `pointer-like trace ${JSON.stringify(updates.at(-1))}`
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
            2,
            benchmark("distinct", async () => {
              const traceStart = trace.length
              await value(update(() => "same", repeated))
              await value(update((value) => value, leaf))
              assert(writes === 0, "distinct produced DOM writes")
              assert(
                trace
                  .slice(traceStart)
                  .some(
                    (event) =>
                      event.type === "binding-update" &&
                      event.logicalTarget.value === "#leaf" &&
                      event.outcome === "equal-skip"
                  ),
                `equal leaf update trace ${JSON.stringify(trace.at(-1))}`
              )
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
    globalThis as typeof globalThis & {
      benchmarkReport?: unknown
      benchmarkDomTrace?: readonly BrowserDomTraceEvent[]
    }
  ).benchmarkReport = report
  ;(
    globalThis as typeof globalThis & {
      benchmarkDomTrace?: readonly BrowserDomTraceEvent[]
    }
  ).benchmarkDomTrace = trace
} finally {
  observer.disconnect()
  await value(unmount(mounted.value))
  assert(root.childNodes.length === 0, "benchmark teardown leaked DOM")
  assert(
    trace.at(-1)?.type === "scope" &&
      trace.at(-1)?.operation === "cleanup" &&
      trace.at(-1)?.activeSubscriptions === 0 &&
      trace.at(-1)?.activeListeners === 0,
    "benchmark teardown leaked subscriptions or listeners"
  )
  root.remove()
}
document.documentElement.dataset.benchmark = "complete"
