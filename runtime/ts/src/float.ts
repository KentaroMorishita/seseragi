import { assertInt } from "./int"
import type { RoundingMode } from "./number"
import { canonicalFloat } from "./show"
import {
  Equal,
  Greater,
  Just,
  Left,
  Less,
  Nothing,
  Right,
  type Either,
  type Maybe,
  type Ordering,
} from "./sum"

export type FloatParseError =
  | Readonly<{ readonly tag: "EmptyFloat" }>
  | Readonly<{
      readonly tag: "InvalidFloat"
      readonly value: Readonly<{ readonly offset: number }>
    }>
  | Readonly<{ readonly tag: "FloatParseOverflow" }>

export const EmptyFloat: FloatParseError = Object.freeze({ tag: "EmptyFloat" })
export const InvalidFloat = (
  value: Readonly<{ readonly offset: number }>
): FloatParseError => ({ tag: "InvalidFloat", value })
export const FloatParseOverflow: FloatParseError = Object.freeze({
  tag: "FloatParseOverflow",
})

export type FloatConversionError =
  | Readonly<{ readonly tag: "FloatNotFinite" }>
  | Readonly<{ readonly tag: "FloatOutsideIntRange" }>

export const FloatNotFinite: FloatConversionError = Object.freeze({
  tag: "FloatNotFinite",
})
export const FloatOutsideIntRange: FloatConversionError = Object.freeze({
  tag: "FloatOutsideIntRange",
})

export function nan(_unit: undefined): number {
  return Number.NaN
}

export function positiveInfinity(_unit: undefined): number {
  return Number.POSITIVE_INFINITY
}

export function negativeInfinity(_unit: undefined): number {
  return Number.NEGATIVE_INFINITY
}

export function parse(text: string): Either<FloatParseError, number> {
  if (text.length === 0) return Left(EmptyFloat)
  if (text === "NaN") return Right(Number.NaN)
  if (text === "Infinity") return Right(Number.POSITIVE_INFINITY)
  if (text === "-Infinity") return Right(Number.NEGATIVE_INFINITY)
  const invalidOffset = floatSyntaxErrorOffset(text)
  if (invalidOffset !== undefined)
    return Left(InvalidFloat({ offset: invalidOffset }))
  const value = Number(text)
  return Number.isFinite(value) ? Right(value) : Left(FloatParseOverflow)
}

export function format(value: number): string {
  return canonicalFloat(value)
}

export function fromInt(value: number): number {
  return assertInt(value)
}

export function toInt(
  rounding: RoundingMode,
  value: number
): Either<FloatConversionError, number> {
  if (!Number.isFinite(value)) return Left(FloatNotFinite)
  const rounded = roundIntegral(rounding, value)
  if (!Number.isSafeInteger(rounded)) return Left(FloatOutsideIntRange)
  return Right(assertInt(rounded))
}

// biome-ignore lint/suspicious/noShadowRestrictedNames: std/float ABI name.
export function isNaN(value: number): boolean {
  return Number.isNaN(value)
}

// biome-ignore lint/suspicious/noShadowRestrictedNames: std/float ABI name.
export function isFinite(value: number): boolean {
  return Number.isFinite(value)
}

export function isInfinite(value: number): boolean {
  return (
    value === Number.POSITIVE_INFINITY || value === Number.NEGATIVE_INFINITY
  )
}

export function isNegativeZero(value: number): boolean {
  return Object.is(value, -0)
}

export function ieeeEq(left: number, right: number): boolean {
  return left === right
}

export function totalCompare(left: number, right: number): Ordering {
  if (Number.isNaN(left)) return Number.isNaN(right) ? Equal : Greater
  if (Number.isNaN(right)) return Less
  if (left === right) {
    if (Object.is(left, -0) && Object.is(right, 0)) return Less
    if (Object.is(left, 0) && Object.is(right, -0)) return Greater
    return Equal
  }
  return left < right ? Less : Greater
}

export function minimumNumber(right: number, left: number): number {
  if (Number.isNaN(left)) return Number.isNaN(right) ? Number.NaN : right
  if (Number.isNaN(right)) return left
  return Math.min(left, right)
}

export function maximumNumber(right: number, left: number): number {
  if (Number.isNaN(left)) return Number.isNaN(right) ? Number.NaN : right
  if (Number.isNaN(right)) return left
  return Math.max(left, right)
}

export function clampNumber(
  lower: number,
  upper: number,
  value: number
): Maybe<number> {
  if (
    Number.isNaN(lower) ||
    Number.isNaN(upper) ||
    Number.isNaN(value) ||
    totalCompare(lower, upper).tag === "Greater"
  ) {
    return Nothing
  }
  if (totalCompare(value, lower).tag === "Less") return Just(lower)
  if (totalCompare(value, upper).tag === "Greater") return Just(upper)
  return Just(value)
}

export function abs(value: number): number {
  return Number.isNaN(value) ? Number.NaN : Math.abs(value)
}

export function sign(value: number): Maybe<number> {
  if (Number.isNaN(value)) return Nothing
  return Just(value < 0 ? -1 : value > 0 ? 1 : 0)
}

export function power(exponent: number, base: number): number {
  return base ** exponent
}

export function roundIntegral(rounding: RoundingMode, value: number): number {
  if (!Number.isFinite(value) || value === 0) return value
  switch (rounding.tag) {
    case "HalfEven": {
      const floor = Math.floor(value)
      const fraction = value - floor
      const rounded =
        fraction < 0.5
          ? floor
          : fraction > 0.5
            ? floor + 1
            : floor % 2 === 0
              ? floor
              : floor + 1
      return rounded === 0 && value < 0 ? -0 : rounded
    }
    case "HalfUp": {
      const magnitude = Math.floor(Math.abs(value) + 0.5)
      return value < 0 ? -magnitude : magnitude
    }
    case "TowardZero":
      return Math.trunc(value)
    case "AwayFromZero":
      return value < 0 ? Math.floor(value) : Math.ceil(value)
    case "Floor":
      return Math.floor(value)
    case "Ceiling":
      return Math.ceil(value)
  }
}

function floatSyntaxErrorOffset(text: string): number | undefined {
  let index = 0
  if (text[index] === "+" || text[index] === "-") index += 1

  if (text[index] === ".") {
    index += 1
    if (!isAsciiDigit(text, index)) return utf8Offset(text, index)
    while (isAsciiDigit(text, index)) index += 1
  } else if (isAsciiDigit(text, index)) {
    while (isAsciiDigit(text, index)) index += 1
    if (text[index] === ".") {
      index += 1
      while (isAsciiDigit(text, index)) index += 1
    }
  } else {
    return utf8Offset(text, index)
  }

  if (text[index] === "e" || text[index] === "E") {
    index += 1
    if (text[index] === "+" || text[index] === "-") index += 1
    if (!isAsciiDigit(text, index)) return utf8Offset(text, index)
    while (isAsciiDigit(text, index)) index += 1
  }
  return index === text.length ? undefined : utf8Offset(text, index)
}

function isAsciiDigit(text: string, index: number): boolean {
  const code = text.charCodeAt(index)
  return code >= 48 && code <= 57
}

function utf8Offset(text: string, index: number): number {
  return new TextEncoder().encode(text.slice(0, index)).length
}
