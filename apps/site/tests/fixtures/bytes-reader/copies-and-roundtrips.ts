import assert from "node:assert/strict"
import * as bytes from "../../../../../runtime/ts/src/bytes.ts"
import * as base64 from "../../../../../runtime/ts/src/bytes-base64.ts"
import * as hex from "../../../../../runtime/ts/src/bytes-hex.ts"

const numbers = [0, 127, 255]
const made = bytes.fromInts(numbers)
assert.equal(made.tag, "Right")
if (made.tag !== "Right") throw new Error("Valid synthetic bytes rejected")
numbers[0] = 9
assert.deepEqual(bytes.toInts(made.value), [0, 127, 255])
const inspected = bytes.toInts(made.value)
Reflect.set(inspected, 1, 9)
assert.deepEqual(bytes.toInts(made.value), [0, 127, 255])
const hostInput = new Uint8Array([3, 4])
const imported = bytes.fromUint8Array(hostInput)
hostInput[0] = 9
assert.deepEqual(bytes.toInts(imported), [3, 4])
const hostOutput = bytes.toUint8Array(imported)
hostOutput[1] = 9
assert.deepEqual(bytes.toInts(imported), [3, 4])
console.log("copies: fromInts, toInts, host input, host output")

const all = bytes.fromInts(Array.from({ length: 256 }, (_, index) => index))
if (all.tag !== "Right") throw new Error("Valid byte range rejected")
assert.deepEqual(hex.decode(hex.encode(all.value)), all)
assert.deepEqual(hex.decode(hex.encode(all.value).toUpperCase()), all)
console.log("hex: every byte, lowercase and uppercase input")
for (let length = 0; length <= 64; length++) {
  const data = bytes.fromInts(
    Array.from({ length }, (_, i) => (i * 67 + length) & 255)
  )
  if (data.tag !== "Right") throw new Error("Valid synthetic bytes rejected")
  assert.deepEqual(base64.decode(base64.encode(data.value)), data)
  assert.deepEqual(base64.decodeUrl(base64.encodeUrl(data.value)), data)
  assert.equal(
    base64.encode(data.value),
    Uint8Array.from(bytes.toInts(data.value)).toBase64()
  )
  assert.equal(
    base64.encodeUrl(data.value),
    Uint8Array.from(bytes.toInts(data.value)).toBase64({
      alphabet: "base64url",
      omitPadding: true,
    })
  )
}
console.log(
  "base64: 65 deterministic lengths, both formats, native encoder parity"
)
for (const value of [-1, 256, 1.5, Number.NaN, Number.POSITIVE_INFINITY]) {
  assert.equal(bytes.byte(value).tag, "Left")
  assert.equal(bytes.fromInts([0, value, 255]).tag, "Left")
}
console.log("runtime numeric boundary: invalid host numbers rejected")
