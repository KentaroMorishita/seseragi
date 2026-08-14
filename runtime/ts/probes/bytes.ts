import {
  append,
  byte,
  concat,
  copy,
  empty,
  fromArray,
  fromInts,
  fromUint8Array,
  get,
  isEmpty,
  length,
  singleton,
  slice,
  toArray,
  toInt,
  toInts,
  toUint8Array,
} from "../src/bytes"
import { decodeUtf8, decodeUtf8Lossy, encodeUtf8 } from "../src/text"

function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const zero = byte(0)
const maximum = byte(255)
require(zero.tag === "Right" && toInt(zero.value) === 0, "Byte 0 failed")
require(
  maximum.tag === "Right" && toInt(maximum.value) === 255,
  "Byte 255 failed"
)
for (const value of [-1, 256]) {
  const result = byte(value)
  require(
    result.tag === "Left" &&
      result.value.tag === "ByteOutOfRange" &&
      result.value.value === value,
    `Byte boundary ${value} was accepted`
  )
}

const invalid = fromInts([1, 256, -1])
require(
  invalid.tag === "Left" && invalid.value.value === 256,
  "fromInts did not report the first invalid Byte"
)
const built = fromInts([1, 2, 255])
require(built.tag === "Right", "fromInts rejected valid Bytes")
const values = built.value
require(length(values) === 3 && !isEmpty(values), "Bytes size failed")
require(isEmpty(empty(undefined)), "empty Bytes failed")
require(
  maximum.tag === "Right" && toInts(singleton(maximum.value))[0] === 255,
  "singleton failed"
)
require(
  JSON.stringify(toArray(values).map(toInt)) === "[1,2,255]",
  "Byte Array conversion failed"
)
require(get(-1, values).tag === "Nothing", "negative get succeeded")
require(get(3, values).tag === "Nothing", "out-of-range get succeeded")
const observed = get(1, values)
require(
  observed.tag === "Just" && toInt(observed.value) === 2,
  "valid get failed"
)

const validSlice = slice(1, 3, values)
require(
  validSlice.tag === "Right" &&
    JSON.stringify(toInts(validSlice.value)) === "[2,255]",
  "valid slice failed"
)
const invalidSlice = slice(2, 1, values)
require(
  invalidSlice.tag === "Left" &&
    invalidSlice.value.value.start === 2 &&
    invalidSlice.value.value.end === 1 &&
    invalidSlice.value.value.length === 3,
  "invalid slice payload failed"
)
require(
  JSON.stringify(toInts(append(values, values))) === "[1,2,255,1,2,255]",
  "append order failed"
)
require(
  JSON.stringify(toInts(concat([values, copy(values)]))) ===
    "[1,2,255,1,2,255]",
  "concat order failed"
)

for (const text of ["ASCII", "せせらぎ", "emoji 🌊"]) {
  const decoded = decodeUtf8(encodeUtf8(text))
  require(
    decoded.tag === "Right" && decoded.value === text,
    `UTF-8 round-trip failed for ${text}`
  )
}
const invalidUtf8 = fromUint8Array(new Uint8Array([0x61, 0xe2, 0x28, 0xa1]))
const strict = decodeUtf8(invalidUtf8)
require(
  strict.tag === "Left" && strict.value.value.offset === 1,
  "strict UTF-8 offset failed"
)
require(
  decodeUtf8Lossy(invalidUtf8) === "a�(�",
  "lossy UTF-8 replacement failed"
)

const hostInput = new Uint8Array([4, 5, 6])
const immutable = fromUint8Array(hostInput)
hostInput[0] = 9
require(toInts(immutable)[0] === 4, "host input mutation aliased Bytes")
const hostOutput = toUint8Array(immutable)
hostOutput[1] = 9
require(toInts(immutable)[1] === 5, "host output mutation aliased Bytes")
const byteArray = [zero.value]
const copiedArray = fromArray(byteArray)
byteArray[0] = maximum.value
require(toInts(copiedArray)[0] === 0, "Array mutation aliased Bytes")

console.log("bytes runtime probe passed")
