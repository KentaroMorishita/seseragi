import type { Unit } from "./effect"

export const MIN_INT = Number.MIN_SAFE_INTEGER
export const MAX_INT = Number.MAX_SAFE_INTEGER

export function assertInt(value: number): number {
  if (!Number.isSafeInteger(value)) {
    throw new RangeError("Seseragi Int overflow")
  }
  return value === 0 ? 0 : value
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
  return assertInt(Math.trunc(left / right))
}

export function remainder(left: number, right: number): number {
  if (right === 0) {
    throw new RangeError("Seseragi Int remainder by zero")
  }
  return assertInt(left % right)
}

export function power(base: number, exponent: number): number {
  if (exponent < 0) {
    throw new RangeError("Seseragi Int negative exponent")
  }
  return assertInt(base ** exponent)
}
