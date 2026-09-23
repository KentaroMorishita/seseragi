import {
  createEffectExecution,
  type Effect,
  type EffectResult,
  fail,
  flatMap,
  mapError,
  run as runEffect,
  throwIfCancelled,
  type Unit,
  unit,
} from "./effect"
import { type ElementRef, elementRefId, type Html } from "./html"
import {
  type ServiceOperation,
  type ServiceResult,
  serviceEffect,
} from "./service"
import {
  constant as constantSignal,
  make as makeSignal,
  map as mapSignal,
  type Signal,
  update as updateSignal,
} from "./signal"
import type { Maybe } from "./sum"

const DOM_TARGET = Symbol("seseragi.dom-target")
const DOM_MOUNT = Symbol("seseragi.dom-mount")
const DOM_OBSERVATION = Symbol("seseragi.dom-observation")
const DOM_CONTENT = Symbol("seseragi.dom-content")
const DOM_BINDING = Symbol("seseragi.dom-binding")
const DOM_BINDING_TARGET = Symbol("seseragi.dom-binding-target")

export type DomTarget = Readonly<{
  readonly [DOM_TARGET]: unknown
}>

export type HydrationMode =
  | Readonly<{ readonly tag: "FreshMount" }>
  | Readonly<{ readonly tag: "HydrateStrict" }>
  | Readonly<{ readonly tag: "HydrateOrReplace" }>

export type CleanupMode =
  | Readonly<{ readonly tag: "ClearRenderedDom" }>
  | Readonly<{ readonly tag: "PreserveRenderedDom" }>

export const FreshMount: HydrationMode = /* @__PURE__ */ Object.freeze({
  tag: "FreshMount",
})
export const HydrateStrict: HydrationMode = /* @__PURE__ */ Object.freeze({
  tag: "HydrateStrict",
})
export const HydrateOrReplace: HydrationMode = /* @__PURE__ */ Object.freeze({
  tag: "HydrateOrReplace",
})
export const ClearRenderedDom: CleanupMode = /* @__PURE__ */ Object.freeze({
  tag: "ClearRenderedDom",
})
export const PreserveRenderedDom: CleanupMode = /* @__PURE__ */ Object.freeze({
  tag: "PreserveRenderedDom",
})

export type DomOptions = Readonly<{
  readonly eventCapacity: number
  readonly hydration: HydrationMode
  readonly cleanup: CleanupMode
}>

export type DomError =
  | Readonly<{ readonly tag: "InvalidSelector"; readonly value: string }>
  | Readonly<{ readonly tag: "DomTargetNotFound"; readonly value: string }>
  | Readonly<{ readonly tag: "DomTargetAlreadyMounted" }>
  | Readonly<{
      readonly tag: "HydrationMismatch"
      readonly value: Readonly<{
        readonly path: readonly number[]
        readonly expected: string
        readonly actual: string
      }>
    }>
  | Readonly<{ readonly tag: "DomEventQueueOverflow"; readonly value: number }>
  | Readonly<{ readonly tag: "DomTargetRemoved" }>
  | Readonly<{ readonly tag: "DomOperationFailed"; readonly value: string }>

export type DomRuntimeError<Failure> =
  | Readonly<{ readonly tag: "DomFailure"; readonly value: DomError }>
  | Readonly<{ readonly tag: "DispatchFailure"; readonly value: Failure }>

export const InvalidSelector = (value: string): DomError =>
  Object.freeze({ tag: "InvalidSelector", value })
export const DomTargetNotFound = (value: string): DomError =>
  Object.freeze({ tag: "DomTargetNotFound", value })
export const DomTargetAlreadyMounted: DomError = /* @__PURE__ */ Object.freeze({
  tag: "DomTargetAlreadyMounted",
})
export const HydrationMismatch = (value: {
  readonly path: readonly number[]
  readonly expected: string
  readonly actual: string
}): DomError => Object.freeze({ tag: "HydrationMismatch", value })
export const DomEventQueueOverflow = (value: number): DomError =>
  Object.freeze({ tag: "DomEventQueueOverflow", value })
