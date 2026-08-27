import { type Bytes, fromUint8Array } from "./bytes"
import {
  type Effect,
  type EffectContext,
  throwIfCancelled,
  type Unit,
} from "./effect"
import { MAX_INT } from "./int"
import {
  type ServiceOperation,
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"
import { fromPull, type Stream } from "./stream"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"

const readSizeBrand: unique symbol = Symbol("seseragi.stdin-read-size")
const lineLimitBrand: unique symbol = Symbol("seseragi.stdin-line-limit")

export const MAX_READ_SIZE = 1024 * 1024
export const MAX_LINE_LIMIT = 64 * 1024 * 1024
export const DEFAULT_READ_SIZE = 64 * 1024
export const DEFAULT_LINE_LIMIT = 1024 * 1024

export type ReadSize = Readonly<{
  readonly [readSizeBrand]: true
  readonly value: number
}>

export type LineLimit = Readonly<{
  readonly [lineLimitBrand]: true
  readonly value: number
}>

export type StdinConfigError =
  | Readonly<{ readonly tag: "NonPositiveReadSize"; readonly value: number }>
  | Readonly<{ readonly tag: "ReadSizeTooLarge"; readonly value: number }>
  | Readonly<{ readonly tag: "NonPositiveLineLimit"; readonly value: number }>
  | Readonly<{ readonly tag: "LineLimitTooLarge"; readonly value: number }>

export const NonPositiveReadSize = (value: number): StdinConfigError => ({
  tag: "NonPositiveReadSize",
  value,
})

export const ReadSizeTooLarge = (value: number): StdinConfigError => ({
  tag: "ReadSizeTooLarge",
  value,
})

export const NonPositiveLineLimit = (value: number): StdinConfigError => ({
  tag: "NonPositiveLineLimit",
  value,
})

export const LineLimitTooLarge = (value: number): StdinConfigError => ({
  tag: "LineLimitTooLarge",
  value,
})

export type StdinUnavailable = Readonly<{
  readonly tag: "StdinUnavailable"
}>

export type StdinReadFailure = Readonly<{
  readonly tag: "StdinReadFailure"
}>

export type ConcurrentStdinRead = Readonly<{
  readonly tag: "ConcurrentStdinRead"
}>

export type InvalidStdinUtf8 = Readonly<{
  readonly tag: "InvalidStdinUtf8"
  readonly value: Readonly<{ readonly offset: number }>
}>

export type StdinLineTooLong = Readonly<{
  readonly tag: "StdinLineTooLong"
  readonly value: Readonly<{ readonly limitBytes: number }>
}>

export type StdinPositionOverflow = Readonly<{
  readonly tag: "StdinPositionOverflow"
}>

export type StdinError =
  | StdinUnavailable
  | StdinReadFailure
  | ConcurrentStdinRead
  | InvalidStdinUtf8
  | StdinLineTooLong
  | StdinPositionOverflow

export const StdinUnavailable: StdinUnavailable = Object.freeze({
  tag: "StdinUnavailable",
})

export const StdinReadFailure: StdinReadFailure = Object.freeze({
  tag: "StdinReadFailure",
})

export const ConcurrentStdinRead: ConcurrentStdinRead = Object.freeze({
  tag: "ConcurrentStdinRead",
})

export const InvalidStdinUtf8 = (
  value: Readonly<{ readonly offset: number }>
): InvalidStdinUtf8 => ({ tag: "InvalidStdinUtf8", value })

export const StdinLineTooLong = (
  value: Readonly<{ readonly limitBytes: number }>
): StdinLineTooLong => ({ tag: "StdinLineTooLong", value })

export const StdinPositionOverflow: StdinPositionOverflow = Object.freeze({
  tag: "StdinPositionOverflow",
})

export type Stdin = Readonly<{
  readChunk: (
    size: ReadSize,
    context: EffectContext
  ) => ServiceOperation<StdinError, Maybe<Bytes>>
  readLine: (
    limit: LineLimit,
    context: EffectContext
  ) => ServiceOperation<StdinError, Maybe<string>>
}>

export type StdinEnvironment = Readonly<{
  readonly stdin: Stdin
}>

export type StdinByteSource = Readonly<{
  read: (
    size: number,
    context: EffectContext
  ) => ServiceOperation<StdinError, Maybe<Uint8Array>>
}>

export type ByteStdinOptions = Readonly<{
  readonly initialOffset?: number
}>

export function readSize(bytes: number): Either<StdinConfigError, ReadSize> {
  if (!Number.isSafeInteger(bytes) || bytes <= 0) {
    return Left(NonPositiveReadSize(bytes))
  }
  if (bytes > MAX_READ_SIZE) return Left(ReadSizeTooLarge(bytes))
  return Right(Object.freeze({ [readSizeBrand]: true as const, value: bytes }))
}

export function lineLimit(bytes: number): Either<StdinConfigError, LineLimit> {
  if (!Number.isSafeInteger(bytes) || bytes <= 0) {
    return Left(NonPositiveLineLimit(bytes))
  }
  if (bytes > MAX_LINE_LIMIT) return Left(LineLimitTooLarge(bytes))
  return Right(Object.freeze({ [lineLimitBrand]: true as const, value: bytes }))
}

export function defaultReadSize(_unit?: Unit): ReadSize {
  return Object.freeze({
    [readSizeBrand]: true as const,
    value: DEFAULT_READ_SIZE,
  })
}

export function defaultLineLimit(_unit?: Unit): LineLimit {
  return Object.freeze({
    [lineLimitBrand]: true as const,
    value: DEFAULT_LINE_LIMIT,
  })
}

export function readChunk(
  size: ReadSize
): Effect<StdinEnvironment, StdinError, Maybe<Bytes>> {
  return serviceEffect((environment: StdinEnvironment, context) =>
    environment.stdin.readChunk(size, context)
  )
}

/** Reads one default-limited line from the supplied Stdin service. */
export function readLine(): Effect<
  StdinEnvironment,
  StdinError,
  Maybe<string>
> {
  return readLineWith(defaultLineLimit())
}

export function readLineWith(
  limit: LineLimit
): Effect<StdinEnvironment, StdinError, Maybe<string>> {
  return serviceEffect((environment: StdinEnvironment, context) =>
    environment.stdin.readLine(limit, context)
  )
}

export function lines(
  limit: LineLimit
): Stream<StdinEnvironment, StdinError, string> {
  return fromPull<StdinEnvironment, StdinError, string>(
    async (environment, context) => ({
      async pull() {
        const value = await readLineWith(limit)(environment, context)
        if (value.tag === "Nothing") {
          return { done: true, value: undefined }
        }
        return { done: false, value: value.value }
      },
      close() {
        // A terminal owns only its active read lease, never the shared cursor.
      },
    })
  )
}

/** Builds the canonical shared cursor over a host-owned byte source. */
export function createByteStdin(
  source: StdinByteSource,
  options: ByteStdinOptions = {}
): Stdin {
  let buffered = new Uint8Array()
  let offset = options.initialOffset ?? 0
  let eof = false
  let active = false
  let overflow = false

  const consume = (count: number): StdinPositionOverflow | undefined => {
    if (offset > MAX_INT - count) {
      overflow = true
      buffered = new Uint8Array()
      return StdinPositionOverflow
    }
    offset += count
    buffered = buffered.subarray(count)
    return undefined
  }

  const fill = async (
    size: number,
    context: EffectContext
  ): Promise<ServiceResult<StdinError, boolean>> => {
    if (buffered.length > 0) return serviceSuccess(true)
    if (eof) return serviceSuccess(false)
    const result = await source.read(size, context)
    if (result.kind === "failure") return result
    if (result.value.tag === "Nothing") {
      eof = true
      return serviceSuccess(false)
    }
    const chunk = result.value.value
    if (chunk.length === 0) {
      throw new TypeError("stdin byte source returned an empty chunk")
    }
    buffered = new Uint8Array(chunk)
    if (context.signal.aborted) {
      // A host chunk that lost the cancellation race remains at the front.
      throwIfCancelled(context)
    }
    return serviceSuccess(true)
  }

  const withLease = async <Value>(
    operation: () => Promise<ServiceResult<StdinError, Value>>
  ): Promise<ServiceResult<StdinError, Value>> => {
    if (overflow) return serviceFailure(StdinPositionOverflow)
    if (active) return serviceFailure(ConcurrentStdinRead)
    active = true
    try {
      return await operation()
    } finally {
      active = false
    }
  }

  return Object.freeze({
    readChunk(size, context) {
      return withLease<Maybe<Bytes>>(async () => {
        const available = await fill(size.value, context)
        if (available.kind === "failure") return available
        if (!available.value) return serviceSuccess(Nothing)
        const count = Math.min(size.value, buffered.length)
        const value = fromUint8Array(buffered.subarray(0, count))
        const failure = consume(count)
        return failure === undefined
          ? serviceSuccess(Just(value))
          : serviceFailure(failure)
      })
    },
    readLine(limit, context) {
      return withLease<Maybe<string>>(async () => {
        const startOffset = offset
        const parts: Uint8Array[] = []
        let length = 0
        let terminated = false
        let tooLong = false

        while (true) {
          const available = await fill(DEFAULT_READ_SIZE, context)
          if (available.kind === "failure") return available
          if (!available.value) break

          const newline = buffered.indexOf(0x0a)
          const count = newline < 0 ? buffered.length : newline + 1
          const segment = buffered.subarray(0, newline < 0 ? count : newline)
          if (!tooLong) {
            if (length + segment.length <= limit.value + 1) {
              parts.push(new Uint8Array(segment))
              length += segment.length
            } else {
              tooLong = true
              parts.length = 0
            }
          }
          const failure = consume(count)
          if (failure !== undefined) return serviceFailure(failure)
          if (newline >= 0) {
            terminated = true
            break
          }
        }

        if (!terminated && length === 0 && !tooLong) {
          return serviceSuccess(Nothing)
        }
        if (tooLong) {
          return serviceFailure(StdinLineTooLong({ limitBytes: limit.value }))
        }

        let bytes = concatenate(parts, length)
        if (terminated && bytes.at(-1) === 0x0d) {
          bytes = bytes.subarray(0, bytes.length - 1)
        }
        if (bytes.length > limit.value) {
          return serviceFailure(StdinLineTooLong({ limitBytes: limit.value }))
        }
        const invalid = invalidUtf8Offset(bytes)
        if (invalid !== undefined) {
          return serviceFailure(
            InvalidStdinUtf8({ offset: startOffset + invalid })
          )
        }
        return serviceSuccess(
          Just(new TextDecoder("utf-8", { fatal: true }).decode(bytes))
        )
      })
    },
  })
}

function concatenate(parts: readonly Uint8Array[], length: number): Uint8Array {
  if (parts.length === 1) return parts[0] as Uint8Array
  const result = new Uint8Array(length)
  let cursor = 0
  for (const part of parts) {
    result.set(part, cursor)
    cursor += part.length
  }
  return result
}

function invalidUtf8Offset(bytes: Uint8Array): number | undefined {
  let index = 0
  while (index < bytes.length) {
    const first = bytes[index] as number
    if (first <= 0x7f) {
      index += 1
      continue
    }
    const width =
      first >= 0xc2 && first <= 0xdf
        ? 2
        : first >= 0xe0 && first <= 0xef
          ? 3
          : first >= 0xf0 && first <= 0xf4
            ? 4
            : 0
    if (width === 0 || index + width > bytes.length) return index
    const second = bytes[index + 1] as number
    if (second < 0x80 || second > 0xbf) return index
    if (first === 0xe0 && second < 0xa0) return index
    if (first === 0xed && second > 0x9f) return index
    if (first === 0xf0 && second < 0x90) return index
    if (first === 0xf4 && second > 0x8f) return index
    for (let continuation = 2; continuation < width; continuation += 1) {
      const value = bytes[index + continuation] as number
      if (value < 0x80 || value > 0xbf) return index
    }
    index += width
  }
  return undefined
}
