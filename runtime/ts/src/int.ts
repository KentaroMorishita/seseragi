import type { Unit } from "./effect"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"

export const MIN_INT = Number.MIN_SAFE_INTEGER
export const MAX_INT = Number.MAX_SAFE_INTEGER

export function assertInt(value: number): number {
  if (!Number.isSafeInteger(value)) {
    throw new RangeError("Seseragi Int overflow")
  }
  return value === 0 ? 0 : value
}

/** Decode an untrusted TypeScript value at a generated foreign boundary. */
export function decodeForeignInt(value: unknown): number {
  return decodeIntBoundary("foreign TypeScript", value)
}

/** Encode a Seseragi Int for a generated TypeScript wrapper. */
export function encodeForeignInt(value: number): number {
  return assertInt(value)
}

/** Decode an untrusted JavaScript JSON value as a Seseragi Int. */
export function decodeJsonInt(value: unknown): number {
  return decodeIntBoundary("JSON", value)
}

/** Encode a Seseragi Int as a JavaScript JSON number value. */
export function encodeJsonInt(value: number): number {
  return assertInt(value)
}

export function add(left: number, right: number): number {
  return assertInt(left + right)
}

export const intZero = {
  zero: (_unit: Unit): number => 0,
} as const

export const intOne = {
  one: (_unit: Unit): number => 1,
} as const

export const intAdd = {
  add:
    (left: number) =>
    (right: number): number =>
      add(left, right),
} as const

export function subtract(left: number, right: number): number {
  return assertInt(left - right)
}

export function multiply(left: number, right: number): number {
  return assertInt(left * right)
}

export const intMul = {
  mul:
    (left: number) =>
    (right: number): number =>
      multiply(left, right),
} as const

export function divide(left: number, right: number): number {
  if (right === 0) {
    throw new RangeError("Seseragi Int division by zero")
  }
  assertInt(left)
  assertInt(right)
  return Number(BigInt(left) / BigInt(right))
}

export function remainder(left: number, right: number): number {
  if (right === 0) {
    throw new RangeError("Seseragi Int remainder by zero")
  }
  assertInt(left)
  assertInt(right)
  return Number(BigInt(left) % BigInt(right))
}

export function power(base: number, exponent: number): number {
  if (exponent < 0) {
    throw new RangeError("Seseragi Int negative exponent")
  }
  return assertInt(base ** exponent)
}

export type IntParseError =
  | Readonly<{ readonly tag: "EmptyInt" }>
  | Readonly<{ readonly tag: "InvalidIntRadix"; readonly value: number }>
  | Readonly<{
      readonly tag: "InvalidIntDigit"
      readonly value: Readonly<{
        readonly offset: number
        readonly radix: number
      }>
    }>
  | Readonly<{ readonly tag: "IntOutsideRange" }>

export const EmptyInt: IntParseError = Object.freeze({ tag: "EmptyInt" })
export const InvalidIntRadix = (value: number): IntParseError => ({
  tag: "InvalidIntRadix",
  value,
})
export const InvalidIntDigit = (
  value: Readonly<{ readonly offset: number; readonly radix: number }>
): IntParseError => ({ tag: "InvalidIntDigit", value })
export const IntOutsideRange: IntParseError = Object.freeze({
  tag: "IntOutsideRange",
})

export type IntDivisionError = Readonly<{ readonly tag: "IntDivisionByZero" }>
export const IntDivisionByZero: IntDivisionError = Object.freeze({
  tag: "IntDivisionByZero",
})

export type IntPowerError =
  | Readonly<{ readonly tag: "NegativeIntExponent"; readonly value: number }>
  | Readonly<{ readonly tag: "IntPowerOverflow" }>
export const NegativeIntExponent = (value: number): IntPowerError => ({
  tag: "NegativeIntExponent",
  value,
})
export const IntPowerOverflow: IntPowerError = Object.freeze({
  tag: "IntPowerOverflow",
})

const MIN_INT_BIGINT = BigInt(MIN_INT)
const MAX_INT_BIGINT = BigInt(MAX_INT)

export function minValue(_unit: Unit): number {
  return MIN_INT
}

export function maxValue(_unit: Unit): number {
  return MAX_INT
}

export function parse(text: string): Either<IntParseError, number> {
  return parseInteger(10, text, true)
}

export function parseRadix(
  radix: number,
  text: string
): Either<IntParseError, number> {
  return parseInteger(radix, text, false)
}

export function format(value: number): string {
  return assertInt(value).toString(10)
}

export function formatRadix(
  radix: number,
  value: number
): Either<IntParseError, string> {
  if (!validRadix(radix)) return Left(InvalidIntRadix(radix))
  return Right(assertInt(value).toString(radix))
}

export function checkedAdd(right: number, left: number): Maybe<number> {
  return checkedExact(BigInt(left) + BigInt(right))
}

export function checkedSubtract(right: number, left: number): Maybe<number> {
  return checkedExact(BigInt(left) - BigInt(right))
}

export function checkedMultiply(right: number, left: number): Maybe<number> {
  return checkedExact(BigInt(left) * BigInt(right))
}

export function saturatingAdd(right: number, left: number): number {
  return saturate(BigInt(left) + BigInt(right))
}

export function saturatingSubtract(right: number, left: number): number {
  return saturate(BigInt(left) - BigInt(right))
}

export function saturatingMultiply(right: number, left: number): number {
  return saturate(BigInt(left) * BigInt(right))
}

