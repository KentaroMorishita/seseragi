import type { ConsoleError } from "./console-service"
import type { StdinError } from "./stdin-service"

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
