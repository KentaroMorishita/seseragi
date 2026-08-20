import { createDuration, type Duration, type Instant } from "./clock-value"
import { durationNanoseconds } from "./clock-value"
import type { Effect, EffectContext, Unit } from "./effect"
import { serviceEffect, serviceSuccess } from "./service"
import { type Either, Left, Right } from "./sum"

export type { Duration, Instant } from "./clock-value"

export type NegativeDuration = Readonly<{
  tag: "NegativeDuration"
  value: number
}>

export type DurationOutsideRange = Readonly<{
  tag: "DurationOutsideRange"
}>

export type DurationError = NegativeDuration | DurationOutsideRange

export function NegativeDuration(value: number): NegativeDuration {
  return Object.freeze({ tag: "NegativeDuration", value })
}

export const DurationOutsideRange: DurationOutsideRange = Object.freeze({
  tag: "DurationOutsideRange",
})

export type Clock = Readonly<{
  now: (context: EffectContext) => Promise<Instant>
  sleep: (duration: Duration, context: EffectContext) => Promise<Unit>
}>

export type ClockEnvironment = Readonly<{ clock: Clock }>

export function zeroDuration(_unit?: Unit): Duration {
  return createDuration(0n)
}

export function nanoseconds(value: number): Either<DurationError, Duration> {
  return durationFromUnit(value, 1n)
}

export function milliseconds(value: number): Either<DurationError, Duration> {
  return durationFromUnit(value, 1_000_000n)
}

export function seconds(value: number): Either<DurationError, Duration> {
  return durationFromUnit(value, 1_000_000_000n)
}

export function minutes(value: number): Either<DurationError, Duration> {
  return durationFromUnit(value, 60_000_000_000n)
}

export function hours(value: number): Either<DurationError, Duration> {
  return durationFromUnit(value, 3_600_000_000_000n)
}

export function toNanoseconds(value: Duration): number {
  return Number(durationNanoseconds(value))
}

export function addDuration(
  right: Duration,
  left: Duration
): Either<DurationError, Duration> {
  try {
    return Right(
      createDuration(durationNanoseconds(left) + durationNanoseconds(right))
    )
  } catch {
    return Left(DurationOutsideRange)
  }
}

function durationFromUnit(
  value: number,
  nanosecondsPerUnit: bigint
): Either<DurationError, Duration> {
  if (!Number.isSafeInteger(value) || value < 0) {
    return value < 0
      ? Left(NegativeDuration(value))
      : Left(DurationOutsideRange)
  }
  try {
    return Right(createDuration(BigInt(value) * nanosecondsPerUnit))
  } catch {
    return Left(DurationOutsideRange)
  }
}

/** Reads the selected Clock without exposing its provider identity. */
export function now(): Effect<ClockEnvironment, never, Instant> {
  return serviceEffect(async (environment, context) =>
    serviceSuccess(await environment.clock.now(context))
  )
}

/** Waits on the selected Clock and preserves Effect cancellation. */
export function sleep(
  duration: Duration
): Effect<ClockEnvironment, never, Unit> {
  return serviceEffect(async (environment, context) =>
    serviceSuccess(await environment.clock.sleep(duration, context))
  )
}
