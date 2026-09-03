import type { Unit } from "./effect"
import type { Eq } from "./equality"
import type { Hash } from "./hash"
import { assertInt, MAX_INT, MIN_INT } from "./int"
import type { Ord } from "./sequence"
import { type Either, Equal, Greater, Left, Less, Right } from "./sum"

declare const bigIntBrand: unique symbol

/** Runtime representation of Seseragi's opaque arbitrary-precision integer. */
export type BigInt = bigint & { readonly [bigIntBrand]: true }

export type BigIntParseError =
  | Readonly<{ readonly tag: "EmptyBigInt" }>
  | Readonly<{ readonly tag: "InvalidBigIntRadix"; readonly value: number }>
  | Readonly<{
      readonly tag: "InvalidBigIntDigit"
      readonly value: Readonly<{
        readonly offset: number
        readonly radix: number
      }>
    }>

export type BigIntDivisionError = Readonly<{
  readonly tag: "BigIntDivisionByZero"
}>

export type BigIntPowerError = Readonly<{
  readonly tag: "NegativeBigIntExponent"
  readonly value: number
}>

export type BigIntConversionError = Readonly<{
  readonly tag: "BigIntOutsideIntRange"
}>

const asBigInt = (value: bigint): BigInt => value as BigInt

export const EmptyBigInt: BigIntParseError = Object.freeze({
  tag: "EmptyBigInt",
})

export const InvalidBigIntRadix = (value: number): BigIntParseError => ({
  tag: "InvalidBigIntRadix",
  value,
})

export const InvalidBigIntDigit = (
  value: Readonly<{ readonly offset: number; readonly radix: number }>
): BigIntParseError => ({ tag: "InvalidBigIntDigit", value })

export const BigIntDivisionByZero: BigIntDivisionError = Object.freeze({
  tag: "BigIntDivisionByZero",
})

export const NegativeBigIntExponent = (value: number): BigIntPowerError => ({
  tag: "NegativeBigIntExponent",
  value,
})

export const BigIntOutsideIntRange: BigIntConversionError = Object.freeze({
  tag: "BigIntOutsideIntRange",
})

export function parse(text: string): Either<BigIntParseError, BigInt> {
  return parseInteger(10, text, true)
}

export function parseRadix(
  radix: number,
  text: string
): Either<BigIntParseError, BigInt> {
  return parseInteger(radix, text, false)
}

export function format(value: BigInt): string {
  return value.toString(10)
}

export function formatRadix(
  radix: number,
  value: BigInt
): Either<BigIntParseError, string> {
  if (!validRadix(radix)) return Left(InvalidBigIntRadix(radix))
  return Right(value.toString(radix))
}

export function fromInt(value: number): BigInt {
  return asBigInt(globalThis.BigInt(assertInt(value)))
}

export function toInt(value: BigInt): Either<BigIntConversionError, number> {
  if (
    value < globalThis.BigInt(MIN_INT) ||
    value > globalThis.BigInt(MAX_INT)
  ) {
    return Left(BigIntOutsideIntRange)
  }
  return Right(Number(value))
}

export function add(left: BigInt, right: BigInt): BigInt {
  return asBigInt(left + right)
}

export function subtract(left: BigInt, right: BigInt): BigInt {
  return asBigInt(left - right)
}

export function multiply(left: BigInt, right: BigInt): BigInt {
  return asBigInt(left * right)
}

export function divide(left: BigInt, right: BigInt): BigInt {
  if (right === 0n) {
    throw new RangeError("Seseragi BigInt division by zero")
  }
  return asBigInt(left / right)
}

export function remainder(left: BigInt, right: BigInt): BigInt {
  if (right === 0n) {
    throw new RangeError("Seseragi BigInt remainder by zero")
  }
  return asBigInt(left % right)
}

export function power(base: BigInt, exponent: number): BigInt {
  assertInt(exponent)
  if (exponent < 0) {
    throw new RangeError("Seseragi BigInt negative exponent")
  }
  return exactPower(base, exponent)
}

export function checkedDivide(
  divisor: BigInt,
  dividend: BigInt
): Either<BigIntDivisionError, BigInt> {
  return divisor === 0n
    ? Left(BigIntDivisionByZero)
    : Right(asBigInt(dividend / divisor))
}

export function checkedRemainder(
  divisor: BigInt,
  dividend: BigInt
): Either<BigIntDivisionError, BigInt> {
  return divisor === 0n
    ? Left(BigIntDivisionByZero)
    : Right(asBigInt(dividend % divisor))
}

export function checkedPower(
  exponent: number,
  base: BigInt
): Either<BigIntPowerError, BigInt> {
  assertInt(exponent)
  return exponent < 0
    ? Left(NegativeBigIntExponent(exponent))
    : Right(exactPower(base, exponent))
}

export function abs(value: BigInt): BigInt {
  return value < 0n ? asBigInt(-value) : value
}

