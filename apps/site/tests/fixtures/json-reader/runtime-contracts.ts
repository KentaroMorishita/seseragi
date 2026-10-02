import {
  array,
  type DecodeError,
  type Decoder,
  decodeString,
  encodeString,
  field,
  intJsonDecode,
  intJsonEncode,
  type Json,
  JsonArray,
  JsonNull,
  JsonString,
  map,
  maybeJsonDecode,
  optionalField,
  parse,
  record,
  stringify,
  stringJsonDecode,
} from "../../../../../runtime/ts/src/json"
import { Left, Right } from "../../../../../runtime/ts/src/sum"

let assertions = 0
const observations: Record<string, unknown> = {}
function check(condition: unknown, message: string): asserts condition {
  assertions++
  if (!condition) throw new Error(message)
}
function input(source: string): Json {
  const value = parse(source)
  if (value.tag === "Left") throw new Error(`Bad test source: ${source}`)
  return value.value
}
function failure(value: ReturnType<Decoder<unknown>>): DecodeError {
  check(value.tag === "Left", "expected failure")
  return value.value
}
const missing = failure(field("name", stringJsonDecode.decodeJson)(input("{}")))
const invalid = failure(
  field("name", stringJsonDecode.decodeJson)(input('{"name":false}'))
)
check(
  missing.path.length === 0 && missing.kind.tag === "MissingJsonField",
  "missing field path"
)
check(
  invalid.path.length === 1 && invalid.path[0]?.tag === "JsonField",
  "invalid field path"
)
observations.field = { missing, invalid }
let visits: string[] = []
const selectedFailure: DecodeError = {
  path: [],
  kind: { tag: "InvalidJsonValue", value: "stop" },
}
const decoder: Decoder<string> = (value) => {
  check(value.tag === "JsonString", "test element type")
  visits.push(value.value)
  return value.value === "stop" ? Left(selectedFailure) : Right(value.value)
}
const stopped = array(decoder)(
  JsonArray([JsonString("a"), JsonString("stop"), JsonString("later")])
)
check(
  JSON.stringify(visits) === '["a","stop"]',
  "array should stop calling children"
)
const stoppedError = failure(stopped)
check(
  stoppedError.path[0]?.tag === "JsonIndex" && stoppedError.path[0].value === 1,
  "array index"
)
visits = []
const empty = array(decoder)(JsonArray([]))
check(
  empty.tag === "Right" && empty.value.length === 0 && visits.length === 0,
  "empty array"
)
observations.array = { stopped, visitsAfterEmpty: visits }
let recordCalls: string[] = []
const declared = record([
  [
    "first",
    (value) => {
      recordCalls.push("first")
      return stringJsonDecode.decodeJson(value)
    },
  ],
  [
    "second",
    (value) => {
      recordCalls.push("second")
      return stringJsonDecode.decodeJson(value)
    },
  ],
])
const unknown = declared(input('{"first":false,"extra":"x"}'))
check(
  recordCalls.length === 0,
  "unknown fields must be rejected before callbacks"
)
check(failure(unknown).kind.tag === "UnknownJsonField", "unknown priority")
const ordered = declared(input('{"second":"B","first":"A"}'))
check(
  JSON.stringify(recordCalls) === '["first","second"]',
  "record declaration callback order"
)
check(
  ordered.tag === "Right" &&
    JSON.stringify(ordered.value) === '[["first","A"],["second","B"]]',
  "record pair order"
)
recordCalls = []
const badFirst = declared(input('{"first":false,"second":"B"}'))
check(
  JSON.stringify(recordCalls) === '["first"]',
  "record first failure should stop later callbacks"
)
observations.record = {
  unknown,
  ordered,
  badFirst,
  callsAfterBadFirst: recordCalls,
}
let optionalCalls = 0
const optional = optionalField("name", (value) => {
  optionalCalls++
  return stringJsonDecode.decodeJson(value)
})
const absent = optional(input("{}"))
check(
  absent.tag === "Right" &&
    absent.value.tag === "Nothing" &&
    optionalCalls === 0,
  "missing skips decoder"
)
const nullString = optional(input('{"name":null}'))
check(
  nullString.tag === "Left" && Number(optionalCalls) === 1,
  "present null reaches decoder"
)
const nullable = optionalField(
  "name",
  maybeJsonDecode(stringJsonDecode).decodeJson
)(input('{"name":null}'))
check(
  nullable.tag === "Right" &&
    nullable.value.tag === "Just" &&
    nullable.value.value.tag === "Nothing",
  "present null differs from absence"
)
observations.optional = { absent, nullString, nullable, optionalCalls }
let transforms = 0
const greeting = map((value) => {
  transforms++
  return `Hello, ${value}`
}, stringJsonDecode.decodeJson)
const badGreeting = greeting(JsonNull)
check(
  badGreeting.tag === "Left" && transforms === 0,
  "failed decode skips transform"
)
const goodGreeting = greeting(JsonString("Mio"))
check(
  goodGreeting.tag === "Right" &&
    goodGreeting.value === "Hello, Mio" &&
    Number(transforms) === 1,
  "successful transform once"
)
const sentinel = new Error("transform defect")
let defect: unknown
try {
  map(() => {
    throw sentinel
  }, stringJsonDecode.decodeJson)(JsonString("Mio"))
} catch (error) {
  defect = error
}
check(
  defect === sentinel,
  "map does not wrap a thrown transform defect as DecodeError"
)
observations.map = {
  badGreeting,
  goodGreeting,
  transforms,
  defectPropagated: defect === sentinel,
}
const lexicalFraction = decodeString("1.0000000000000001", intJsonDecode)
check(
  lexicalFraction.tag === "Left",
  "exact fractional JSON must fail Int decoding"
)
check(
  Number.isSafeInteger(JSON.parse("1.0000000000000001")),
  "native JSON has already rounded this value before the guard"
)
const nonfinite: unknown[] = []
for (const value of [
  Number.NaN,
  Number.POSITIVE_INFINITY,
  Number.NEGATIVE_INFINITY,
]) {
  let error: unknown
  try {
    encodeString(value, intJsonEncode)
  } catch (caught) {
    error = caught
  }
  check(
    error instanceof Error,
    "invalid runtime Int must not encode nonfinite number"
  )
  nonfinite.push({
    nativeStringify: JSON.stringify(value),
    runtimeIntEncoderThrows: true,
  })
}
observations.numeric = {
  lexicalFraction,
  nativeParsedLexicalFraction: JSON.parse("1.0000000000000001"),
  nonfinite,
}
const duplicate = parse('{"outer":{"same":1,"same":2}}')
check(
  duplicate.tag === "Left" && duplicate.value.tag === "DuplicateJsonField",
  "duplicate keys are rejected"
)
if (duplicate.tag === "Left" && duplicate.value.tag === "DuplicateJsonField") {
  check(
    duplicate.value.value.field === "same" &&
      duplicate.value.value.path[0]?.tag === "JsonField" &&
      duplicate.value.value.path[0].value === "outer",
    "duplicate has containing object path"
  )
}
const offset = parse('"🌊" nope')
check(
  offset.tag === "Left" &&
    offset.value.tag === "InvalidJsonSyntax" &&
    offset.value.value.offset === 7,
  "parse offset counts UTF-8 bytes"
)
const exact = input("9007199254740993")
check(
  stringify(exact) === "9007199254740993",
  "parsed number keeps exact decimal digits"
)
check(
  JSON.stringify(JSON.parse("9007199254740993")) === "9007199254740992",
  "native JSON number differs for this fixture"
)
const orderedKeys = '{"2":"two","1":"one"}'
check(
  stringify(input(orderedKeys)) === orderedKeys,
  "JSON Map keeps input key order"
)
check(
  JSON.stringify(JSON.parse(orderedKeys)) === '{"1":"one","2":"two"}',
  "native JS integer keys have their own ordering"
)
observations.parseDifferences = {
  duplicate,
  offset,
  exact: stringify(exact),
  keyOrder: stringify(input(orderedKeys)),
}
console.log(JSON.stringify({ assertions, observations }, null, 2))
