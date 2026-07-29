import {
  fromInt,
  isNegativeZero,
  parse as parseFloatValue,
  roundIntegral,
  toInt,
  totalCompare,
} from "../src/float"
import {
  checkedAdd,
  checkedDivide,
  checkedMultiply,
  checkedPower,
  checkedRemainder,
  checkedSubtract,
  decodeForeignInt,
  decodeJsonInt,
  encodeForeignInt,
  encodeJsonInt,
  format,
  formatRadix,
  MAX_INT,
  MIN_INT,
  parse,
  parseRadix,
  saturatingAdd,
  saturatingMultiply,
  saturatingPower,
  saturatingSubtract,
} from "../src/int"
import { HalfEven, HalfUp, TowardZero } from "../src/number"

function assert(condition: boolean, message: string): void {
  if (!condition) throw new Error(message)
}

function assertTag(
  value: Readonly<{ readonly tag: string }>,
  tag: string
): void {
  assert(value.tag === tag, `expected ${tag}, got ${value.tag}`)
}

function assertThrows(
  operation: () => unknown,
  errorType: typeof TypeError | typeof RangeError,
  message: string
): void {
  try {
    operation()
  } catch (error) {
    assert(error instanceof errorType, message)
    return
  }
  throw new Error(message)
}

for (const decode of [decodeForeignInt, decodeJsonInt]) {
  assert(decode(MAX_INT) === MAX_INT, "boundary accepts maximum Int")
  const decodedZero = decode(-0)
  assert(
    decodedZero === 0 && !Object.is(decodedZero, -0),
    "boundary normalizes negative zero"
  )
  assertThrows(() => decode("42"), TypeError, "boundary rejects non-number")
  assertThrows(() => decode(Number.NaN), RangeError, "boundary rejects NaN")
  assertThrows(
    () => decode(Number.POSITIVE_INFINITY),
    RangeError,
    "boundary rejects infinity"
  )
  assertThrows(() => decode(1.5), RangeError, "boundary rejects fraction")
  assertThrows(
    () => decode(MAX_INT + 1),
    RangeError,
    "boundary rejects unsafe integer"
  )
}
assert(encodeForeignInt(MIN_INT) === MIN_INT, "foreign encode uses number")
assert(encodeJsonInt(MAX_INT) === MAX_INT, "JSON encode uses number")
assertThrows(
  () => encodeForeignInt(MAX_INT + 1),
  RangeError,
  "foreign encode validates Int"
)
assertThrows(
  () => encodeJsonInt(Number.NaN),
  RangeError,
  "JSON encode validates Int"
)

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
assert(
  isNegativeZero(roundIntegral(TowardZero, -0.25)),
  "Float keeps signed zero"
)
assert(
  isNegativeZero(roundIntegral(HalfEven, -0.5)),
  "half-even keeps negative zero"
)
const invalidExponent = parseFloatValue("1e+")
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
