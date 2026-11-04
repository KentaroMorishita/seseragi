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

let pendingStderrWrites = 0

function handleStderrWriteError() {
  // The write callback reports this failure as ConsoleError.
}

function releaseStderrWrite() {
  if (--pendingStderrWrites === 0) {
    stderr.off("error", handleStderrWriteError)
  }
}

function writeStderr(value: string): ServiceOperation<ConsoleError, Unit> {
  return new Promise((resolve, reject) => {
    if (pendingStderrWrites++ === 0) {
      stderr.on("error", handleStderrWriteError)
    }
    try {
      stderr.write(value, (writeError) => {
        if (writeError === null || writeError === undefined) {
          releaseStderrWrite()
        } else {
          // Node emits the stream error after the write callback.
          queueMicrotask(releaseStderrWrite)
        }
        resolve(
          writeError === null || writeError === undefined
            ? serviceSuccess(unit)
            : serviceFailure(consoleError(writeError))
        )
      })
    } catch (writeError) {
      releaseStderrWrite()
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
