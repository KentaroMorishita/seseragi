import type { Effect, Unit } from "./effect"
import { toArray, type List } from "./list"
import { type ServiceOperation, serviceEffect } from "./service"

export type LogLevel =
  | Readonly<{ readonly tag: "LogTrace" }>
  | Readonly<{ readonly tag: "LogDebug" }>
  | Readonly<{ readonly tag: "LogInfo" }>
  | Readonly<{ readonly tag: "LogWarn" }>
  | Readonly<{ readonly tag: "LogFailure" }>

export type LogValue =
  | Readonly<{ readonly tag: "LogString"; readonly value: string }>
  | Readonly<{ readonly tag: "LogInt"; readonly value: number }>
  | Readonly<{ readonly tag: "LogFloat"; readonly value: number }>
  | Readonly<{ readonly tag: "LogBool"; readonly value: boolean }>

export type LogEvent = Readonly<{
  readonly level: LogLevel
  readonly message: string
  readonly fields: List<readonly [string, LogValue]>
}>

export type LogError = Readonly<{
  readonly kind: "log-error"
  readonly message: string
}>

export type Logger = Readonly<{
  readonly log: (event: LogEvent) => ServiceOperation<LogError, Unit>
}>

export type LoggerEnvironment = Readonly<{
  readonly logger: Logger
}>

export const LogTrace: LogLevel = Object.freeze({ tag: "LogTrace" })
export const LogDebug: LogLevel = Object.freeze({ tag: "LogDebug" })
export const LogInfo: LogLevel = Object.freeze({ tag: "LogInfo" })
export const LogWarn: LogLevel = Object.freeze({ tag: "LogWarn" })
export const LogFailure: LogLevel = Object.freeze({ tag: "LogFailure" })

export const LogString = (value: string): LogValue => ({
  tag: "LogString",
  value,
})
export const LogInt = (value: number): LogValue => ({ tag: "LogInt", value })
export const LogFloat = (value: number): LogValue => ({
  tag: "LogFloat",
  value,
})
export const LogBool = (value: boolean): LogValue => ({
  tag: "LogBool",
  value,
})

export function log(
  event: LogEvent
): Effect<LoggerEnvironment, LogError, Unit> {
  return serviceEffect((environment: LoggerEnvironment) =>
    environment.logger.log(event)
  )
}

export function renderLogEvent(event: LogEvent): string {
  return JSON.stringify({
    level: event.level.tag.slice(3).toLowerCase(),
    message: event.message,
    fields: toArray(event.fields).map(([name, value]) => [
      name,
      logValue(value),
    ]),
  })
}

function logValue(value: LogValue): string | number | boolean {
  return value.value
}
