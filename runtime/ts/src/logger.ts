import { stderr } from "node:process"
import { type Unit, unit } from "./effect"
import type { LogError, LogEvent, Logger } from "./logger-service"
import { renderLogEvent } from "./logger-service"
import {
  type ServiceOperation,
  serviceFailure,
  serviceSuccess,
} from "./service"

export type {
  LogError,
  LogEvent,
  Logger,
  LoggerEnvironment,
  LogLevel,
  LogValue,
} from "./logger-service"
export {
  LogBool,
  LogDebug,
  LogFailure,
  LogFloat,
  LogInfo,
  LogInt,
  LogString,
  LogTrace,
  LogWarn,
  log,
  renderLogEvent,
} from "./logger-service"

export const liveLogger: Logger = Object.freeze({
  log(event) {
    return writeEvent(event)
  },
})

function writeEvent(event: LogEvent): ServiceOperation<LogError, Unit> {
  return new Promise((resolve, reject) => {
    try {
      stderr.write(`${renderLogEvent(event)}\n`, (error) => {
        resolve(
          error === null || error === undefined
            ? serviceSuccess(unit)
            : serviceFailure(logError(error))
        )
      })
    } catch (error) {
      if (error instanceof Error) {
        resolve(serviceFailure(logError(error)))
        return
      }
      reject(error)
    }
  })
}

function logError(error: Error): LogError {
  return { kind: "log-error", message: error.message }
}
