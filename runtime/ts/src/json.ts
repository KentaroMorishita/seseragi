import type { Unit } from "./effect"
import type { List } from "./list"
import { toArray as listToArray } from "./list"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"

declare const decimalBrand: unique symbol

/** Exact, canonical finite decimal used by the JsonNumber ABI. */
export type Decimal = Readonly<{
  readonly negative: boolean
  readonly digits: string
  readonly scale: bigint
  readonly [decimalBrand]: true
}>

export type JsonMap<Key = string, Value = Json> = ReadonlyArray<
  readonly [Key, Value]
>

export type Json =
  | Readonly<{ readonly tag: "JsonNull" }>
  | Readonly<{ readonly tag: "JsonBool"; readonly value: boolean }>
  | Readonly<{ readonly tag: "JsonNumber"; readonly value: Decimal }>
  | Readonly<{ readonly tag: "JsonString"; readonly value: string }>
  | Readonly<{ readonly tag: "JsonArray"; readonly value: ReadonlyArray<Json> }>
  | Readonly<{
      readonly tag: "JsonObject"
      readonly value: JsonMap<string, Json>
    }>

export type JsonPathSegment =
  | Readonly<{ readonly tag: "JsonField"; readonly value: string }>
  | Readonly<{ readonly tag: "JsonIndex"; readonly value: number }>

export type DecodeErrorKind =
  | Readonly<{ readonly tag: "ExpectedJsonType"; readonly value: string }>
  | Readonly<{ readonly tag: "MissingJsonField"; readonly value: string }>
  | Readonly<{ readonly tag: "UnknownJsonField"; readonly value: string }>
  | Readonly<{ readonly tag: "UnknownJsonTag"; readonly value: string }>
  | Readonly<{ readonly tag: "InvalidJsonValue"; readonly value: string }>

export type DecodeError = Readonly<{
  readonly path: ReadonlyArray<JsonPathSegment>
  readonly kind: DecodeErrorKind
}>

export type JsonParseError =
  | Readonly<{
      readonly tag: "InvalidJsonSyntax"
      readonly value: Readonly<{
        readonly offset: number
        readonly message: string
      }>
    }>
  | Readonly<{
      readonly tag: "DuplicateJsonField"
      readonly value: Readonly<{
        readonly path: ReadonlyArray<JsonPathSegment>
        readonly field: string
      }>
    }>

export type JsonReadError =
  | Readonly<{
      readonly tag: "JsonSyntaxFailure"
      readonly value: JsonParseError
    }>
  | Readonly<{ readonly tag: "JsonDecodeFailure"; readonly value: DecodeError }>

export type Decoder<A> = (value: Json) => Either<DecodeError, A>
export type Encoder<A> = (value: A) => Json

export type JsonEncode<A> = Readonly<{ readonly encodeJson: Encoder<A> }>
export type JsonDecode<A> = Readonly<{ readonly decodeJson: Decoder<A> }>

export const JsonNull: Json = Object.freeze({ tag: "JsonNull" })
export const JsonBool = (value: boolean): Json => ({ tag: "JsonBool", value })
export const JsonNumber = (value: Decimal): Json => ({
  tag: "JsonNumber",
  value,
})
export const JsonString = (value: string): Json => ({
  tag: "JsonString",
  value,
})
export const JsonArray = (value: ReadonlyArray<Json>): Json => ({
  tag: "JsonArray",
  value,
})
export const JsonObject = (value: JsonMap<string, Json>): Json => ({
  tag: "JsonObject",
  value,
})

export const JsonField = (value: string): JsonPathSegment => ({
  tag: "JsonField",
  value,
})
export const JsonIndex = (value: number): JsonPathSegment => ({
  tag: "JsonIndex",
  value,
})

export const ExpectedJsonType = (value: string): DecodeErrorKind => ({
  tag: "ExpectedJsonType",
  value,
})
export const MissingJsonField = (value: string): DecodeErrorKind => ({
  tag: "MissingJsonField",
  value,
})
export const UnknownJsonField = (value: string): DecodeErrorKind => ({
  tag: "UnknownJsonField",
  value,
})
export const UnknownJsonTag = (value: string): DecodeErrorKind => ({
  tag: "UnknownJsonTag",
  value,
})
export const InvalidJsonValue = (value: string): DecodeErrorKind => ({
  tag: "InvalidJsonValue",
  value,
})