export const DomTargetRemoved: DomError = /* @__PURE__ */ Object.freeze({
  tag: "DomTargetRemoved",
})
export const DomOperationFailed = (value: string): DomError =>
  Object.freeze({ tag: "DomOperationFailed", value })
export const DomFailure = <Failure>(
  value: DomError
): DomRuntimeError<Failure> => Object.freeze({ tag: "DomFailure", value })
export const DispatchFailure = <Failure>(
  value: Failure
): DomRuntimeError<Failure> => Object.freeze({ tag: "DispatchFailure", value })

export type DomDispatch<Failure, Action> = (
  action: Action
) => Promise<EffectResult<Failure, Unit>>

export type ElementRect = Readonly<{
  readonly x: number
  readonly y: number
  readonly width: number
  readonly height: number
  readonly top: number
  readonly right: number
  readonly bottom: number
  readonly left: number
}>

type PhantomAction<Action> = Readonly<{ readonly __action?: Action }>
type PhantomValue<Value> = Readonly<{
  readonly __bindingValue?: (value: Value) => Value
}>

export type BindingElementExpectation = Readonly<{
  readonly namespace?: "html" | "svg"
  readonly tags?: ReadonlyArray<string>
}>

type BindingTargetDescriptor = BindingElementExpectation &
  (
    | Readonly<{ readonly kind: "text" }>
    | Readonly<{ readonly kind: "attribute"; readonly name: string }>
    | Readonly<{ readonly kind: "string-attribute"; readonly name: string }>
    | Readonly<{ readonly kind: "number-attribute"; readonly name: string }>
    | Readonly<{ readonly kind: "boolean-attribute"; readonly name: string }>
    | Readonly<{ readonly kind: "aria-boolean"; readonly name: string }>
    | Readonly<{ readonly kind: "value" }>
    | Readonly<{ readonly kind: "checked" }>
    | Readonly<{ readonly kind: "style"; readonly name: string }>
    | Readonly<{ readonly kind: "region" }>
  )

export type BindingTarget<Action, Value> = PhantomAction<Action> &
  PhantomValue<Value> &
  Readonly<{
    readonly [DOM_BINDING_TARGET]: true
    readonly reference: ElementRef
    readonly descriptor: BindingTargetDescriptor
  }>

type DomBindingLocation =
  | Readonly<{ readonly selector: string; readonly reference?: never }>
  | Readonly<{ readonly selector?: never; readonly reference: ElementRef }>

export type DomContent<Action> = PhantomAction<Action> &
  Readonly<{
    readonly [DOM_CONTENT]: true
    readonly initial: Html<Action>
    readonly bindings: ReadonlyArray<DomBinding<Action>>
  }>

export type DomBinding<Action> = PhantomAction<Action> &
  DomBindingLocation &
  (
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "text"
        readonly selector: string
        readonly source: Signal<string>
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "attribute"
        readonly selector: string
        readonly name: string
        readonly source: Signal<Maybe<string>>
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "string-attribute"
        readonly name: string
        readonly source: Signal<string>
        readonly expectation?: BindingElementExpectation
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "number-attribute"
        readonly name: string
        readonly source: Signal<number>
        readonly expectation?: BindingElementExpectation
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "boolean-attribute"
        readonly name: string
        readonly source: Signal<boolean>
        readonly expectation?: BindingElementExpectation
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "aria-boolean"
        readonly name: string
        readonly source: Signal<boolean>
        readonly expectation?: BindingElementExpectation
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "value"
        readonly selector: string
        readonly source: Signal<string>
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "checked"
        readonly selector: string
        readonly source: Signal<boolean>
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "style"
        readonly selector: string
        readonly name: string
        readonly source: Signal<Maybe<string>>
      }>
    | Readonly<{
        readonly [DOM_BINDING]: true
        readonly kind: "region"
        readonly selector: string
        readonly source: Signal<DomContent<Action>>
      }>
  )

