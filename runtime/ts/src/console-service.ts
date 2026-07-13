import { type Effect, type Unit } from "./effect"
import { serviceEffect, type ServiceOperation } from "./service"

export type Console = {
  readonly print: (value: string) => ServiceOperation<ConsoleError, Unit>
  readonly println: (value: string) => ServiceOperation<ConsoleError, Unit>
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
