import { expect, test } from "bun:test"
import {
  addDuration,
  type Duration,
  type DurationError,
  durationBetween,
  hours,
  milliseconds,
  minutes,
  nanoseconds,
  seconds,
  toNanoseconds,
  zeroDuration,
} from "../src/clock"
import { createInstant } from "../src/clock-value"
import type { Either } from "../src/sum"

const scaledUnits = [
  [milliseconds, 1_000_000],
  [seconds, 1_000_000_000],
  [minutes, 60_000_000_000],
  [hours, 3_600_000_000_000],
] as const

function duration(result: Either<DurationError, Duration>): Duration {
  if (result.tag !== "Right") throw new Error("Expected a valid Duration")
  return result.value
}

test("toNanoseconds preserves exact values throughout the Int range", () => {
  expect(toNanoseconds(zeroDuration())).toBe(0)
  for (const value of [
    0,
    1,
    999_999_999,
    Number.MAX_SAFE_INTEGER - 1,
    Number.MAX_SAFE_INTEGER,
  ]) {
    expect(toNanoseconds(duration(nanoseconds(value)))).toBe(value)
  }
  for (const [make, scale] of scaledUnits) {
    const lastSafe = Math.floor(Number.MAX_SAFE_INTEGER / scale)
    expect(toNanoseconds(duration(make(lastSafe)))).toBe(lastSafe * scale)
  }
})

test("toNanoseconds rejects valid Durations outside the Int range", () => {
  const maximum = duration(nanoseconds(Number.MAX_SAFE_INTEGER))
  const values = [
    ...[1, 2, 3, Number.MAX_SAFE_INTEGER].map((extra) =>
      duration(addDuration(duration(nanoseconds(extra)), maximum))
    ),
    ...scaledUnits.map(([make, scale]) =>
      duration(make(Math.floor(Number.MAX_SAFE_INTEGER / scale) + 1))
    ),
    duration(seconds(9_223_372_036)),
    duration(milliseconds(9_223_372_036_854)),
    duration(
      addDuration(
        duration(nanoseconds(854_775_807)),
        duration(seconds(9_223_372_036))
      )
    ),
  ]
  for (const value of values) {
    expect(() => toNanoseconds(value)).toThrow(RangeError)
    expect(() => toNanoseconds(value)).toThrow(/^Seseragi Int overflow$/)
  }
})

test("durationBetween measures ordered Instants exactly", () => {
  const between = (earlier: bigint, later: bigint) =>
    durationBetween(createInstant(earlier), createInstant(later))
  for (const [earlier, later, expected] of [
    [0n, 0n, 0],
    [1_700_000_000_000_000_000n, 1_700_000_000_250_000_000n, 250_000_000],
    [-5n, 7n, 12],
  ] as const) {
    expect(toNanoseconds(duration(between(earlier, later)))).toBe(expected)
  }
  expect(between(-9_223_372_036_854_775_807n, 0n).tag).toBe("Right")
})

test("durationBetween reports reversed and out-of-range Instants", () => {
  const between = (earlier: bigint, later: bigint) =>
    durationBetween(createInstant(earlier), createInstant(later))
  expect(between(1_000n, 1n)).toEqual({
    tag: "Left",
    value: { tag: "NegativeDuration", value: -999 },
  })
  expect(between(9_007_199_254_740_991n, 0n)).toEqual({
    tag: "Left",
    value: { tag: "NegativeDuration", value: -9_007_199_254_740_991 },
  })
  for (const [earlier, later] of [
    [0n, 9_223_372_036_854_775_808n],
    [9_007_199_254_740_992n, 0n],
  ] as const) {
    expect(between(earlier, later)).toEqual({
      tag: "Left",
      value: { tag: "DurationOutsideRange" },
    })
  }
})
