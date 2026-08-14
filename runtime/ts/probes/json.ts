import {
  arrayJsonDecode,
  arrayJsonEncode,
  boolJsonDecode,
  decimalFromCanonical,
  decodeString,
  eitherJsonDecode,
  eitherJsonEncode,
  encodeString,
  field,
  intJsonDecode,
  intJsonEncode,
  JsonArray,
  JsonBool,
  JsonNull,
  JsonNumber,
  JsonObject,
  JsonString,
  listJsonDecode,
  listJsonEncode,
  optionalField,
  parse,
  record,
  recordJsonDecode,
  recordJsonEncode,
  stringify,
  stringJsonDecode,
  tupleJsonDecode,
  tupleJsonEncode,
} from "../src/json"
import { fromArray as listFromArray, toArray as listToArray } from "../src/list"
import { Just, Left, Nothing, Right } from "../src/sum"

function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

function roundTrip(source: string, expected: string): void {
  const parsed = parse(source)
  require(parsed.tag === "Right", `parse failed for ${source}`)
  require(stringify(parsed.value) ===
    expected, `expected ${expected}, got ${stringify(parsed.value)}`)
}

roundTrip(
  ' { "first": [null, true, false], "number": -12.3400e+2 } ',
  '{"first":[null,true,false],"number":-1234}'
)
for (const [source, expected] of [
  ["0", "0"],
  ["-0", "0"],
  ["1.00", "1"],
  ["1e3", "1000"],
  ["1e-3", "0.001"],
  ["123.45e-1", "12.345"],
] as const) {
  roundTrip(source, expected)
}

roundTrip(
  '"quote: \\" slash: / backslash: \\\\ unicode: せせらぎ \\uD83C\\uDF0A"',
  '"quote: \\" slash: / backslash: \\\\ unicode: せせらぎ 🌊"'
)
roundTrip('"\\b\\t\\n\\f\\r\\u0000\\u001F"', '"\\b\\t\\n\\f\\r\\u0000\\u001F"')

for (const source of [
  "01",
  "1.",
  "1e",
  "[1,]",
  '{"x":1,}',
  "NaN",
  "Infinity",
  "/* comment */ null",
  '"\\uD800"',
]) {
  const result = parse(source)
  require(result.tag === "Left", `invalid JSON was accepted: ${source}`)
}

const offset = parse('"🌊" nope')
require(offset.tag === "Left" &&
  offset.value.tag === "InvalidJsonSyntax" &&
  offset.value.value.offset === 7, "UTF-8 byte offset was not reported")
const duplicate = parse('{"outer":{"same":1,"same":2}}')
require(duplicate.tag === "Left" &&
  duplicate.value.tag === "DuplicateJsonField" &&
  duplicate.value.value.field === "same" &&
  duplicate.value.value.path.length === 1 &&
  duplicate.value.value.path[0]?.tag === "JsonField" &&
  duplicate.value.value.path[0].value ===
    "outer", "duplicate field path was not preserved")

const nested = field(
  "items",
  arrayJsonDecode(intJsonDecode).decodeJson
)(
  JsonObject([
    [
      "items",
      JsonArray([JsonNumber(decimalFromCanonical("1")), JsonString("bad")]),
    ],
  ])
)
require(nested.tag === "Left" &&
  nested.value.path[0]?.tag === "JsonField" &&
  nested.value.path[0].value === "items" &&
  nested.value.path[1]?.tag === "JsonIndex" &&
  nested.value.path[1].value ===
    1, "decoder path did not preserve field/index order")

require(optionalField("missing", stringJsonDecode.decodeJson)(JsonObject([]))
  .tag === "Right", "optional field failed")
const explicitNull = optionalField(
  "value",
  stringJsonDecode.decodeJson
)(JsonObject([["value", JsonNull]]))
require(explicitNull.tag === "Left", "explicit null was treated as absence")
const homogeneousRecord = record([
  ["first", stringJsonDecode.decodeJson],
  ["second", stringJsonDecode.decodeJson],
])(
  JsonObject([
    ["first", JsonString("a")],
    ["second", JsonString("b")],
  ])
)
require(homogeneousRecord.tag === "Right" &&
  homogeneousRecord.value[1]?.[1] === "b", "homogeneous record decoder failed")

