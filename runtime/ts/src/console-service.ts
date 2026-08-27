import type { Effect, Unit } from "./effect"
import { type ServiceOperation, serviceEffect } from "./service"
import { renderShow, type Show } from "./show"

export type Console = {
  readonly print: (value: string) => ServiceOperation<ConsoleError, Unit>
  readonly println: (value: string) => ServiceOperation<ConsoleError, Unit>
  readonly error: (value: string) => ServiceOperation<ConsoleError, Unit>
  readonly errorLine: (value: string) => ServiceOperation<ConsoleError, Unit>
  readonly flush: () => ServiceOperation<ConsoleError, Unit>
}

export type ConsoleEnvironment = {
  readonly console: Console
}

export type ConsoleError = {
  readonly kind: "console-error"
  readonly message: string
}

export function print(
  value: unknown
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.print(String(value))
  )
}

export function println(
  value: unknown
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.println(String(value))
  )
}

export function printValue<Value>(
  value: Value,
  dictionary: Show<Value>
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return print(renderShow(dictionary, value, { layout: "compact" }))
}

export function error(
  value: unknown
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.error(String(value))
  )
}

export function errorLine(
  value: unknown
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.errorLine(String(value))
  )
}

export function flush(): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.flush()
  )
}