export const InvalidJsonSyntax = (value: {
  readonly offset: number
  readonly message: string
}): JsonParseError => ({ tag: "InvalidJsonSyntax", value })
export const DuplicateJsonField = (value: {
  readonly path: ReadonlyArray<JsonPathSegment>
  readonly field: string
}): JsonParseError => ({ tag: "DuplicateJsonField", value })
export const JsonSyntaxFailure = (value: JsonParseError): JsonReadError => ({
  tag: "JsonSyntaxFailure",
  value,
})
export const JsonDecodeFailure = (value: DecodeError): JsonReadError => ({
  tag: "JsonDecodeFailure",
  value,
})

function decodeError(kind: DecodeErrorKind): Either<DecodeError, never> {
  return Left({ path: [], kind })
}

function prependPath(
  segment: JsonPathSegment,
  result: Either<DecodeError, never>
): Either<DecodeError, never> {
  return Left({
    path: [segment, ...result.value.path],
    kind: result.value.kind,
  })
}

function canonicalDecimal(source: string): Decimal {
  const match =
    /^(-?)(0|[1-9][0-9]*)(?:\.([0-9]+))?(?:[eE]([+-]?[0-9]+))?$/.exec(source)
  if (match === null) throw new Error(`invalid JSON number: ${source}`)
  const fraction = match[3] ?? ""
  const exponent = BigInt(match[4] ?? "0")
  let digits = `${match[2]}${fraction}`.replace(/^0+/, "")
  if (digits.length === 0) {
    return Object.freeze({
      negative: false,
      digits: "0",
      scale: 0n,
    }) as Decimal
  }
  let scale = BigInt(fraction.length) - exponent
  const trailing = /0+$/.exec(digits)?.[0].length ?? 0
  if (trailing > 0) {
    digits = digits.slice(0, -trailing)
    scale -= BigInt(trailing)
  }
  return Object.freeze({
    negative: match[1] === "-",
    digits,
    scale,
  }) as Decimal
}

/** Runtime bridge for the future std/decimal implementation and codec tests. */
export function decimalFromCanonical(source: string): Decimal {
  return canonicalDecimal(source)
}

export function decimalToCanonical(value: Decimal): string {
  if (value.digits === "0") return "0"
  const sign = value.negative ? "-" : ""
  if (value.scale <= 0n) {
    return `${sign}${value.digits}${"0".repeat(Number(-value.scale))}`
  }
  const scale = Number(value.scale)
  if (scale < value.digits.length) {
    const split = value.digits.length - scale
    return `${sign}${value.digits.slice(0, split)}.${value.digits.slice(split)}`
  }
  return `${sign}0.${"0".repeat(scale - value.digits.length)}${value.digits}`
}

class ParseFailure {
  constructor(readonly error: JsonParseError) {}
}

class Parser {
  private index = 0
  private readonly encoder = new TextEncoder()

  constructor(private readonly source: string) {}

  parse(): Json {
    this.skipWhitespace()
    const result = this.value([])
    this.skipWhitespace()
    if (this.index !== this.source.length) {
      this.fail("unexpected trailing input")
    }
    return result
  }

  private byteOffset(index = this.index): number {
    return this.encoder.encode(this.source.slice(0, index)).length
  }

  private fail(message: string, index = this.index): never {
    throw new ParseFailure(
      InvalidJsonSyntax({ offset: this.byteOffset(index), message })
    )
  }

  private skipWhitespace(): void {
    while (
      this.source[this.index] === " " ||
      this.source[this.index] === "\t" ||
      this.source[this.index] === "\n" ||
      this.source[this.index] === "\r"
    ) {
      this.index += 1
    }
  }

  private value(path: ReadonlyArray<JsonPathSegment>): Json {
    const character = this.source[this.index]
    if (character === '"') return JsonString(this.string())
    if (character === "[") return this.array(path)
    if (character === "{") return this.object(path)
    if (character === "t") return this.literal("true", JsonBool(true))
    if (character === "f") return this.literal("false", JsonBool(false))
    if (character === "n") return this.literal("null", JsonNull)
    if (
      character === "-" ||
      (character !== undefined && /[0-9]/.test(character))
    ) {
      return JsonNumber(this.number())
    }
    this.fail(
      character === undefined ? "expected JSON value" : "invalid JSON value"
    )
  }

