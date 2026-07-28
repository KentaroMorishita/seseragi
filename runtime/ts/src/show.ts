import type { ConsoleError } from "./console-service"
import type { List } from "./list"
import type { StdinError } from "./stdin-service"
import type { Either, Maybe } from "./sum"

export type RenderLayout = "compact" | "multiline" | "auto"

export type RenderOptions = {
  readonly layout?: RenderLayout
  readonly indentWidth?: number
  readonly maxWidth?: number
}

export type RenderDocument =
  | {
      readonly kind: "text"
      readonly value: string
    }
  | {
      readonly kind: "line"
    }
  | {
      readonly kind: "concat"
      readonly documents: readonly RenderDocument[]
    }
  | {
      readonly kind: "indent"
      readonly document: RenderDocument
    }
  | {
      readonly kind: "delimited"
      readonly open: string
      readonly close: string
      readonly separator: string
      readonly compactPadding: boolean
      readonly items: readonly RenderDocument[]
    }

type DocumentBacked<Value> = {
  readonly document?: (value: Value) => RenderDocument
}

/**
 * The runtime dictionary representation of Seseragi's pure `Show<A>` trait.
 *
 * Compiler-generated dictionaries use this same shape. The dictionary does
 * not perform I/O and must not expose host error objects or stack traces.
 */
export type Show<Value> = DocumentBacked<Value> & {
  readonly show: (value: Value) => string
}

/** The runtime dictionary representation of Seseragi's pure `Debug<A>` trait. */
export type Debug<Value> = DocumentBacked<Value> & {
  readonly debug: (value: Value) => string
}

type ErasedEvidence = Readonly<Record<string, (...arguments_: any[]) => any>>
type ShowEvidence<Value> = Show<Value> | ErasedEvidence
type DebugEvidence<Value> = Debug<Value> | ErasedEvidence

type DisplayRange<Value> = Readonly<{
  start: Value
  end: Value
  inclusive: boolean
}>

export function text(value: string): RenderDocument {
  return Object.freeze({ kind: "text", value })
}

export const line: RenderDocument = Object.freeze({ kind: "line" })

export function concat(documents: readonly RenderDocument[]): RenderDocument {
  return Object.freeze({
    kind: "concat",
    documents: Object.freeze([...documents]),
  })
}

export function indent(document: RenderDocument): RenderDocument {
  return Object.freeze({ kind: "indent", document })
}

export function delimited(
  open: string,
  items: readonly RenderDocument[],
  close: string,
  separator = ",",
  compactPadding = false
): RenderDocument {
  return Object.freeze({
    kind: "delimited",
    open,
    close,
    separator,
    compactPadding,
    items: Object.freeze([...items]),
  })
}

export function renderDocument(
  document: RenderDocument,
  options: RenderOptions = {}
): string {
  const indentWidth = naturalOption(options.indentWidth, 2, "indentWidth")
  const maxWidth = naturalOption(options.maxWidth, 80, "maxWidth")
  const layout = options.layout ?? "compact"
  if (layout === "auto") {
    const compact = renderWithLayout(document, "compact", indentWidth)
    return scalarLength(compact) <= maxWidth && !compact.includes("\n")
      ? compact
      : renderWithLayout(document, "multiline", indentWidth)
  }
  return renderWithLayout(document, layout, indentWidth)
}

export function renderShow<Value>(
  instance: Show<Value>,
  value: Value,
  options: RenderOptions = {}
): string {
  const document = instance.document?.(value)
  return document === undefined
    ? instance.show(value)
    : renderDocument(document, options)
}

export function renderDebug<Value>(
  instance: Debug<Value>,
  value: Value,
  options: RenderOptions = {}
): string {
  const document = instance.document?.(value)
  return document === undefined
    ? instance.debug(value)
    : renderDocument(document, options)
}

function defineShow<Value>(
  document: (value: Value) => RenderDocument
): Show<Value> {
  return Object.freeze({
    document,
    show(value: Value): string {
      return renderDocument(document(value))
    },
  })
}

function defineDebug<Value>(
  document: (value: Value) => RenderDocument
): Debug<Value> {
  return Object.freeze({
    document,
    debug(value: Value): string {
      return renderDocument(document(value))
    },
  })
}

/** String Show is identity: user-facing output does not add quotes. */
export const stringShow = defineShow((value: string) => text(value))

/** Int Show uses the canonical signed base-10 spelling without separators. */
export const intShow = defineShow((value: bigint) => text(value.toString(10)))

