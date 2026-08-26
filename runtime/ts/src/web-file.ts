import { type Bytes, fromUint8Array, toUint8Array } from "./bytes"
import {
  createEffectExecution,
  type Effect,
  fail,
  throwIfCancelled,
} from "./effect"
import { type Body, streamBody } from "./http-client"
import { fromPull, type Stream } from "./stream"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"

const BLOB = Symbol("seseragi.web-blob")
const FILE = Symbol("seseragi.web-file")
const MAX_CHUNK_BYTES = 64 * 1024
type EmptyEnvironment = Readonly<Record<string, never>>

export type Blob = Readonly<{
  readonly [BLOB]: globalThis.Blob
}>

export type File = Blob &
  Readonly<{
    readonly [FILE]: globalThis.File
    readonly name: string
    readonly lastModifiedMillis: number
  }>

export type BlobBuildError = Readonly<{
  readonly tag: "InvalidBlobMimeType"
  readonly value: string
}>

export type BlobReadError =
  | Readonly<{
      readonly tag: "BlobReadLimitExceeded"
      readonly value: Readonly<{ limitBytes: number; sizeBytes: number }>
    }>
  | Readonly<{ readonly tag: "BlobReadFailure"; readonly value: string }>

export const InvalidBlobMimeType = (value: string): BlobBuildError =>
  Object.freeze({ tag: "InvalidBlobMimeType", value })

export const BlobReadLimitExceeded = (value: {
  readonly limitBytes: number
  readonly sizeBytes: number
}): BlobReadError => Object.freeze({ tag: "BlobReadLimitExceeded", value })

export const BlobReadFailure = (value: string): BlobReadError =>
  Object.freeze({ tag: "BlobReadFailure", value })

export function fromBytes(
  mimeType: Maybe<string>,
  content: Bytes
): Either<BlobBuildError, Blob> {
  const resolvedMime = maybeValue(mimeType)
  if (resolvedMime !== undefined && !validMimeType(resolvedMime)) {
    return Left(InvalidBlobMimeType(resolvedMime))
  }
  const source = toUint8Array(content)
  const copy = new Uint8Array(source.length)
  copy.set(source)
  return Right(
    wrapBlob(
      new globalThis.Blob([copy.buffer], {
        ...(resolvedMime === undefined ? {} : { type: resolvedMime }),
      })
    )
  )
}

export function asBlob(file: File): Blob {
  return wrapBlob(file[FILE])
}

export function name(file: File): string {
  return file.name
}

export function mimeType(blob: Blob): Maybe<string> {
  return blob[BLOB].type === "" ? Nothing : Just(blob[BLOB].type)
}

export function sizeBytes(blob: Blob): number {
  return blob[BLOB].size
}

export function lastModifiedMillis(file: File): number {
  return file.lastModifiedMillis
}

export function readBytes(
  limitBytes: number,
  blob: Blob
): Effect<EmptyEnvironment, BlobReadError, Bytes> {
  return async (environment, context) => {
    const activeContext = context ?? createEffectExecution().context
    throwIfCancelled(activeContext)
    if (!Number.isSafeInteger(limitBytes) || limitBytes < 0) {
      return await fail(BlobReadFailure("invalid blob read limit"))(
        environment,
        activeContext
      )
    }
    if (blob[BLOB].size > limitBytes) {
      return await fail(
        BlobReadLimitExceeded({ limitBytes, sizeBytes: blob[BLOB].size })
      )(environment, activeContext)
    }
    try {
      const result = await blob[BLOB].arrayBuffer()
      throwIfCancelled(activeContext)
      return fromUint8Array(new Uint8Array(result))
    } catch (cause) {
      throwIfCancelled(activeContext)
      return await fail(BlobReadFailure(errorMessage(cause)))(
        environment,
        activeContext
      )
    }
  }
}

export function readChunks(
  blob: Blob
): Stream<EmptyEnvironment, BlobReadError, Bytes> {
  return fromPull<EmptyEnvironment, BlobReadError, Bytes>(
    async (_environment, context) => {
      const reader = blob[BLOB].stream().getReader()
      let pending = new Uint8Array()
      let closed = false
      const close = async (): Promise<void> => {
        if (closed) return
        closed = true
        try {
          await reader.cancel()
        } catch {
          // The stream may already have completed or failed.
        }
        reader.releaseLock()
      }
      const releaseCancellation = context.onCancel(close)
      return Object.freeze({
        async pull() {
          try {
            while (pending.length === 0) {
              const next = await reader.read()
              if (next.done) {
                releaseCancellation()
                await close()
                return { done: true, value: undefined }
              }
              if (next.value === undefined) continue
              pending = next.value
            }
            const chunk = pending.slice(0, MAX_CHUNK_BYTES)
            pending = pending.slice(chunk.length)
            return { done: false, value: fromUint8Array(chunk) }
          } catch (cause) {
            releaseCancellation()
            await close()
            throwIfCancelled(context)
            throw BlobReadFailure(errorMessage(cause))
          }
        },
        async close() {
          releaseCancellation()
          await close()
        },
      })
    }
  )
}

export function body(blob: Blob): Body<EmptyEnvironment, BlobReadError> {
  return streamBody(readChunks(blob))
}

/** Runtime-only bridge used by the DOM event adapter. */
export function wrapFile(value: globalThis.File): File {
  return Object.freeze({
    [BLOB]: value,
    [FILE]: value,
    name: value.name,
    lastModifiedMillis: value.lastModified,
  })
}

function wrapBlob(value: globalThis.Blob): Blob {
  return Object.freeze({ [BLOB]: value })
}

function maybeValue<Value>(value: Maybe<Value>): Value | undefined {
  return value.tag === "Just" ? value.value : undefined
}

function validMimeType(value: string): boolean {
  const token = "[!#$%&'*+.^_`|~0-9A-Za-z-]+"
  return new RegExp(
    `^${token}/${token}(?:\\s*;\\s*${token}=(?:${token}|\"[^\"\\r\\n]*\"))*$`,
    "u"
  ).test(value)
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause)
}
