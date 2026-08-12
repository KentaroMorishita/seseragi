const durationBrand: unique symbol = Symbol("seseragi.duration")
const instantBrand: unique symbol = Symbol("seseragi.instant")

export type Duration = Readonly<{ readonly [durationBrand]: true }>
export type Instant = Readonly<{ readonly [instantBrand]: true }>

const durations = new WeakMap<object, bigint>()
const instants = new WeakMap<object, bigint>()

export function createDuration(nanoseconds: bigint): Duration {
  if (nanoseconds < 0n || nanoseconds > 9_223_372_036_854_775_807n) {
    throw new RangeError("Duration nanoseconds are outside the supported range")
  }
  const value = Object.freeze({}) as Duration
  durations.set(value, nanoseconds)
  return value
}

export function durationNanoseconds(value: Duration): bigint {
  const nanoseconds = durations.get(value)
  if (nanoseconds === undefined) {
    throw new TypeError("Duration value does not use the runtime brand")
  }
  return nanoseconds
}

export function createInstant(nanoseconds: bigint): Instant {
  if (nanoseconds < 0n) {
    throw new RangeError("Instant nanoseconds must not be negative")
  }
  const value = Object.freeze({}) as Instant
  instants.set(value, nanoseconds)
  return value
}

export function instantNanoseconds(value: Instant): bigint {
  const nanoseconds = instants.get(value)
  if (nanoseconds === undefined) {
    throw new TypeError("Instant value does not use the runtime brand")
  }
  return nanoseconds
}