/** Int Debug uses the same canonical spelling as Show. */
export const intDebug = defineDebug((value: bigint) => text(value.toString(10)))

/**
 * Float display is the shortest decimal spelling that round-trips to the same
 * binary64 value while retaining an unambiguous Float spelling.
 */
export const floatShow = defineShow((value: number) =>
  text(canonicalFloat(value))
)

export const floatDebug = defineDebug((value: number) =>
  text(canonicalFloat(value))
)

/**
 * Never has no runtime inhabitants. These dictionaries exist so conditional
 * evidence such as Show<Maybe<Never>> can still be materialized.
 */
export const neverShow = defineShow((value: never) => unreachableNever(value))

export const neverDebug = defineDebug((value: never) => unreachableNever(value))

/** Bool uses Seseragi's canonical constructor spelling. */
export const boolShow = defineShow((value: boolean) =>
  text(value ? "True" : "False")
)

export const boolDebug = defineDebug((value: boolean) =>
  text(value ? "True" : "False")
)

/** Unit has one source-level spelling in both user and developer output. */
export const unitShow = defineShow((_value: undefined) => text("()"))

export const unitDebug = defineDebug((_value: undefined) => text("()"))

/** Char Show emits the scalar; Char Debug emits a quoted, escaped literal. */
export const charShow = defineShow((value: string) =>
  text(requireScalar(value))
)

export const charDebug = defineDebug((value: string) =>
  text(`'${escapeCharacter(requireScalar(value), "'")}'`)
)

/** String Debug is a quoted Seseragi literal with deterministic escapes. */
export const stringDebug = defineDebug((value: string) =>
  text(`"${escapeText(value, '"')}"`)
)

/** Ordered, recursively composed display dictionaries for standard containers. */
export function arrayShow<Value>(
  element: ShowEvidence<Value>
): Show<ReadonlyArray<Value>> {
  return defineShow((values) =>
    delimited(
      "[",
      values.map((value) => showDocument(element, value)),
      "]"
    )
  )
}

export function arrayDebug<Value>(
  element: DebugEvidence<Value>
): Debug<ReadonlyArray<Value>> {
  return defineDebug((values) =>
    delimited(
      "[",
      values.map((value) => debugDocument(element, value)),
      "]"
    )
  )
}

export function listShow<Value>(
  element: ShowEvidence<Value>
): Show<List<Value>> {
  return defineShow((values) =>
    delimited("`[", listDocuments(values, element, showDocument), "]")
  )
}

export function listDebug<Value>(
  element: DebugEvidence<Value>
): Debug<List<Value>> {
  return defineDebug((values) =>
    delimited("`[", listDocuments(values, element, debugDocument), "]")
  )
}

export function maybeShow<Value>(
  element: ShowEvidence<Value>
): Show<Maybe<Value>> {
  return defineShow((value) =>
    value.tag === "Nothing"
      ? text("Nothing")
      : constructorDocument("Just", showDocument(element, value.value))
  )
}

export function maybeDebug<Value>(
  element: DebugEvidence<Value>
): Debug<Maybe<Value>> {
  return defineDebug((value) =>
    value.tag === "Nothing"
      ? text("Nothing")
      : constructorDocument("Just", debugDocument(element, value.value))
  )
}

export function eitherShow<Error, Value>(
  error: ShowEvidence<Error>,
  value: ShowEvidence<Value>
): Show<Either<Error, Value>> {
  return defineShow((either) =>
    either.tag === "Left"
      ? constructorDocument("Left", showDocument(error, either.value))
      : constructorDocument("Right", showDocument(value, either.value))
  )
}

export function eitherDebug<Error, Value>(
  error: DebugEvidence<Error>,
  value: DebugEvidence<Value>
): Debug<Either<Error, Value>> {
  return defineDebug((either) =>
    either.tag === "Left"
      ? constructorDocument("Left", debugDocument(error, either.value))
      : constructorDocument("Right", debugDocument(value, either.value))
  )
}

/**
 * Range renders its bounds and inclusivity directly. It never iterates or
 * expands the values between the bounds.
 */
export function rangeShow<Value>(
  bound: ShowEvidence<Value>
): Show<DisplayRange<Value>> {
  return defineShow((range) =>
    concat([
      showDocument(bound, range.start),
      text(range.inclusive ? "..=" : ".."),
      showDocument(bound, range.end),
    ])
  )
}

export function rangeDebug<Value>(
  bound: DebugEvidence<Value>
): Debug<DisplayRange<Value>> {
  return defineDebug((range) =>
    concat([
      debugDocument(bound, range.start),
      text(range.inclusive ? "..=" : ".."),
      debugDocument(bound, range.end),
    ])
  )
}

