import { describe, expect, test } from "bun:test"
import {
  FloatNotFinite,
  FloatOutsideIntRange,
  roundIntegral,
  toInt,
} from "../src/float"
import {
  AwayFromZero,
  Ceiling,
  Floor,
  HalfEven,
  HalfUp,
  type RoundingMode,
  TowardZero,
} from "../src/number"

const modes: readonly RoundingMode[] = [
  HalfEven,
  HalfUp,
  TowardZero,
  AwayFromZero,
  Floor,
  Ceiling,
]

// Expected columns follow modes above; Object.is distinguishes both zero signs.
const boundaries: ReadonlyArray<readonly [number, readonly number[]]> = [
  [0.49999999999999994, [0, 0, 0, 1, 0, 1]],
  [-0.49999999999999994, [-0, -0, -0, -1, -1, -0]],
  [0.5, [0, 1, 0, 1, 0, 1]],
  [-0.5, [-0, -1, -0, -1, -1, -0]],
  [0.5000000000000001, [1, 1, 0, 1, 0, 1]],
  [-0.5000000000000001, [-1, -1, -0, -1, -1, -0]],
  [1.4999999999999998, [1, 1, 1, 2, 1, 2]],
  [-1.4999999999999998, [-1, -1, -1, -2, -2, -1]],
  [1.5, [2, 2, 1, 2, 1, 2]],
  [-1.5, [-2, -2, -1, -2, -2, -1]],
  [1.5000000000000002, [2, 2, 1, 2, 1, 2]],
  [-1.5000000000000002, [-2, -2, -1, -2, -2, -1]],
  [2.5, [2, 3, 2, 3, 2, 3]],
  [-2.5, [-2, -3, -2, -3, -3, -2]],
  [3.5, [4, 4, 3, 4, 3, 4]],
  [-3.5, [-4, -4, -3, -4, -4, -3]],
  [Number.MIN_VALUE, [0, 0, 0, 1, 0, 1]],
  [-Number.MIN_VALUE, [-0, -0, -0, -1, -1, -0]],
]

describe("Float integral rounding", () => {
  test("retains all six modes around positive and negative midpoints", () => {
    for (const [value, expected] of boundaries) {
      for (const [index, mode] of modes.entries()) {
        expect(roundIntegral(mode, value)).toBe(expected[index])
      }
    }
  })

  test("preserves exact integers, including odd large integers", () => {
    for (const magnitude of [
      1,
      2,
      2 ** 51 + 1,
      2 ** 52 - 1,
      2 ** 52,
      2 ** 52 + 1,
      Number.MAX_SAFE_INTEGER,
      2 ** 53,
      Number.MAX_VALUE,
    ]) {
      for (const mode of modes) {
        expect(roundIntegral(mode, magnitude)).toBe(magnitude)
        expect(roundIntegral(mode, -magnitude)).toBe(-magnitude)
      }
    }
  })

  test("preserves signed zero, NaN and both infinities in every mode", () => {
    for (const mode of modes) {
      for (const value of [0, -0, Number.NaN, Infinity, -Infinity]) {
        expect(roundIntegral(mode, value)).toBe(value)
      }
    }
  })

  test("converts both safe Int endpoints without changing their values", () => {
    for (const mode of modes) {
      for (const value of [
        Number.MIN_SAFE_INTEGER,
        -(2 ** 52 + 1),
        2 ** 52 + 1,
        Number.MAX_SAFE_INTEGER,
      ]) {
        expect(toInt(mode, value)).toEqual({ tag: "Right", value })
      }
    }
  })

  test("normalizes rounded zero only when converting Float to Int", () => {
    for (const value of [0, -0, 0.49999999999999994, -0.49999999999999994]) {
      const converted = toInt(HalfUp, value)
      expect(converted).toEqual({ tag: "Right", value: 0 })
      if (converted.tag === "Right") expect(converted.value).toBe(0)
    }
    for (const [value, expected] of [
      [0.5, 1],
      [-0.5, -1],
      [2.5, 3],
      [-2.5, -3],
    ]) {
      expect(toInt(HalfUp, value!)).toEqual({
        tag: "Right",
        value: expected,
      })
    }
  })

  test("retains genuine out-of-range and non-finite conversion errors", () => {
    for (const mode of modes) {
      for (const value of [
        2 ** 53,
        -(2 ** 53),
        Number.MAX_VALUE,
        -Number.MAX_VALUE,
      ]) {
        expect(toInt(mode, value)).toEqual({
          tag: "Left",
          value: FloatOutsideIntRange,
        })
      }
      for (const value of [Number.NaN, Infinity, -Infinity]) {
        expect(toInt(mode, value)).toEqual({
          tag: "Left",
          value: FloatNotFinite,
        })
      }
    }
  })

  test("HalfUp agrees with exact binary64 rational arithmetic across all finite exponents", () => {
    const view = new DataView(new ArrayBuffer(8))
    const fractions = [
      0n,
      1n,
      (1n << 51n) - 1n,
      1n << 51n,
      (1n << 51n) + 1n,
      (1n << 52n) - 2n,
      (1n << 52n) - 1n,
    ]
    for (let exponent = 0; exponent < 2047; exponent += 1) {
      for (const fraction of fractions) {
        view.setBigUint64(0, (BigInt(exponent) << 52n) | fraction)
        const value = view.getFloat64(0)
        const significand = exponent === 0 ? fraction : (1n << 52n) | fraction
        const shift = exponent === 0 ? -1074 : exponent - 1023 - 52
        let expected = value
        if (shift < 0) {
          const denominator = 1n << BigInt(-shift)
          const whole = significand / denominator
          const remainder = significand % denominator
          expected = Number(whole + (2n * remainder >= denominator ? 1n : 0n))
        }
        expect(roundIntegral(HalfUp, value)).toBe(expected)
        expect(roundIntegral(HalfUp, -value)).toBe(-expected)
      }
    }
  })
})
