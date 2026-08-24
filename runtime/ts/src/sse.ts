import { type Bytes, toUint8Array } from "./bytes"
import { attempt, type Effect, type EffectContext, fail } from "./effect"
import {
  type Headers,
  type HttpEvent,
  headerValues,
  isSuccess,
  type Request,
  statusCode,
  withRequestHeader,
} from "./http-client"
import {
  type HttpHeader,
  type HttpServerResponse,
  streamResponse,
} from "./http-server"
import {
  fromPull,
  map,
  type PullStreamSource,
  type Stream,
  type StreamCursor,
} from "./stream"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"
import { encodeUtf8 } from "./text"

declare const eventBrand: unique symbol
declare const decodeLimitBrand: unique symbol

export type Event = Readonly<{
  data: string
  event: Maybe<string>
  id: Maybe<string>
  retryMillis: Maybe<number>
  readonly [eventBrand]: true
}>

export type DecodeLimit = number & { readonly [decodeLimitBrand]: true }

export type SseBuildError =
  | Readonly<{ tag: "InvalidSseEventName"; value: string }>
  | Readonly<{ tag: "InvalidSseEventId"; value: string }>
  | Readonly<{ tag: "InvalidSseRetryMillis"; value: number }>
  | Readonly<{ tag: "InvalidSseComment"; value: string }>
  | Readonly<{ tag: "InvalidSseDecodeLimit"; value: number }>

export type SseParseError =
  | Readonly<{ tag: "SseUnexpectedStatus"; value: number }>
  | Readonly<{ tag: "SseInvalidContentType"; value: string }>
  | Readonly<{ tag: "SseInvalidUtf8" }>
  | Readonly<{ tag: "SseEventTooLarge"; value: number }>
  | Readonly<{ tag: "SseMalformedId" }>
  | Readonly<{ tag: "SseMalformedRetry"; value: string }>
  | Readonly<{ tag: "SseMalformedHttpEvents"; value: string }>

const DEFAULT_DECODE_LIMIT = 1024 * 1024

export function event(data: string): Event {
  return eventValue(data, Nothing, Nothing, Nothing)
}

export function withEventName(
  name: string,
  value: Event
): Either<SseBuildError, Event> {
  return containsLineBreak(name)
    ? Left(InvalidSseEventName(name))
    : Right(eventValue(value.data, Just(name), value.id, value.retryMillis))
}

export function withId(id: string, value: Event): Either<SseBuildError, Event> {
  return containsLineBreak(id) || id.includes("\0")
    ? Left(InvalidSseEventId(id))
    : Right(eventValue(value.data, value.event, Just(id), value.retryMillis))
}

export function withRetryMillis(
  retryMillis: number,
  value: Event
): Either<SseBuildError, Event> {
  return !Number.isSafeInteger(retryMillis) || retryMillis < 0
    ? Left(InvalidSseRetryMillis(retryMillis))
    : Right(eventValue(value.data, value.event, value.id, Just(retryMillis)))
}

export function eventData(value: Event): string {
  return value.data
}

export function eventName(value: Event): Maybe<string> {
  return value.event
}

export function eventId(value: Event): Maybe<string> {
  return value.id
}

export function eventRetryMillis(value: Event): Maybe<number> {
  return value.retryMillis
}

export function encode(value: Event): Bytes {
  const lines: string[] = []
  if (value.event.tag === "Just") lines.push(`event: ${value.event.value}`)
  if (value.id.tag === "Just") lines.push(`id: ${value.id.value}`)
  if (value.retryMillis.tag === "Just") {
    lines.push(`retry: ${value.retryMillis.value}`)
  }
  for (const line of value.data.split(/\r\n|\r|\n/u)) {
    lines.push(`data: ${line}`)
  }
  return encodeUtf8(`${lines.join("\n")}\n\n`)
}

/** Encodes an explicit comment frame for keepalive use. */
export function keepAlive(comment: string): Either<SseBuildError, Bytes> {
  return containsLineBreak(comment)
    ? Left(InvalidSseComment(comment))
    : Right(encodeUtf8(`: ${comment}\n\n`))
}

export function decodeLimit(bytes: number): Either<SseBuildError, DecodeLimit> {
  return Number.isSafeInteger(bytes) && bytes > 0
    ? Right(bytes as DecodeLimit)
    : Left(InvalidSseDecodeLimit(bytes))
}

export function defaultDecodeLimit(_unit?: undefined): DecodeLimit {
  return DEFAULT_DECODE_LIMIT as DecodeLimit
}

export function withLastEventId(
  id: string,
  request: Request
): Either<SseBuildError, Request> {
  if (containsLineBreak(id) || id.includes("\0")) {
    return Left(InvalidSseEventId(id))
  }
  const result = withRequestHeader("last-event-id", id, request)
  if (result.tag === "Left") {
    throw new TypeError("validated Last-Event-ID was rejected", {
      cause: result.value,
    })
  }
  return Right(result.value)
}