  private literal(text: string, value: Json): Json {
    if (this.source.slice(this.index, this.index + text.length) !== text) {
      this.fail(`expected ${text}`)
    }
    this.index += text.length
    return value
  }

  private number(): Decimal {
    const start = this.index
    if (this.source[this.index] === "-") this.index += 1
    if (this.source[this.index] === "0") {
      this.index += 1
      if (/[0-9]/.test(this.source[this.index] ?? "")) {
        this.fail("leading zero in JSON number")
      }
    } else if (/[1-9]/.test(this.source[this.index] ?? "")) {
      while (/[0-9]/.test(this.source[this.index] ?? "")) this.index += 1
    } else {
      this.fail("expected digit in JSON number")
    }
    if (this.source[this.index] === ".") {
      this.index += 1
      if (!/[0-9]/.test(this.source[this.index] ?? "")) {
        this.fail("expected digit after decimal point")
      }
      while (/[0-9]/.test(this.source[this.index] ?? "")) this.index += 1
    }
    if (this.source[this.index] === "e" || this.source[this.index] === "E") {
      this.index += 1
      if (this.source[this.index] === "+" || this.source[this.index] === "-") {
        this.index += 1
      }
      if (!/[0-9]/.test(this.source[this.index] ?? "")) {
        this.fail("expected exponent digit")
      }
      while (/[0-9]/.test(this.source[this.index] ?? "")) this.index += 1
    }
    return canonicalDecimal(this.source.slice(start, this.index))
  }

  private string(): string {
    this.index += 1
    let result = ""
    while (this.index < this.source.length) {
      const character = this.source[this.index] as string
      if (character === '"') {
        this.index += 1
        return result
      }
      if (character === "\\") {
        result += this.escape()
        continue
      }
      const code = character.charCodeAt(0)
      if (code <= 0x1f) this.fail("unescaped control character")
      if (code >= 0xd800 && code <= 0xdbff) {
        const low = this.source.charCodeAt(this.index + 1)
        if (low < 0xdc00 || low > 0xdfff) this.fail("unpaired high surrogate")
        result += character + this.source[this.index + 1]
        this.index += 2
        continue
      }
      if (code >= 0xdc00 && code <= 0xdfff) this.fail("unpaired low surrogate")
      result += character
      this.index += 1
    }
    this.fail("unterminated JSON string")
  }

  private escape(): string {
    const escapeStart = this.index
    this.index += 1
    const character = this.source[this.index]
    this.index += 1
    switch (character) {
      case '"':
      case "\\":
      case "/":
        return character
      case "b":
        return "\b"
      case "f":
        return "\f"
      case "n":
        return "\n"
      case "r":
        return "\r"
      case "t":
        return "\t"
      case "u": {
        const first = this.hexCodeUnit(escapeStart)
        if (first >= 0xd800 && first <= 0xdbff) {
          if (this.source.slice(this.index, this.index + 2) !== "\\u") {
            this.fail("unpaired escaped high surrogate", escapeStart)
          }
          this.index += 2
          const second = this.hexCodeUnit(escapeStart)
          if (second < 0xdc00 || second > 0xdfff) {
            this.fail("unpaired escaped high surrogate", escapeStart)
          }
          return String.fromCodePoint(
            0x10000 + ((first - 0xd800) << 10) + second - 0xdc00
          )
        }
        if (first >= 0xdc00 && first <= 0xdfff) {
          this.fail("unpaired escaped low surrogate", escapeStart)
        }
        return String.fromCharCode(first)
      }
      default:
        this.fail("invalid JSON escape", escapeStart)
    }
  }

  private hexCodeUnit(escapeStart: number): number {
    const digits = this.source.slice(this.index, this.index + 4)
    if (!/^[0-9a-fA-F]{4}$/.test(digits)) {
      this.fail("invalid Unicode escape", escapeStart)
    }
    this.index += 4
    return Number.parseInt(digits, 16)
  }

  private array(path: ReadonlyArray<JsonPathSegment>): Json {
    this.index += 1
    this.skipWhitespace()
    const values: Json[] = []
    if (this.source[this.index] === "]") {
      this.index += 1
      return JsonArray(values)
    }
    while (true) {
      values.push(this.value([...path, JsonIndex(values.length)]))
      this.skipWhitespace()
      if (this.source[this.index] === "]") {
        this.index += 1
        return JsonArray(values)
      }
      if (this.source[this.index] !== ",") this.fail("expected ',' or ']'")
      this.index += 1
      this.skipWhitespace()
      if (this.source[this.index] === "]") this.fail("trailing array comma")
    }
  }

