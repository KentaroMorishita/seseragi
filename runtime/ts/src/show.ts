import type {
  BigIntConversionError,
  BigIntDivisionError,
  BigIntParseError,
  BigIntPowerError,
  BigInt as SeseragiBigInt,
} from "./big-int"
import type { ByteError, BytesSliceError } from "./bytes"
import type { Base64DecodeError } from "./bytes-base64"
import type { HexDecodeError } from "./bytes-hex"
import type {
  ChildExitStatus,
  ChildProcessConfigError,
  ChildProcessError,
} from "./child-process"
import type { DurationError } from "./clock"
import type { SizeError } from "./collection"
import type { ConsoleError } from "./console-service"
import type { DomError, DomRuntimeError } from "./dom"
import type { ParallelismError, ScheduleError } from "./effect"
import type { EntropyConfigError, EntropyError } from "./entropy"
import type {
  DirectoryEntry,
  FileMetadata,
  FileSystemError,
  FileSystemErrorKind,
  FileSystemOperation,
  FileTextError,
  FileType,
  WriteMode,
} from "./filesystem"
import type { JsError } from "./foreign"
import type { GraphemeSliceError } from "./grapheme"
import type { HtmlBuildError } from "./html"
import type { HttpBuildError, HttpError } from "./http-client"
import type { List, NonEmptyList } from "./list"
import type { LogError } from "./logger-service"
import { entries as mapEntries, type Map as PersistentMap } from "./map"
import type { NavigationError, UrlBuildError } from "./navigation"
import { type PathError, render as renderPath } from "./path"
import type { ProcessError, ProcessSignal } from "./process"
import type { QueueClosed, QueueCreateError } from "./queue"
import type { RandomConfigError, RandomRangeError } from "./random"
import type {
  RegexCapture,
  RegexCompileError,
  RegexCompileErrorKind,
  RegexMatch,
  RegexOptions,
  RegexSpan,
} from "./regex"
import type { SemaphoreCreateError } from "./semaphore"
import { type Set as PersistentSet, toArray as setValues } from "./set"
import type { StdinConfigError, StdinError } from "./stdin-service"
import type { StorageArea, StorageError } from "./storage"
import type { BufferCapacityError } from "./stream"
import type { Either, Maybe } from "./sum"
import type { TextSliceError, Utf8DecodeError } from "./text"
import type { DateTimeError, TimeZoneError } from "./time"
import type { NormalizationForm, UnicodeGeneralCategory } from "./unicode"
import type { Validation } from "./validation"

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

export const displayDepthLimit = 128
const depthLimitDocument = text("…")
let displayDepth = 0

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
  const boundedDocument = (value: Value): RenderDocument =>
    withDisplayDepth(() => document(value), depthLimitDocument)
  return Object.freeze({
    document: boundedDocument,
    show(value: Value): string {
      return renderDocument(boundedDocument(value))
    },
  })
}

function defineDebug<Value>(
  document: (value: Value) => RenderDocument
): Debug<Value> {
  const boundedDocument = (value: Value): RenderDocument =>
    withDisplayDepth(() => document(value), depthLimitDocument)
  return Object.freeze({
    document: boundedDocument,
    debug(value: Value): string {
      return renderDocument(boundedDocument(value))
    },
  })
}

/**
 * Wraps compiler-derived Show implementations with the shared recursion bound.
 * User-defined instances remain responsible for their own recursion policy.
 */
export function boundedShow<Value>(
  show: (value: Value) => string
): Show<Value> {
  return Object.freeze({
    show(value: Value): string {
      return withDisplayDepth(() => show(value), "…")
    },
  })
}

/** Wraps compiler-derived Debug implementations with the shared depth bound. */
export function boundedDebug<Value>(
  debug: (value: Value) => string
): Debug<Value> {
  return Object.freeze({
    debug(value: Value): string {
      return withDisplayDepth(() => debug(value), "…")
    },
  })
}

/** String Show is identity: user-facing output does not add quotes. */
export const stringShow = defineShow((value: string) => text(value))

/** Int Show uses the canonical signed base-10 spelling without separators. */
export const intShow = defineShow((value: number) => text(value.toString(10)))

/** Int Debug uses the same canonical spelling as Show. */
export const intDebug = defineDebug((value: number) => text(value.toString(10)))

/** BigInt Show uses the canonical signed base-10 spelling. */
export const bigIntShow = defineShow((value: SeseragiBigInt) =>
  text(value.toString(10))
)

/** BigInt Debug deliberately matches its canonical Show spelling. */
export const bigIntDebug = defineDebug((value: SeseragiBigInt) =>
  text(value.toString(10))
)

function bigIntParseErrorDocument(error: BigIntParseError): RenderDocument {
  switch (error.tag) {
    case "EmptyBigInt":
      return text(error.tag)
    case "InvalidBigIntRadix":
      return constructorDocument(error.tag, showDocument(intShow, error.value))
    case "InvalidBigIntDigit":
      return recordConstructorDocument(error.tag, [
        ["offset", String(error.value.offset)],
        ["radix", String(error.value.radix)],
      ])
  }
}

