import { describe, expect, test } from "bun:test"
import {
  abs,
  add,
  BigIntDivisionByZero,
  BigIntOutsideIntRange,
  bigIntAdd,
  bigIntConversionErrorEq,
  bigIntDivisionErrorEq,
  bigIntEq,
  bigIntHash,
  bigIntOrd,
  bigIntParseErrorEq,
  bigIntPowerErrorEq,
  checkedDivide,
  checkedPower,
  checkedRemainder,
  divide,
  EmptyBigInt,
  format,
  formatRadix,
  fromInt,
  InvalidBigIntDigit,
  InvalidBigIntRadix,
  multiply,
  NegativeBigIntExponent,
  parse,
  parseRadix,
  power,
  remainder,
  sign,
  subtract,
  toInt,
} from "../src/big-int"

const expectRight = <Value>(result: {
  readonly tag: string
  readonly value?: Value
}): Value => {
  expect(result.tag).toBe("Right")
  return result.value as Value
}

describe("BigInt runtime", () => {
  test("parses and formats arbitrary-precision values without number loss", () => {
    const decimal = "12345678901234567890123456789012345678901234567890"
    const value = expectRight(parse(decimal))
    expect(format(value)).toBe(decimal)
    expect(format(expectRight(parse("-0")))).toBe("0")
    expect(format(expectRight(parse("+42")))).toBe("42")
    expect(formatRadix(16, value)).toEqual({
      tag: "Right",
      value: value.toString(16),
    })
    expect(parseRadix(16, "000FF")).toEqual({
      tag: "Right",
      value: 255n,
    })
  })

  test("reports canonical syntax, radix, and UTF-8 byte offsets", () => {
    expect(parse("")).toEqual({ tag: "Left", value: EmptyBigInt })
    expect(parse("01")).toEqual({
      tag: "Left",
      value: InvalidBigIntDigit({ offset: 1, radix: 10 }),
    })
    expect(parse("12é")).toEqual({
      tag: "Left",
      value: InvalidBigIntDigit({ offset: 2, radix: 10 }),
    })
    expect(parseRadix(1, "0")).toEqual({
      tag: "Left",
      value: InvalidBigIntRadix(1),
    })
    expect(formatRadix(37, fromInt(1))).toEqual({
      tag: "Left",
      value: InvalidBigIntRadix(37),
    })
  })

  test("keeps exact arithmetic, truncating division, and remainder laws", () => {
    const huge = expectRight(parse("999999999999999999999999999999999999"))
    expect(format(add(huge, fromInt(1)))).toBe(
      "1000000000000000000000000000000000000"
    )
    expect(format(subtract(fromInt(2), fromInt(5)))).toBe("-3")
    expect(format(multiply(huge, fromInt(3)))).toBe(
      "2999999999999999999999999999999999997"
    )
    for (const [dividend, divisor, quotient, rest] of [
      [-17, 5, -3, -2],
      [17, -5, -3, 2],
      [-17, -5, 3, -2],
    ] as const) {
      expect(format(divide(fromInt(dividend), fromInt(divisor)))).toBe(
        String(quotient)
      )
      expect(format(remainder(fromInt(dividend), fromInt(divisor)))).toBe(
        String(rest)
      )
    }
    expect(() => divide(fromInt(1), fromInt(0))).toThrow("division by zero")
    expect(() => remainder(fromInt(1), fromInt(0))).toThrow("remainder by zero")
  })

  test("uses exponentiation by squaring semantics and typed checked failures", () => {
    expect(format(power(fromInt(2), 100))).toBe(
      "1267650600228229401496703205376"
    )
    expect(format(power(fromInt(0), 0))).toBe("1")
    expect(() => power(fromInt(2), -1)).toThrow("negative exponent")
    expect(checkedPower(-7, fromInt(2))).toEqual({
      tag: "Left",
      value: NegativeBigIntExponent(-7),
    })
    expect(checkedDivide(fromInt(0), fromInt(1))).toEqual({
      tag: "Left",
      value: BigIntDivisionByZero,
    })
    expect(checkedRemainder(fromInt(0), fromInt(1))).toEqual({
      tag: "Left",
      value: BigIntDivisionByZero,
    })
  })

  test("converts Int explicitly and rejects only the narrowing boundary", () => {
    expect(toInt(fromInt(Number.MAX_SAFE_INTEGER))).toEqual({
      tag: "Right",
      value: Number.MAX_SAFE_INTEGER,
    })
    expect(toInt(expectRight(parse("9007199254740992")))).toEqual({
      tag: "Left",
      value: BigIntOutsideIntRange,
    })
    expect(format(abs(expectRight(parse("-999999999999999999"))))).toBe(
      "999999999999999999"
    )
    expect(sign(fromInt(-1))).toBe(-1)
    expect(sign(fromInt(0))).toBe(0)
    expect(sign(fromInt(1))).toBe(1)
  })

  test("provides coherent value, arithmetic, and error dictionaries", () => {
    const two = fromInt(2)
    const three = fromInt(3)
    expect(bigIntEq.eq(two)(fromInt(2))).toBe(true)
    expect(bigIntOrd.compare(two)(three).tag).toBe("Less")
    expect(bigIntHash.hash(two)).toBe(bigIntHash.hash(fromInt(2)))
    expect(format(bigIntAdd.add(two)(three))).toBe("5")
    expect(bigIntParseErrorEq.eq(EmptyBigInt)(EmptyBigInt)).toBe(true)
    expect(
      bigIntParseErrorEq.eq(InvalidBigIntRadix(1))(InvalidBigIntRadix(1))
    ).toBe(true)
    expect(
      bigIntDivisionErrorEq.eq(BigIntDivisionByZero)(BigIntDivisionByZero)
    ).toBe(true)
    expect(
      bigIntPowerErrorEq.eq(NegativeBigIntExponent(-1))(
        NegativeBigIntExponent(-1)
      )
    ).toBe(true)
    expect(
      bigIntConversionErrorEq.eq(BigIntOutsideIntRange)(BigIntOutsideIntRange)
    ).toBe(true)
  })
})