export function sign(value: BigInt): number {
  return value < 0n ? -1 : value > 0n ? 1 : 0
}

export const bigIntEq: Eq<BigInt> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      left === right,
})

export const bigIntOrd: Ord<BigInt> & Eq<BigInt> = Object.freeze({
  ...bigIntEq,
  compare: (left) => (right) =>
    left < right ? Less : left > right ? Greater : Equal,
})

export const bigIntHash: Hash<BigInt> = Object.freeze({
  hash: (value): number => {
    let magnitude = value < 0n ? -value : value
    let state = Math.imul(0x811c9dc5 ^ (value < 0n ? 1 : 0), 0x01000193)
    do {
      state = Math.imul(state ^ Number(magnitude & 0xffff_ffffn), 0x01000193)
      magnitude >>= 32n
    } while (magnitude !== 0n)
    return state | 0
  },
})

export const bigIntZero = Object.freeze({
  zero: (_unit: Unit): BigInt => asBigInt(0n),
})

export const bigIntOne = Object.freeze({
  one: (_unit: Unit): BigInt => asBigInt(1n),
})

export const bigIntAdd = Object.freeze({
  add:
    (left: BigInt) =>
    (right: BigInt): BigInt =>
      add(left, right),
})

export const bigIntSub = Object.freeze({
  sub:
    (left: BigInt) =>
    (right: BigInt): BigInt =>
      subtract(left, right),
})

export const bigIntMul = Object.freeze({
  mul:
    (left: BigInt) =>
    (right: BigInt): BigInt =>
      multiply(left, right),
})

export const bigIntDiv = Object.freeze({
  div:
    (left: BigInt) =>
    (right: BigInt): BigInt =>
      divide(left, right),
})

export const bigIntRem = Object.freeze({
  rem:
    (left: BigInt) =>
    (right: BigInt): BigInt =>
      remainder(left, right),
})

export const bigIntPow = Object.freeze({
  pow:
    (base: BigInt) =>
    (exponent: number): BigInt =>
      power(base, exponent),
})

export const bigIntParseErrorEq: Eq<BigIntParseError> = Object.freeze({
  eq:
    (left) =>
    (right): boolean => {
      if (left.tag !== right.tag) return false
      switch (left.tag) {
        case "EmptyBigInt":
          return true
        case "InvalidBigIntRadix":
          return (
            right.tag === "InvalidBigIntRadix" && left.value === right.value
          )
        case "InvalidBigIntDigit":
          return (
            right.tag === "InvalidBigIntDigit" &&
            left.value.offset === right.value.offset &&
            left.value.radix === right.value.radix
          )
      }
    },
})

export const bigIntDivisionErrorEq: Eq<BigIntDivisionError> = Object.freeze({
  eq:
    (_left) =>
    (_right): boolean =>
      true,
})

export const bigIntPowerErrorEq: Eq<BigIntPowerError> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      left.value === right.value,
})

export const bigIntConversionErrorEq: Eq<BigIntConversionError> = Object.freeze(
  {
    eq:
      (_left) =>
      (_right): boolean =>
        true,
  }
)

function parseInteger(
  radix: number,
  text: string,
  canonicalDecimal: boolean
): Either<BigIntParseError, BigInt> {
  if (!validRadix(radix)) return Left(InvalidBigIntRadix(radix))
  if (text.length === 0) return Left(EmptyBigInt)
  let index = 0
  let negative = false
  if (text[0] === "+" || text[0] === "-") {
    negative = text[0] === "-"
    index = 1
  }
  if (index === text.length) {
    return Left(InvalidBigIntDigit({ offset: utf8Offset(text, index), radix }))
  }
  if (canonicalDecimal && text[index] === "0" && index + 1 < text.length) {
    return Left(
      InvalidBigIntDigit({ offset: utf8Offset(text, index + 1), radix })
    )
  }
  let result = 0n
  const radixBigInt = globalThis.BigInt(radix)
  for (; index < text.length; index += 1) {
    const digit = digitValue(text.charCodeAt(index))
    if (digit < 0 || digit >= radix) {
      return Left(
        InvalidBigIntDigit({ offset: utf8Offset(text, index), radix })
      )
    }
    result = result * radixBigInt + globalThis.BigInt(digit)
  }
  return Right(asBigInt(negative ? -result : result))
}

function exactPower(base: BigInt, exponent: number): BigInt {
  let result = 1n
  let factor: bigint = base
  let remaining = globalThis.BigInt(exponent)
  while (remaining > 0n) {
    if ((remaining & 1n) === 1n) result *= factor
    remaining >>= 1n
    if (remaining > 0n) factor *= factor
  }
  return asBigInt(result)
}

function validRadix(radix: number): boolean {
  return Number.isSafeInteger(radix) && radix >= 2 && radix <= 36
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
