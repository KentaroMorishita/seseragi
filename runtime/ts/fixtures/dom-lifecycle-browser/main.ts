import { createBrowserDom } from "../../src/browser/dom"
import {
  awaitMount,
  bindAttribute,
  bindChecked,
  bindRegion,
  bindStyle,
  bindText,
  bindValue,
  ClearRenderedDom,
  createDomTarget,
  type DomContent,
  type DomMount,
  defaultOptions,
  HydrateOrReplace,
  HydrateStrict,
  mount,
  mountContent,
  PreserveRenderedDom,
  content as reactiveContent,
  unmount,
} from "../../src/dom"
import { createEffectExecution, type Effect, run, unit } from "../../src/effect"
import {
  button,
  type ChangeEvent,
  div,
  fragment,
  input,
  option,
  p,
  select,
  span,
  style,
  textarea,
} from "../../src/html"
import { serviceSuccess } from "../../src/service"
import {
  combine,
  constant,
  distinct,
  type MutableSignal,
  make,
  map,
  planSet,
  transaction,
  update,
} from "../../src/signal"
import { Just, type Maybe } from "../../src/sum"

declare global {
  interface Window {
    domLifecycleResult?: Readonly<{
      readonly changeSnapshots: readonly ChangeEvent[]
      readonly strictMismatchPath: readonly number[]
      readonly dispatched: number
      readonly duplicateTargetRejected: boolean
      readonly coarseUpdateRendered: boolean
      readonly hydrationPreservedIdentity: boolean
      readonly replacementPreservedAncestor: boolean
      readonly reactiveLeafIsolation: boolean
      readonly reactiveRegionIsolation: boolean
      readonly reactiveRegionCleanup: boolean
      readonly reactiveTransactionStable: boolean
      readonly reactiveDistinctSkippedWrite: boolean
      readonly reactiveHydrationPreservedIdentity: boolean
      readonly reactiveUnmountStoppedUpdates: boolean
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

const reactiveRoot = document.createElement("div")
host.append(reactiveRoot)
const reactiveDom = createBrowserDom(document, () => undefined)
const textSource = await effectValue<MutableSignal<string>>(make("zero"))
const attributeSource = await effectValue<MutableSignal<Maybe<string>>>(
  make(Just("zero"))
)
const valueSource = await effectValue<MutableSignal<string>>(make("start"))
const checkedSource = await effectValue<MutableSignal<boolean>>(make(false))
const styleSource = await effectValue<MutableSignal<Maybe<string>>>(
  make(Just("red"))
)
const leftSource = await effectValue<MutableSignal<number>>(make(0))
const rightSource = await effectValue<MutableSignal<number>>(make(0))
const transactionSource = combine(
  (left: number) => (right: number) => `${left}:${right}`,
  leftSource,
  rightSource
)
const rawDistinctSource = await effectValue<MutableSignal<string>>(make("same"))
const distinctSource = distinct(
  (left: string) => (right: string) => left === right,
  rawDistinctSource
)
const oldInnerSource = await effectValue<MutableSignal<string>>(make("old"))
const oldRegion = reactiveContent<string>(
  fragment([
    span({ id: "inner", children: "old" }),
    button({ id: "old-action", onClick: "old", children: "old action" }),
  ]),
  [bindText<string>("#inner", oldInnerSource)]
)
const regionSource = await effectValue<MutableSignal<DomContent<string>>>(
  make(oldRegion)
)
const initialReactive = div<string>({
  children: [
    span({ id: "static", children: "static" }),
    span({ id: "bound-text", title: "zero", children: "zero" }),
    input({
      id: "bound-input",
      value: "start",
      checked: false,
      onInput: () => "input",
    }),
    span({
      id: "bound-style",
      style: style({ color: "red" }),
      children: "styled",
    }),
    span({ id: "transaction", children: "0:0" }),
    span({ id: "distinct", children: "same" }),
    div({
      id: "region",
      children: [
        span({ id: "inner", children: "old" }),
        button({ id: "old-action", onClick: "old", children: "old action" }),
      ],
    }),
  ],
})
const reactive = reactiveContent<string>(initialReactive, [
  bindText<string>("#bound-text", textSource),
  bindAttribute<string>("#bound-text", "title", attributeSource),
  bindValue<string>("#bound-input", valueSource),
  bindChecked<string>("#bound-input", checkedSource),
  bindStyle<string>("#bound-style", "color", styleSource),
  bindText<string>("#transaction", transactionSource),
  bindText<string>("#distinct", distinctSource),
  bindRegion<string>("#region", regionSource),
])
let reactiveDispatches = 0
const reactiveMounted = await run(
  mountContent(
    {
      ...defaultOptions(unit),
      cleanup: PreserveRenderedDom,
    },
    createDomTarget(reactiveRoot),
    () => async () => {
      reactiveDispatches += 1
      return unit
    },
    reactive
  ),
  { dom: reactiveDom.service }
)
assert(reactiveMounted.kind === "success", "reactive content must mount")
const staticSibling = reactiveRoot.querySelector("#static")
const boundText = reactiveRoot.querySelector("#bound-text")
const boundInput = reactiveRoot.querySelector<HTMLInputElement>("#bound-input")
const boundStyle = reactiveRoot.querySelector<HTMLElement>("#bound-style")
const regionElement = reactiveRoot.querySelector("#region")
const oldInner = reactiveRoot.querySelector("#inner")
const oldAction = reactiveRoot.querySelector<HTMLButtonElement>("#old-action")
assert(
  staticSibling !== null &&
    boundText !== null &&
    boundInput !== null &&
    boundStyle !== null &&
    regionElement !== null &&
    oldInner !== null &&
    oldAction !== null,
  "reactive DOM fixture must render every target"
)
const distinctWrites: MutationRecord[] = []
const distinctObserver = new MutationObserver((records) =>
  distinctWrites.push(...records)
)
distinctObserver.observe(reactiveRoot.querySelector("#distinct")!, {
  childList: true,
  characterData: true,
  subtree: true,
})
const transactionValues: string[] = []
const transactionObserver = new MutationObserver(() => {
  transactionValues.push(
    reactiveRoot.querySelector("#transaction")?.textContent ?? "missing"
  )
})
transactionObserver.observe(reactiveRoot.querySelector("#transaction")!, {
  childList: true,
  characterData: true,
  subtree: true,
})
await effectValue(update(() => "one", textSource))
await effectValue(update(() => Just("one"), attributeSource))
await effectValue(update(() => "next", valueSource))
await effectValue(update(() => true, checkedSource))
await effectValue(update(() => Just("blue"), styleSource))
await effectValue(
  transaction([planSet(1, leftSource), planSet(1, rightSource)])
)
await effectValue(update(() => "same", rawDistinctSource))
await new Promise((resolve) => setTimeout(resolve, 0))
const reactiveLeafIsolation =
  reactiveRoot.querySelector("#static") === staticSibling &&
  reactiveRoot.querySelector("#bound-text") === boundText &&
  boundText.textContent === "one" &&
  boundText.getAttribute("title") === "one" &&
  boundInput.value === "next" &&
  boundInput.checked &&
  boundStyle.style.getPropertyValue("color") === "blue"
const reactiveTransactionStable =
  reactiveRoot.querySelector("#transaction")?.textContent === "1:1" &&
  transactionValues.every((value) => value === "1:1")
const reactiveDistinctSkippedWrite = distinctWrites.length === 0

const newInnerSource = await effectValue<MutableSignal<string>>(make("new"))
const newRegion = reactiveContent<string>(
  fragment([
    span({ id: "new-inner", children: "new" }),
    button({ id: "new-action", onClick: "new", children: "new action" }),
  ]),
  [bindText<string>("#new-inner", newInnerSource)]
)
await effectValue(update(() => newRegion, regionSource))
const reactiveRegionIsolation =
  reactiveRoot.querySelector("#static") === staticSibling &&
  reactiveRoot.querySelector("#region") === regionElement &&
  reactiveRoot.querySelector("#new-inner")?.textContent === "new"
await effectValue(update(() => "stale", oldInnerSource))
regionElement.append(oldAction)
oldAction.click()
reactiveRoot.querySelector<HTMLButtonElement>("#new-action")?.click()
await Promise.resolve()
const reactiveRegionCleanup =
  oldInner.textContent === "old" && reactiveDispatches === 1
await unmountTwice(reactiveMounted.value)
await effectValue(update(() => "after-unmount", textSource))
const reactiveUnmountStoppedUpdates = boundText.textContent === "one"
distinctObserver.disconnect()
transactionObserver.disconnect()

const reactiveHydrationRoot = document.createElement("div")
reactiveHydrationRoot.innerHTML =
  '<div><span id="hydrated-static">static</span><span id="hydrated-leaf">server</span></div>'
host.append(reactiveHydrationRoot)
const hydratedStatic = reactiveHydrationRoot.querySelector("#hydrated-static")
const hydratedLeaf = reactiveHydrationRoot.querySelector("#hydrated-leaf")
const hydratedSource = await effectValue<MutableSignal<string>>(make("server"))
const hydratedContent = reactiveContent<string>(
  div({
    children: [
      span({ id: "hydrated-static", children: "static" }),
      span({ id: "hydrated-leaf", children: "server" }),
    ],
  }),
  [bindText<string>("#hydrated-leaf", hydratedSource)]
)
const reactiveHydrationDom = createBrowserDom(document, () => undefined)
const hydratedMounted = await run(
  mountContent(
    {
      ...defaultOptions(unit),
      hydration: HydrateStrict,
      cleanup: PreserveRenderedDom,
    },
    createDomTarget(reactiveHydrationRoot),
    () => async () => unit,
    hydratedContent
  ),
  { dom: reactiveHydrationDom.service }
)
assert(hydratedMounted.kind === "success", "reactive hydration must mount")
await effectValue(update(() => "client", hydratedSource))
const reactiveHydrationPreservedIdentity =
  reactiveHydrationRoot.querySelector("#hydrated-static") === hydratedStatic &&
  reactiveHydrationRoot.querySelector("#hydrated-leaf") === hydratedLeaf &&
  hydratedLeaf?.textContent === "client"
await unmountTwice(hydratedMounted.value)

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

const changesRoot = document.createElement("div")
host.append(changesRoot)
const changesDom = createBrowserDom(document, () => undefined)
const changeSnapshots: ChangeEvent[] = []
const changes = await changesDom.service.mount(
  defaultOptions(unit),
  createDomTarget(changesRoot),
  async (snapshot: ChangeEvent) => {
    changeSnapshots.push(snapshot)
    return serviceSuccess(unit)
  },
  constant(
    div({
      children: [
        input({ inputType: "text", onChange: (event: ChangeEvent) => event }),
        input({ inputType: "number", onChange: (event: ChangeEvent) => event }),
        input({
          inputType: "checkbox",
          onChange: (event: ChangeEvent) => event,
        }),
        input({ inputType: "radio", onChange: (event: ChangeEvent) => event }),
        textarea({ onChange: (event: ChangeEvent) => event }),
        select({
          onChange: (event: ChangeEvent) => event,
          children: option({ value: "changed", children: "Changed" }),
        }),
      ],
    })
  )
)
assert(changes.kind === "success", "change controls must mount")
for (const element of changesRoot.querySelectorAll<
  HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
>("input,textarea,select")) {
  element.value = element.type === "number" ? "42" : "changed"
  if (
    element instanceof HTMLInputElement &&
    (element.type === "checkbox" || element.type === "radio")
  )
    element.checked = true
  element.dispatchEvent(new Event("change", { bubbles: true }))
  await new Promise((resolve) => setTimeout(resolve, 0))
}
await unmountTwice(changes.value)

window.domLifecycleResult = Object.freeze({
  changeSnapshots,
  strictMismatchPath: mismatch.error.value.path,
  dispatched,
  duplicateTargetRejected,
  coarseUpdateRendered,
  hydrationPreservedIdentity,
  replacementPreservedAncestor,
  reactiveLeafIsolation,
  reactiveRegionIsolation,
  reactiveRegionCleanup,
  reactiveTransactionStable,
  reactiveDistinctSkippedWrite,
  reactiveHydrationPreservedIdentity,
  reactiveUnmountStoppedUpdates,
  cancellationReleasedTarget,
  targetRemoval,
})
document.documentElement.dataset.domLifecycle = "complete"