type DomMountControl<Failure> = Readonly<{
  readonly awaitResult: () => Promise<
    ServiceResult<DomRuntimeError<Failure>, Unit>
  >
  readonly unmount: () => Promise<void>
  readonly bindCancellation: (release: () => void) => void
  readonly attachContent?: (
    content: DomContent<unknown>
  ) => Promise<ServiceResult<DomError, Unit>>
}>

export type DomMount<Failure> = Readonly<{
  readonly [DOM_MOUNT]: DomMountControl<Failure>
}>

type DomObservationControl<Failure> = Readonly<{
  readonly awaitResult: () => Promise<ServiceResult<Failure, Unit>>
  readonly disconnect: () => Promise<void>
  readonly bindCancellation: (release: () => void) => void
}>

export type DomObservation<Failure> = Readonly<{
  readonly [DOM_OBSERVATION]: DomObservationControl<Failure>
}>

export type Dom = {
  readonly query: (selector: string) => ServiceOperation<DomError, DomTarget>
  readonly capturePointer: (
    target: DomTarget,
    pointerId: number
  ) => ServiceOperation<DomError, Unit>
  readonly releasePointer: (
    target: DomTarget,
    pointerId: number
  ) => ServiceOperation<DomError, Unit>
  readonly measure: (
    target: DomTarget
  ) => ServiceOperation<DomError, ElementRect>
  readonly observeResize: <Failure>(
    target: DomTarget,
    callback: (rect: ElementRect) => Promise<EffectResult<Failure, Unit>>
  ) => ServiceOperation<DomError, DomObservation<Failure>>
  readonly mount: <Failure, Action>(
    options: DomOptions,
    target: DomTarget,
    dispatch: DomDispatch<Failure, Action>,
    content: Signal<Html<Action>>
  ) => ServiceOperation<DomError, DomMount<Failure>>
}

export type DomEnvironment = {
  readonly dom: Dom
}

type DomRequirements<Environment> = [Environment] extends [
  Readonly<Record<string, never>>,
]
  ? DomEnvironment
  : DomEnvironment & Environment

export type DomApp<State, Action> = Readonly<{
  readonly target: string
  readonly initial: NoInfer<State>
  readonly update: (action: Action) => (state: State) => State
  readonly view: (state: State) => Html<Action>
}>

export function defaultOptions(_unit: Unit): DomOptions {
  return Object.freeze({
    eventCapacity: 1024,
    hydration: FreshMount,
    cleanup: ClearRenderedDom,
  })
}

export function query(
  selector: string
): Effect<DomEnvironment, DomError, DomTarget> {
  return serviceEffect((environment: DomEnvironment) =>
    environment.dom.query(selector)
  )
}

export function capturePointer(
  target: DomTarget,
  pointerId: number
): Effect<DomEnvironment, DomError, Unit> {
  return serviceEffect((environment: DomEnvironment) =>
    environment.dom.capturePointer(target, pointerId)
  )
}

export function releasePointer(
  target: DomTarget,
  pointerId: number
): Effect<DomEnvironment, DomError, Unit> {
  return serviceEffect((environment: DomEnvironment) =>
    environment.dom.releasePointer(target, pointerId)
  )
}

export function measure(
  target: DomTarget
): Effect<DomEnvironment, DomError, ElementRect> {
  return serviceEffect((environment: DomEnvironment) =>
    environment.dom.measure(target)
  )
}

export function observeResize<Environment, Failure>(
  target: DomTarget,
  callback: (rect: ElementRect) => Effect<Environment, Failure, Unit>
): Effect<DomRequirements<Environment>, DomError, DomObservation<Failure>> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    throwIfCancelled(activeContext)
    const result = await environment.dom.observeResize(target, (rect) =>
      runEffect(callback(rect), environment as Environment, activeContext)
    )
    if (result.kind === "failure") {
      return fail(result.error)(environment, activeContext)
    }
    const observation = result.value
    const release = activeContext.onCancel(() =>
      domObservationControl(observation).disconnect()
    )
    domObservationControl(observation).bindCancellation(release)
    throwIfCancelled(activeContext)
    return observation
  }
}