/**
 * Converts a cold HTTP exchange into a cold SSE event stream. Underlying HTTP
 * failures stay in Left, while wire/parser failures use Right. Normal remote
 * end remains normal Stream completion and cancellation remains cancellation.
 */
export function events<Environment, Failure>(
  limit: DecodeLimit,
  source: Stream<Environment, Failure, HttpEvent>
): Stream<Environment, Either<Failure, SseParseError>, Event> {
  return fromPull(async (environment, context) => {
    const opened = await attempt((() =>
      source.open(environment, context)) as Effect<
      Environment,
      Failure,
      StreamCursor<HttpEvent>
    >)(environment, context)
    if (opened.tag === "Left") {
      return await fail(Left(opened.value))(environment, context)
    }
    return eventSource(opened.value, limit, environment, context)
  })
}

/** Builds the canonical streaming SSE response without adding retry policy. */
export function response<Environment>(
  headers: ReadonlyArray<HttpHeader>,
  source: Stream<Environment, never, Event>
): Effect<Environment, never, HttpServerResponse> {
  const body = map(encode, source)
  return streamResponse(200, responseHeaders(headers), body)
}

export const InvalidSseEventName = (value: string): SseBuildError =>
  Object.freeze({ tag: "InvalidSseEventName", value })
export const InvalidSseEventId = (value: string): SseBuildError =>
  Object.freeze({ tag: "InvalidSseEventId", value })
export const InvalidSseRetryMillis = (value: number): SseBuildError =>
  Object.freeze({ tag: "InvalidSseRetryMillis", value })
export const InvalidSseComment = (value: string): SseBuildError =>
  Object.freeze({ tag: "InvalidSseComment", value })
export const InvalidSseDecodeLimit = (value: number): SseBuildError =>
  Object.freeze({ tag: "InvalidSseDecodeLimit", value })
export const SseUnexpectedStatus = (value: number): SseParseError =>
  Object.freeze({ tag: "SseUnexpectedStatus", value })
export const SseInvalidContentType = (value: string): SseParseError =>
  Object.freeze({ tag: "SseInvalidContentType", value })
export const SseInvalidUtf8: SseParseError = Object.freeze({
  tag: "SseInvalidUtf8",
})
export const SseEventTooLarge = (value: number): SseParseError =>
  Object.freeze({ tag: "SseEventTooLarge", value })
export const SseMalformedId: SseParseError = Object.freeze({
  tag: "SseMalformedId",
})
export const SseMalformedRetry = (value: string): SseParseError =>
  Object.freeze({ tag: "SseMalformedRetry", value })
export const SseMalformedHttpEvents = (value: string): SseParseError =>
  Object.freeze({ tag: "SseMalformedHttpEvents", value })

function eventValue(
  data: string,
  eventName: Maybe<string>,
  id: Maybe<string>,
  retryMillis: Maybe<number>
): Event {
  return Object.freeze({ data, event: eventName, id, retryMillis }) as Event
}

function eventSource<Environment, Failure>(
  source: StreamCursor<HttpEvent>,
  limit: DecodeLimit,
  environment: Environment,
  context: EffectContext
): PullStreamSource<Event> {
  const parser = new EventStreamParser(limit)
  let started = false
  let trailers = false
  return Object.freeze({
    async pull() {
      while (true) {
        const queued = parser.shift()
        if (queued !== undefined) {
          return { done: false as const, value: queued }
        }
        const next = await attempt((() => source.next()) as Effect<
          Environment,
          Failure,
          IteratorResult<HttpEvent>
        >)(environment, context)
        if (next.tag === "Left") {
          return await fail(Left(next.value))(environment, context)
        }
        if (next.value.done) {
          const failure = parser.finish()
          if (failure !== undefined) {
            return await fail(Right(failure))(environment, context)
          }
          if (!started) {
            return await fail(
              Right(SseMalformedHttpEvents("response did not start"))
            )(environment, context)
          }
          return { done: true as const, value: undefined }
        }
        const httpEvent = next.value.value
        let failure: SseParseError | undefined
        switch (httpEvent.tag) {
          case "InformationalResponse":
            if (started) {
              failure = SseMalformedHttpEvents(
                "informational response followed final response"
              )
            }
            break
          case "ResponseStarted":
            if (started) {
              failure = SseMalformedHttpEvents("response started twice")
              break
            }
            started = true
            if (!isSuccess(httpEvent.value.status)) {
              failure = SseUnexpectedStatus(statusCode(httpEvent.value.status))
              break
            }
            failure = validateContentType(httpEvent.value.headers)
            break
          case "ResponseBodyChunk":
            if (!started || trailers) {
              failure = SseMalformedHttpEvents(
                !started
                  ? "body arrived before response head"
                  : "body arrived after trailers"
              )
              break
            }
            failure = parser.push(toUint8Array(httpEvent.value))
            break
          case "ResponseTrailers":
            if (!started || trailers) {
              failure = SseMalformedHttpEvents("trailers are out of order")
            } else {
              trailers = true
            }
            break
        }
        if (failure !== undefined) {
          return await fail(Right(failure))(environment, context)
        }
      }
    },
    close: source.close,
  })
}