/**
 * Tuple dictionaries are compiler-provided from the tuple's ordered element
 * types. Runtime values never supply their own inspection policy.
 */
export function tupleShow<Value extends readonly unknown[]>(
  ...elements: ShowEvidence<any>[]
): Show<Value> {
  return defineShow((value) =>
    delimited(
      "(",
      elements.map((element, index) => showDocument(element, value[index])),
      ")"
    )
  )
}

export function tupleDebug<Value extends readonly unknown[]>(
  ...elements: DebugEvidence<any>[]
): Debug<Value> {
  return defineDebug((value) =>
    delimited(
      "(",
      elements.map((element, index) => debugDocument(element, value[index])),
      ")"
    )
  )
}

/**
 * Record field names, order, and optionality come from the compiler's closed
 * structural type. The runtime never enumerates host object keys.
 */
export function recordShow<Value extends object>(
  fieldNames: readonly string[],
  optionalFields: readonly boolean[],
  ...fields: ShowEvidence<any>[]
): Show<Value> {
  requireRecordDescriptor(fieldNames, optionalFields, fields)
  return defineShow((value) => {
    const record = value as Readonly<Record<string, unknown>>
    return delimited(
      "{",
      fieldNames.map((name, index) => {
        const optional = optionalFields[index] === true
        if (optional && !hasOwn(record, name)) {
          return text(`${name}?: Nothing`)
        }
        const field = fields[index]
        if (field === undefined) {
          throw new RangeError("record Show descriptor is incomplete")
        }
        const rendered = showDocument(field, record[name])
        return concat([
          text(optional ? `${name}?: Just ` : `${name}: `),
          rendered,
        ])
      }),
      "}",
      ",",
      true
    )
  })
}

export function recordDebug<Value extends object>(
  fieldNames: readonly string[],
  optionalFields: readonly boolean[],
  ...fields: DebugEvidence<any>[]
): Debug<Value> {
  requireRecordDescriptor(fieldNames, optionalFields, fields)
  return defineDebug((value) => {
    const record = value as Readonly<Record<string, unknown>>
    return delimited(
      "{",
      fieldNames.map((name, index) => {
        const optional = optionalFields[index] === true
        if (optional && !hasOwn(record, name)) {
          return text(`${name}?: Nothing`)
        }
        const field = fields[index]
        if (field === undefined) {
          throw new RangeError("record Debug descriptor is incomplete")
        }
        const rendered = debugDocument(field, record[name])
        return concat([
          text(optional ? `${name}?: Just ` : `${name}: `),
          rendered,
        ])
      }),
      "}",
      ",",
      true
    )
  })
}

/** Stable, user-facing rendering for the opaque Console failure boundary. */
export const consoleErrorShow = defineShow((error: ConsoleError) =>
  text(`ConsoleError: ${error.message}`)
)

/** Source-like rendering for the standard Stdin failure ADT. */
export const stdinErrorShow = defineShow((error: StdinError) => {
  switch (error.tag) {
    case "StdinUnavailable":
    case "StdinReadFailure":
    case "ConcurrentStdinRead":
    case "StdinPositionOverflow":
      return text(error.tag)
    case "InvalidStdinUtf8":
      return delimited(
        "InvalidStdinUtf8 {",
        [text(`offset: ${error.value.offset}`)],
        "}",
        ",",
        true
      )
    case "StdinLineTooLong":
      return delimited(
        "StdinLineTooLong {",
        [text(`limitBytes: ${error.value.limitBytes}`)],
        "}",
        ",",
        true
      )
  }
})

function showDocument<Value>(
  evidence: ShowEvidence<Value>,
  value: Value
): RenderDocument {
  const instance = evidence as Show<Value>
  return instance.document?.(value) ?? text(instance.show(value))
}

function debugDocument<Value>(
  evidence: DebugEvidence<Value>,
  value: Value
): RenderDocument {
  const instance = evidence as Debug<Value>
  return instance.document?.(value) ?? text(instance.debug(value))
}

function requireRecordDescriptor(
  fieldNames: readonly string[],
  optionalFields: readonly boolean[],
  fields: readonly unknown[]
): void {
  if (
    fieldNames.length !== optionalFields.length ||
    fieldNames.length !== fields.length
  ) {
    throw new RangeError("record display descriptor length mismatch")
  }
}

function hasOwn(
  record: Readonly<Record<string, unknown>>,
  field: string
): boolean {
  return Object.hasOwn(record, field)
}