export function awaitObservation<Failure>(
  observation: DomObservation<Failure>
): Effect<{}, Failure, Unit> {
  return serviceEffect(() => domObservationControl(observation).awaitResult())
}

export function disconnect<Failure>(
  observation: DomObservation<Failure>
): Effect<{}, never, Unit> {
  return async () => {
    await domObservationControl(observation).disconnect()
    return unit
  }
}

export function mount<Environment, Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<Environment, Failure, Unit>,
  content: Signal<Html<Action>>
): Effect<DomRequirements<Environment>, DomError, DomMount<Failure>> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    throwIfCancelled(activeContext)
    const result = await environment.dom.mount(
      options,
      target,
      (action) =>
        runEffect(dispatch(action), environment as Environment, activeContext),
      content
    )
    if (result.kind === "failure") {
      return fail(result.error)(environment, activeContext)
    }
    const mounted = result.value
    const release = activeContext.onCancel(() =>
      domMountControl(mounted).unmount()
    )
    domMountControl(mounted).bindCancellation(release)
    throwIfCancelled(activeContext)
    return mounted
  }
}

export function awaitMount<Failure>(
  mounted: DomMount<Failure>
): Effect<{}, DomRuntimeError<Failure>, Unit> {
  return serviceEffect(() => domMountControl(mounted).awaitResult())
}

export function unmount<Failure>(
  mounted: DomMount<Failure>
): Effect<{}, never, Unit> {
  return async () => {
    await domMountControl(mounted).unmount()
    return unit
  }
}

export function content<Action>(
  initial: Html<Action>,
  bindings: ReadonlyArray<DomBinding<Action>>
): DomContent<Action> {
  return Object.freeze({
    [DOM_CONTENT]: true as const,
    initial,
    bindings: Object.freeze([...bindings]),
  })
}

export function initialHtml<Action>(value: DomContent<Action>): Html<Action> {
  return value.initial
}

function bindingTarget<Action, Value>(
  reference: ElementRef,
  descriptor: BindingTargetDescriptor
): BindingTarget<Action, Value> {
  elementRefId(reference)
  return Object.freeze({
    [DOM_BINDING_TARGET]: true as const,
    reference,
    descriptor: Object.freeze({
      ...descriptor,
      tags:
        descriptor.tags === undefined
          ? undefined
          : Object.freeze([...descriptor.tags]),
    }),
  }) as BindingTarget<Action, Value>
}

function bindingTargetDescriptor<Action, Value>(
  target: BindingTarget<Action, Value>
): BindingTargetDescriptor {
  if (
    typeof target !== "object" ||
    target === null ||
    !(DOM_BINDING_TARGET in target)
  ) {
    throw new TypeError("DOM binding target must be created by std/web/dom")
  }
  return target.descriptor
}

export function bind<Action, Value>(
  target: BindingTarget<Action, Value>,
  source: Signal<Value>
): DomBinding<Action> {
  const descriptor = bindingTargetDescriptor(target)
  return Object.freeze({
    [DOM_BINDING]: true as const,
    kind: descriptor.kind,
    reference: target.reference,
    ...(descriptor.kind === "attribute" ||
    descriptor.kind === "string-attribute" ||
    descriptor.kind === "number-attribute" ||
    descriptor.kind === "boolean-attribute" ||
    descriptor.kind === "aria-boolean" ||
    descriptor.kind === "style"
      ? { name: descriptor.name }
      : {}),
    ...(descriptor.namespace === undefined && descriptor.tags === undefined
      ? {}
      : {
          expectation: Object.freeze({
            namespace: descriptor.namespace,
            tags: descriptor.tags,
          }),
        }),
    source,
  }) as DomBinding<Action>
}

export function textTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return bindingTarget(reference, { kind: "text" })
}

export function attributeTarget<Action>(
  reference: ElementRef,
  name: string
): BindingTarget<Action, Maybe<string>> {
  return bindingTarget(reference, { kind: "attribute", name })
}

export function stringAttributeTarget<Action>(
  reference: ElementRef,
  name: string,
  expectation: BindingElementExpectation = {}
): BindingTarget<Action, string> {
  return bindingTarget(reference, {
    kind: "string-attribute",
    name,
    ...expectation,
  })
}

