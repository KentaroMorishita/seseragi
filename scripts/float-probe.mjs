import assert from "node:assert/strict"

const module = await import(process.argv[2])
const view = new DataView(new ArrayBuffer(8))
let state = 0x5eed6123n
let digest = 0xcbf29ce484222325n
let formatCases = 0
function check(bits) {
  view.setBigUint64(0, bits)
  const value = view.getFloat64(0)
  const actual = module.display(value)
  let expected = Number.isNaN(value)
    ? "NaN"
    : Object.is(value, -0)
      ? "-0.0"
      : value.toString().replace("e+", "e")
  if (!/[.e]|Infinity|NaN/.test(expected)) expected += ".0"
  assert.equal(actual, expected, `binary64 0x${bits.toString(16)}`)
  for (const byte of new TextEncoder().encode(`${actual}\0`)) {
    digest = BigInt.asUintN(64, (digest ^ BigInt(byte)) * 0x100000001b3n)
  }
  formatCases++
}
for (let i = 0; i < 100000; i++) {
  state = BigInt.asUintN(64, state ^ (state << 13n))
  state ^= state >> 7n
  state = BigInt.asUintN(64, state ^ (state << 17n))
  check(state)
}
for (let exponent = -1074; exponent <= 1023; exponent++) {
  view.setFloat64(0, 2 ** exponent)
  const bits = view.getBigUint64(0)
  check(bits)
  check(bits + 1n)
  check(bits - 1n)
}
assert.equal(formatCases, 106294)
console.log(JSON.stringify({ formatCases, digest: digest.toString(16) }))
