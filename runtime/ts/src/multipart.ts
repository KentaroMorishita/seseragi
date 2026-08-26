import { type Bytes, fromUint8Array, toUint8Array } from "./bytes"
import type { Unit } from "./effect"
import { type Body, streamBody } from "./http-client"
import { fromPull, type Stream, type StreamCursor, singleton } from "./stream"
import { type Either, Left, type Maybe, Right } from "./sum"

const MULTIPART = Symbol("seseragi.multipart")
const encoder = new TextEncoder()

export type Multipart<Environment, Failure> = Readonly<{
  readonly [MULTIPART]: true
  readonly boundary: string
  readonly parts: ReadonlyArray<MultipartPart<Environment, Failure>>
}>

export type MultipartBuildError =
  | Readonly<{
      readonly tag: "InvalidMultipartFieldName"
      readonly value: string
    }>
  | Readonly<{
      readonly tag: "InvalidMultipartFileName"
      readonly value: string
    }>
  | Readonly<{
      readonly tag: "InvalidMultipartMimeType"
      readonly value: string
    }>

type MultipartPart<Environment, Failure> = Readonly<{
  readonly name: string
  readonly filename: string | undefined
  readonly mimeType: string | undefined
  readonly body: Body<Environment, Failure>
}>

export const InvalidMultipartFieldName = (value: string): MultipartBuildError =>
  Object.freeze({ tag: "InvalidMultipartFieldName", value })

export const InvalidMultipartFileName = (value: string): MultipartBuildError =>
  Object.freeze({ tag: "InvalidMultipartFileName", value })

export const InvalidMultipartMimeType = (value: string): MultipartBuildError =>
  Object.freeze({ tag: "InvalidMultipartMimeType", value })

export function empty<Environment, Failure>(
  _unit: Unit
): Multipart<Environment, Failure> {
  return multipartValue(randomBoundary(), [])
}

export function appendText<Environment, Failure>(
  name: string,
  value: string,
  multipart: Multipart<Environment, Failure>
): Either<MultipartBuildError, Multipart<Environment, Failure>> {
  return appendBytes(
    name,
    { tag: "Nothing" },
    { tag: "Just", value: "text/plain; charset=utf-8" },
    fromUint8Array(encoder.encode(value)),
    multipart
  )
}

export function appendBytes<Environment, Failure>(
  name: string,
  filename: Maybe<string>,
  mimeType: Maybe<string>,
  content: Bytes,
  multipart: Multipart<Environment, Failure>
): Either<MultipartBuildError, Multipart<Environment, Failure>> {
  return appendBody(
    name,
    filename,
    mimeType,
    streamBody(
      singleton(fromUint8Array(toUint8Array(content))) as Stream<
        Environment,
        Failure,
        Bytes
      >
    ),
    multipart
  )
}

export function appendBody<Environment, Failure>(
  name: string,
  filename: Maybe<string>,
  mimeType: Maybe<string>,
  content: Body<Environment, Failure>,
  multipart: Multipart<Environment, Failure>
): Either<MultipartBuildError, Multipart<Environment, Failure>> {
  const nameFailure = validateDispositionValue(name, InvalidMultipartFieldName)
  if (nameFailure !== undefined) return Left(nameFailure)
  const resolvedFilename = maybeValue(filename)
  if (resolvedFilename !== undefined) {
    const filenameFailure = validateDispositionValue(
      resolvedFilename,
      InvalidMultipartFileName
    )
    if (filenameFailure !== undefined) return Left(filenameFailure)
  }
  const resolvedMime = maybeValue(mimeType)
  if (resolvedMime !== undefined && !validMimeType(resolvedMime)) {
    return Left(InvalidMultipartMimeType(resolvedMime))
  }
  return Right(
    multipartValue(multipart.boundary, [
      ...multipart.parts,
      Object.freeze({
        name,
        filename: resolvedFilename,
        mimeType: resolvedMime,
        body: content,
      }),
    ])
  )
}

