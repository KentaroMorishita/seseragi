import type { Unit } from "./effect"
import type { Eq } from "./equality"
import type { Hash } from "./hash"
import { assertInt, MAX_INT, MIN_INT } from "./int"
import type { RoundingMode } from "./number"
import type { Ord } from "./sequence"
import { type Either, Equal, Greater, Left, Less, Right } from "./sum"

declare const decimalBrand: unique symbol
declare const decimalContextBrand: unique symbol

/** Canonical exact value `coefficient * 10^-scale`. */
export type Decimal = Readonly<{
  readonly coefficient: bigint
  readonly scale: bigint
  readonly [decimalBrand]: true
}>

/** Explicit precision and rounding policy for inexact decimal operations. */
export type DecimalContext = Readonly<{
  readonly digits: number
  readonly mode: RoundingMode
  readonly [decimalContextBrand]: true
}>

export type DecimalParseError = Readonly<{
  readonly tag: "InvalidDecimal"
  readonly value: Readonly<{ readonly offset: number }>
}>

export type DecimalContextError = Readonly<{
  readonly tag: "NonPositiveDecimalPrecision"
  readonly value: number
}>

export type DecimalArithmeticError =
  | Readonly<{ readonly tag: "DecimalDivisionByZero" }>
  | Readonly<{ readonly tag: "NonTerminatingDecimal" }>

export type DecimalConversionError =
  | Readonly<{ readonly tag: "DecimalNotIntegral" }>
  | Readonly<{ readonly tag: "DecimalOutsideIntRange" }>
  | Readonly<{ readonly tag: "DecimalOutsideFloatRange" }>
  | Readonly<{ readonly tag: "FloatNotFinite" }>

export const InvalidDecimal = (
  value: Readonly<{ readonly offset: number }>
): DecimalParseError => ({ tag: "InvalidDecimal", value })

export const NonPositiveDecimalPrecision = (
  value: number
): DecimalContextError => ({ tag: "NonPositiveDecimalPrecision", value })

export const DecimalDivisionByZero: DecimalArithmeticError = Object.freeze({
  tag: "DecimalDivisionByZero",
})

export const NonTerminatingDecimal: DecimalArithmeticError = Object.freeze({
  tag: "NonTerminatingDecimal",
})

export const DecimalNotIntegral: DecimalConversionError = Object.freeze({
  tag: "DecimalNotIntegral",
})

export const DecimalOutsideIntRange: DecimalConversionError = Object.freeze({
  tag: "DecimalOutsideIntRange",
})

export const DecimalOutsideFloatRange: DecimalConversionError = Object.freeze({
  tag: "DecimalOutsideFloatRange",
})

export const FloatNotFinite: DecimalConversionError = Object.freeze({
  tag: "FloatNotFinite",
})

export function parse(text: string): Either<DecimalParseError, Decimal> {
  const syntax = decimalSyntax(text)
  if (typeof syntax === "number") {
    return Left(InvalidDecimal({ offset: utf8Offset(text, syntax) }))
  }
  const coefficient = globalThis.BigInt(syntax.integer + syntax.fraction)
  const signed = syntax.negative ? -coefficient : coefficient
  const scale = globalThis.BigInt(syntax.fraction.length) - syntax.exponent
  return Right(makeDecimal(signed, scale))
}

export function format(value: Decimal): string {
  if (value.coefficient === 0n) return "0"
  const negative = value.coefficient < 0n
  const digits = (negative ? -value.coefficient : value.coefficient).toString()
  let magnitude: string
  if (value.scale <= 0n) {
    magnitude = digits + zeros(-value.scale)
  } else if (value.scale >= globalThis.BigInt(digits.length)) {
    magnitude = `0.${zeros(value.scale - globalThis.BigInt(digits.length))}${digits}`
  } else {
    const point = digits.length - Number(value.scale)
    magnitude = `${digits.slice(0, point)}.${digits.slice(point)}`
  }
  return negative ? `-${magnitude}` : magnitude
}

export function fromInt(value: number): Decimal {
  return makeDecimal(globalThis.BigInt(assertInt(value)), 0n)
}

export function toIntExact(
  value: Decimal
): Either<DecimalConversionError, number> {
  if (value.scale > 0n) return Left(DecimalNotIntegral)
  const magnitudeDigits = absBigInt(value.coefficient).toString().length
  const integerDigits = BigInt(magnitudeDigits) - value.scale
  if (integerDigits > 16n) return Left(DecimalOutsideIntRange)
  const integer = value.coefficient * powerOfTen(-value.scale)
  if (
    integer < globalThis.BigInt(MIN_INT) ||
    integer > globalThis.BigInt(MAX_INT)
  ) {
    return Left(DecimalOutsideIntRange)
  }
  return Right(Number(integer))
}

