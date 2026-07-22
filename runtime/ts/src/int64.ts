import type { Unit } from "./effect"

const MIN_INT64 = -(1n << 63n)
const MAX_INT64 = (1n << 63n) - 1n

function assertInt64(value: bigint): bigint {
  if (value < MIN_INT64 || value > MAX_INT64) {
    throw new RangeError("Seseragi Int overflow")
  }
  return value
}

export function add(left: bigint, right: bigint): bigint {
  return assertInt64(left + right)
}

export const intZero = {
  zero: (_unit: Unit): bigint => 0n,
} as const

export const intAdd = {
  add:
    (left: bigint) =>
    (right: bigint): bigint =>
      add(left, right),
} as const

export function subtract(left: bigint, right: bigint): bigint {
  return assertInt64(left - right)
}

export function multiply(left: bigint, right: bigint): bigint {
  return assertInt64(left * right)
}

export function divide(left: bigint, right: bigint): bigint {
  if (right === 0n) {
    throw new RangeError("Seseragi Int division by zero")
  }
  if (left === MIN_INT64 && right === -1n) {
    throw new RangeError("Seseragi Int division overflow")
  }
  return left / right
}

export function remainder(left: bigint, right: bigint): bigint {
  if (right === 0n) {
    throw new RangeError("Seseragi Int remainder by zero")
  }
  if (left === MIN_INT64 && right === -1n) {
    throw new RangeError("Seseragi Int remainder overflow")
  }
  return left % right
}

export function power(base: bigint, exponent: bigint): bigint {
  if (exponent < 0n) {
    throw new RangeError("Seseragi Int negative exponent")
  }
  let result = 1n
  let factor = base
  let remaining = exponent
  while (remaining > 0n) {
    if ((remaining & 1n) === 1n) {
      result = assertInt64(result * factor)
    }
    remaining >>= 1n
    if (remaining > 0n) {
      factor = assertInt64(factor * factor)
    }
  }
  return result
}