export function contentType<Environment, Failure>(
  multipart: Multipart<Environment, Failure>
): string {
  return `multipart/form-data; boundary=${multipart.boundary}`
}

export function body<Environment, Failure>(
  multipart: Multipart<Environment, Failure>
): Body<Environment, Failure> {
  const content = fromPull<Environment, Failure, Bytes>(
    async (environment, context) => {
      let index = 0
      let phase: "header" | "body" | "ending" | "final" = "header"
      let cursor: StreamCursor<Bytes> | undefined
      let closed = false
      const closeCursor = async (): Promise<void> => {
        const current = cursor
        cursor = undefined
        if (current !== undefined) await current.close()
      }
      const close = async (): Promise<void> => {
        if (closed) return
        closed = true
        await closeCursor()
      }
      return Object.freeze({
        async pull() {
          if (closed) return { done: true, value: undefined }
          while (true) {
            if (index >= multipart.parts.length) {
              if (phase === "final") {
                await close()
                return { done: true, value: undefined }
              }
              phase = "final"
              return item(bytes(`--${multipart.boundary}--\r\n`))
            }
            const part = multipart.parts[index] as MultipartPart<
              Environment,
              Failure
            >
            if (phase === "header") {
              phase = "body"
              cursor = await part.body.stream.open(environment, context)
              return item(partHeader(multipart.boundary, part))
            }
            if (phase === "body") {
              const next = await (cursor as StreamCursor<Bytes>).next()
              if (!next.done) {
                if (next.value.length === 0) continue
                return item(fromUint8Array(toUint8Array(next.value)))
              }
              await closeCursor()
              phase = "ending"
            }
            if (phase === "ending") {
              index += 1
              phase = "header"
              return item(bytes("\r\n"))
            }
          }
        },
        close,
      })
    }
  )
  return streamBody(content)
}

function multipartValue<Environment, Failure>(
  boundary: string,
  parts: ReadonlyArray<MultipartPart<Environment, Failure>>
): Multipart<Environment, Failure> {
  return Object.freeze({
    [MULTIPART]: true as const,
    boundary,
    parts: Object.freeze(parts.slice()),
  })
}

function randomBoundary(): string {
  const random = new Uint8Array(18)
  globalThis.crypto.getRandomValues(random)
  return `seseragi-${Array.from(random, (value) =>
    value.toString(16).padStart(2, "0")
  ).join("")}`
}

function partHeader<Environment, Failure>(
  boundary: string,
  part: MultipartPart<Environment, Failure>
): Bytes {
  let disposition = `Content-Disposition: form-data; name="${quote(part.name)}"`
  if (part.filename !== undefined) {
    disposition += `; filename="${quote(part.filename)}"`
  }
  const mime =
    part.mimeType === undefined ? "" : `Content-Type: ${part.mimeType}\r\n`
  return bytes(`--${boundary}\r\n${disposition}\r\n${mime}\r\n`)
}

function validateDispositionValue(
  value: string,
  failure: (value: string) => MultipartBuildError
): MultipartBuildError | undefined {
  if (
    value.length === 0 ||
    Array.from(value).some((character) => {
      const code = character.codePointAt(0) as number
      return code < 0x20 || code === 0x7f
    })
  ) {
    return failure(value)
  }
  return undefined
}

function validMimeType(value: string): boolean {
  const token = "[!#$%&'*+.^_`|~0-9A-Za-z-]+"
  return new RegExp(
    `^${token}/${token}(?:\\s*;\\s*${token}=(?:${token}|\"[^\"\\r\\n]*\"))*$`,
    "u"
  ).test(value)
}

function quote(value: string): string {
  return value.replaceAll("\\", "\\\\").replaceAll('"', '\\"')
}

function maybeValue<Value>(value: Maybe<Value>): Value | undefined {
  return value.tag === "Just" ? value.value : undefined
}

function bytes(value: string): Bytes {
  return fromUint8Array(encoder.encode(value))
}

function item<Value>(value: Value): IteratorResult<Value> {
  return { done: false, value }
}
