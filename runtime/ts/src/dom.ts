import {
  type Effect,
  type EffectResult,
  type Unit,
  run as runEffect,
} from "./effect"
import type { Html } from "./html"
import { serviceEffect, type ServiceOperation } from "./service"
import type { Signal } from "./signal"

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

export type DomDispatch<Failure, Message> = (
  message: Message
) => Promise<EffectResult<Failure, Unit>>

export type Dom = {
  readonly query: (selector: string) => ServiceOperation<DomError, DomTarget>
  readonly run: <Failure, Message>(
    options: DomOptions,
    target: DomTarget,
    dispatch: DomDispatch<Failure, Message>,
    content: Signal<Html<Message>>
  ) => ServiceOperation<DomRuntimeError<Failure>, Unit>
}

export type DomEnvironment = {
  readonly dom: Dom
}

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

export function run<Failure, Message>(
  options: DomOptions,
  target: DomTarget,
  dispatch: (message: Message) => Effect<unknown, Failure, Unit>,
  content: Signal<Html<Message>>
): Effect<DomEnvironment, DomRuntimeError<Failure>, Unit> {
  return serviceEffect((environment: DomEnvironment) =>
    environment.dom.run(
      options,
      target,
      (message) => runEffect(dispatch(message), environment),
      content
    )
  )
}

/** Host-adapter boundary; never exposed as a Seseragi value constructor. */
export function createDomTarget(value: unknown): DomTarget {
  return Object.freeze({ [DOM_TARGET]: value })
}

/** Host-adapter boundary paired with createDomTarget. */
export function domTargetValue(target: DomTarget): unknown {
  return target[DOM_TARGET]
}
