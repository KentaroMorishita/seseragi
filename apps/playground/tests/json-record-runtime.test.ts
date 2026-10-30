import { expect, test } from "bun:test"
import {
  derivedStructJsonDecode,
  type Json,
  type JsonDecode,
  jsonJsonDecode,
  parse,
  recordJsonDecode,
  recordJsonEncode,
  stringify,
  stringJsonDecode,
  stringJsonEncode,
} from "../../../runtime/ts/src/json"

function parsed(source: string): Json {
  const result = parse(source)
  if (result.tag === "Left") throw new Error(JSON.stringify(result.value))
  return result.value
}

function decoded(codec: JsonDecode<Record<string, unknown>>, source: string) {
  const result = codec.decodeJson(parsed(source))
  if (result.tag === "Left") throw new Error(JSON.stringify(result.value))
  return result.value
}

for (const derived of [false, true]) {
  const kind = derived ? "derived struct" : "structural record"
  test(`${kind} decodes prototype names as own ordinary data properties`, () => {
    const names = ["__proto__", "constructor", "toString"]
    const codec = derived
      ? derivedStructJsonDecode<Record<string, unknown>>(
          names,
          names.map(() => () => stringJsonDecode)
        )
      : recordJsonDecode(
          names,
          [false, false, false],
          ...names.map(() => stringJsonDecode)
        )
    const source = '{"__proto__":"雪🙂","constructor":"ctor","toString":"text"}'
    const value = decoded(codec, source)
    expect(Object.getPrototypeOf(value)).toBe(Object.prototype)
    expect(Object.keys(value)).toEqual(names)
    for (const name of names) {
      expect(Object.getOwnPropertyDescriptor(value, name)).toEqual({
        value:
          name === "__proto__"
            ? "雪🙂"
            : name === "constructor"
              ? "ctor"
              : "text",
        enumerable: true,
        configurable: true,
        writable: true,
      })
    }
    const encoder = recordJsonEncode(
      names,
      [false, false, false],
      ...names.map(() => stringJsonEncode)
    )
    expect(stringify(encoder.encodeJson(value))).toBe(source)
  })

  test(`${kind} keeps object values in __proto__ without changing its prototype`, () => {
    const codec = derived
      ? derivedStructJsonDecode<Record<string, unknown>>(
          ["__proto__"],
          [() => jsonJsonDecode]
        )
      : recordJsonDecode(["__proto__"], [false], jsonJsonDecode)
    const source = '{"__proto__":{"marker":"data"}}'
    const value = decoded(codec, source)
    expect(Object.getPrototypeOf(value)).toBe(Object.prototype)
    expect(Object.hasOwn(value, "__proto__")).toBe(true)
    expect(stringify(value.__proto__ as Json)).toBe('{"marker":"data"}')
    expect(Object.hasOwn(value, "marker")).toBe(false)
  })
}

test("optional prototype fields keep absence, null rejection, and unknown-first order", () => {
  const codec = recordJsonDecode(["__proto__"], [true], stringJsonDecode)
  const absent = decoded(codec, "{}")
  expect(Object.hasOwn(absent, "__proto__")).toBe(false)
  expect(Object.getPrototypeOf(absent)).toBe(Object.prototype)
  expect(decoded(codec, '{"__proto__":"present"}').__proto__).toBe("present")
  expect(codec.decodeJson(parsed('{"__proto__":null}'))).toEqual({
    tag: "Left",
    value: {
      path: [{ tag: "JsonField", value: "__proto__" }],
      kind: { tag: "ExpectedJsonType", value: "JsonString" },
    },
  })
  expect(codec.decodeJson(parsed('{"__proto__":null,"extra":false}'))).toEqual({
    tag: "Left",
    value: {
      path: [],
      kind: { tag: "UnknownJsonField", value: "extra" },
    },
  })
})
