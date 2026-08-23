import { createBrowserDom } from "../../src/browser/dom"
import {
  awaitMount,
  ClearRenderedDom,
  createDomTarget,
  type DomMount,
  defaultOptions,
  HydrateOrReplace,
  HydrateStrict,
  mount,
  PreserveRenderedDom,
  unmount,
} from "../../src/dom"
import { createEffectExecution, type Effect, run, unit } from "../../src/effect"
import { button, div, p, span } from "../../src/html"
import { serviceSuccess } from "../../src/service"
import {
  constant,
  type MutableSignal,
  make,
  map,
  update,
} from "../../src/signal"

declare global {
  interface Window {
    domLifecycleResult?: Readonly<{
      readonly strictMismatchPath: readonly number[]
      readonly dispatched: number
      readonly duplicateTargetRejected: boolean
      readonly coarseUpdateRendered: boolean
      readonly hydrationPreservedIdentity: boolean
      readonly replacementPreservedAncestor: boolean
      readonly cancellationReleasedTarget: boolean
      readonly targetRemoval: string
    }>
  }
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

async function effectValue<Value>(
  effect: Effect<Record<string, never>, unknown, Value>
) {
  const result = await run(effect, {})
  assert(result.kind === "success", "expected effect success")
  return result.value as Value
}

async function unmountTwice<Failure>(mounted: DomMount<Failure>) {
  for (let index = 0; index < 2; index += 1) {
    const result = await run(unmount(mounted), {})
    assert(result.kind === "success", "unmount must be idempotent")
  }
}

const host = document.createElement("main")
document.body.append(host)

const mismatchRoot = document.createElement("div")
mismatchRoot.innerHTML = "<p>server</p>"
host.append(mismatchRoot)
const mismatchDom = createBrowserDom(document, () => undefined)
const mismatch = await mismatchDom.service.mount(
  {
    ...defaultOptions(unit),
    hydration: HydrateStrict,
    cleanup: PreserveRenderedDom,
  },
  createDomTarget(mismatchRoot),
  async () => serviceSuccess(unit),
  constant(p({ children: "client" }))
)
assert(mismatch.kind === "failure", "strict mismatch must reject mount")
assert(mismatch.error.tag === "HydrationMismatch", "expected mismatch error")
assert(
  mismatchRoot.innerHTML === "<p>server</p>",
  "strict mismatch must not mutate server DOM"
)

const hydrationRoot = document.createElement("div")
hydrationRoot.innerHTML = '<button type="button">go</button>'
host.append(hydrationRoot)
const hydratedButton = hydrationRoot.firstElementChild
let dispatched = 0
const hydrationDom = createBrowserDom(document, () => undefined)
const hydration = await hydrationDom.service.mount(
  {
    ...defaultOptions(unit),
    hydration: HydrateStrict,
    cleanup: PreserveRenderedDom,
  },
  createDomTarget(hydrationRoot),
  async () => {
    dispatched += 1
    return serviceSuccess(unit)
  },
  constant(button({ onClick: "go", children: "go" }))
)
assert(hydration.kind === "success", "matching hydration must mount")
const hydrationPreservedIdentity =
  hydrationRoot.firstElementChild === hydratedButton
const duplicate = await hydrationDom.service.mount(
  defaultOptions(unit),
  createDomTarget(hydrationRoot),
  async () => serviceSuccess(unit),
  constant(p({ children: "duplicate" }))
)
const duplicateTargetRejected =
  duplicate.kind === "failure" &&
  duplicate.error.tag === "DomTargetAlreadyMounted"
hydrationRoot.querySelector("button")?.click()
await Promise.resolve()
await unmountTwice(hydration.value)
hydrationRoot.querySelector("button")?.click()
await Promise.resolve()
assert(dispatched === 1, "preserved DOM listener must be removed on unmount")

const replacementRoot = document.createElement("div")
replacementRoot.innerHTML = "<div><p>stable</p><span>server</span></div>"
host.append(replacementRoot)
const replacementSection = replacementRoot.firstElementChild
const stableParagraph = replacementSection?.firstElementChild
const replacementDom = createBrowserDom(document, () => undefined)
const replacement = await replacementDom.service.mount(
  {
    ...defaultOptions(unit),
    hydration: HydrateOrReplace,
    cleanup: ClearRenderedDom,
  },
  createDomTarget(replacementRoot),
  async () => serviceSuccess(unit),
  constant(
    div({
      children: [p({ children: "stable" }), span({ children: "client" })],
    })
  )
)
assert(replacement.kind === "success", "replace hydration must mount")
const replacementPreservedAncestor =
  replacementRoot.firstElementChild === replacementSection &&
  replacementRoot.querySelector("p") === stableParagraph &&
  replacementRoot.querySelector("span")?.textContent === "client"
await unmountTwice(replacement.value)

const updateRoot = document.createElement("div")
host.append(updateRoot)
const updateDom = createBrowserDom(document, () => undefined)
const count = await effectValue<MutableSignal<number>>(make(0))
const content = map(
  (value: number) => span({ id: "count", children: String(value) }),
  count
)
const updating = await updateDom.service.mount(
  defaultOptions(unit),
  createDomTarget(updateRoot),
  async () => serviceSuccess(unit),
  content
)
assert(updating.kind === "success", "fresh mount must succeed")
await effectValue(update((value: number) => value + 1, count))
await Promise.resolve()
const coarseUpdateRendered =
  updateRoot.querySelector("#count")?.textContent === "1"
assert(updateRoot.textContent === "1", "signal update must render text")
await unmountTwice(updating.value)
assert(
  updateRoot.childNodes.length === 0,
  "clear cleanup must remove managed DOM"
)
await effectValue(update((value: number) => value + 1, count))
await Promise.resolve()
assert(
  updateRoot.childNodes.length === 0,
  "unmount must unsubscribe from Signal"
)

const cancellationRoot = document.createElement("div")
host.append(cancellationRoot)
const cancellationDom = createBrowserDom(document, () => undefined)
const execution = createEffectExecution()
const mountedByEffect = await run(
  mount(
    defaultOptions(unit),
    createDomTarget(cancellationRoot),
    () => async () => unit,
    constant(p({ children: "cancel me" }))
  ),
  { dom: cancellationDom.service },
  execution.context
)
assert(mountedByEffect.kind === "success", "effect mount must succeed")
await execution.cancel()
assert(
  cancellationRoot.childNodes.length === 0,
  "root cancellation must cleanup"
)
const remounted = await cancellationDom.service.mount(
  defaultOptions(unit),
  createDomTarget(cancellationRoot),
  async () => serviceSuccess(unit),
  constant(p({ children: "again" }))
)
const cancellationReleasedTarget = remounted.kind === "success"
if (remounted.kind === "success") await unmountTwice(remounted.value)

const removedRoot = document.createElement("div")
host.append(removedRoot)
const removedDom = createBrowserDom(document, () => undefined)
const removed = await removedDom.service.mount(
  defaultOptions(unit),
  createDomTarget(removedRoot),
  async () => serviceSuccess(unit),
  constant(p({ children: "remove me" }))
)
assert(removed.kind === "success", "removal mount must succeed")
removedRoot.remove()
await new Promise((resolve) => setTimeout(resolve, 0))
const removalResult = await run(awaitMount(removed.value), {})
assert(removalResult.kind === "failure", "target removal must fail awaitMount")
const targetRemoval =
  removalResult.kind === "failure" && removalResult.error.tag === "DomFailure"
    ? removalResult.error.value.tag
    : "unexpected"

window.domLifecycleResult = Object.freeze({
  strictMismatchPath: mismatch.error.value.path,
  dispatched,
  duplicateTargetRejected,
  coarseUpdateRendered,
  hydrationPreservedIdentity,
  replacementPreservedAncestor,
  cancellationReleasedTarget,
  targetRemoval,
})
document.documentElement.dataset.domLifecycle = "complete"
