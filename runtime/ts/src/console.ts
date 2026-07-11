import { stdout } from "node:process"
import { unit, type Effect, type Unit } from "./effect"
import {
  serviceEffect,
  serviceFailure,
  serviceSuccess,
  type ServiceOperation,
} from "./service"

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

export const liveConsole: Console = {
  print(value) {
    return writeStdout(value)
  },
  println(value) {
    return writeStdout(`${value}\n`)
  },
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

function writeStdout(value: string): ServiceOperation<ConsoleError, Unit> {
  return new Promise((resolve, reject) => {
    try {
      stdout.write(value, (error) => {
        resolve(
          error === null || error === undefined
            ? serviceSuccess(unit)
            : serviceFailure(consoleError(error))
        )
      })
    } catch (error) {
      if (error instanceof Error) {
        resolve(serviceFailure(consoleError(error)))
        return
      }
      reject(error)
    }
  })
}

function consoleError(error: Error): ConsoleError {
  return {
    kind: "console-error",
    message: error.message,
  }
}