  private object(path: ReadonlyArray<JsonPathSegment>): Json {
    this.index += 1
    this.skipWhitespace()
    const entries: Array<readonly [string, Json]> = []
    const names = new Set<string>()
    if (this.source[this.index] === "}") {
      this.index += 1
      return JsonObject(entries)
    }
    while (true) {
      if (this.source[this.index] !== '"') this.fail("expected object field")
      const name = this.string()
      if (names.has(name)) {
        throw new ParseFailure(DuplicateJsonField({ path, field: name }))
      }
      names.add(name)
      this.skipWhitespace()
      if (this.source[this.index] !== ":") this.fail("expected ':'")
      this.index += 1
      this.skipWhitespace()
      entries.push([name, this.value([...path, JsonField(name)])])
      this.skipWhitespace()
      if (this.source[this.index] === "}") {
        this.index += 1
        return JsonObject(entries)
      }
      if (this.source[this.index] !== ",") this.fail("expected ',' or '}'")
      this.index += 1
      this.skipWhitespace()
      if (this.source[this.index] === "}") this.fail("trailing object comma")
    }
  }
}

export function parse(text: string): Either<JsonParseError, Json> {
  try {
    return Right(new Parser(text).parse())
  } catch (error) {
    if (error instanceof ParseFailure) return Left(error.error)
    throw error
  }
}

function escapeString(value: string): string {
  let result = '"'
  for (const character of value) {
    const code = character.codePointAt(0) as number
    switch (code) {
      case 0x08:
        result += "\\b"
        break
      case 0x09:
        result += "\\t"
        break
      case 0x0a:
        result += "\\n"
        break
      case 0x0c:
        result += "\\f"
        break
      case 0x0d:
        result += "\\r"
        break
      case 0x22:
        result += '\\"'
        break
      case 0x5c:
        result += "\\\\"
        break
      default:
        if (code <= 0x1f)
          result += `\\u${code.toString(16).toUpperCase().padStart(4, "0")}`
        else if (code >= 0xd800 && code <= 0xdfff)
          throw new Error("JsonString contains an unpaired surrogate")
        else result += character
    }
  }
  return `${result}"`
}

export function stringify(value: Json): string {
  switch (value.tag) {
    case "JsonNull":
      return "null"
    case "JsonBool":
      return value.value ? "true" : "false"
    case "JsonNumber":
      return decimalToCanonical(value.value)
    case "JsonString":
      return escapeString(value.value)
    case "JsonArray":
      return `[${value.value.map(stringify).join(",")}]`
    case "JsonObject":
      return `{${value.value
        .map(
          ([name, fieldValue]) =>
            `${escapeString(name)}:${stringify(fieldValue)}`
        )
        .join(",")}}`
  }
}

function objectEntries(
  value: Json
): Either<DecodeError, JsonMap<string, Json>> {
  return value.tag === "JsonObject"
    ? Right(value.value)
    : decodeError(ExpectedJsonType("object"))
}

export function field<A>(name: string, decoder: Decoder<A>): Decoder<A> {
  return (value) => {
    const object = objectEntries(value)
    if (object.tag === "Left") return object
    const entry = object.value.find(([fieldName]) => fieldName === name)
    if (entry === undefined) return decodeError(MissingJsonField(name))
    const decoded = decoder(entry[1])
    return decoded.tag === "Left"
      ? prependPath(JsonField(name), decoded)
      : decoded
  }
}

export function optionalField<A>(
  name: string,
  decoder: Decoder<A>
): Decoder<Maybe<A>> {
  return (value) => {
    const object = objectEntries(value)
    if (object.tag === "Left") return object
    const entry = object.value.find(([fieldName]) => fieldName === name)
    if (entry === undefined) return Right(Nothing)
    const decoded = decoder(entry[1])
    return decoded.tag === "Left"
      ? prependPath(JsonField(name), decoded)
      : Right(Just(decoded.value))
  }
}

export function index<A>(position: number, decoder: Decoder<A>): Decoder<A> {
  return (value) => {
    if (value.tag !== "JsonArray") return decodeError(ExpectedJsonType("array"))
    if (position < 0 || position >= value.value.length) {
      return decodeError(InvalidJsonValue(`missing array index ${position}`))
    }
    const decoded = decoder(value.value[position] as Json)
    return decoded.tag === "Left"
      ? prependPath(JsonIndex(position), decoded)
      : decoded
  }
}