export function numberAttributeTarget<Action>(
  reference: ElementRef,
  name: string,
  expectation: BindingElementExpectation = {}
): BindingTarget<Action, number> {
  return bindingTarget(reference, {
    kind: "number-attribute",
    name,
    ...expectation,
  })
}

export function booleanAttributeTarget<Action>(
  reference: ElementRef,
  name: string,
  expectation: BindingElementExpectation = {}
): BindingTarget<Action, boolean> {
  return bindingTarget(reference, {
    kind: "boolean-attribute",
    name,
    ...expectation,
  })
}

export function ariaBooleanTarget<Action>(
  reference: ElementRef,
  name: string
): BindingTarget<Action, boolean> {
  return bindingTarget(reference, { kind: "aria-boolean", name })
}

export function classTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return stringAttributeTarget(reference, "class")
}

export function titleTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return stringAttributeTarget(reference, "title", { namespace: "html" })
}

export function hiddenTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, boolean> {
  return booleanAttributeTarget(reference, "hidden", { namespace: "html" })
}

export function disabledTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, boolean> {
  return booleanAttributeTarget(reference, "disabled", { namespace: "html" })
}

export function inertTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, boolean> {
  return booleanAttributeTarget(reference, "inert", { namespace: "html" })
}

export function ariaLabelTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return stringAttributeTarget(reference, "aria-label")
}

export function ariaBusyTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, boolean> {
  return ariaBooleanTarget(reference, "aria-busy")
}

export function ariaExpandedTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, boolean> {
  return ariaBooleanTarget(reference, "aria-expanded")
}

export function ariaHiddenTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, boolean> {
  return ariaBooleanTarget(reference, "aria-hidden")
}

export function ariaSelectedTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, boolean> {
  return ariaBooleanTarget(reference, "aria-selected")
}

export function valueTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, string> {
  return bindingTarget(reference, { kind: "value", namespace: "html" })
}

export function checkedTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, boolean> {
  return bindingTarget(reference, {
    kind: "checked",
    namespace: "html",
    tags: ["input"],
  })
}

export function styleTarget<Action>(
  reference: ElementRef,
  name: string
): BindingTarget<Action, Maybe<string>> {
  return bindingTarget(reference, { kind: "style", name })
}

export function regionTarget<Action>(
  reference: ElementRef
): BindingTarget<Action, DomContent<Action>> {
  return bindingTarget(reference, { kind: "region" })
}

export function bindText<Action>(
  selector: string,
  source: Signal<string>
): DomBinding<Action> {
  return Object.freeze({
    [DOM_BINDING]: true as const,
    kind: "text" as const,
    selector,
    source,
  })
}

export function bindAttribute<Action>(
  selector: string,
  name: string,
  source: Signal<Maybe<string>>
): DomBinding<Action> {
  return Object.freeze({
    [DOM_BINDING]: true as const,
    kind: "attribute" as const,
    selector,
    name,
    source,
  })
}

export function bindValue<Action>(
  selector: string,
  source: Signal<string>
): DomBinding<Action> {
  return Object.freeze({
    [DOM_BINDING]: true as const,
    kind: "value" as const,
    selector,
    source,
  })
}

export function bindChecked<Action>(
  selector: string,
  source: Signal<boolean>
): DomBinding<Action> {
  return Object.freeze({
    [DOM_BINDING]: true as const,
    kind: "checked" as const,
    selector,
    source,
  })
}

export function bindStyle<Action>(
  selector: string,
  name: string,
  source: Signal<Maybe<string>>
): DomBinding<Action> {
  return Object.freeze({
    [DOM_BINDING]: true as const,
    kind: "style" as const,
    selector,
    name,
    source,
  })
}

export function bindRegion<Action>(
  selector: string,
  source: Signal<DomContent<Action>>
): DomBinding<Action> {
  return Object.freeze({
    [DOM_BINDING]: true as const,
    kind: "region" as const,
    selector,
    source,
  })
}