export function fromFloat(
  decimalContext: DecimalContext,
  value: number
): Either<DecimalConversionError, Decimal> {
  if (!Number.isFinite(value)) return Left(FloatNotFinite)
  if (value === 0) return Right(ZERO)

  const view = new DataView(new ArrayBuffer(8))
  view.setFloat64(0, value, false)
  const bits = view.getBigUint64(0, false)
  const negative = bits >> 63n !== 0n
  const exponentBits = Number((bits >> 52n) & 0x7ffn)
  const fraction = bits & 0x000f_ffff_ffff_ffffn
  const significand = exponentBits === 0 ? fraction : (1n << 52n) | fraction
  const binaryExponent = exponentBits === 0 ? -1074 : exponentBits - 1023 - 52
  let exact: Decimal
  if (binaryExponent >= 0) {
    exact = makeDecimal(
      sign(negative, significand << BigInt(binaryExponent)),
      0n
    )
  } else {
    const places = BigInt(-binaryExponent)
    exact = makeDecimal(sign(negative, significand * 5n ** places), places)
  }
  return Right(roundSignificant(exact, decimalContext))
}

export function toFloat(
  value: Decimal
): Either<DecimalConversionError, number> {
  if (value.coefficient === 0n) return Right(0)
  const exponent =
    BigInt(absBigInt(value.coefficient).toString().length - 1) - value.scale
  if (exponent > 308n) return Left(DecimalOutsideFloatRange)
  if (exponent < -324n) return Right(value.coefficient < 0n ? -0 : 0)
  const magnitude =
    value.coefficient < 0n
      ? makeDecimal(-value.coefficient, value.scale)
      : value
  if (compareDecimal(magnitude, MAX_FINITE_FLOAT) > 0) {
    return Left(DecimalOutsideFloatRange)
  }
  const converted = Number(format(value))
  return Number.isFinite(converted)
    ? Right(converted)
    : Left(DecimalOutsideFloatRange)
}

export function context(
  digits: number,
  mode: RoundingMode
): Either<DecimalContextError, DecimalContext> {
  assertInt(digits)
  return digits <= 0
    ? Left(NonPositiveDecimalPrecision(digits))
    : Right(Object.freeze({ digits, mode }) as DecimalContext)
}

export function precision(decimalContext: DecimalContext): number {
  return decimalContext.digits
}

export function rounding(decimalContext: DecimalContext): RoundingMode {
  return decimalContext.mode
}

export function add(left: Decimal, right: Decimal): Decimal {
  const scale = left.scale > right.scale ? left.scale : right.scale
  return makeDecimal(
    left.coefficient * powerOfTen(scale - left.scale) +
      right.coefficient * powerOfTen(scale - right.scale),
    scale
  )
}

export function subtract(left: Decimal, right: Decimal): Decimal {
  const scale = left.scale > right.scale ? left.scale : right.scale
  return makeDecimal(
    left.coefficient * powerOfTen(scale - left.scale) -
      right.coefficient * powerOfTen(scale - right.scale),
    scale
  )
}

export function multiply(left: Decimal, right: Decimal): Decimal {
  return makeDecimal(
    left.coefficient * right.coefficient,
    left.scale + right.scale
  )
}

export function divideExact(
  divisor: Decimal,
  dividend: Decimal
): Either<DecimalArithmeticError, Decimal> {
  if (divisor.coefficient === 0n) return Left(DecimalDivisionByZero)
  if (dividend.coefficient === 0n) return Right(ZERO)

  const common = gcd(
    absBigInt(dividend.coefficient),
    absBigInt(divisor.coefficient)
  )
  let numerator = dividend.coefficient / common
  let denominator = divisor.coefficient / common
  if (denominator < 0n) {
    numerator = -numerator
    denominator = -denominator
  }
  let twos = 0n
  let fives = 0n
  while (denominator % 2n === 0n) {
    denominator /= 2n
    twos += 1n
  }
  while (denominator % 5n === 0n) {
    denominator /= 5n
    fives += 1n
  }
  if (denominator !== 1n) return Left(NonTerminatingDecimal)

  const places = twos > fives ? twos : fives
  const coefficient = numerator * 2n ** (places - twos) * 5n ** (places - fives)
  return Right(
    makeDecimal(coefficient, places + dividend.scale - divisor.scale)
  )
}

