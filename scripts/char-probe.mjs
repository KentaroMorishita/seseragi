import assert from "node:assert/strict"

const module = await import(process.argv[2])
let validScalars = 0
let checksum = 0
for (let value = 0; value <= 0x110000; value++) {
  const valid = value <= 0x10ffff && !(value >= 0xd800 && value <= 0xdfff)
  const expected = valid ? value : -1
  const actual = module.pointRoundtrip(value)
  if (actual !== expected) {
    assert.fail(`codePoint roundtrip at U+${value.toString(16)}`)
  }
  if (valid) validScalars++
  checksum += actual
}
assert.equal(validScalars, 1112064)
for (const value of [-9007199254740991, -1, 9007199254740991]) {
  assert.equal(module.pointRoundtrip(value), -1)
}

const cases = [
  [0, "'\\0'"],
  [8, "'\\b'"],
  [9, "'\\t'"],
  [10, "'\\n'"],
  [12, "'\\f'"],
  [13, "'\\r'"],
  [0x1f, "'\\u{1F}'"],
  [0x27, "'\\''"],
  [0x5c, "'\\\\'"],
  [0x7f, "'\\u{7F}'"],
  [0x9f, "'\\u{9F}'"],
  [0x301, "'\u0301'"],
  [0x3bb, "'λ'"],
  [0xe000, "'\ue000'"],
  [0xffff, "'\uffff'"],
  [0x10000, "'\u{10000}'"],
  [0x1f642, "'🙂'"],
  [0x10ffff, "'\u{10ffff}'"],
]
for (const [point, debug] of cases) {
  const left = String.fromCodePoint(point)
  assert.equal(module.displayChar(left), debug)
  for (const [other] of cases) {
    const right = String.fromCodePoint(other)
    assert.equal(module.scalarSame(left)(right), point === other)
    assert.equal(module.scalarBefore(left)(right), point < other)
  }
}
const probe = module.probe(undefined)
assert.equal(probe.length, 45)
assert.deepEqual(probe.slice(0, 4), ["0", "955", "128578", "1114111"])
assert.deepEqual(probe.slice(-2), ["'\u0301'", "'🙂'"])
console.log(
  JSON.stringify({
    unicodeCodePoints: 1114113,
    validScalars,
    checksum,
    comparisonPairs: cases.length ** 2,
    probe,
  })
)
