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
import type { Html } from "./html"
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
const DOM_CONTENT = Symbol("seseragi.dom-content")
const DOM_BINDING = Symbol("seseragi.dom-binding")

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

export const FreshMount: HydrationMode = Object.freeze({ tag: "FreshMount" })
export const HydrateStrict: HydrationMode = Object.freeze({
  tag: "HydrateStrict",
})
export const HydrateOrReplace: HydrationMode = Object.freeze({
  tag: "HydrateOrReplace",
})
export const ClearRenderedDom: CleanupMode = Object.freeze({
  tag: "ClearRenderedDom",
})
export const PreserveRenderedDom: CleanupMode = Object.freeze({
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
export const DomTargetAlreadyMounted: DomError = Object.freeze({
  tag: "DomTargetAlreadyMounted",
})
export const HydrationMismatch = (value: {
  readonly path: readonly number[]
  readonly expected: string
  readonly actual: string
}): DomError => Object.freeze({ tag: "HydrationMismatch", value })
export const DomEventQueueOverflow = (value: number): DomError =>
  Object.freeze({ tag: "DomEventQueueOverflow", value })
export const DomTargetRemoved: DomError = Object.freeze({
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

type PhantomAction<Action> = Readonly<{ readonly __action?: Action }>

export type DomContent<Action> = PhantomAction<Action> &
  Readonly<{
    readonly [DOM_CONTENT]: true
    readonly initial: Html<Action>
    readonly bindings: ReadonlyArray<DomBinding<Action>>
  }>

export type DomBinding<Action> = PhantomAction<Action> &
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

export type Dom = {
  readonly query: (selector: string) => ServiceOperation<DomError, DomTarget>
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

export function mount<Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<{}, Failure, Unit>,
  content: Signal<Html<Action>>
): Effect<DomEnvironment, DomError, DomMount<Failure>> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    throwIfCancelled(activeContext)
    const result = await environment.dom.mount(
      options,
      target,
      (action) => runEffect(dispatch(action), environment, activeContext),
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

export function mountContent<Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<{}, Failure, Unit>,
  value: DomContent<Action>
): Effect<DomEnvironment, DomError, DomMount<Failure>> {
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

export function runContent<Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<{}, Failure, Unit>,
  value: DomContent<Action>
): Effect<DomEnvironment, DomRuntimeError<Failure>, Unit> {
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

export function run<Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<{}, Failure, Unit>,
  content: Signal<Html<Action>>
): Effect<DomEnvironment, DomRuntimeError<Failure>, Unit> {
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

function domMountControl<Failure>(
  mounted: DomMount<Failure>
): DomMountControl<Failure> {
  return mounted[DOM_MOUNT]
}
