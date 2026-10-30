import { describe, expect, test } from "bun:test"
import { decimalOrd, fromInt, parse } from "../src/decimal"

const value = (source: string) => {
  const result = parse(source)
  if (result.tag !== "Right") throw new Error(`Invalid test Decimal: ${source}`)
  return result.value
}

describe("Decimal zero ordering", () => {
  test("zero follows every negative value and precedes every positive value", () => {
    const exponent = "999999999999999999999999999999999999"
    const negative = [
      "-100",
      "-1",
      "-0.01",
      `-1e-${exponent}`,
      `-1e${exponent}`,
    ]
    const positive = ["0.01", "1", "100", `1e-${exponent}`, `1e${exponent}`]
    for (const zero of [fromInt(0), value("0"), value(`-0.000e${exponent}`)]) {
      for (const source of negative) {
        const other = value(source)
        expect(decimalOrd.compare(zero)(other).tag).toBe("Greater")
        expect(decimalOrd.compare(other)(zero).tag).toBe("Less")
      }
      for (const source of positive) {
        const other = value(source)
        expect(decimalOrd.compare(zero)(other).tag).toBe("Less")
        expect(decimalOrd.compare(other)(zero).tag).toBe("Greater")
      }
    }
  })

  test("normalized zero spellings compare equally in both directions", () => {
    for (const left of [
      "0",
      "-0",
      "0.000e-999999999999999999999999999999999999",
    ]) {
      for (const right of [
        "+0",
        "-0.00",
        "0e999999999999999999999999999999999999",
      ]) {
        expect(decimalOrd.compare(value(left))(value(right)).tag).toBe("Equal")
        expect(decimalOrd.compare(value(right))(value(left)).tag).toBe("Equal")
      }
    }
  })

  test("unequal nonzero values keep their sign, exponent and aligned digit order", () => {
    const ordered = [
      "-100",
      "-1.21",
      "-1.2",
      "-0.01",
      "0.01",
      "1.2",
      "1.21",
      "100",
    ]
    for (let index = 1; index < ordered.length; index += 1) {
      const left = value(ordered[index - 1]!)
      const right = value(ordered[index]!)
      expect(decimalOrd.compare(left)(right).tag).toBe("Less")
      expect(decimalOrd.compare(right)(left).tag).toBe("Greater")
    }
  })
})