export function divide(
  decimalContext: DecimalContext,
  divisor: Decimal,
  dividend: Decimal
): Either<DecimalArithmeticError, Decimal> {
  if (divisor.coefficient === 0n) return Left(DecimalDivisionByZero)
  if (dividend.coefficient === 0n) return Right(ZERO)

  const ratioExponent = floorLog10Ratio(
    absBigInt(dividend.coefficient),
    absBigInt(divisor.coefficient)
  )
  const exponent = ratioExponent + divisor.scale - dividend.scale
  const targetScale = BigInt(decimalContext.digits - 1) - exponent
  const decimalPower = divisor.scale - dividend.scale + targetScale
  const coefficient = roundedRatio(
    dividend.coefficient,
    divisor.coefficient,
    decimalPower,
    decimalContext.mode
  )
  return Right(makeDecimal(coefficient, targetScale))
}

export function quantize(
  targetScale: number,
  mode: RoundingMode,
  value: Decimal
): Decimal {
  assertInt(targetScale)
  return quantizeAtScale(BigInt(targetScale), mode, value)
}

export const decimalEq: Eq<Decimal> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      left.coefficient === right.coefficient && left.scale === right.scale,
})

export const decimalOrd: Ord<Decimal> & Eq<Decimal> = Object.freeze({
  ...decimalEq,
  compare: (left) => (right) => {
    const result = compareDecimal(left, right)
    return result < 0 ? Less : result > 0 ? Greater : Equal
  },
})

export const decimalHash: Hash<Decimal> = Object.freeze({
  hash: (value): number => {
    let state = hashBigInt(value.coefficient, 0x811c9dc5)
    state = hashBigInt(value.scale, state)
    return state | 0
  },
})

export const decimalZero = Object.freeze({
  zero: (_unit: Unit): Decimal => ZERO,
})

export const decimalOne = Object.freeze({
  one: (_unit: Unit): Decimal => ONE,
})

export const decimalAdd = Object.freeze({
  add:
    (left: Decimal) =>
    (right: Decimal): Decimal =>
      add(left, right),
})

export const decimalSub = Object.freeze({
  sub:
    (left: Decimal) =>
    (right: Decimal): Decimal =>
      subtract(left, right),
})

export const decimalMul = Object.freeze({
  mul:
    (left: Decimal) =>
    (right: Decimal): Decimal =>
      multiply(left, right),
})

export const decimalParseErrorEq: Eq<DecimalParseError> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      left.value.offset === right.value.offset,
})

export const decimalContextErrorEq: Eq<DecimalContextError> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      left.value === right.value,
})

export const decimalArithmeticErrorEq: Eq<DecimalArithmeticError> =
  Object.freeze({
    eq:
      (left) =>
      (right): boolean =>
        left.tag === right.tag,
  })

export const decimalConversionErrorEq: Eq<DecimalConversionError> =
  Object.freeze({
    eq:
      (left) =>
      (right): boolean =>
        left.tag === right.tag,
  })

const ZERO = makeDecimal(0n, 0n)
const ONE = makeDecimal(1n, 0n)
const MAX_FINITE_FLOAT = makeDecimal(((1n << 53n) - 1n) << 971n, 0n)

function makeDecimal(coefficient: bigint, scale: bigint): Decimal {
  if (coefficient === 0n) {
    return Object.freeze({ coefficient: 0n, scale: 0n }) as Decimal
  }
  while (coefficient % 10n === 0n) {
    coefficient /= 10n
    scale -= 1n
  }
  return Object.freeze({ coefficient, scale }) as Decimal
}

function quantizeAtScale(
  targetScale: bigint,
  mode: RoundingMode,
  value: Decimal
): Decimal {
  if (targetScale >= value.scale) return value
  const divisor = powerOfTen(value.scale - targetScale)
  return makeDecimal(
    roundQuotient(value.coefficient, divisor, mode),
    targetScale
  )
}

function roundSignificant(
  value: Decimal,
  decimalContext: DecimalContext
): Decimal {
  if (value.coefficient === 0n) return value
  const exponent =
    BigInt(absBigInt(value.coefficient).toString().length - 1) - value.scale
  const targetScale = BigInt(decimalContext.digits - 1) - exponent
  return quantizeAtScale(targetScale, decimalContext.mode, value)
}

function roundedRatio(
  numerator: bigint,
  denominator: bigint,
  decimalPower: bigint,
  mode: RoundingMode
): bigint {
  if (decimalPower >= 0n) {
    numerator *= powerOfTen(decimalPower)
  } else {
    denominator *= powerOfTen(-decimalPower)
  }
  if (denominator < 0n) {
    numerator = -numerator
    denominator = -denominator
  }
  return roundQuotient(numerator, denominator, mode)
}

