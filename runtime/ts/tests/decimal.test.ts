import { describe, expect, test } from "bun:test"
import {
  add,
  context,
  type Decimal,
  type DecimalContext,
  DecimalDivisionByZero,
  DecimalNotIntegral,
  DecimalOutsideFloatRange,
  DecimalOutsideIntRange,
  decimalAdd,
  decimalEq,
  decimalHash,
  decimalMul,
  decimalOne,
  decimalOrd,
  decimalSub,
  decimalZero,
  divide,
  divideExact,
  FloatNotFinite,
  format,
  fromFloat,
  fromInt,
  multiply,
  NonTerminatingDecimal,
  parse,
  precision,
  quantize,
  rounding,
  subtract,
  toFloat,
  toIntExact,
} from "../src/decimal"
import {
  decimalJsonDecode,
  decimalJsonEncode,
  decodeString,
  encodeString,
} from "../src/json"
import {
  AwayFromZero,
  Ceiling,
  Floor,
  HalfEven,
  HalfUp,
  TowardZero,
} from "../src/number"

const value = (text: string): Decimal => {
  const result = parse(text)
  if (result.tag === "Left") throw new Error(`failed to parse ${text}`)
  return result.value
}

const decimalContext = (digits: number, mode = HalfEven): DecimalContext => {
  const result = context(digits, mode)
  if (result.tag === "Left") throw new Error("invalid context")
  return result.value
}

describe("exact Decimal representation", () => {
  test("parses the grammar and produces canonical non-exponent spelling", () => {
    for (const [source, expected] of [
      ["0", "0"],
      ["-0.000", "0"],
      ["+12.3400", "12.34"],
      ["1.25e3", "1250"],
      ["1.25E-3", "0.00125"],
      [
        "123456789012345678901234567890.0001",
        "123456789012345678901234567890.0001",
      ],
    ]) {
      expect(format(value(source))).toBe(expected)
    }
  })

  test("reports the first invalid UTF-8 byte offset", () => {
    for (const [source, offset] of [
      ["", 0],
      ["+", 1],
      ["01", 1],
      ["1.", 2],
      ["1e+", 3],
      ["12_3", 2],
      ["1.2猫", 3],
      [" NaN", 0],
    ] as const) {
      const result = parse(source)
      expect(result).toEqual({
        tag: "Left",
        value: { tag: "InvalidDecimal", value: { offset } },
      })
    }
  })

  test("adds, subtracts, and multiplies without rounding", () => {
    expect(format(add(value("0.1"), value("0.2")))).toBe("0.3")
    expect(format(subtract(value("1000"), value("0.001")))).toBe("999.999")
    expect(format(multiply(value("12.5"), value("0.08")))).toBe("1")
    expect(format(decimalAdd.add(value("1.2"))(value("3.4")))).toBe("4.6")
    expect(format(decimalSub.sub(value("1.2"))(value("3.4")))).toBe("-2.2")
    expect(format(decimalMul.mul(value("1.2"))(value("3")))).toBe("3.6")
  })

  test("performs exact division only for terminating decimal results", () => {
    expect(divideExact(value("8"), value("1"))).toEqual({
      tag: "Right",
      value: value("0.125"),
    })
    expect(divideExact(value("0.2"), value("1"))).toEqual({
      tag: "Right",
      value: value("5"),
    })
    expect(divideExact(value("3"), value("1"))).toEqual({
      tag: "Left",
      value: NonTerminatingDecimal,
    })
    expect(divideExact(value("0"), value("1"))).toEqual({
      tag: "Left",
      value: DecimalDivisionByZero,
    })
  })
})

describe("explicit Decimal rounding", () => {
  test("validates and exposes context values", () => {
    expect(context(0, HalfEven)).toEqual({
      tag: "Left",
      value: { tag: "NonPositiveDecimalPrecision", value: 0 },
    })
    const created = decimalContext(12, HalfUp)
    expect(precision(created)).toBe(12)
    expect(rounding(created)).toBe(HalfUp)
  })

  test("rounds division to significant precision", () => {
    expect(
      format(right(divide(decimalContext(2), value("8"), value("1"))))
    ).toBe("0.12")
    expect(
      format(right(divide(decimalContext(2, HalfUp), value("8"), value("1"))))
    ).toBe("0.13")
    expect(
      format(right(divide(decimalContext(3), value("3"), value("2"))))
    ).toBe("0.667")
    expect(
      format(right(divide(decimalContext(4), value("2"), value("1000"))))
    ).toBe("500")
  })

  test("implements all six quantize directions for both signs", () => {
    const positive = value("2.5")
    const negative = value("-2.5")
    expect(format(quantize(0, HalfEven, positive))).toBe("2")
    expect(format(quantize(0, HalfUp, positive))).toBe("3")
    expect(format(quantize(0, TowardZero, negative))).toBe("-2")
    expect(format(quantize(0, AwayFromZero, negative))).toBe("-3")
    expect(format(quantize(0, Floor, negative))).toBe("-3")
    expect(format(quantize(0, Ceiling, negative))).toBe("-2")
    expect(format(quantize(-2, HalfEven, value("1250")))).toBe("1200")
    expect(format(quantize(-2, HalfUp, value("1250")))).toBe("1300")
  })
})

