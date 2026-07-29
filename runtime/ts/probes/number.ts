import {
  MAX_INT,
  MIN_INT,
  checkedAdd,
  checkedDivide,
  checkedMultiply,
  checkedPower,
  checkedRemainder,
  checkedSubtract,
  format,
  formatRadix,
  parse,
  parseRadix,
  saturatingAdd,
  saturatingMultiply,
  saturatingPower,
  saturatingSubtract,
} from "../src/int"
import {
  fromInt,
  parse as parseFloat,
  isNegativeZero,
  roundIntegral,
  toInt,
  totalCompare,
} from "../src/float"
import { HalfEven, HalfUp, TowardZero } from "../src/number"

function assert(condition: boolean, message: string): void {
  if (!condition) throw new Error(message)
}

function assertTag(value: Readonly<{ readonly tag: string }>, tag: string): void {
  assert(value.tag === tag, `expected ${tag}, got ${value.tag}`)
}

assertTag(parse(""), "Left")
assertTag(parse("9007199254740992"), "Left")
assertTag(parseRadix(1, "0"), "Left")
assertTag(parseRadix(16, "7fffffffffffffff"), "Left")
assertTag(parse("9007199254740991"), "Right")
assertTag(parse("-9007199254740991"), "Right")
const negativeZero = parse("-0")
assertTag(negativeZero, "Right")
assert(
  negativeZero.tag === "Right" && !Object.is(negativeZero.value, -0),
  "Int parse must normalize negative zero"
)
assert(format(MIN_INT) === "-9007199254740991", "Int minimum format")
const hex = formatRadix(16, MAX_INT)
assert(hex.tag === "Right" && hex.value === "1fffffffffffff", "radix format")

assertTag(checkedAdd(1, MAX_INT), "Nothing")
assertTag(checkedSubtract(1, MIN_INT), "Nothing")
assertTag(checkedMultiply(2, MAX_INT), "Nothing")
assertTag(checkedDivide(0, 1), "Left")
assertTag(checkedRemainder(0, 1), "Left")
assertTag(checkedPower(-1, 2), "Left")
assertTag(checkedPower(53, 2), "Left")
assert(saturatingAdd(1, MAX_INT) === MAX_INT, "saturating add")
assert(saturatingSubtract(1, MIN_INT) === MIN_INT, "saturating subtract")
assert(saturatingMultiply(2, MIN_INT) === MIN_INT, "saturating multiply")
const saturatedPower = saturatingPower(53, -2)
assert(
  saturatedPower.tag === "Right" && saturatedPower.value === MIN_INT,
  "saturating odd negative power"
)

assert(fromInt(MAX_INT) === MAX_INT, "every Int must convert exactly to Float")
const even = toInt(HalfEven, 2.5)
assert(even.tag === "Right" && even.value === 2, "half-even tie")
const halfUp = toInt(HalfUp, -1.5)
assert(halfUp.tag === "Right" && halfUp.value === -2, "half-up tie")
assertTag(toInt(TowardZero, Number.POSITIVE_INFINITY), "Left")
assertTag(toInt(TowardZero, MAX_INT + 1), "Left")
const floatNegativeZero = toInt(TowardZero, -0.25)
assert(
  floatNegativeZero.tag === "Right" &&
    floatNegativeZero.value === 0 &&
    !Object.is(floatNegativeZero.value, -0),
  "Float to Int must normalize negative zero"
)
assert(roundIntegral(HalfEven, 3.5) === 4, "half-even odd tie")
assert(isNegativeZero(roundIntegral(TowardZero, -0.25)), "Float keeps signed zero")
assert(isNegativeZero(roundIntegral(HalfEven, -0.5)), "half-even keeps negative zero")
const invalidExponent = parseFloat("1e+")
assert(
  invalidExponent.tag === "Left" &&
    invalidExponent.value.tag === "InvalidFloat" &&
    invalidExponent.value.value.offset === 3,
  "Float parse reports the first invalid UTF-8 offset"
)
assert(totalCompare(-0, 0).tag === "Less", "negative zero total order")
assert(
  totalCompare(Number.NaN, Number.POSITIVE_INFINITY).tag === "Greater",
  "canonical NaN total order"
)

process.stdout.write("numeric runtime probe passed\n")