export function mountContent<Environment, Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<Environment, Failure, Unit>,
  value: DomContent<Action>
): Effect<DomRequirements<Environment>, DomError, DomMount<Failure>> {
  return async (environment, context) => {
    const mounted = await mount(
      options,
      target,
      dispatch,
      constantSignal(value.initial)
    )(environment, context)
    const attachContent = domMountControl(mounted).attachContent
    const attached =
      attachContent === undefined
        ? ({
            kind: "failure",
            error: DomOperationFailed(
              "DOM adapter does not support reactive content"
            ),
          } as const)
        : await attachContent(value as DomContent<unknown>)
    if (attached.kind === "failure") {
      await domMountControl(mounted).unmount()
      return fail(attached.error)(environment, context)
    }
    return mounted
  }
}

export function runContent<Environment, Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<Environment, Failure, Unit>,
  value: DomContent<Action>
): Effect<DomRequirements<Environment>, DomRuntimeError<Failure>, Unit> {
  return async (environment, context) => {
    let mounted: DomMount<Failure> | undefined
    try {
      mounted = await mapError(
        (error): DomRuntimeError<Failure> => ({
          tag: "DomFailure",
          value: error,
        }),
        mountContent(options, target, dispatch, value)
      )(environment, context)
      return await awaitMount(mounted)(environment, context)
    } finally {
      if (mounted !== undefined) {
        await domMountControl(mounted).unmount()
      }
    }
  }
}

export function run<Environment, Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<Environment, Failure, Unit>,
  content: Signal<Html<Action>>
): Effect<DomRequirements<Environment>, DomRuntimeError<Failure>, Unit> {
  return async (environment, context) => {
    let mounted: DomMount<Failure> | undefined
    try {
      mounted = await mapError(
        (error): DomRuntimeError<Failure> => ({
          tag: "DomFailure",
          value: error,
        }),
        mount(options, target, dispatch, content)
      )(environment, context)
      return await awaitMount(mounted)(environment, context)
    } finally {
      if (mounted !== undefined) {
        await domMountControl(mounted).unmount()
      }
    }
  }
}

/**
 * Mounts the common pure reducer + Signal application shape.
 *
 * Lower-level query/run remain available for custom lifecycles and dispatch
 * failures. This helper owns the standard setup and presents portable String
 * failures so a compact executable main can infer its complete Effect type.
 */
export function app<State, Action>(
  config: DomApp<State, Action>
): Effect<DomEnvironment, string, Unit> {
  return flatMap(makeSignal(config.initial), (state) => {
    const content = mapSignal(config.view, state)
    return flatMap(
      mapError(
        () => `DOM target unavailable: ${config.target}`,
        query(config.target)
      ),
      (target) =>
        mapError(
          () => "DOM runtime failed",
          run(
            defaultOptions(unit),
            target,
            (action) => updateSignal(config.update(action), state),
            content
          )
        )
    )
  })
}

/** Host-adapter boundary; never exposed as a Seseragi value constructor. */
export function createDomTarget(value: unknown): DomTarget {
  return Object.freeze({ [DOM_TARGET]: value })
}

/** Host-adapter boundary paired with createDomTarget. */
export function domTargetValue(target: DomTarget): unknown {
  return target[DOM_TARGET]
}

/** Host-adapter boundary; never exposed as a Seseragi value constructor. */
export function createDomMount<Failure>(
  control: DomMountControl<Failure>
): DomMount<Failure> {
  return Object.freeze({ [DOM_MOUNT]: control })
}

/** Host-adapter boundary; never exposed as a Seseragi value constructor. */
export function createDomObservation<Failure>(
  control: DomObservationControl<Failure>
): DomObservation<Failure> {
  return Object.freeze({ [DOM_OBSERVATION]: control })
}

function domMountControl<Failure>(
  mounted: DomMount<Failure>
): DomMountControl<Failure> {
  return mounted[DOM_MOUNT]
}

function domObservationControl<Failure>(
  observation: DomObservation<Failure>
): DomObservationControl<Failure> {
  return observation[DOM_OBSERVATION]
}