class EventStreamParser {
  readonly #decoder = new TextDecoder("utf-8", { fatal: true })
  readonly #encoder = new TextEncoder()
  readonly #limit: number
  readonly #events: Event[] = []
  #text = ""
  #blockSize = 0
  #data: string[] = []
  #event: Maybe<string> = Nothing
  #id: Maybe<string> = Nothing
  #retryMillis: Maybe<number> = Nothing
  #recognized = false

  constructor(limit: DecodeLimit) {
    this.#limit = limit
  }

  push(chunk: Uint8Array): SseParseError | undefined {
    try {
      this.#text += this.#decoder.decode(chunk, { stream: true })
    } catch {
      return SseInvalidUtf8
    }
    return this.#drain(false)
  }

  finish(): SseParseError | undefined {
    try {
      this.#text += this.#decoder.decode()
    } catch {
      return SseInvalidUtf8
    }
    // The wire contract follows EventSource framing: EOF is not a blank line,
    // so an incomplete final block is deliberately discarded.
    return this.#drain(true)
  }

  shift(): Event | undefined {
    return this.#events.shift()
  }

  #drain(end: boolean): SseParseError | undefined {
    while (true) {
      const ending = lineEnding(this.#text, end)
      if (ending === undefined) {
        if (
          this.#blockSize + this.#encoder.encode(this.#text).length >
          this.#limit
        ) {
          return SseEventTooLarge(this.#limit)
        }
        if (end) this.#text = ""
        return undefined
      }
      const line = this.#text.slice(0, ending.index)
      const framedLine = this.#text.slice(0, ending.index + ending.length)
      this.#text = this.#text.slice(ending.index + ending.length)
      this.#blockSize += this.#encoder.encode(framedLine).length
      if (this.#blockSize > this.#limit) return SseEventTooLarge(this.#limit)
      const failure = this.#line(line)
      if (failure !== undefined) return failure
    }
  }

  #line(line: string): SseParseError | undefined {
    if (line.length === 0) {
      if (this.#recognized) {
        this.#events.push(
          eventValue(
            this.#data.join("\n"),
            this.#event,
            this.#id,
            this.#retryMillis
          )
        )
      }
      this.#blockSize = 0
      this.#data = []
      this.#event = Nothing
      this.#id = Nothing
      this.#retryMillis = Nothing
      this.#recognized = false
      return undefined
    }
    if (line.startsWith(":")) return undefined
    const separator = line.indexOf(":")
    const name = separator < 0 ? line : line.slice(0, separator)
    const rawValue = separator < 0 ? "" : line.slice(separator + 1)
    const value = rawValue.startsWith(" ") ? rawValue.slice(1) : rawValue
    switch (name) {
      case "data":
        this.#recognized = true
        this.#data.push(value)
        return undefined
      case "event":
        this.#recognized = true
        this.#event = Just(value)
        return undefined
      case "id":
        if (value.includes("\0")) return SseMalformedId
        this.#recognized = true
        this.#id = Just(value)
        return undefined
      case "retry": {
        if (!/^[0-9]+$/u.test(value)) return SseMalformedRetry(value)
        const retryMillis = Number(value)
        if (!Number.isSafeInteger(retryMillis)) return SseMalformedRetry(value)
        this.#recognized = true
        this.#retryMillis = Just(retryMillis)
        return undefined
      }
      default:
        return undefined
    }
  }
}

function lineEnding(
  text: string,
  end: boolean
): Readonly<{ index: number; length: number }> | undefined {
  for (let index = 0; index < text.length; index += 1) {
    const character = text[index]
    if (character === "\n") return { index, length: 1 }
    if (character === "\r") {
      if (index + 1 >= text.length && !end) return undefined
      return { index, length: text[index + 1] === "\n" ? 2 : 1 }
    }
  }
  return undefined
}

function validateContentType(headers: Headers): SseParseError | undefined {
  const values = headerValues("content-type", headers)
  const value = values[0] ?? ""
  const mediaType = value.split(";", 1)[0]?.trim().toLowerCase()
  return values.length === 1 && mediaType === "text/event-stream"
    ? undefined
    : SseInvalidContentType(value)
}

function responseHeaders(
  headers: ReadonlyArray<HttpHeader>
): ReadonlyArray<HttpHeader> {
  let result = headers.slice()
  if (!hasHeader("content-type", result)) {
    result = [...result, { name: "content-type", value: "text/event-stream" }]
  }
  if (!hasHeader("cache-control", result)) {
    result = [...result, { name: "cache-control", value: "no-cache" }]
  }
  return Object.freeze(result.map((entry) => Object.freeze({ ...entry })))
}

function hasHeader(name: string, headers: ReadonlyArray<HttpHeader>): boolean {
  return headers.some((entry) => entry.name.toLowerCase() === name)
}

function containsLineBreak(value: string): boolean {
  return value.includes("\r") || value.includes("\n")
}