export function checkedDivide(
  divisor: number,
  dividend: number
): Either<IntDivisionError, number> {
  if (divisor === 0) return Left(IntDivisionByZero)
  assertInt(divisor)
  assertInt(dividend)
  return Right(Number(BigInt(dividend) / BigInt(divisor)))
}

export function checkedRemainder(
  divisor: number,
  dividend: number
): Either<IntDivisionError, number> {
  if (divisor === 0) return Left(IntDivisionByZero)
  assertInt(divisor)
  assertInt(dividend)
  return Right(Number(BigInt(dividend) % BigInt(divisor)))
}

export function checkedPower(
  exponent: number,
  base: number
): Either<IntPowerError, number> {
  assertInt(exponent)
  assertInt(base)
  if (exponent < 0) return Left(NegativeIntExponent(exponent))
  const result = boundedPower(base, exponent)
  return result === undefined ? Left(IntPowerOverflow) : Right(result)
}

export function saturatingPower(
  exponent: number,
  base: number
): Either<IntPowerError, number> {
  assertInt(exponent)
  assertInt(base)
  if (exponent < 0) return Left(NegativeIntExponent(exponent))
  const result = boundedPower(base, exponent)
  if (result !== undefined) return Right(result)
  return Right(base < 0 && exponent % 2 === 1 ? MIN_INT : MAX_INT)
}

export function abs(value: number): number {
  return Math.abs(assertInt(value))
}

export function minimum(right: number, left: number): number {
  return Math.min(assertInt(left), assertInt(right))
}

export function maximum(right: number, left: number): number {
  return Math.max(assertInt(left), assertInt(right))
}

export function clamp(lower: number, upper: number, value: number): number {
  assertInt(lower)
  assertInt(upper)
  assertInt(value)
  if (lower > upper) {
    throw new RangeError("Seseragi Int clamp lower bound exceeds upper bound")
  }
  return Math.min(upper, Math.max(lower, value))
}

export function sign(value: number): number {
  const integer = assertInt(value)
  return integer < 0 ? -1 : integer > 0 ? 1 : 0
}

function parseInteger(
  radix: number,
  text: string,
  canonicalDecimal: boolean
): Either<IntParseError, number> {
  if (!validRadix(radix)) return Left(InvalidIntRadix(radix))
  if (text.length === 0) return Left(EmptyInt)
  let index = 0
  let negative = false
  if (text[0] === "+" || text[0] === "-") {
    negative = text[0] === "-"
    index = 1
  }
  if (index === text.length) {
    return Left(InvalidIntDigit({ offset: utf8Offset(text, index), radix }))
  }
  if (canonicalDecimal && text[index] === "0" && index + 1 < text.length) {
    return Left(InvalidIntDigit({ offset: utf8Offset(text, index + 1), radix }))
  }
  let result = 0n
  const radixBigInt = BigInt(radix)
  for (; index < text.length; index += 1) {
    const digit = digitValue(text.charCodeAt(index))
    if (digit < 0 || digit >= radix) {
      return Left(InvalidIntDigit({ offset: utf8Offset(text, index), radix }))
    }
    result = result * radixBigInt + BigInt(digit)
    const signed = negative ? -result : result
    if (signed < MIN_INT_BIGINT || signed > MAX_INT_BIGINT) {
      return Left(IntOutsideRange)
    }
  }
  return Right(Number(negative ? -result : result))
}

function validRadix(radix: number): boolean {
  return Number.isSafeInteger(radix) && radix >= 2 && radix <= 36
}

function decodeIntBoundary(boundary: string, value: unknown): number {
  if (typeof value !== "number") {
    throw new TypeError(`${boundary} Int input must be a number`)
  }
  if (!Number.isFinite(value)) {
    throw new RangeError(`${boundary} Int input must be finite`)
  }
  if (!Number.isInteger(value)) {
    throw new RangeError(`${boundary} Int input must be integral`)
  }
  if (!Number.isSafeInteger(value)) {
    throw new RangeError(`${boundary} Int input must be a safe integer`)
  }
  return value === 0 ? 0 : value
}

function digitValue(code: number): number {
  if (code >= 48 && code <= 57) return code - 48
  if (code >= 65 && code <= 90) return code - 65 + 10
  if (code >= 97 && code <= 122) return code - 97 + 10
  return -1
}

function utf8Offset(text: string, index: number): number {
  return new TextEncoder().encode(text.slice(0, index)).length
}

function checkedExact(value: bigint): Maybe<number> {
  return value < MIN_INT_BIGINT || value > MAX_INT_BIGINT
    ? Nothing
    : Just(Number(value))
}

function saturate(value: bigint): number {
  if (value < MIN_INT_BIGINT) return MIN_INT
  if (value > MAX_INT_BIGINT) return MAX_INT
  return Number(value)
}

function boundedPower(base: number, exponent: number): number | undefined {
  if (exponent === 0) return 1
  if (base === 0) return 0
  if (base === 1) return 1
  if (base === -1) return exponent % 2 === 0 ? 1 : -1
  let result = 1n
  let factor = BigInt(base)
  let remaining = BigInt(exponent)
  while (remaining > 0n) {
    if ((remaining & 1n) === 1n) {
      result *= factor
      if (result < MIN_INT_BIGINT || result > MAX_INT_BIGINT) return undefined
    }
    remaining >>= 1n
    if (remaining > 0n) {
      factor *= factor
      if (factor < MIN_INT_BIGINT || factor > MAX_INT_BIGINT) return undefined
    }
  }
  return Number(result)
}