describe("explicit Decimal conversions", () => {
  test("converts Int exactly and rejects non-integral or out-of-range values", () => {
    expect(format(fromInt(9_007_199_254_740_991))).toBe("9007199254740991")
    expect(toIntExact(value("42.0"))).toEqual({ tag: "Right", value: 42 })
    expect(toIntExact(value("0.1"))).toEqual({
      tag: "Left",
      value: DecimalNotIntegral,
    })
    expect(toIntExact(value("9007199254740992"))).toEqual({
      tag: "Left",
      value: DecimalOutsideIntRange,
    })
    expect(toIntExact(value("1e999999999999999999"))).toEqual({
      tag: "Left",
      value: DecimalOutsideIntRange,
    })
  })

  test("uses the exact binary64 value before applying Decimal context", () => {
    expect(format(right(fromFloat(decimalContext(17), 0.1)))).toBe(
      "0.10000000000000001"
    )
    expect(format(right(fromFloat(decimalContext(7), 0.1)))).toBe("0.1")
    expect(fromFloat(decimalContext(5), Number.NaN)).toEqual({
      tag: "Left",
      value: FloatNotFinite,
    })
    expect(fromFloat(decimalContext(5), Number.POSITIVE_INFINITY)).toEqual({
      tag: "Left",
      value: FloatNotFinite,
    })
  })

  test("rounds explicitly to binary64 and rejects overflow", () => {
    expect(toFloat(value("0.1"))).toEqual({ tag: "Right", value: 0.1 })
    const maxFloat = right(fromFloat(decimalContext(309), Number.MAX_VALUE))
    expect(toFloat(maxFloat)).toEqual({
      tag: "Right",
      value: Number.MAX_VALUE,
    })
    expect(toFloat(add(maxFloat, fromInt(1)))).toEqual({
      tag: "Left",
      value: DecimalOutsideFloatRange,
    })
    expect(toFloat(subtract(fromInt(-1), maxFloat))).toEqual({
      tag: "Left",
      value: DecimalOutsideFloatRange,
    })
    expect(toFloat(value("1e400"))).toEqual({
      tag: "Left",
      value: DecimalOutsideFloatRange,
    })
    expect(toFloat(value("1e999999999999999999"))).toEqual({
      tag: "Left",
      value: DecimalOutsideFloatRange,
    })
    expect(toFloat(value("-1e-999999999999999999"))).toEqual({
      tag: "Right",
      value: -0,
    })
  })
})

describe("Decimal standard dictionaries", () => {
  test("use canonical value equality, ordering, hashing, zero, and one", () => {
    const one = value("1.00")
    expect(decimalEq.eq(one)(value("1"))).toBe(true)
    expect(decimalOrd.compare(value("-10"))(value("-2"))).toEqual({
      tag: "Less",
    })
    expect(decimalOrd.compare(value("0.01"))(value("0.0101"))).toEqual({
      tag: "Less",
    })
    expect(decimalHash.hash(one)).toBe(decimalHash.hash(value("1e0")))
    expect(format(decimalZero.zero(undefined))).toBe("0")
    expect(format(decimalOne.one(undefined))).toBe("1")
  })

  test("round-trips exact JSON numbers without a JS number boundary", () => {
    const exact = value("12345678901234567890.000000000000000001")
    const encoded = encodeString(exact, decimalJsonEncode)
    expect(encoded).toBe("12345678901234567890.000000000000000001")
    const decoded = decodeString(encoded, decimalJsonDecode)
    expect(decoded).toEqual({ tag: "Right", value: exact })
  })
})

function right<E>(
  result: { tag: "Left"; value: E } | { tag: "Right"; value: Decimal }
): Decimal {
  if (result.tag === "Left")
    throw new Error(`unexpected ${JSON.stringify(result.value)}`)
  return result.value
}
