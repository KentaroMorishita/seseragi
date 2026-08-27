import { stderr, stdout } from "node:process"
import type { Console, ConsoleError } from "./console-service"
import { type Unit, unit } from "./effect"
import {
  type ServiceOperation,
  serviceFailure,
  serviceSuccess,
} from "./service"

export type {
  Console,
  ConsoleEnvironment,
  ConsoleError,
} from "./console-service"
export {
  error,
  errorLine,
  flush,
  print,
  println,
  printValue,
} from "./console-service"

export const liveConsole: Console = {
  print(value) {
    return writeStdout(value)
  },
  println(value) {
    return writeStdout(`${value}\n`)
  },
  error(value) {
    return writeStderr(value)
  },
  errorLine(value) {
    return writeStderr(`${value}\n`)
  },
  flush() {
    return serviceSuccess(unit)
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

function writeStderr(value: string): ServiceOperation<ConsoleError, Unit> {
  return new Promise((resolve, reject) => {
    try {
      stderr.write(value, (writeError) => {
        resolve(
          writeError === null || writeError === undefined
            ? serviceSuccess(unit)
            : serviceFailure(consoleError(writeError))
        )
      })
    } catch (writeError) {
      if (writeError instanceof Error) {
        resolve(serviceFailure(consoleError(writeError)))
        return
      }
      reject(writeError)
    }
  })
}

function consoleError(error: Error): ConsoleError {
  return {
    kind: "console-error",
    message: error.message,
  }
}