export function array<A>(decoder: Decoder<A>): Decoder<ReadonlyArray<A>> {
  return (value) => {
    if (value.tag !== "JsonArray") return decodeError(ExpectedJsonType("array"))
    const result: A[] = []
    for (let position = 0; position < value.value.length; position += 1) {
      const decoded = decoder(value.value[position] as Json)
      if (decoded.tag === "Left") {
        return prependPath(JsonIndex(position), decoded)
      }
      result.push(decoded.value)
    }
    return Right(result)
  }
}

export function record<A>(
  fields: ReadonlyArray<readonly [string, Decoder<A>]>
): Decoder<ReadonlyArray<readonly [string, A]>> {
  return (value) => {
    if (value.tag !== "JsonObject")
      return decodeError(ExpectedJsonType("object"))
    const expected = new Set(fields.map(([name]) => name))
    const unknown = value.value.find(([name]) => !expected.has(name))
    if (unknown !== undefined) return decodeError(UnknownJsonField(unknown[0]))
    const result: Array<readonly [string, A]> = []
    for (const [name, decoder] of fields) {
      const decoded = field(name, decoder)(value)
      if (decoded.tag === "Left") return decoded
      result.push([name, decoded.value])
    }
    return Right(result)
  }
}

export function oneOf<A>(decoders: ReadonlyArray<Decoder<A>>): Decoder<A> {
  return (value) => {
    let last: Either<DecodeError, A> | undefined
    for (const decoder of decoders) {
      const decoded = decoder(value)
      if (decoded.tag === "Right") return decoded
      last = decoded
    }
    return last ?? decodeError(InvalidJsonValue("no JSON decoder matched"))
  }
}

export function map<A, B>(
  transform: (value: A) => B,
  decoder: Decoder<A>
): Decoder<B> {
  return (value) => {
    const decoded = decoder(value)
    return decoded.tag === "Left" ? decoded : Right(transform(decoded.value))
  }
}

export function flatMap<A, B>(
  transform: (value: A) => Decoder<B>,
  decoder: Decoder<A>
): Decoder<B> {
  return (value) => {
    const decoded = decoder(value)
    return decoded.tag === "Left" ? decoded : transform(decoded.value)(value)
  }
}

export function encodeString<A>(value: A, dictionary: JsonEncode<A>): string {
  return stringify(dictionary.encodeJson(value))
}

export function decodeString<A>(
  text: string,
  dictionary: JsonDecode<A>
): Either<JsonReadError, A> {
  const parsed = parse(text)
  if (parsed.tag === "Left") return Left(JsonSyntaxFailure(parsed.value))
  const decoded = dictionary.decodeJson(parsed.value)
  return decoded.tag === "Left"
    ? Left(JsonDecodeFailure(decoded.value))
    : decoded
}

const expect =
  <A>(tag: Json["tag"], project: (value: Json) => A): Decoder<A> =>
  (value) =>
    value.tag === tag
      ? Right(project(value))
      : decodeError(ExpectedJsonType(tag))

export const boolJsonEncode: JsonEncode<boolean> = Object.freeze({
  encodeJson: JsonBool,
})
export const boolJsonDecode: JsonDecode<boolean> = Object.freeze({
  decodeJson: expect(
    "JsonBool",
    (value) => (value as { value: boolean }).value
  ),
})
export const stringJsonEncode: JsonEncode<string> = Object.freeze({
  encodeJson: JsonString,
})
export const stringJsonDecode: JsonDecode<string> = Object.freeze({
  decodeJson: expect(
    "JsonString",
    (value) => (value as { value: string }).value
  ),
})
export const intJsonEncode: JsonEncode<number> = Object.freeze({
  encodeJson: (value) => JsonNumber(canonicalDecimal(String(value))),
})
const maximumSafeIntMagnitude = "9007199254740991"