export const bigIntParseErrorShow = defineShow(bigIntParseErrorDocument)
export const bigIntParseErrorDebug = defineDebug(bigIntParseErrorDocument)

export const bigIntDivisionErrorShow = defineShow(
  (error: BigIntDivisionError) => text(error.tag)
)
export const bigIntDivisionErrorDebug = defineDebug(
  (error: BigIntDivisionError) => text(error.tag)
)

const bigIntPowerErrorDocument = (error: BigIntPowerError): RenderDocument =>
  constructorDocument(error.tag, showDocument(intShow, error.value))

export const bigIntPowerErrorShow = defineShow(bigIntPowerErrorDocument)
export const bigIntPowerErrorDebug = defineDebug(bigIntPowerErrorDocument)

export const bigIntConversionErrorShow = defineShow(
  (error: BigIntConversionError) => text(error.tag)
)
export const bigIntConversionErrorDebug = defineDebug(
  (error: BigIntConversionError) => text(error.tag)
)

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

export function nonEmptyListShow<Value>(
  element: ShowEvidence<Value>
): Show<NonEmptyList<Value>> {
  return defineShow((values) =>
    delimited(
      "`[",
      [
        showDocument(element, values.head),
        ...listDocuments(values.tail, element, showDocument),
      ],
      "]"
    )
  )
}

export function nonEmptyListDebug<Value>(
  element: DebugEvidence<Value>
): Debug<NonEmptyList<Value>> {
  return defineDebug((values) =>
    delimited(
      "`[",
      [
        debugDocument(element, values.head),
        ...listDocuments(values.tail, element, debugDocument),
      ],
      "]"
    )
  )
}

export function mapShow<K, V>(
  key: ShowEvidence<K>,
  value: ShowEvidence<V>
): Show<PersistentMap<K, V>> {
  return defineShow((values) =>
    constructorDocument(
      "Map",
      delimited(
        "[",
        mapEntries(values).map(([k, v]) =>
          delimited("(", [showDocument(key, k), showDocument(value, v)], ")")
        ),
        "]"
      )
    )
  )
}

export function mapDebug<K, V>(
  key: DebugEvidence<K>,
  value: DebugEvidence<V>
): Debug<PersistentMap<K, V>> {
  return defineDebug((values) =>
    constructorDocument(
      "Map",
      delimited(
        "[",
        mapEntries(values).map(([k, v]) =>
          delimited("(", [debugDocument(key, k), debugDocument(value, v)], ")")
        ),
        "]"
      )
    )
  )
}

export function setShow<A>(element: ShowEvidence<A>): Show<PersistentSet<A>> {
  return defineShow((values) =>
    constructorDocument(
      "Set",
      delimited(
        "[",
        setValues(values).map((value) => showDocument(element, value)),
        "]"
      )
    )
  )
}

