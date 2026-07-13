import { stdout } from "node:process"
import { unit, type Unit } from "./effect"
import {
  serviceFailure,
  serviceSuccess,
  type ServiceOperation,
} from "./service"
import type { Console, ConsoleError } from "./console-service"

export type {
  Console,
  ConsoleEnvironment,
  ConsoleError,
} from "./console-service"
export { print, println } from "./console-service"

export const liveConsole: Console = {
  print(value) {
    return writeStdout(value)
  },
  println(value) {
    return writeStdout(`${value}\n`)
  },
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