function decimalToSafeInt(
  value: Decimal
): Either<DecodeError, number> {
  if (value.digits === "0") return Right(0)
  if (value.scale > 0n)
    return decodeError(InvalidJsonValue("expected integer"))

  const integerDigits = BigInt(value.digits.length) - value.scale
  if (integerDigits > BigInt(maximumSafeIntMagnitude.length)) {
    return decodeError(
      InvalidJsonValue("integer is outside the safe Int range")
    )
  }
  const magnitude = `${value.digits}${"0".repeat(Number(-value.scale))}`
  if (
    magnitude.length === maximumSafeIntMagnitude.length &&
    magnitude > maximumSafeIntMagnitude
  ) {
    return decodeError(
      InvalidJsonValue("integer is outside the safe Int range")
    )
  }
  const result = Number(`${value.negative ? "-" : ""}${magnitude}`)
  return Right(Object.is(result, -0) ? 0 : result)
}

export const intJsonDecode: JsonDecode<number> = Object.freeze({
  decodeJson: (value) => {
    if (value.tag !== "JsonNumber")
      return decodeError(ExpectedJsonType("number"))
    return decimalToSafeInt(value.value)
  },
})
export const unitJsonEncode: JsonEncode<Unit> = Object.freeze({
  encodeJson: () => JsonNull,
})
export const unitJsonDecode: JsonDecode<Unit> = Object.freeze({
  decodeJson: (value) =>
    value.tag === "JsonNull"
      ? Right(undefined)
      : decodeError(ExpectedJsonType("null")),
})
export const jsonJsonEncode: JsonEncode<Json> = Object.freeze({
  encodeJson: (value) => value,
})
export const jsonJsonDecode: JsonDecode<Json> = Object.freeze({
  decodeJson: Right,
})

export const maybeJsonEncode = <A>(
  dictionary: JsonEncode<A>
): JsonEncode<Maybe<A>> =>
  Object.freeze({
    encodeJson: (value) =>
      value.tag === "Nothing" ? JsonNull : dictionary.encodeJson(value.value),
  })

export const maybeJsonDecode = <A>(
  dictionary: JsonDecode<A>
): JsonDecode<Maybe<A>> =>
  Object.freeze({
    decodeJson: (value) =>
      value.tag === "JsonNull"
        ? Right(Nothing)
        : map(Just, dictionary.decodeJson)(value),
  })

export const eitherJsonEncode = <E, A>(
  left: JsonEncode<E>,
  right: JsonEncode<A>
): JsonEncode<Either<E, A>> =>
  Object.freeze({
    encodeJson: (value) =>
      JsonObject([
        ["tag", JsonString(value.tag)],
        [
          "value",
          value.tag === "Left"
            ? left.encodeJson(value.value)
            : right.encodeJson(value.value),
        ],
      ]),
  })

export const eitherJsonDecode = <E, A>(
  left: JsonDecode<E>,
  right: JsonDecode<A>
): JsonDecode<Either<E, A>> =>
  Object.freeze({
    decodeJson: (value) => {
      const tag = field("tag", stringJsonDecode.decodeJson)(value)
      if (tag.tag === "Left") return tag
      if (tag.value === "Left") {
        const decoded = field("value", left.decodeJson)(value)
        return decoded.tag === "Left" ? decoded : Right(Left(decoded.value))
      }
      if (tag.value === "Right") {
        const decoded = field("value", right.decodeJson)(value)
        return decoded.tag === "Left" ? decoded : Right(Right(decoded.value))
      }
      return prependPath(
        JsonField("tag"),
        decodeError(UnknownJsonTag(tag.value))
      )
    },
  })

export const arrayJsonEncode = <A>(
  dictionary: JsonEncode<A>
): JsonEncode<ReadonlyArray<A>> =>
  Object.freeze({
    encodeJson: (values) => JsonArray(values.map(dictionary.encodeJson)),
  })
export const arrayJsonDecode = <A>(
  dictionary: JsonDecode<A>
): JsonDecode<ReadonlyArray<A>> =>
  Object.freeze({ decodeJson: array(dictionary.decodeJson) })

export const listJsonEncode = <A>(
  dictionary: JsonEncode<A>
): JsonEncode<List<A>> =>
  Object.freeze({
    encodeJson: (values) =>
      JsonArray(listToArray(values).map(dictionary.encodeJson)),
  })
export const listJsonDecode = <A>(
  dictionary: JsonDecode<A>
): JsonDecode<List<A>> =>
  Object.freeze({
    decodeJson: (value) => {
      const decoded = array(dictionary.decodeJson)(value)
      if (decoded.tag === "Left") return decoded
      let result: List<A> = { tag: "Empty" }
      for (
        let position = decoded.value.length - 1;
        position >= 0;
        position -= 1
      ) {
        result = {
          tag: "Cons",
          head: decoded.value[position] as A,
          tail: result,
        }
      }
      return Right(result)
    },
  })

