import {
  type Effect,
  type EffectResult,
  type Unit,
  flatMap,
  mapError,
  run as runEffect,
  unit,
} from "./effect"
import type { Html } from "./html"
import { serviceEffect, type ServiceOperation } from "./service"
import {
  make as makeSignal,
  map as mapSignal,
  type Signal,
  update as updateSignal,
} from "./signal"

const DOM_TARGET = Symbol("seseragi.dom-target")

export type DomTarget = Readonly<{
  readonly [DOM_TARGET]: unknown
}>

export type DomOptions = Readonly<{
  readonly eventCapacity: number
}>

export type DomError =
  | Readonly<{ readonly tag: "InvalidSelector"; readonly value: string }>
  | Readonly<{ readonly tag: "DomTargetNotFound"; readonly value: string }>
  | Readonly<{ readonly tag: "DomTargetAlreadyMounted" }>
  | Readonly<{ readonly tag: "DomEventQueueOverflow"; readonly value: bigint }>
  | Readonly<{ readonly tag: "DomTargetRemoved" }>
  | Readonly<{ readonly tag: "DomOperationFailed"; readonly value: string }>

export type DomRuntimeError<Failure> =
  | Readonly<{ readonly tag: "DomFailure"; readonly value: DomError }>
  | Readonly<{ readonly tag: "DispatchFailure"; readonly value: Failure }>

export type DomDispatch<Failure, Action> = (
  action: Action
) => Promise<EffectResult<Failure, Unit>>

export type Dom = {
  readonly query: (selector: string) => ServiceOperation<DomError, DomTarget>
  readonly run: <Failure, Action>(
    options: DomOptions,
    target: DomTarget,
    dispatch: DomDispatch<Failure, Action>,
    content: Signal<Html<Action>>
  ) => ServiceOperation<DomRuntimeError<Failure>, Unit>
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
  return Object.freeze({ eventCapacity: 1024 })
}

export function query(
  selector: string
): Effect<DomEnvironment, DomError, DomTarget> {
  return serviceEffect((environment: DomEnvironment) =>
    environment.dom.query(selector)
  )
}

export function run<Failure, Action>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (action: Action) => Effect<{}, Failure, Unit>,
  content: Signal<Html<Action>>
): Effect<DomEnvironment, DomRuntimeError<Failure>, Unit> {
  return serviceEffect((environment: DomEnvironment) =>
    environment.dom.run(
      options,
      target,
      (action) => runEffect(dispatch(action), environment),
      content
    )
  )
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