require(encodeString(42, intJsonEncode) === "42", "Int encodeString failed")
const decodedInt = decodeString("42", intJsonDecode)
require(decodedInt.tag === "Right" &&
  decodedInt.value === 42, "Int decodeString failed")
for (const boundary of ["-9007199254740991", "9007199254740991"]) {
  require(decodeString(boundary, intJsonDecode).tag ===
    "Right", `safe Int boundary was rejected: ${boundary}`)
}
for (const source of ["1.5", "9007199254740992"]) {
  require(decodeString(source, intJsonDecode).tag ===
    "Left", `invalid Int decoded: ${source}`)
}
const malformedRead = decodeString("[", intJsonDecode)
require(malformedRead.tag === "Left" &&
  malformedRead.value.tag ===
    "JsonSyntaxFailure", "malformed JSON was not a syntax failure")
const mismatchedRead = decodeString('"not an int"', intJsonDecode)
require(mismatchedRead.tag === "Left" &&
  mismatchedRead.value.tag ===
    "JsonDecodeFailure", "wrong JSON value was not a decode failure")

const maybeDictionary = {
  encodeJson: (value: typeof Nothing | ReturnType<typeof Just<string>>) =>
    value.tag === "Nothing" ? JsonNull : JsonString(value.value),
}
require(encodeString(Nothing, maybeDictionary) ===
  "null", "Nothing encoding failed")
require(encodeString(Just("yes"), maybeDictionary) ===
  '"yes"', "Just encoding failed")

const actualEitherEncode = eitherJsonEncode(
  { encodeJson: JsonString },
  intJsonEncode
)
require(encodeString(Left("bad"), actualEitherEncode) ===
  '{"tag":"Left","value":"bad"}', "Left encoding failed")
require(encodeString(Right(7), actualEitherEncode) ===
  '{"tag":"Right","value":7}', "Right encoding failed")
const eitherDecode = eitherJsonDecode(stringJsonDecode, intJsonDecode)
const decodedRight = decodeString('{"tag":"Right","value":7}', eitherDecode)
require(decodedRight.tag === "Right" &&
  decodedRight.value.tag === "Right", "Either decoding failed")

const arrayCodec = [
  arrayJsonEncode(intJsonEncode),
  arrayJsonDecode(intJsonDecode),
] as const
require(encodeString([1, 2, 3], arrayCodec[0]) ===
  "[1,2,3]", "Array encoding failed")
const decodedArray = decodeString("[1,2,3]", arrayCodec[1])
require(decodedArray.tag === "Right" &&
  decodedArray.value[2] === 3, "Array decoding failed")

const listEncode = listJsonEncode(intJsonEncode)
const listDecode = listJsonDecode(intJsonDecode)
require(encodeString(listFromArray([1, 2]), listEncode) ===
  "[1,2]", "List encoding failed")
const decodedList = decodeString("[1,2]", listDecode)
require(decodedList.tag === "Right" &&
  listToArray(decodedList.value).join(",") === "1,2", "List decoding failed")

const actualTupleEncode = tupleJsonEncode(
  { encodeJson: JsonString },
  intJsonEncode
)
require(encodeString(["x", 1], actualTupleEncode) ===
  '["x",1]', "tuple encoding failed")
const tupleDecode = tupleJsonDecode(stringJsonDecode, intJsonDecode)
require(decodeString('["x",1]', tupleDecode).tag ===
  "Right", "tuple decoding failed")

const actualRecordEncode = recordJsonEncode(
  ["a", "optional", "z"],
  [false, true, false],
  intJsonEncode,
  { encodeJson: JsonString },
  { encodeJson: JsonBool }
)
require(encodeString({ z: true, a: 1 }, actualRecordEncode) ===
  '{"a":1,"z":true}', "record order/optional encoding failed")
const recordDecode = recordJsonDecode(
  ["a", "optional", "z"],
  [false, true, false],
  intJsonDecode,
  stringJsonDecode,
  boolJsonDecode
)
require(decodeString('{"a":1,"z":true}', recordDecode).tag ===
  "Right", "record optional decoding failed")
require(decodeString('{"a":1,"z":true,"unknown":0}', recordDecode).tag ===
  "Left", "unknown record field was accepted")

console.log("json runtime probe passed")