export function setDebug<A>(
  element: DebugEvidence<A>
): Debug<PersistentSet<A>> {
  return defineDebug((values) =>
    constructorDocument(
      "Set",
      delimited(
        "[",
        setValues(values).map((value) => debugDocument(element, value)),
        "]"
      )
    )
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

export function validationShow<E, A>(
  error: ShowEvidence<E>,
  value: ShowEvidence<A>
): Show<Validation<E, A>> {
  const errors = nonEmptyListShow(error)
  return defineShow((validation) =>
    validation.tag === "Invalid"
      ? constructorDocument("Invalid", showDocument(errors, validation.value))
      : constructorDocument("Valid", showDocument(value, validation.value))
  )
}

export function validationDebug<E, A>(
  error: DebugEvidence<E>,
  value: DebugEvidence<A>
): Debug<Validation<E, A>> {
  const errors = nonEmptyListDebug(error)
  return defineDebug((validation) =>
    validation.tag === "Invalid"
      ? constructorDocument("Invalid", debugDocument(errors, validation.value))
      : constructorDocument("Valid", debugDocument(value, validation.value))
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

export const jsErrorShow = defineShow((error: JsError) =>
  text(`Js.Error(${error.phase}): ${error.message}`)
)

/**
 * Console failures cross a host boundary. Debug preserves the failure kind but
 * redacts the host-provided message so it cannot become an accidental secret
 * or stack-trace channel.
 */
export const consoleErrorDebug = defineDebug((_error: ConsoleError) =>
  text('ConsoleError { message: "<redacted>" }')
)

/** Source-like rendering for the standard Stdin failure ADT. */
export const stdinErrorShow = defineShow((error: StdinError) =>
  stdinErrorDocument(error)
)

export const stdinErrorDebug = defineDebug((error: StdinError) =>
  stdinErrorDocument(error)
)

export const stdinConfigErrorShow = defineShow((error: StdinConfigError) =>
  constructorDocument(error.tag, text(String(error.value)))
)

export const stdinConfigErrorDebug = defineDebug((error: StdinConfigError) =>
  constructorDocument(error.tag, text(String(error.value)))
)

export const logErrorShow = defineShow((error: LogError) =>
  text(`LogError: ${error.message}`)
)

export const logErrorDebug = defineDebug((_error: LogError) =>
  text('LogError { message: "<redacted>" }')
)

export const processSignalShow = defineShow((signal: ProcessSignal) =>
  text(signal.tag)
)

export const processSignalDebug = defineDebug((signal: ProcessSignal) =>
  text(signal.tag)
)

export const processErrorShow = defineShow((error: ProcessError) =>
  error.tag === "UnsupportedProcessSignal" ||
  error.tag === "ReservedProcessSignal"
    ? constructorDocument(
        error.tag,
        showDocument(processSignalShow, error.value)
      )
    : error.tag === "InvalidArgumentEncoding"
      ? constructorDocument(error.tag, showDocument(intShow, error.value))
      : error.tag === "InvalidEnvironmentName" ||
          error.tag === "InvalidEnvironmentEncoding"
        ? constructorDocument(error.tag, showDocument(stringShow, error.value))
        : text(error.tag)
)

export const processErrorDebug = defineDebug((error: ProcessError) =>
  error.tag === "UnsupportedProcessSignal" ||
  error.tag === "ReservedProcessSignal"
    ? constructorDocument(
        error.tag,
        debugDocument(processSignalDebug, error.value)
      )
    : error.tag === "InvalidArgumentEncoding"
      ? constructorDocument(error.tag, debugDocument(intDebug, error.value))
      : error.tag === "InvalidEnvironmentName" ||
          error.tag === "InvalidEnvironmentEncoding"
        ? constructorDocument(
            error.tag,
            debugDocument(stringDebug, error.value)
          )
        : text(error.tag)
)

export const childProcessConfigErrorShow = defineShow(
  childProcessConfigErrorDocument
)
export const childProcessConfigErrorDebug = defineDebug(
  childProcessConfigErrorDocument
)
export const childProcessErrorShow = defineShow(childProcessErrorDocument)
export const childProcessErrorDebug = defineDebug(childProcessErrorDocument)
export const childExitStatusShow = defineShow(childExitStatusDocument)
export const childExitStatusDebug = defineDebug(childExitStatusDocument)

const randomRangeErrorDocument = (error: RandomRangeError): RenderDocument =>
  error.tag === "InvalidProbability"
    ? constructorDocument(error.tag, showDocument(floatShow, error.value))
    : recordConstructorDocument(error.tag, [
        ["lower", String(error.value.lower)],
        ["upperExclusive", String(error.value.upperExclusive)],
      ])
const randomConfigErrorDocument = (error: RandomConfigError): RenderDocument =>
  constructorDocument(error.tag, showDocument(intShow, error.value))
const entropyConfigErrorDocument = (
  error: EntropyConfigError
): RenderDocument =>
  constructorDocument(error.tag, showDocument(intShow, error.value))
const entropyErrorDocument = (error: EntropyError): RenderDocument =>
  text(error.tag)

export const randomRangeErrorShow = defineShow(randomRangeErrorDocument)
export const randomRangeErrorDebug = defineDebug(randomRangeErrorDocument)
export const randomConfigErrorShow = defineShow(randomConfigErrorDocument)
export const randomConfigErrorDebug = defineDebug(randomConfigErrorDocument)
export const entropyConfigErrorShow = defineShow(entropyConfigErrorDocument)
export const entropyConfigErrorDebug = defineDebug(entropyConfigErrorDocument)
export const entropyErrorShow = defineShow(entropyErrorDocument)
export const entropyErrorDebug = defineDebug(entropyErrorDocument)

function childProcessConfigErrorDocument(
  error: ChildProcessConfigError
): RenderDocument {
  switch (error.tag) {
    case "EmptyExecutableName":
      return text(error.tag)
    case "ArgumentContainsNul":
      return recordConstructorDocument(error.tag, [
        ["index", String(error.value.index)],
        ["offset", String(error.value.offset)],
      ])
    default:
      return constructorDocument(
        error.tag,
        text(
          typeof error.value === "string"
            ? JSON.stringify(error.value)
            : String(error.value)
        )
      )
  }
}

function childProcessErrorDocument(error: ChildProcessError): RenderDocument {
  switch (error.tag) {
    case "ChildInputAfterClose":
      return text(error.tag)
    case "UnsupportedChildSignal":
      return constructorDocument(error.tag, text(error.value.tag))
    case "ChildInputFailed":
    case "ChildWaitFailed":
    case "ChildTerminationFailed":
      return constructorDocument(error.tag, text(JSON.stringify(error.value)))
    case "ChildSpawnFailed":
      return recordConstructorDocument(error.tag, [
        [
          "executable",
          error.value.executable.tag === "SearchPath"
            ? `SearchPath ${JSON.stringify(error.value.executable.value)}`
            : `ExecutablePath ${JSON.stringify(
                renderPath(error.value.executable.value)
              )}`,
        ],
        ["detail", JSON.stringify(error.value.detail)],
      ])
    case "ChildOutputReadFailed":
      return recordConstructorDocument(error.tag, [
        ["channel", error.value.channel.tag],
        ["detail", JSON.stringify(error.value.detail)],
      ])
    case "ChildOutputLimitExceeded":
      return recordConstructorDocument(error.tag, [
        ["channel", error.value.channel.tag],
        ["limitBytes", String(error.value.limitBytes)],
      ])
  }
}

function childExitStatusDocument(status: ChildExitStatus): RenderDocument {
  switch (status.tag) {
    case "ChildExited":
      return constructorDocument(status.tag, text(String(status.value)))
    case "ChildSignaled":
      return constructorDocument(status.tag, text(status.value.tag))
    case "ChildHostTerminated":
      return constructorDocument(status.tag, text(JSON.stringify(status.value)))
  }
}

export const byteErrorShow = defineShow((error: ByteError) =>
  constructorDocument("ByteOutOfRange", text(String(error.value)))
)

export const byteErrorDebug = defineDebug((error: ByteError) =>
  constructorDocument("ByteOutOfRange", text(String(error.value)))
)

export const bytesSliceErrorShow = defineShow((error: BytesSliceError) =>
  bytesSliceErrorDocument(error)
)

export const bytesSliceErrorDebug = defineDebug((error: BytesSliceError) =>
  bytesSliceErrorDocument(error)
)

function bytesSliceErrorDocument(error: BytesSliceError): RenderDocument {
  return delimited(
    "InvalidByteRange {",
    [
      text(`start: ${error.value.start}`),
      text(`end: ${error.value.end}`),
      text(`length: ${error.value.length}`),
    ],
    "}",
    ",",
    true
  )
}

function offsetErrorDocument(tag: string, offset: number): RenderDocument {
  return delimited(`${tag} {`, [text(`offset: ${offset}`)], "}", ",", true)
}

function hexDecodeErrorDocument(error: HexDecodeError): RenderDocument {
  return error.tag === "OddHexLength"
    ? constructorDocument(error.tag, text(String(error.value)))
    : offsetErrorDocument(error.tag, error.value.offset)
}

export const hexDecodeErrorShow = defineShow((error: HexDecodeError) =>
  hexDecodeErrorDocument(error)
)

export const hexDecodeErrorDebug = defineDebug((error: HexDecodeError) =>
  hexDecodeErrorDocument(error)
)

function base64DecodeErrorDocument(error: Base64DecodeError): RenderDocument {
  return error.tag === "InvalidBase64Length"
    ? constructorDocument(error.tag, text(String(error.value)))
    : offsetErrorDocument(error.tag, error.value.offset)
}

export const base64DecodeErrorShow = defineShow((error: Base64DecodeError) =>
  base64DecodeErrorDocument(error)
)

export const base64DecodeErrorDebug = defineDebug((error: Base64DecodeError) =>
  base64DecodeErrorDocument(error)
)

export const utf8DecodeErrorShow = defineShow((error: Utf8DecodeError) =>
  utf8DecodeErrorDocument(error)
)

export const utf8DecodeErrorDebug = defineDebug((error: Utf8DecodeError) =>
  utf8DecodeErrorDocument(error)
)

export const scheduleErrorShow = defineShow((error: ScheduleError) =>
  constructorDocument(error.tag, showDocument(intShow, error.value))
)

export const scheduleErrorDebug = defineDebug((error: ScheduleError) =>
  constructorDocument(error.tag, debugDocument(intDebug, error.value))
)

export const parallelismErrorShow = defineShow((error: ParallelismError) =>
  constructorDocument(error.tag, showDocument(intShow, error.value))
)

export const parallelismErrorDebug = defineDebug((error: ParallelismError) =>
  constructorDocument(error.tag, debugDocument(intDebug, error.value))
)

export const sizeErrorShow = defineShow((error: SizeError) =>
  constructorDocument(error.tag, showDocument(intShow, error.value))
)

export const sizeErrorDebug = defineDebug((error: SizeError) =>
  constructorDocument(error.tag, debugDocument(intDebug, error.value))
)

export const bufferCapacityErrorShow = defineShow(
  (error: BufferCapacityError) =>
    constructorDocument(error.tag, showDocument(intShow, error.value))
)

export const bufferCapacityErrorDebug = defineDebug(
  (error: BufferCapacityError) =>
    constructorDocument(error.tag, debugDocument(intDebug, error.value))
)

export const queueCreateErrorShow = defineShow((error: QueueCreateError) =>
  constructorDocument(error.tag, showDocument(intShow, error.value))
)

export const queueCreateErrorDebug = defineDebug((error: QueueCreateError) =>
  constructorDocument(error.tag, debugDocument(intDebug, error.value))
)

export const queueClosedShow = defineShow((error: QueueClosed) =>
  text(error.tag)
)

export const queueClosedDebug = defineDebug((error: QueueClosed) =>
  text(error.tag)
)

export const semaphoreCreateErrorShow = defineShow(
  (error: SemaphoreCreateError) =>
    constructorDocument(error.tag, showDocument(intShow, error.value))
)

export const semaphoreCreateErrorDebug = defineDebug(
  (error: SemaphoreCreateError) =>
    constructorDocument(error.tag, debugDocument(intDebug, error.value))
)

export const durationErrorShow = defineShow((error: DurationError) =>
  error.tag === "NegativeDuration"
    ? constructorDocument(error.tag, showDocument(intShow, error.value))
    : text(error.tag)
)

export const durationErrorDebug = defineDebug((error: DurationError) =>
  error.tag === "NegativeDuration"
    ? constructorDocument(error.tag, debugDocument(intDebug, error.value))
    : text(error.tag)
)

function dateTimeErrorDocument(error: DateTimeError): RenderDocument {
  switch (error.tag) {
    case "InvalidDate":
      return text(
        `InvalidDate { year: ${error.value.year}, month: ${error.value.month}, day: ${error.value.day} }`
      )
    case "InvalidTime":
      return text(
        `InvalidTime { hour: ${error.value.hour}, minute: ${error.value.minute}, second: ${error.value.second}, nanosecond: ${error.value.nanosecond} }`
      )
    case "InvalidUtcOffsetSeconds":
      return text(`InvalidUtcOffsetSeconds ${error.value}`)
    case "InvalidDateTimeText":
      return text(`InvalidDateTimeText { offset: ${error.value.offset} }`)
  }
}

function timeZoneErrorDocument(error: TimeZoneError): RenderDocument {
  switch (error.tag) {
    case "UnknownTimeZone":
      return text(`UnknownTimeZone ${JSON.stringify(error.value)}`)
    case "TimeZoneDatabaseUnavailable":
      return text(`TimeZoneDatabaseUnavailable ${JSON.stringify(error.value)}`)
    case "TimeZoneDatabaseVersionMismatch":
      return text(
        `TimeZoneDatabaseVersionMismatch { required: ${JSON.stringify(error.value.required)}, actual: ${JSON.stringify(error.value.actual)} }`
      )
  }
}

export const dateTimeErrorShow = defineShow(dateTimeErrorDocument)
export const dateTimeErrorDebug = defineDebug(dateTimeErrorDocument)
export const timeZoneErrorShow = defineShow(timeZoneErrorDocument)
export const timeZoneErrorDebug = defineDebug(timeZoneErrorDocument)

export const pathErrorShow = defineShow(pathErrorDocument)
export const pathErrorDebug = defineDebug(pathErrorDocument)
export const fileTypeShow = defineShow((value: FileType) => text(value.tag))
export const fileTypeDebug = defineDebug((value: FileType) => text(value.tag))
export const fileSystemOperationShow = defineShow(
  (value: FileSystemOperation) => text(value.tag)
)
export const fileSystemOperationDebug = defineDebug(
  (value: FileSystemOperation) => text(value.tag)
)
export const fileSystemErrorKindShow = defineShow(fileSystemErrorKindDocument)
export const fileSystemErrorKindDebug = defineDebug(fileSystemErrorKindDocument)
export const fileSystemErrorShow = defineShow(fileSystemErrorDocument)
export const fileSystemErrorDebug = defineDebug(fileSystemErrorDocument)
export const fileMetadataShow = defineShow(fileMetadataDocument)
export const fileMetadataDebug = defineDebug(fileMetadataDocument)
export const directoryEntryShow = defineShow(directoryEntryDocument)
export const directoryEntryDebug = defineDebug(directoryEntryDocument)
export const writeModeShow = defineShow((value: WriteMode) => text(value.tag))
export const writeModeDebug = defineDebug((value: WriteMode) => text(value.tag))
export const fileTextErrorShow = defineShow(fileTextErrorDocument)
export const fileTextErrorDebug = defineDebug(fileTextErrorDocument)

function pathErrorDocument(error: PathError): RenderDocument {
  switch (error.tag) {
    case "PathContainsNul":
    case "PathContainsBackslash":
      return recordConstructorDocument(error.tag, [
        ["offset", String(error.value.offset)],
      ])
    case "InvalidPathSegment":
      return constructorDocument(error.tag, text(JSON.stringify(error.value)))
    default:
      return text(error.tag)
  }
}

function fileSystemErrorKindDocument(
  value: FileSystemErrorKind
): RenderDocument {
  return value.tag === "OtherFileSystemError"
    ? constructorDocument(value.tag, text(JSON.stringify(value.value)))
    : text(value.tag)
}

function fileSystemErrorDocument(value: FileSystemError): RenderDocument {
  return recordConstructorDocument("FileSystemError", [
    ["operation", value.operation.tag],
    ["path", JSON.stringify(renderPath(value.path))],
    [
      "otherPath",
      value.otherPath.tag === "Nothing"
        ? "Nothing"
        : `Just ${JSON.stringify(renderPath(value.otherPath.value))}`,
    ],
    [
      "kind",
      value.kind.tag === "OtherFileSystemError"
        ? `${value.kind.tag} ${JSON.stringify(value.kind.value)}`
        : value.kind.tag,
    ],
  ])
}

function fileMetadataDocument(value: FileMetadata): RenderDocument {
  return recordConstructorDocument("FileMetadata", [
    ["fileType", value.fileType.tag],
    ["sizeBytes", String(value.sizeBytes)],
    ["modified", value.modified.tag],
    ["created", value.created.tag],
  ])
}

function directoryEntryDocument(value: DirectoryEntry): RenderDocument {
  return recordConstructorDocument("DirectoryEntry", [
    ["name", JSON.stringify(value.name)],
    ["path", JSON.stringify(renderPath(value.path))],
    [
      "fileType",
      value.fileType.tag === "Nothing"
        ? "Nothing"
        : `Just ${value.fileType.value.tag}`,
    ],
  ])
}

function fileTextErrorDocument(value: FileTextError): RenderDocument {
  return value.tag === "FileAccessFailure"
    ? constructorDocument(value.tag, fileSystemErrorDocument(value.value))
    : constructorDocument(value.tag, utf8DecodeErrorDocument(value.value))
}

export const httpBuildErrorShow = defineShow((error: HttpBuildError) =>
  httpBuildErrorDocument(error)
)

export const httpBuildErrorDebug = defineDebug((error: HttpBuildError) =>
  httpBuildErrorDocument(error)
)

export const httpErrorShow = defineShow((error: HttpError) =>
  httpErrorDocument(error)
)

export const httpErrorDebug = defineDebug((error: HttpError) =>
  httpErrorDocument(error)
)

export const urlBuildErrorShow = defineShow((error: UrlBuildError) =>
  urlBuildErrorDocument(error)
)

export const urlBuildErrorDebug = defineDebug((error: UrlBuildError) =>
  urlBuildErrorDocument(error)
)

export const navigationErrorShow = defineShow((error: NavigationError) =>
  navigationErrorDocument(error)
)

export const navigationErrorDebug = defineDebug((error: NavigationError) =>
  navigationErrorDocument(error)
)

export const storageAreaShow = defineShow((area: StorageArea) => text(area.tag))

export const storageAreaDebug = defineDebug((area: StorageArea) =>
  text(area.tag)
)

export const storageErrorShow = defineShow((error: StorageError) =>
  storageErrorDocument(error)
)

export const storageErrorDebug = defineDebug((error: StorageError) =>
  storageErrorDocument(error)
)

function urlBuildErrorDocument(error: UrlBuildError): RenderDocument {
  switch (error.tag) {
    case "UrlContainsUserInfo":
      return text(error.tag)
    case "InvalidUrl":
    case "InvalidPercentEncoding":
      return recordConstructorDocument(error.tag, [
        ["offset", String(error.value.offset)],
      ])
    case "UnsupportedUrlScheme":
      return constructorDocument(error.tag, text(JSON.stringify(error.value)))
  }
}

function navigationErrorDocument(error: NavigationError): RenderDocument {
  switch (error.tag) {
    case "CrossOriginNavigation":
      return recordConstructorDocument(error.tag, [
        ["expected", JSON.stringify(error.value.expected)],
        ["actual", JSON.stringify(error.value.actual)],
      ])
    case "NavigationUnavailable":
    case "NavigationSecurityFailure":
      return constructorDocument(error.tag, text(JSON.stringify(error.value)))
  }
}

function storageErrorDocument(error: StorageError): RenderDocument {
  const area = error.value.area.tag
  switch (error.tag) {
    case "StorageQuotaExceeded":
      return recordConstructorDocument(error.tag, [
        ["area", area],
        ["key", JSON.stringify(error.value.key)],
        ["message", JSON.stringify(error.value.message)],
      ])
    case "StorageSecurityFailure":
    case "StorageUnavailable":
      return recordConstructorDocument(error.tag, [
        ["area", area],
        ["message", JSON.stringify(error.value.message)],
      ])
  }
}

function httpBuildErrorDocument(error: HttpBuildError): RenderDocument {
  switch (error.tag) {
    case "HttpUrlContainsUserInfo":
    case "HttpUrlContainsFragment":
      return text(error.tag)
    case "InvalidHttpUrl":
      return recordConstructorDocument(error.tag, [
        ["offset", String(error.value.offset)],
      ])
    case "InvalidHeaderValue":
      return recordConstructorDocument(error.tag, [
        ["name", JSON.stringify(error.value.name)],
        ["offset", String(error.value.offset)],
      ])
    case "InvalidHttpStatus":
    case "InvalidHttpBodyLimit":
      return constructorDocument(error.tag, text(String(error.value)))
    case "UnsupportedHttpScheme":
    case "InvalidHttpMethod":
    case "InvalidHeaderName":
    case "ManagedHttpHeader":
      return constructorDocument(error.tag, text(JSON.stringify(error.value)))
  }
}

function httpErrorDocument(error: HttpError): RenderDocument {
  switch (error.tag) {
    case "HttpClientUnavailable":
      return text(error.tag)
    case "HttpRequestLengthMismatch":
      return recordConstructorDocument(error.tag, [
        ["declared", String(error.value.declared)],
        ["actual", String(error.value.actual)],
      ])
    case "HttpResponseBodyLimitExceeded":
      return recordConstructorDocument(error.tag, [
        ["limitBytes", String(error.value.limitBytes)],
      ])
    case "HttpDnsFailure":
    case "HttpConnectionFailure":
    case "HttpTlsFailure":
    case "HttpProtocolFailure":
    case "HttpRequestBodyFailure":
      return constructorDocument(error.tag, text(JSON.stringify(error.value)))
  }
}

function recordConstructorDocument(
  name: string,
  fields: ReadonlyArray<readonly [string, string]>
): RenderDocument {
  return delimited(
    `${name} {`,
    fields.map(([field, value]) => text(`${field}: ${value}`)),
    "}",
    ",",
    true
  )
}

function textRangeErrorDocument(
  error: TextSliceError | GraphemeSliceError
): RenderDocument {
  return recordConstructorDocument(error.tag, [
    ["start", String(error.value.start)],
    ["end", String(error.value.end)],
    ["length", String(error.value.length)],
  ])
}

export const textSliceErrorShow = defineShow<TextSliceError>(
  textRangeErrorDocument
)
export const textSliceErrorDebug = defineDebug<TextSliceError>(
  textRangeErrorDocument
)
export const graphemeSliceErrorShow = defineShow<GraphemeSliceError>(
  textRangeErrorDocument
)
export const graphemeSliceErrorDebug = defineDebug<GraphemeSliceError>(
  textRangeErrorDocument
)
export const normalizationFormShow = defineShow((value: NormalizationForm) =>
  text(value.tag)
)
export const normalizationFormDebug = defineDebug((value: NormalizationForm) =>
  text(value.tag)
)
export const unicodeGeneralCategoryShow = defineShow(
  (value: UnicodeGeneralCategory) => text(value.tag)
)
export const unicodeGeneralCategoryDebug = defineDebug(
  (value: UnicodeGeneralCategory) => text(value.tag)
)

function recordFieldsDocument(
  name: string,
  fields: ReadonlyArray<readonly [string, RenderDocument]>
): RenderDocument {
  return delimited(
    `${name} {`,
    fields.map(([field, value]) => concat([text(`${field}: `), value])),
    "}",
    ",",
    true
  )
}

function regexCompileErrorKindDocument(
  value: RegexCompileErrorKind,
  debug: boolean
): RenderDocument {
  switch (value.tag) {
    case "UnexpectedRegexEnd":
    case "InvalidRegexEscape":
    case "InvalidRegexRange":
    case "InvalidRegexQuantifier":
      return text(value.tag)
    case "UnexpectedRegexToken":
      return constructorDocument(
        value.tag,
        debug
          ? debugDocument(charDebug, value.value)
          : showDocument(charShow, value.value)
      )
    case "DuplicateCaptureName":
    case "UnsupportedRegexFeature":
      return constructorDocument(
        value.tag,
        debug
          ? debugDocument(stringDebug, value.value)
          : showDocument(stringShow, value.value)
      )
  }
}

export const regexCompileErrorKindShow = defineShow(
  (value: RegexCompileErrorKind) => regexCompileErrorKindDocument(value, false)
)
export const regexCompileErrorKindDebug = defineDebug(
  (value: RegexCompileErrorKind) => regexCompileErrorKindDocument(value, true)
)

function regexCompileErrorDocument(
  value: RegexCompileError,
  debug: boolean
): RenderDocument {
  return recordFieldsDocument("RegexCompileError", [
    ["kind", regexCompileErrorKindDocument(value.kind, debug)],
    ["offset", text(String(value.offset))],
  ])
}

export const regexCompileErrorShow = defineShow((value: RegexCompileError) =>
  regexCompileErrorDocument(value, false)
)
export const regexCompileErrorDebug = defineDebug((value: RegexCompileError) =>
  regexCompileErrorDocument(value, true)
)

function regexOptionsDocument(value: RegexOptions): RenderDocument {
  return recordFieldsDocument("RegexOptions", [
    ["caseInsensitive", text(value.caseInsensitive ? "True" : "False")],
    ["multiline", text(value.multiline ? "True" : "False")],
    ["dotMatchesNewline", text(value.dotMatchesNewline ? "True" : "False")],
  ])
}

export const regexOptionsShow = defineShow(regexOptionsDocument)
export const regexOptionsDebug = defineDebug(regexOptionsDocument)

function regexSpanDocument(value: RegexSpan): RenderDocument {
  return recordFieldsDocument("RegexSpan", [
    ["start", text(String(value.start))],
    ["end", text(String(value.end))],
  ])
}

export const regexSpanShow = defineShow(regexSpanDocument)
export const regexSpanDebug = defineDebug(regexSpanDocument)

function regexCaptureDocument(
  value: RegexCapture,
  debug: boolean
): RenderDocument {
  return recordFieldsDocument("RegexCapture", [
    ["span", regexSpanDocument(value.span)],
    [
      "text",
      debug
        ? debugDocument(stringDebug, value.text)
        : showDocument(stringShow, value.text),
    ],
  ])
}

export const regexCaptureShow = defineShow((value: RegexCapture) =>
  regexCaptureDocument(value, false)
)
export const regexCaptureDebug = defineDebug((value: RegexCapture) =>
  regexCaptureDocument(value, true)
)

function regexMatchDocument(value: RegexMatch, debug: boolean): RenderDocument {
  const capture = debug ? regexCaptureDebug : regexCaptureShow
  const maybeCapture = debug ? maybeDebug(capture) : maybeShow(capture)
  const captures = debug ? arrayDebug(maybeCapture) : arrayShow(maybeCapture)
  const named = debug
    ? mapDebug(stringDebug, maybeCapture)
    : mapShow(stringShow, maybeCapture)
  return recordFieldsDocument("RegexMatch", [
    ["span", regexSpanDocument(value.span)],
    [
      "text",
      debug
        ? debugDocument(stringDebug, value.text)
        : showDocument(stringShow, value.text),
    ],
    [
      "captures",
      debug
        ? debugDocument(
            captures as Debug<ReadonlyArray<Maybe<RegexCapture>>>,
            value.captures
          )
        : showDocument(
            captures as Show<ReadonlyArray<Maybe<RegexCapture>>>,
            value.captures
          ),
    ],
    [
      "named",
      debug
        ? debugDocument(
            named as Debug<PersistentMap<string, Maybe<RegexCapture>>>,
            value.named
          )
        : showDocument(
            named as Show<PersistentMap<string, Maybe<RegexCapture>>>,
            value.named
          ),
    ],
  ])
}

export const regexMatchShow = defineShow((value: RegexMatch) =>
  regexMatchDocument(value, false)
)
export const regexMatchDebug = defineDebug((value: RegexMatch) =>
  regexMatchDocument(value, true)
)

function utf8DecodeErrorDocument(error: Utf8DecodeError): RenderDocument {
  return delimited(
    "InvalidUtf8 {",
    [text(`offset: ${error.value.offset}`)],
    "}",
    ",",
    true
  )
}

function stdinErrorDocument(error: StdinError): RenderDocument {
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
}

export const domErrorShow = defineShow((error: DomError) =>
  domErrorDocument(error, (value) => showDocument(stringShow, value))
)

export const domErrorDebug = defineDebug((error: DomError) =>
  domErrorDocument(error, (value) => debugDocument(stringDebug, value))
)

export function domRuntimeErrorShow<Failure>(
  failure: ShowEvidence<Failure>
): Show<DomRuntimeError<Failure>> {
  return defineShow((error) =>
    error.tag === "DomFailure"
      ? constructorDocument(
          "DomFailure",
          showDocument(domErrorShow, error.value)
        )
      : constructorDocument(
          "DispatchFailure",
          showDocument(failure, error.value)
        )
  )
}

export function domRuntimeErrorDebug<Failure>(
  failure: DebugEvidence<Failure>
): Debug<DomRuntimeError<Failure>> {
  return defineDebug((error) =>
    error.tag === "DomFailure"
      ? constructorDocument(
          "DomFailure",
          debugDocument(domErrorDebug, error.value)
        )
      : constructorDocument(
          "DispatchFailure",
          debugDocument(failure, error.value)
        )
  )
}

export const htmlBuildErrorShow = defineShow((error: HtmlBuildError) =>
  constructorDocument(error.tag, showDocument(stringShow, error.value))
)

export const htmlBuildErrorDebug = defineDebug((error: HtmlBuildError) =>
  constructorDocument(error.tag, debugDocument(stringDebug, error.value))
)

function domErrorDocument(
  error: DomError,
  renderString: (value: string) => RenderDocument
): RenderDocument {
  switch (error.tag) {
    case "DomTargetAlreadyMounted":
    case "DomTargetRemoved":
      return text(error.tag)
    case "InvalidSelector":
    case "DomTargetNotFound":
    case "DomOperationFailed":
      return constructorDocument(error.tag, renderString(error.value))
    case "HydrationMismatch":
      return constructorDocument(
        error.tag,
        delimited(
          "{",
          [
            concat([text("path: "), text(`[${error.value.path.join(", ")}]`)]),
            concat([text("expected: "), renderString(error.value.expected)]),
            concat([text("actual: "), renderString(error.value.actual)]),
          ],
          "}",
          ",",
          true
        )
      )
    case "DomEventQueueOverflow":
      return constructorDocument(error.tag, showDocument(intShow, error.value))
  }
}

function showDocument<Value>(
  evidence: ShowEvidence<Value>,
  value: Value
): RenderDocument {
  const instance = evidence as Show<Value>
  return instance.document?.(value) ?? text(instance.show(value))
}

function withDisplayDepth<Value>(render: () => Value, fallback: Value): Value {
  if (displayDepth >= displayDepthLimit) {
    return fallback
  }
  displayDepth += 1
  try {
    return render()
  } finally {
    displayDepth -= 1
  }
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

export function canonicalFloat(value: number): string {
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
