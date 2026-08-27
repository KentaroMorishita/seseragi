import { unit } from "../effect"
import type { Logger } from "../logger-service"
import { renderLogEvent } from "../logger-service"
import { serviceSuccess } from "../service"

export type {
  LogError,
  LogEvent,
  Logger,
  LoggerEnvironment,
  LogLevel,
  LogValue,
} from "../logger-service"
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
} from "../logger-service"

export function createCapturedLogger(write: (value: string) => void): Logger {
  return Object.freeze({
    log(event) {
      write(`${renderLogEvent(event)}\n`)
      return serviceSuccess(unit)
    },
  })
}
