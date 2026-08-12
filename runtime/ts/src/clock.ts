import { createDuration, type Duration, type Instant } from "./clock-value"
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

export type Clock = Readonly<{
  now: (context: EffectContext) => Promise<Instant>
  sleep: (duration: Duration, context: EffectContext) => Promise<Unit>
}>

export type ClockEnvironment = Readonly<{ clock: Clock }>

export function zeroDuration(): Duration {
  return createDuration(0n)
}

export function milliseconds(value: number): Either<DurationError, Duration> {
  if (!Number.isSafeInteger(value) || value < 0) {
    return value < 0
      ? Left(Object.freeze({ tag: "NegativeDuration", value }))
      : Left(Object.freeze({ tag: "DurationOutsideRange" }))
  }
  try {
    return Right(createDuration(BigInt(value) * 1_000_000n))
  } catch {
    return Left(Object.freeze({ tag: "DurationOutsideRange" }))
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