export const tupleJsonEncode = <T extends ReadonlyArray<unknown>>(
  ...dictionaries: ReadonlyArray<JsonEncode<never>>
): JsonEncode<T> =>
  Object.freeze({
    encodeJson: (values) =>
      JsonArray(
        dictionaries.map((dictionary, position) =>
          (dictionary.encodeJson as unknown as Encoder<unknown>)(
            values[position]
          )
        )
      ),
  })
export const tupleJsonDecode = <T extends ReadonlyArray<unknown>>(
  ...dictionaries: ReadonlyArray<JsonDecode<unknown>>
): JsonDecode<T> =>
  Object.freeze({
    decodeJson: (value) => {
      if (value.tag !== "JsonArray")
        return decodeError(ExpectedJsonType("array"))
      if (value.value.length !== dictionaries.length) {
        return decodeError(
          InvalidJsonValue(`expected tuple length ${dictionaries.length}`)
        )
      }
      const result: unknown[] = []
      for (let position = 0; position < dictionaries.length; position += 1) {
        const decoded = (
          dictionaries[position] as JsonDecode<unknown>
        ).decodeJson(value.value[position] as Json)
        if (decoded.tag === "Left")
          return prependPath(JsonIndex(position), decoded)
        result.push(decoded.value)
      }
      return Right(result as unknown as T)
    },
  })

export const recordJsonEncode = <R extends Readonly<Record<string, unknown>>>(
  names: ReadonlyArray<string>,
  optional: ReadonlyArray<boolean>,
  ...dictionaries: ReadonlyArray<JsonEncode<never>>
): JsonEncode<R> =>
  Object.freeze({
    encodeJson: (value) => {
      const entries: Array<readonly [string, Json]> = []
      for (let position = 0; position < names.length; position += 1) {
        const name = names[position] as string
        if (optional[position] && !Object.hasOwn(value, name)) continue
        entries.push([
          name,
          (
            (dictionaries[position] as JsonEncode<never>)
              .encodeJson as unknown as Encoder<unknown>
          )(value[name]),
        ])
      }
      return JsonObject(entries)
    },
  })

export const recordJsonDecode = <R extends Readonly<Record<string, unknown>>>(
  names: ReadonlyArray<string>,
  optional: ReadonlyArray<boolean>,
  ...dictionaries: ReadonlyArray<JsonDecode<unknown>>
): JsonDecode<R> =>
  Object.freeze({
    decodeJson: (value) => {
      if (value.tag !== "JsonObject")
        return decodeError(ExpectedJsonType("object"))
      const expected = new Set(names)
      const entries = new Map<string, Json>()
      for (const [name, fieldValue] of value.value) {
        if (!expected.has(name)) return decodeError(UnknownJsonField(name))
        if (!entries.has(name)) entries.set(name, fieldValue)
      }
      const result: Record<string, unknown> = {}
      for (let position = 0; position < names.length; position += 1) {
        const name = names[position] as string
        const fieldValue = entries.get(name)
        if (fieldValue === undefined && optional[position]) continue
        if (fieldValue === undefined) return decodeError(MissingJsonField(name))
        const decoded = (
          dictionaries[position] as JsonDecode<unknown>
        ).decodeJson(fieldValue)
        if (decoded.tag === "Left") return prependPath(JsonField(name), decoded)
        result[name] = decoded.value
      }
      return Right(result as R)
    },
  })

type JsonEncodeThunk = () => JsonEncode<unknown>
type JsonDecodeThunk = () => JsonDecode<unknown>

/** Compiler support for declaration-ordered, strict named struct codecs. */
export const derivedStructJsonEncode = <A>(
  names: ReadonlyArray<string>,
  dictionaries: ReadonlyArray<JsonEncodeThunk>
): JsonEncode<A> =>
  Object.freeze({
    encodeJson: (value) => {
      const record = value as Readonly<Record<string, unknown>>
      return JsonObject(
        names.map((name, position) => [
          name,
          (dictionaries[position] as JsonEncodeThunk)().encodeJson(record[name]),
        ])
      )
    },
  })

