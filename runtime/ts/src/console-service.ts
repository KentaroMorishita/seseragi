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
  value: string
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.print(consoleText(value))
  )
}

export function println(
  value: string
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.println(consoleText(value))
  )
}

export function printValue<Value>(
  value: Value,
  dictionary: Show<Value>
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return print(renderShow(dictionary, value, { layout: "compact" }))
}

export function error(
  value: string
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.error(consoleText(value))
  )
}

export function errorLine(
  value: string
): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.errorLine(consoleText(value))
  )
}

export function flush(): Effect<ConsoleEnvironment, ConsoleError, Unit> {
  return serviceEffect((environment: ConsoleEnvironment) =>
    environment.console.flush()
  )
}

function consoleText(value: string): string {
  if (typeof value !== "string") {
    throw new TypeError(
      "Console requires String; render values with show or printValue"
    )
  }
  return value
}
