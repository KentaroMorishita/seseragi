import { createBrowserDom } from "../../src/browser/dom"
import {
  ariaExpandedTarget,
  awaitMount,
  bind,
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
  disabledTarget,
  HydrateOrReplace,
  HydrateStrict,
  mount,
  mountContent,
  PreserveRenderedDom,
  content as reactiveContent,
  textTarget,
  unmount,
} from "../../src/dom"
import { createEffectExecution, type Effect, run, unit } from "../../src/effect"
import {
  button,
  type ChangeEvent,
  div,
  elementRef,
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
import { g, rect, svg, toHtml, viewBoxTarget, xTarget } from "../../src/svg"

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
      readonly keyedRegionHydrationPreservedIdentity: boolean
      readonly keyedRegionPreservedIdentity: boolean
      readonly keyedRegionFocusSelection: boolean
      readonly keyedRegionBoundedMutations: boolean
      readonly keyedRegionCleanup: boolean
      readonly keyedRegionDiagnostics: boolean
      readonly keyedSvgNamespacePreserved: boolean
      readonly typedBindingPreservedHydrationIdentity: boolean
      readonly typedBindingValuesUpdated: boolean
      readonly typedBindingMissingRefRejected: boolean
      readonly typedBindingKindMismatchRejected: boolean
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

const keyedRoot = document.createElement("div")
keyedRoot.innerHTML =
  '<div><div id="keyed-region"><input id="key-a" value="abcdef" type="text"><button id="key-b" type="button">Beta</button><p id="key-c">Gamma</p></div></div>'
host.append(keyedRoot)
const hydratedKeyA = keyedRoot.querySelector<HTMLInputElement>("#key-a")
const hydratedKeyB = keyedRoot.querySelector<HTMLButtonElement>("#key-b")
const hydratedKeyC = keyedRoot.querySelector<HTMLElement>("#key-c")
const keyedTextSource = await effectValue<MutableSignal<string>>(make("Beta"))
const initialKeyedRegion = reactiveContent<string>(
  fragment([
    input({ key: "a", id: "key-a", value: "abcdef" }),
    button({
      key: "b",
      id: "key-b",
      onClick: "before",
      children: "Beta",
    }),
    p({ key: "c", id: "key-c", children: "Gamma" }),
  ]),
  [bindText<string>("#key-b", keyedTextSource)]
)
const keyedRegionSource = await effectValue<MutableSignal<DomContent<string>>>(
  make(initialKeyedRegion)
)
const keyedDispatches: string[] = []
const keyedInitial = div<string>({
  children: div({
    id: "keyed-region",
    children: [
      input({ key: "a", id: "key-a", value: "abcdef" }),
      button({
        key: "b",
        id: "key-b",
        onClick: "before",
        children: "Beta",
      }),
      p({ key: "c", id: "key-c", children: "Gamma" }),
    ],
  }),
})
const keyedMounted = await run(
  mountContent(
    {
      ...defaultOptions(unit),
      hydration: HydrateStrict,
      cleanup: PreserveRenderedDom,
    },
    createDomTarget(keyedRoot),
    (action: string) => async () => {
      keyedDispatches.push(action)
      return unit
    },
    reactiveContent(keyedInitial, [
      bindRegion<string>("#keyed-region", keyedRegionSource),
    ])
  ),
  { dom: createBrowserDom(document, () => undefined).service }
)
assert(
  keyedMounted.kind === "success",
  `keyed region must hydrate: ${JSON.stringify(keyedMounted)}`
)
const keyedRegionHydrationPreservedIdentity =
  keyedRoot.querySelector("#key-a") === hydratedKeyA &&
  keyedRoot.querySelector("#key-b") === hydratedKeyB &&
  keyedRoot.querySelector("#key-c") === hydratedKeyC
hydratedKeyA?.focus()
hydratedKeyA?.setSelectionRange(2, 4)
const keyedRegion = keyedRoot.querySelector("#keyed-region")!
const keyedMutations: MutationRecord[] = []
const keyedObserver = new MutationObserver((records) =>
  keyedMutations.push(...records)
)
keyedObserver.observe(keyedRegion, { childList: true })

const movedKeyedRegion = reactiveContent<string>(
  fragment([
    p({ key: "c", id: "key-c", children: "Gamma updated" }),
    button({
      key: "b",
      id: "key-b",
      onClick: "after",
      children: "Beta",
    }),
    span({ key: "d", id: "key-d", children: "Delta" }),
    input({ key: "a", id: "key-a", value: "abcdef" }),
  ]),
  [bindText<string>("#key-b", keyedTextSource)]
)
await effectValue(update(() => movedKeyedRegion, keyedRegionSource))
await new Promise((resolve) => setTimeout(resolve, 0))
const keyedRegionPreservedIdentity =
  keyedRoot.querySelector("#key-a") === hydratedKeyA &&
  keyedRoot.querySelector("#key-b") === hydratedKeyB &&
  keyedRoot.querySelector("#key-c") === hydratedKeyC &&
  [...keyedRegion.children].map((child) => child.id).join(",") ===
    "key-c,key-b,key-d,key-a" &&
  hydratedKeyC?.textContent === "Gamma updated"
const keyedRegionFocusSelection =
  document.activeElement === hydratedKeyA &&
  hydratedKeyA?.selectionStart === 2 &&
  hydratedKeyA.selectionEnd === 4
hydratedKeyB?.click()
await Promise.resolve()
assert(
  keyedDispatches.join(",") === "after",
  "retained keyed node must use the current event handler"
)
await effectValue(update(() => "Beta updated", keyedTextSource))
assert(hydratedKeyB?.textContent === "Beta updated", "binding must reattach")

const reducedKeyedRegion = reactiveContent<string>(
  fragment([
    input({ key: "a", id: "key-a", value: "abcdef" }),
    p({ key: "c", id: "key-c", children: "Gamma updated" }),
  ]),
  []
)
await effectValue(update(() => reducedKeyedRegion, keyedRegionSource))
await new Promise((resolve) => setTimeout(resolve, 0))
const detachedKeyBText = hydratedKeyB?.textContent
await effectValue(update(() => "stale", keyedTextSource))
const keyedRegionCleanup =
  !keyedRegion.contains(hydratedKeyB) &&
  hydratedKeyB?.textContent === detachedKeyBText
const keyedRegionBoundedMutations = !keyedMutations.some(
  (record) =>
    [...record.removedNodes].includes(hydratedKeyA!) &&
    [...record.removedNodes].includes(hydratedKeyB!) &&
    [...record.removedNodes].includes(hydratedKeyC!)
)
keyedObserver.disconnect()
await unmountTwice(keyedMounted.value)

async function rejectedKeyedRegion(
  children: readonly ReturnType<typeof span>[]
): Promise<string> {
  const root = document.createElement("div")
  host.append(root)
  const region = reactiveContent<string>(fragment(children), [])
  const mounted = await run(
    mountContent(
      defaultOptions(unit),
      createDomTarget(root),
      () => async () => unit,
      reactiveContent(
        div<string>({
          children: div({ id: "invalid-keyed-region", children }),
        }),
        [bindRegion<string>("#invalid-keyed-region", constant(region))]
      )
    ),
    { dom: createBrowserDom(document, () => undefined).service }
  )
  assert(mounted.kind === "failure", "invalid keyed region must fail")
  assert(
    mounted.error.tag === "DomOperationFailed",
    "invalid keyed region must report a DOM operation failure"
  )
  return mounted.error.value
}

const mixedKeyFailure = await rejectedKeyedRegion([
  span({ key: "one", children: "one" }),
  span({ children: "missing" }),
])
const duplicateKeyFailure = await rejectedKeyedRegion([
  span({ key: "same", children: "one" }),
  span({ key: "same", children: "two" }),
])
const emptyKeyFailure = await rejectedKeyedRegion([
  span({ key: "", children: "empty" }),
])
const keyedRegionDiagnostics =
  mixedKeyFailure.includes("cannot mix keyed and unkeyed") &&
  duplicateKeyFailure.includes('duplicate key "same"') &&
  emptyKeyFailure.includes("keys must be non-empty Strings")

const keyedSvgRoot = document.createElement("div")
host.append(keyedSvgRoot)
const keyedSvgChild = rect<string>({
  key: "node",
  id: "keyed-svg-node",
  x: 1,
  y: 2,
  width: 3,
  height: 4,
})
const keyedSvgRegion = reactiveContent<string>(
  fragment([toHtml(keyedSvgChild)]),
  []
)
const keyedSvgMounted = await run(
  mountContent(
    defaultOptions(unit),
    createDomTarget(keyedSvgRoot),
    () => async () => unit,
    reactiveContent(
      toHtml(
        svg({
          children: g({ id: "keyed-svg-region", children: [keyedSvgChild] }),
        })
      ),
      [bindRegion<string>("#keyed-svg-region", constant(keyedSvgRegion))]
    )
  ),
  { dom: createBrowserDom(document, () => undefined).service }
)
assert(keyedSvgMounted.kind === "success", "keyed SVG region must mount")
const keyedSvgNode = keyedSvgRoot.querySelector("#keyed-svg-node")
const keyedSvgNamespacePreserved =
  keyedSvgNode?.namespaceURI === "http://www.w3.org/2000/svg" &&
  keyedSvgNode.parentElement?.namespaceURI === "http://www.w3.org/2000/svg"
await unmountTwice(keyedSvgMounted.value)

const typedRoot = document.createElement("div")
typedRoot.innerHTML =
  '<div><span>zero</span><button aria-expanded="false" type="button">toggle</button><svg viewBox="0 0 10 10"><rect x="0" y="0" width="5" height="5"></rect></svg></div>'
host.append(typedRoot)
const typedTextElement = typedRoot.querySelector("span")
const typedButtonElement = typedRoot.querySelector("button")
const typedSvgElement = typedRoot.querySelector("svg")
const typedRectElement = typedRoot.querySelector("rect")
const typedTextRef = elementRef("typed-text")
const typedButtonRef = elementRef("typed-button")
const typedSvgRef = elementRef("typed-svg")
const typedRectRef = elementRef("typed-rect")
const typedTextSource = await effectValue<MutableSignal<string>>(make("zero"))
const typedDisabledSource = await effectValue<MutableSignal<boolean>>(
  make(false)
)
const typedExpandedSource = await effectValue<MutableSignal<boolean>>(
  make(false)
)
const typedViewBoxSource = await effectValue<MutableSignal<string>>(
  make("0 0 10 10")
)
const typedXSource = await effectValue<MutableSignal<number>>(make(0))
const typedInitial = div<string>({
  children: [
    span({ elementRef: typedTextRef, children: "zero" }),
    button({
      elementRef: typedButtonRef,
      ariaExpanded: false,
      children: "toggle",
    }),
    toHtml(
      svg({
        elementRef: typedSvgRef,
        viewBox: "0 0 10 10",
        children: [
          rect({
            elementRef: typedRectRef,
            x: 0,
            y: 0,
            width: 5,
            height: 5,
          }),
        ],
      })
    ),
  ],
})
const typedContent = reactiveContent<string>(typedInitial, [
  bind(textTarget(typedTextRef), typedTextSource),
  bind(disabledTarget(typedButtonRef), typedDisabledSource),
  bind(ariaExpandedTarget(typedButtonRef), typedExpandedSource),
  bind(viewBoxTarget(typedSvgRef), typedViewBoxSource),
  bind(xTarget(typedRectRef), typedXSource),
])
const typedDom = createBrowserDom(document, () => undefined)
const typedMounted = await run(
  mountContent(
    {
      ...defaultOptions(unit),
      hydration: HydrateStrict,
      cleanup: PreserveRenderedDom,
    },
    createDomTarget(typedRoot),
    () => async () => unit,
    typedContent
  ),
  { dom: typedDom.service }
)
assert(
  typedMounted.kind === "success",
  `typed bindings must hydrate: ${JSON.stringify(typedMounted)}`
)
const typedBindingPreservedHydrationIdentity =
  typedRoot.querySelector("span") === typedTextElement &&
  typedRoot.querySelector("button") === typedButtonElement &&
  typedRoot.querySelector("svg") === typedSvgElement &&
  typedRoot.querySelector("rect") === typedRectElement
await effectValue(update(() => "one", typedTextSource))
await effectValue(update(() => true, typedDisabledSource))
await effectValue(update(() => true, typedExpandedSource))
await effectValue(update(() => "0 0 20 20", typedViewBoxSource))
await effectValue(update(() => 7.5, typedXSource))
const typedBindingValuesUpdated =
  typedTextElement?.textContent === "one" &&
  typedButtonElement?.hasAttribute("disabled") === true &&
  typedButtonElement?.getAttribute("aria-expanded") === "true" &&
  typedSvgElement?.getAttribute("viewBox") === "0 0 20 20" &&
  typedRectElement?.getAttribute("x") === "7.5"
await unmountTwice(typedMounted.value)

async function rejectedTypedBinding(
  referenceKind: "missing" | "mismatch"
): Promise<string> {
  const root = document.createElement("div")
  host.append(root)
  const reference = elementRef(`invalid-${referenceKind}`)
  const source = await effectValue<MutableSignal<string>>(make("value"))
  const initial =
    referenceKind === "missing"
      ? div<string>({ children: span({ children: "value" }) })
      : div<string>({
          children: span({ elementRef: reference, children: "value" }),
        })
  const binding =
    referenceKind === "missing"
      ? bind(textTarget(reference), source)
      : bind(viewBoxTarget(reference), source)
  const mounted = await run(
    mountContent(
      defaultOptions(unit),
      createDomTarget(root),
      () => async () => unit,
      reactiveContent(initial, [binding])
    ),
    { dom: createBrowserDom(document, () => undefined).service }
  )
  assert(mounted.kind === "failure", "invalid typed binding must fail")
  assert(
    mounted.error.tag === "DomOperationFailed",
    "typed binding must report a DOM operation failure"
  )
  return mounted.error.value
}

const missingRefFailure = await rejectedTypedBinding("missing")
const kindMismatchFailure = await rejectedTypedBinding("mismatch")
const typedBindingMissingRefRejected =
  missingRefFailure.includes("matched 0 elements")
const typedBindingKindMismatchRejected = kindMismatchFailure.includes(
  "requires svg but found html"
)

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
  keyedRegionHydrationPreservedIdentity,
  keyedRegionPreservedIdentity,
  keyedRegionFocusSelection,
  keyedRegionBoundedMutations,
  keyedRegionCleanup,
  keyedRegionDiagnostics,
  keyedSvgNamespacePreserved,
  typedBindingPreservedHydrationIdentity,
  typedBindingValuesUpdated,
  typedBindingMissingRefRejected,
  typedBindingKindMismatchRejected,
  cancellationReleasedTarget,
  targetRemoval,
})
document.documentElement.dataset.domLifecycle = "complete"