/** Compiler support for declaration-ordered, strict named struct codecs. */
export const derivedStructJsonDecode = <A>(
  names: ReadonlyArray<string>,
  dictionaries: ReadonlyArray<JsonDecodeThunk>
): JsonDecode<A> =>
  Object.freeze({
    decodeJson: (value) => {
      if (value.tag !== "JsonObject")
        return decodeError(ExpectedJsonType("object"))
      const expected = new Set(names)
      const unknown = value.value.find(([name]) => !expected.has(name))
      if (unknown !== undefined)
        return decodeError(UnknownJsonField(unknown[0]))
      const result: Record<string, unknown> = {}
      for (let position = 0; position < names.length; position += 1) {
        const name = names[position] as string
        const entry = value.value.find(([fieldName]) => fieldName === name)
        if (entry === undefined) return decodeError(MissingJsonField(name))
        const decoded = (
          dictionaries[position] as JsonDecodeThunk
        )().decodeJson(entry[1])
        if (decoded.tag === "Left") return prependPath(JsonField(name), decoded)
        result[name] = decoded.value
      }
      return Right(result as A)
    },
  })

type DerivedAdtEncodeCase = readonly [
  string,
  JsonEncodeThunk | undefined,
]
type DerivedAdtDecodeCase = readonly [
  string,
  JsonDecodeThunk | undefined,
]

/** Compiler support for the canonical tagged nominal ADT wire contract. */
export const derivedAdtJsonEncode = <A>(
  cases: ReadonlyArray<DerivedAdtEncodeCase>
): JsonEncode<A> =>
  Object.freeze({
    encodeJson: (value) => {
      const tagged = value as Readonly<{ tag: string; value?: unknown }>
      const selected = cases.find(([tag]) => tag === tagged.tag)
      if (selected === undefined)
        throw new Error(`unknown derived JSON constructor: ${tagged.tag}`)
      const payload = selected[1]
      return JsonObject(
        payload === undefined
          ? [["tag", JsonString(tagged.tag)]]
          : [
              ["tag", JsonString(tagged.tag)],
              ["value", payload().encodeJson(tagged.value)],
            ]
      )
    },
  })

/** Compiler support for the canonical tagged nominal ADT wire contract. */
export const derivedAdtJsonDecode = <A>(
  cases: ReadonlyArray<DerivedAdtDecodeCase>
): JsonDecode<A> =>
  Object.freeze({
    decodeJson: (value) => {
      if (value.tag !== "JsonObject")
        return decodeError(ExpectedJsonType("object"))
      const tagEntry = value.value.find(([name]) => name === "tag")
      if (tagEntry === undefined) return decodeError(MissingJsonField("tag"))
      const decodedTag = stringJsonDecode.decodeJson(tagEntry[1])
      if (decodedTag.tag === "Left")
        return prependPath(JsonField("tag"), decodedTag)
      const selected = cases.find(([tag]) => tag === decodedTag.value)
      if (selected === undefined)
        return decodeError(UnknownJsonTag(decodedTag.value))
      const payload = selected[1]
      const expected =
        payload === undefined ? new Set(["tag"]) : new Set(["tag", "value"])
      const unknown = value.value.find(([name]) => !expected.has(name))
      if (unknown !== undefined)
        return decodeError(UnknownJsonField(unknown[0]))
      if (payload === undefined)
        return Right({ tag: decodedTag.value } as A)
      const payloadEntry = value.value.find(([name]) => name === "value")
      if (payloadEntry === undefined)
        return decodeError(MissingJsonField("value"))
      const decoded = payload().decodeJson(payloadEntry[1])
      return decoded.tag === "Left"
        ? prependPath(JsonField("value"), decoded)
        : Right({ tag: decodedTag.value, value: decoded.value } as A)
    },
  })

/** Compiler support for transparent newtype encoding. */
export const derivedNewtypeJsonEncode = <A>(
  dictionary: JsonEncodeThunk
): JsonEncode<A> =>
  Object.freeze({
    encodeJson: (value) =>
      dictionary().encodeJson(
        (value as Readonly<{ value: unknown }>).value
      ),
  })

/** Compiler support for transparent newtype decoding. */
export const derivedNewtypeJsonDecode = <A>(
  tag: string,
  dictionary: JsonDecodeThunk
): JsonDecode<A> =>
  Object.freeze({
    decodeJson: (value) => {
      const decoded = dictionary().decodeJson(value)
      return decoded.tag === "Left"
        ? decoded
        : Right({ tag, value: decoded.value } as A)
    },
  })