function roundQuotient(
  numerator: bigint,
  denominator: bigint,
  mode: RoundingMode
): bigint {
  const quotient = numerator / denominator
  const remainder = numerator % denominator
  if (remainder === 0n) return quotient
  const direction = numerator < 0n ? -1n : 1n
  let increment = false
  switch (mode.tag) {
    case "TowardZero":
      break
    case "AwayFromZero":
      increment = true
      break
    case "Floor":
      increment = direction < 0n
      break
    case "Ceiling":
      increment = direction > 0n
      break
    case "HalfUp":
      increment = absBigInt(remainder) * 2n >= denominator
      break
    case "HalfEven": {
      const twice = absBigInt(remainder) * 2n
      increment =
        twice > denominator ||
        (twice === denominator && absBigInt(quotient) % 2n === 1n)
      break
    }
  }
  return increment ? quotient + direction : quotient
}

function compareDecimal(left: Decimal, right: Decimal): number {
  if (left.coefficient === right.coefficient && left.scale === right.scale) {
    return 0
  }
  if (left.coefficient < 0n && right.coefficient >= 0n) return -1
  if (left.coefficient >= 0n && right.coefficient < 0n) return 1
  const polarity = left.coefficient < 0n ? -1 : 1
  const leftDigits = absBigInt(left.coefficient).toString()
  const rightDigits = absBigInt(right.coefficient).toString()
  const leftExponent = BigInt(leftDigits.length) - left.scale
  const rightExponent = BigInt(rightDigits.length) - right.scale
  if (leftExponent !== rightExponent) {
    return (leftExponent < rightExponent ? -1 : 1) * polarity
  }
  const width = Math.max(leftDigits.length, rightDigits.length)
  const leftAligned = leftDigits.padEnd(width, "0")
  const rightAligned = rightDigits.padEnd(width, "0")
  return (leftAligned < rightAligned ? -1 : 1) * polarity
}

function floorLog10Ratio(numerator: bigint, denominator: bigint): bigint {
  const candidate = BigInt(
    numerator.toString().length - denominator.toString().length
  )
  const atLeastCandidatePower =
    candidate >= 0n
      ? numerator >= denominator * powerOfTen(candidate)
      : numerator * powerOfTen(-candidate) >= denominator
  return atLeastCandidatePower ? candidate : candidate - 1n
}

function powerOfTen(exponent: bigint): bigint {
  return 10n ** exponent
}

function zeros(count: bigint): string {
  if (count > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new RangeError("Decimal canonical spelling exceeds host limits")
  }
  return "0".repeat(Number(count))
}

function gcd(left: bigint, right: bigint): bigint {
  while (right !== 0n) {
    ;[left, right] = [right, left % right]
  }
  return left
}

function absBigInt(value: bigint): bigint {
  return value < 0n ? -value : value
}

function sign(negative: boolean, value: bigint): bigint {
  return negative ? -value : value
}

function hashBigInt(value: bigint, initial: number): number {
  let magnitude = value < 0n ? -value : value
  let state = Math.imul(initial ^ (value < 0n ? 1 : 0), 0x01000193)
  do {
    state = Math.imul(state ^ Number(magnitude & 0xffff_ffffn), 0x01000193)
    magnitude >>= 32n
  } while (magnitude !== 0n)
  return state
}

type DecimalSyntax = Readonly<{
  readonly negative: boolean
  readonly integer: string
  readonly fraction: string
  readonly exponent: bigint
}>

function decimalSyntax(text: string): DecimalSyntax | number {
  let index = 0
  let negative = false
  if (text[index] === "+" || text[index] === "-") {
    negative = text[index] === "-"
    index += 1
  }
  const integerStart = index
  if (text[index] === "0") {
    index += 1
    if (isDigit(text[index])) return index
  } else if (isNonZeroDigit(text[index])) {
    while (isDigit(text[index])) index += 1
  } else {
    return index
  }
  const integer = text.slice(integerStart, index)
  let fraction = ""
  if (text[index] === ".") {
    index += 1
    const fractionStart = index
    if (!isDigit(text[index])) return index
    while (isDigit(text[index])) index += 1
    fraction = text.slice(fractionStart, index)
  }
  let exponent = 0n
  if (text[index] === "e" || text[index] === "E") {
    index += 1
    let exponentNegative = false
    if (text[index] === "+" || text[index] === "-") {
      exponentNegative = text[index] === "-"
      index += 1
    }
    const exponentStart = index
    if (!isDigit(text[index])) return index
    while (isDigit(text[index])) index += 1
    exponent = globalThis.BigInt(text.slice(exponentStart, index))
    if (exponentNegative) exponent = -exponent
  }
  if (index !== text.length) return index
  return { negative, integer, fraction, exponent }
}

function isDigit(character: string | undefined): boolean {
  return character !== undefined && character >= "0" && character <= "9"
}

function isNonZeroDigit(character: string | undefined): boolean {
  return character !== undefined && character >= "1" && character <= "9"
}

function utf8Offset(text: string, index: number): number {
  return new TextEncoder().encode(text.slice(0, index)).length
}
