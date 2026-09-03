import { afterEach, expect, test } from "bun:test"
import { arrayIterable } from "../src/array"
import { intEq, stringEq } from "../src/equality"
import { intHash, resetProcessHashSeedForTest, stringHash } from "../src/hash"
import * as json from "../src/json"
import * as maps from "../src/map"
import * as sets from "../src/set"
import * as display from "../src/show"

const host = globalThis as typeof globalThis & {
  __SESERAGI_HASH_SEED__?: number
}
const savedSeed = host.__SESERAGI_HASH_SEED__
afterEach(() => {
  if (savedSeed === undefined) delete host.__SESERAGI_HASH_SEED__
  else host.__SESERAGI_HASH_SEED__ = savedSeed
  resetProcessHashSeedForTest()
})

const mapEncode = json.mapJsonEncode(json.stringJsonEncode, json.intJsonEncode)
const mapDecode = json.mapJsonDecode(
  stringEq,
  stringHash,
  json.stringJsonDecode,
  json.intJsonDecode
)
const setEncode = json.setJsonEncode(json.intJsonEncode)
const setDecode = json.setJsonDecode(intEq, intHash, json.intJsonDecode)

test("Map JSON preserves first position and last value; Set preserves first value", () => {
  const decoded = json.decodeString('[["b",2],["a",1],["b",5]]', mapDecode)
  expect(decoded.tag).toBe("Right")
  if (decoded.tag !== "Right") throw new Error("expected Map")
  expect(maps.entries(decoded.value)).toEqual([
    ["b", 5],
    ["a", 1],
  ])
  expect(json.encodeString(decoded.value, mapEncode)).toBe('[["b",5],["a",1]]')
  const set = json.decodeString("[3,1,3,2]", setDecode)
  expect(set.tag).toBe("Right")
  if (set.tag !== "Right") throw new Error("expected Set")
  expect(json.encodeString(set.value, setEncode)).toBe("[3,1,2]")
  expect(json.encodeString(maps.empty<string, number>(), mapEncode)).toBe("[]")
  expect(json.encodeString(sets.empty<number>(), setEncode)).toBe("[]")
})

test("collection decoding keeps nested error paths and rejects object coercion", () => {
  const badValue = json.decodeString('[["a",1],["b","bad"]]', mapDecode)
  expect(badValue).toMatchObject({
    tag: "Left",
    value: {
      tag: "JsonDecodeFailure",
      value: { path: [json.JsonIndex(1), json.JsonIndex(1)] },
    },
  })
  const badKey = json.decodeString("[[1,2]]", mapDecode)
  expect(badKey).toMatchObject({
    tag: "Left",
    value: {
      tag: "JsonDecodeFailure",
      value: { path: [json.JsonIndex(0), json.JsonIndex(0)] },
    },
  })
  expect(json.decodeString('[["a",1,2]]', mapDecode).tag).toBe("Left")
  expect(json.decodeString('{"a":1}', mapDecode).tag).toBe("Left")
  expect(json.decodeString('[1,"bad"]', setDecode)).toMatchObject({
    tag: "Left",
    value: { tag: "JsonDecodeFailure", value: { path: [json.JsonIndex(1)] } },
  })
})

test("JsonObject's payload is the public persistent Map, preserving textual key order", () => {
  const parsed = json.parse('{"10":true,"2":false,"a":null}')
  if (parsed.tag !== "Right" || parsed.value.tag !== "JsonObject")
    throw new Error("expected object")
  const fields = parsed.value.value
  expect(maps.keys(fields)).toEqual(["10", "2", "a"])
  const changed = maps.insert(stringEq, stringHash, "2", json.JsonNull, fields)
  expect(json.stringify(json.JsonObject(changed))).toBe(
    '{"10":true,"2":null,"a":null}'
  )
  expect(json.stringify(parsed.value)).toBe('{"10":true,"2":false,"a":null}')
  expect(json.parse('{"x":1,"x":2}').tag).toBe("Left")
})

test("serialization and display do not expose the source or decoding process seed", () => {
  host.__SESERAGI_HASH_SEED__ = 7
  resetProcessHashSeedForTest()
  const before = maps.fromEntries(arrayIterable, stringEq, stringHash, [
    ["b", 2],
    ["a", 1],
  ] as const)
  const bytes = json.encodeString(before, mapEncode)
  host.__SESERAGI_HASH_SEED__ = 31
  resetProcessHashSeedForTest()
  const decoded = json.decodeString(bytes, mapDecode)
  if (decoded.tag !== "Right") throw new Error("expected Map")
  expect(
    maps.mapEq(stringEq, stringHash, intEq).eq(before)(decoded.value)
  ).toBe(true)
  expect(json.encodeString(decoded.value, mapEncode)).toBe(bytes)
  const shown = display.mapShow(display.stringShow, display.intShow)
  const debugged = display.mapDebug(display.stringDebug, display.intDebug)
  expect(shown.show(before)).toBe("Map [(b, 2), (a, 1)]")
  expect(debugged.debug(decoded.value)).toBe('Map [("b", 2), ("a", 1)]')
  expect(shown.show(before)).toBe(shown.show(decoded.value))
})

test("Map / Set display retains nested documents for compact, multiline and auto layouts", () => {
  const values = maps.singleton(stringEq, stringHash, "nested", [1, 2])
  const shown = display.mapShow(
    display.stringShow,
    display.arrayShow(display.intShow)
  )
  expect(display.renderShow(shown, values)).toBe("Map [(nested, [1, 2])]")
  const multiline = display.renderShow(shown, values, { layout: "multiline" })
  expect(multiline).toContain("\n")
  expect(
    display.renderShow(shown, values, { layout: "auto", maxWidth: 5 })
  ).toBe(multiline)
  expect(
    display.renderShow(shown, values, { layout: "auto", maxWidth: 100 })
  ).toBe(shown.show(values))
  const unique = sets.fromIterable(arrayIterable, stringEq, stringHash, [
    "b",
    "a",
    "b",
  ])
  expect(display.setShow(display.stringShow).show(unique)).toBe("Set [b, a]")
  const debugged = display.setDebug(display.stringDebug)
  expect(debugged.debug(unique)).toBe('Set ["b", "a"]')
  expect(
    display.renderDebug(debugged, unique, { layout: "multiline" })
  ).toContain("\n")
})