function canonicalFloat(value: number): string {
  if (Number.isNaN(value)) {
    return "NaN"
  }
  if (value === Number.POSITIVE_INFINITY) {
    return "Infinity"
  }
  if (value === Number.NEGATIVE_INFINITY) {
    return "-Infinity"
  }
  if (Object.is(value, -0)) {
    return "-0.0"
  }

  const rendered = value.toString()
  if (rendered.includes("e")) {
    return rendered.replace("e+", "e").replace(/e(-?)0+(\d+)/, "e$1$2")
  }
  return rendered.includes(".") ? rendered : `${rendered}.0`
}

function unreachableNever(_value: never): never {
  throw new TypeError("Never display dictionary received a runtime value")
}

function listDocuments<
  Value,
  Dictionary extends ShowEvidence<Value> | DebugEvidence<Value>,
>(
  values: List<Value>,
  element: Dictionary,
  document: (instance: Dictionary, value: Value) => RenderDocument
): RenderDocument[] {
  const documents: RenderDocument[] = []
  let cursor = values
  while (cursor.tag === "Cons") {
    documents.push(document(element, cursor.head))
    cursor = cursor.tail
  }
  return documents
}

function constructorDocument(
  name: string,
  payload: RenderDocument
): RenderDocument {
  return concat([text(name), indent(concat([line, payload]))])
}

function renderWithLayout(
  document: RenderDocument,
  layout: Exclude<RenderLayout, "auto">,
  indentWidth: number
): string {
  let output = ""

  function append(current: RenderDocument, depth: number): void {
    switch (current.kind) {
      case "text":
        output += current.value
        break
      case "line":
        output +=
          layout === "compact" ? " " : `\n${" ".repeat(depth * indentWidth)}`
        break
      case "concat":
        for (const child of current.documents) {
          append(child, depth)
        }
        break
      case "indent":
        append(current.document, depth + 1)
        break
      case "delimited":
        output += current.open
        if (current.items.length === 0) {
          output += current.close
          break
        }
        if (layout === "compact") {
          if (current.compactPadding) {
            output += " "
          }
          for (const [index, item] of current.items.entries()) {
            if (index > 0) {
              output += `${current.separator} `
            }
            append(item, depth + 1)
          }
          if (current.compactPadding) {
            output += " "
          }
          output += current.close
          break
        }
        output += `\n${" ".repeat((depth + 1) * indentWidth)}`
        for (const [index, item] of current.items.entries()) {
          if (index > 0) {
            output += `${current.separator}\n${" ".repeat(
              (depth + 1) * indentWidth
            )}`
          }
          append(item, depth + 1)
        }
        output += `\n${" ".repeat(depth * indentWidth)}${current.close}`
        break
    }
  }

  append(document, 0)
  return output
}

function naturalOption(
  value: number | undefined,
  fallback: number,
  name: string
): number {
  if (value === undefined) {
    return fallback
  }
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new RangeError(`${name} must be a non-negative safe integer`)
  }
  return value
}

function scalarLength(value: string): number {
  return Array.from(value).length
}

function requireScalar(value: string): string {
  const scalars = Array.from(value)
  if (scalars.length !== 1) {
    throw new RangeError("Char runtime value must contain exactly one scalar")
  }
  const scalar = scalars[0]
  if (scalar === undefined) {
    throw new RangeError("Char runtime value must contain exactly one scalar")
  }
  const codePoint = scalar.codePointAt(0)
  if (codePoint === undefined || (codePoint >= 0xd800 && codePoint <= 0xdfff)) {
    throw new RangeError("Char runtime value must be a Unicode scalar")
  }
  return scalar
}

function escapeText(value: string, quote: '"' | "'"): string {
  let escaped = ""
  for (const character of value) {
    escaped += escapeCharacter(character, quote)
  }
  return escaped
}

function escapeCharacter(character: string, quote: '"' | "'"): string {
  switch (character) {
    case "\\":
      return "\\\\"
    case "\0":
      return "\\0"
    case "\b":
      return "\\b"
    case "\t":
      return "\\t"
    case "\n":
      return "\\n"
    case "\f":
      return "\\f"
    case "\r":
      return "\\r"
    default: {
      if (character === quote) {
        return `\\${quote}`
      }
      const codePoint = character.codePointAt(0)
      if (
        codePoint === undefined ||
        codePoint < 0x20 ||
        (codePoint >= 0x7f && codePoint <= 0x9f)
      ) {
        return `\\u{${(codePoint ?? 0).toString(16).toUpperCase()}}`
      }
      return character
    }
  }
}
