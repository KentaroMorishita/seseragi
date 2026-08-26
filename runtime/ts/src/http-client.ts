import { type Bytes, fromUint8Array, toUint8Array } from "./bytes"
import {
  attempt,
  type Effect,
  type EffectContext,
  fail,
  type Unit,
} from "./effect"
import type { ServiceResult } from "./service"
import { serviceEffect, serviceFailure, serviceSuccess } from "./service"
import {
  empty as emptyStream,
  fromPull,
  type PullStreamSource,
  type Stream,
  singleton as singletonStream,
} from "./stream"
import { type Either, Left, Right } from "./sum"

declare const methodBrand: unique symbol
declare const statusBrand: unique symbol
declare const headersBrand: unique symbol
declare const urlBrand: unique symbol
declare const requestBrand: unique symbol
declare const responseBrand: unique symbol
declare const bodyLimitBrand: unique symbol
declare const bodyBrand: unique symbol

export type Method = string & { readonly [methodBrand]: true }
export type Status = number & { readonly [statusBrand]: true }
export type Headers = ReadonlyArray<HttpClientHeader> & {
  readonly [headersBrand]: true
}
export type HttpUrl = string & { readonly [urlBrand]: true }
export type Request = Readonly<{
  readonly method: Method
  readonly url: HttpUrl
  readonly headers: Headers
  readonly [requestBrand]: true
}>
export type HttpVersion =
  | Readonly<{ tag: "HttpVersionUnknown" }>
  | Readonly<{ tag: "Http1_0" }>
  | Readonly<{ tag: "Http1_1" }>
  | Readonly<{ tag: "Http2" }>
  | Readonly<{ tag: "Http3" }>
export type ResponseHead = Readonly<{
  version: HttpVersion
  status: Status
  headers: Headers
}>
export type Response = Readonly<{
  readonly status: Status
  readonly headers: Headers
  readonly body: Bytes
  readonly [responseBrand]: true
}>
export type HttpEvent =
  | Readonly<{ tag: "InformationalResponse"; value: ResponseHead }>
  | Readonly<{ tag: "ResponseStarted"; value: ResponseHead }>
  | Readonly<{ tag: "ResponseBodyChunk"; value: Bytes }>
  | Readonly<{ tag: "ResponseTrailers"; value: Headers }>
export type Body<Environment, Failure> = Readonly<{
  readonly [bodyBrand]: true
  readonly stream: Stream<Environment, Failure, Bytes>
  readonly knownLength?: number
}>
export type HttpBodyLimit = number & { readonly [bodyLimitBrand]: true }

export type InvalidHttpUrl = Readonly<{
  tag: "InvalidHttpUrl"
  value: Readonly<{ offset: number }>
}>
export type UnsupportedHttpScheme = Readonly<{
  tag: "UnsupportedHttpScheme"
  value: string
}>
export type HttpUrlContainsUserInfo = Readonly<{
  tag: "HttpUrlContainsUserInfo"
}>
export type HttpUrlContainsFragment = Readonly<{
  tag: "HttpUrlContainsFragment"
}>
export type InvalidHttpMethod = Readonly<{
  tag: "InvalidHttpMethod"
  value: string
}>
export type InvalidHeaderName = Readonly<{
  tag: "InvalidHeaderName"
  value: string
}>
export type InvalidHeaderValue = Readonly<{
  tag: "InvalidHeaderValue"
  value: Readonly<{ name: string; offset: number }>
}>
export type ManagedHttpHeader = Readonly<{
  tag: "ManagedHttpHeader"
  value: string
}>
export type InvalidHttpStatus = Readonly<{
  tag: "InvalidHttpStatus"
  value: number
}>
export type InvalidHttpBodyLimit = Readonly<{
  tag: "InvalidHttpBodyLimit"
  value: number
}>
export type HttpBuildError =
  | InvalidHttpUrl
  | UnsupportedHttpScheme
  | HttpUrlContainsUserInfo
  | HttpUrlContainsFragment
  | InvalidHttpMethod
  | InvalidHeaderName
  | InvalidHeaderValue
  | ManagedHttpHeader
  | InvalidHttpStatus
  | InvalidHttpBodyLimit

export type HttpDnsFailure = Readonly<{
  tag: "HttpDnsFailure"
  value: string
}>
export type HttpConnectionFailure = Readonly<{
  tag: "HttpConnectionFailure"
  value: string
}>
export type HttpTlsFailure = Readonly<{
  tag: "HttpTlsFailure"
  value: string
}>
export type HttpProtocolFailure = Readonly<{
  tag: "HttpProtocolFailure"
  value: string
}>
export type HttpRequestBodyFailure = Readonly<{
  tag: "HttpRequestBodyFailure"
  value: string
}>
export type HttpRequestLengthMismatch = Readonly<{
  tag: "HttpRequestLengthMismatch"
  value: Readonly<{ declared: number; actual: number }>
}>
export type HttpResponseBodyLimitExceeded = Readonly<{
  tag: "HttpResponseBodyLimitExceeded"
  value: Readonly<{ limitBytes: number }>
}>
export type HttpClientUnavailable = Readonly<{
  tag: "HttpClientUnavailable"
}>
export type HttpError =
  | HttpDnsFailure
  | HttpConnectionFailure
  | HttpTlsFailure
  | HttpProtocolFailure
  | HttpRequestBodyFailure
  | HttpRequestLengthMismatch
  | HttpResponseBodyLimitExceeded
  | HttpClientUnavailable

export type HttpClientHeader = Readonly<{ name: string; value: string }>
export type HttpClientRequest = Readonly<{
  method: string
  url: string
  headers: ReadonlyArray<HttpClientHeader>
  body: Uint8Array
}>
export type HttpClientResponse = Readonly<{
  status: number
  headers: ReadonlyArray<HttpClientHeader>
  body: Uint8Array
}>
export type HttpClientVersion =
  | "HttpVersionUnknown"
  | "Http1_0"
  | "Http1_1"
  | "Http2"
  | "Http3"
export type HttpClientResponseHead = Readonly<{
  version: HttpClientVersion
  status: number
  headers: ReadonlyArray<HttpClientHeader>
}>
export type HttpClientEvent =
  | Readonly<{ kind: "InformationalResponse"; head: HttpClientResponseHead }>
  | Readonly<{ kind: "ResponseStarted"; head: HttpClientResponseHead }>
  | Readonly<{ kind: "ResponseBodyChunk"; bytes: Uint8Array }>
  | Readonly<{
      kind: "ResponseTrailers"
      headers: ReadonlyArray<HttpClientHeader>
    }>
export type HttpClientRequestBody = Readonly<{
  pull: () => Promise<IteratorResult<Uint8Array>>
  cancel: () => Promise<void>
  knownLength?: number
}>
export type HttpClientError = HttpError
export type HttpClient = Readonly<{
  send: (
    request: HttpClientRequest,
    context: EffectContext
  ) => Promise<ServiceResult<HttpClientError, HttpClientResponse>>
  exchange: (
    request: Omit<HttpClientRequest, "body">,
    body: HttpClientRequestBody,
    context: EffectContext
  ) => PullStreamSource<HttpClientEvent>
}>
export type HttpClientEnvironment = Readonly<{ httpClient: HttpClient }>

const method = (value: string): Method => value as Method
const statusValue = (value: number): Status => value as Status
const urlValue = (value: string): HttpUrl => value as HttpUrl
const limitValue = (value: number): HttpBodyLimit => value as HttpBodyLimit

export const get: Method = method("GET")
export const head: Method = method("HEAD")
export const post: Method = method("POST")
export const put: Method = method("PUT")
export const patch: Method = method("PATCH")
export const delete_: Method = method("DELETE")
export { delete_ as delete }
export const options: Method = method("OPTIONS")
export const connect: Method = method("CONNECT")
export const trace: Method = method("TRACE")

export const HttpVersionUnknown: HttpVersion = Object.freeze({
  tag: "HttpVersionUnknown",
})
export const Http1_0: HttpVersion = Object.freeze({ tag: "Http1_0" })
export const Http1_1: HttpVersion = Object.freeze({ tag: "Http1_1" })
export const Http2: HttpVersion = Object.freeze({ tag: "Http2" })
export const Http3: HttpVersion = Object.freeze({ tag: "Http3" })

export const InformationalResponse = (value: ResponseHead): HttpEvent =>
  Object.freeze({ tag: "InformationalResponse", value })
export const ResponseStarted = (value: ResponseHead): HttpEvent =>
  Object.freeze({ tag: "ResponseStarted", value })
export const ResponseBodyChunk = (value: Bytes): HttpEvent =>
  Object.freeze({ tag: "ResponseBodyChunk", value })
export const ResponseTrailers = (value: Headers): HttpEvent =>
  Object.freeze({ tag: "ResponseTrailers", value })

export const emptyHeaders: Headers = freezeHeaders([])
const defaultBodyLimitValue: HttpBodyLimit = limitValue(8 * 1024 * 1024)
const MAX_HTTP_CHUNK_BYTES = 64 * 1024

const tokenPattern = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/
const managedHeaders = new Set([
  "connection",
  "proxy-connection",
  "keep-alive",
  "transfer-encoding",
  "upgrade",
  "te",
  "host",
])

export const InvalidHttpUrl = (value: {
  readonly offset: number
}): InvalidHttpUrl => Object.freeze({ tag: "InvalidHttpUrl", value })
export const UnsupportedHttpScheme = (value: string): UnsupportedHttpScheme =>
  Object.freeze({ tag: "UnsupportedHttpScheme", value })
export const HttpUrlContainsUserInfo: HttpUrlContainsUserInfo = Object.freeze({
  tag: "HttpUrlContainsUserInfo",
})
export const HttpUrlContainsFragment: HttpUrlContainsFragment = Object.freeze({
  tag: "HttpUrlContainsFragment",
})
export const InvalidHttpMethod = (value: string): InvalidHttpMethod =>
  Object.freeze({ tag: "InvalidHttpMethod", value })
export const InvalidHeaderName = (value: string): InvalidHeaderName =>
  Object.freeze({ tag: "InvalidHeaderName", value })
export const InvalidHeaderValue = (value: {
  readonly name: string
  readonly offset: number
}): InvalidHeaderValue => Object.freeze({ tag: "InvalidHeaderValue", value })
export const ManagedHttpHeader = (value: string): ManagedHttpHeader =>
  Object.freeze({ tag: "ManagedHttpHeader", value })
export const InvalidHttpStatus = (value: number): InvalidHttpStatus =>
  Object.freeze({ tag: "InvalidHttpStatus", value })
export const InvalidHttpBodyLimit = (value: number): InvalidHttpBodyLimit =>
  Object.freeze({ tag: "InvalidHttpBodyLimit", value })

export const HttpDnsFailure = (value: string): HttpDnsFailure =>
  Object.freeze({ tag: "HttpDnsFailure", value })
export const HttpConnectionFailure = (value: string): HttpConnectionFailure =>
  Object.freeze({ tag: "HttpConnectionFailure", value })
export const HttpTlsFailure = (value: string): HttpTlsFailure =>
  Object.freeze({ tag: "HttpTlsFailure", value })
export const HttpProtocolFailure = (value: string): HttpProtocolFailure =>
  Object.freeze({ tag: "HttpProtocolFailure", value })
export const HttpRequestBodyFailure = (value: string): HttpRequestBodyFailure =>
  Object.freeze({ tag: "HttpRequestBodyFailure", value })
export const HttpRequestLengthMismatch = (value: {
  readonly declared: number
  readonly actual: number
}): HttpRequestLengthMismatch =>
  Object.freeze({ tag: "HttpRequestLengthMismatch", value })
export const HttpResponseBodyLimitExceeded = (value: {
  readonly limitBytes: number
}): HttpResponseBodyLimitExceeded =>
  Object.freeze({ tag: "HttpResponseBodyLimitExceeded", value })
export const HttpClientUnavailable: HttpClientUnavailable = Object.freeze({
  tag: "HttpClientUnavailable",
})

export function customMethod(text: string): Either<HttpBuildError, Method> {
  return tokenPattern.test(text) && text === text.toUpperCase()
    ? Right(method(text))
    : Left(InvalidHttpMethod(text))
}

export function methodText(value: Method): string {
  return value
}

export function status(code: number): Either<HttpBuildError, Status> {
  return Number.isSafeInteger(code) && code >= 100 && code <= 999
    ? Right(statusValue(code))
    : Left(InvalidHttpStatus(code))
}

export function statusCode(value: Status): number {
  return value
}

export function isInformational(value: Status): boolean {
  return value >= 100 && value < 200
}

export function isSuccess(value: Status): boolean {
  return value >= 200 && value < 300
}

export function isRedirection(value: Status): boolean {
  return value >= 300 && value < 400
}

export function isClientError(value: Status): boolean {
  return value >= 400 && value < 500
}

export function isServerError(value: Status): boolean {
  return value >= 500 && value < 600
}

export function parseUrl(text: string): Either<HttpBuildError, HttpUrl> {
  for (let offset = 0; offset < text.length; offset += 1) {
    if (text.charCodeAt(offset) > 127) {
      return Left(InvalidHttpUrl({ offset }))
    }
    if (
      text[offset] === "%" &&
      !/^[0-9A-Fa-f]{2}$/.test(text.slice(offset + 1, offset + 3))
    ) {
      return Left(InvalidHttpUrl({ offset }))
    }
  }
  let parsed: URL
  try {
    parsed = new URL(text)
  } catch {
    return Left(InvalidHttpUrl({ offset: 0 }))
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    return Left(UnsupportedHttpScheme(parsed.protocol.slice(0, -1)))
  }
  if (parsed.username !== "" || parsed.password !== "") {
    return Left(HttpUrlContainsUserInfo)
  }
  if (parsed.hash !== "") return Left(HttpUrlContainsFragment)
  return Right(urlValue(uppercasePercentEscapes(parsed.toString())))
}

export function renderUrl(value: HttpUrl): string {
  return value
}

export function appendHeader(
  name: string,
  value: string,
  headers: Headers
): Either<HttpBuildError, Headers> {
  const validation = validateHeader(name, value)
  if (validation !== undefined) return Left(validation)
  return Right(
    freezeHeaders([
      ...headers,
      Object.freeze({ name: name.toLowerCase(), value }),
    ])
  )
}

export function setHeader(
  name: string,
  value: string,
  headers: Headers
): Either<HttpBuildError, Headers> {
  const validation = validateHeader(name, value)
  if (validation !== undefined) return Left(validation)
  const normalized = name.toLowerCase()
  const first = headers.findIndex((entry) => entry.name === normalized)
  const retained = headers.filter((entry) => entry.name !== normalized)
  retained.splice(
    first < 0 ? retained.length : first,
    0,
    Object.freeze({ name: normalized, value })
  )
  return Right(freezeHeaders(retained))
}

export function removeHeader(name: string, headers: Headers): Headers {
  const normalized = name.toLowerCase()
  return freezeHeaders(headers.filter((entry) => entry.name !== normalized))
}

export function headerValues(
  name: string,
  headers: Headers
): ReadonlyArray<string> {
  const normalized = name.toLowerCase()
  return Object.freeze(
    headers
      .filter((entry) => entry.name === normalized)
      .map((entry) => entry.value)
  )
}

export function headerEntries(
  headers: Headers
): ReadonlyArray<readonly [string, string]> {
  return Object.freeze(
    headers.map((entry) => Object.freeze([entry.name, entry.value] as const))
  )
}

export function request(methodValue: Method, url: HttpUrl): Request {
  return Object.freeze({
    method: methodValue,
    url,
    headers: emptyHeaders,
  }) as Request
}

export function withRequestHeader(
  name: string,
  value: string,
  requestValue: Request
): Either<HttpBuildError, Request> {
  const result = appendHeader(name, value, requestValue.headers)
  return result.tag === "Left"
    ? result
    : Right(
        Object.freeze({
          method: requestValue.method,
          url: requestValue.url,
          headers: result.value,
        }) as Request
      )
}

export function withoutRequestHeader(
  name: string,
  requestValue: Request
): Request {
  return Object.freeze({
    method: requestValue.method,
    url: requestValue.url,
    headers: removeHeader(name, requestValue.headers),
  }) as Request
}

export function emptyBody<Environment, Failure>(
  _unit?: Unit
): Body<Environment, Failure> {
  return bodyValue(emptyStream as Stream<Environment, Failure, Bytes>, 0)
}

export function bytesBody<Environment, Failure>(
  content: Bytes
): Body<Environment, Failure> {
  return bodyValue(
    singletonStream(fromUint8Array(toUint8Array(content))) as Stream<
      Environment,
      Failure,
      Bytes
    >,
    toUint8Array(content).length
  )
}

export function streamBody<Environment, Failure>(
  content: Stream<Environment, Failure, Bytes>
): Body<Environment, Failure> {
  return bodyValue(content)
}

export function exchange<Environment, Failure>(
  body: Body<Environment, Failure>,
  requestValue: Request
): Stream<
  Environment & HttpClientEnvironment,
  Either<Failure, HttpError>,
  HttpEvent
> {
  return fromPull(async (environment, context) => {
    const openedBody = await attempt((() =>
      body.stream.open(environment, context)) as Effect<
      Environment,
      Failure,
      Awaited<ReturnType<typeof body.stream.open>>
    >)(environment, context)
    if (openedBody.tag === "Left") {
      return await fail(Left(openedBody.value))(environment, context)
    }
    const bodyCursor = openedBody.value
    const declaredLength = parseContentLength(requestValue.headers)
    if (typeof declaredLength !== "number" && declaredLength !== undefined) {
      await bodyCursor.close()
      return await fail(Right(declaredLength))(environment, context)
    }

    let pendingBody: Uint8Array<ArrayBufferLike> = new Uint8Array()
    let sentBytes = 0
    let observation = 0
    let bodyFailure: Readonly<{ order: number; error: Failure }> | undefined
    let requestFailure:
      | Readonly<{ order: number; error: HttpError }>
      | undefined

    const observeBodyFailure = (error: Failure): void => {
      bodyFailure ??= { order: observation++, error }
    }
    const observeRequestFailure = (error: HttpError): void => {
      requestFailure ??= { order: observation++, error }
    }

    const requestBody: HttpClientRequestBody = Object.freeze({
      ...(body.knownLength === undefined
        ? {}
        : { knownLength: body.knownLength }),
      async pull() {
        if (pendingBody.length > 0) {
          const chunk = pendingBody.slice(0, MAX_HTTP_CHUNK_BYTES)
          pendingBody = pendingBody.slice(chunk.length)
          sentBytes += chunk.length
          return { done: false, value: chunk }
        }
        while (true) {
          const bodyResult = await attempt((() => bodyCursor.next()) as Effect<
            Environment,
            Failure,
            IteratorResult<Bytes>
          >)(environment, context)
          if (bodyResult.tag === "Left") {
            observeBodyFailure(bodyResult.value)
            throw bodyResult.value
          }
          const next = bodyResult.value
          if (next.done) {
            if (declaredLength !== undefined && declaredLength !== sentBytes) {
              const failure = HttpRequestLengthMismatch({
                declared: declaredLength,
                actual: sentBytes,
              })
              observeRequestFailure(failure)
              throw failure
            }
            return { done: true, value: undefined }
          }
          const bytes = toUint8Array(next.value)
          if (bytes.length === 0) continue
          pendingBody = bytes
          const chunk = pendingBody.slice(0, MAX_HTTP_CHUNK_BYTES)
          pendingBody = pendingBody.slice(chunk.length)
          sentBytes += chunk.length
          return { done: false, value: chunk }
        }
      },
      async cancel() {
        await bodyCursor.close()
      },
    })

    let source: PullStreamSource<HttpClientEvent>
    try {
      source = environment.httpClient.exchange(
        {
          method: requestValue.method,
          url: requestValue.url,
          headers: requestValue.headers,
        },
        requestBody,
        context
      )
    } catch (cause) {
      await bodyCursor.close()
      return await fail(Right(cause as HttpError))(environment, context)
    }

    return Object.freeze({
      async pull(pullContext: EffectContext) {
        try {
          const next = await source.pull(pullContext)
          if (next.done) return { done: true, value: undefined }
          return {
            done: false,
            value: publicEvent(next.value),
          } as IteratorResult<HttpEvent>
        } catch (cause) {
          const providerFailure: Readonly<{
            order: number
            error: HttpError
          }> = {
            order: observation++,
            error: cause as HttpError,
          }
          let failure: Either<Failure, HttpError>
          if (
            bodyFailure !== undefined &&
            bodyFailure.order < providerFailure.order &&
            (requestFailure === undefined ||
              bodyFailure.order < requestFailure.order)
          ) {
            failure = Left(bodyFailure.error)
          } else if (
            requestFailure !== undefined &&
            requestFailure.order < providerFailure.order
          ) {
            failure = Right(requestFailure.error)
          } else {
            failure = Right(providerFailure.error)
          }
          return await fail(failure)(environment, pullContext)
        }
      },
      async close() {
        let defect: unknown
        try {
          await source.close()
        } catch (cause) {
          defect = cause
        }
        try {
          await bodyCursor.close()
        } catch (cause) {
          defect ??= cause
        }
        if (defect !== undefined) throw defect
      },
    })
  })
}

export function bodyLimit(
  bytes: number
): Either<HttpBuildError, HttpBodyLimit> {
  return Number.isSafeInteger(bytes) && bytes > 0
    ? Right(limitValue(bytes))
    : Left(InvalidHttpBodyLimit(bytes))
}

export function defaultBodyLimit(_unit?: Unit): HttpBodyLimit {
  return defaultBodyLimitValue
}

export function sendBytes(
  limit: HttpBodyLimit,
  body: Bytes,
  requestValue: Request
): Effect<HttpClientEnvironment, HttpError, Response> {
  return sendApplicationRequest(limit, body, requestValue)
}

export function sendEmpty(
  limit: HttpBodyLimit,
  requestValue: Request
): Effect<HttpClientEnvironment, HttpError, Response> {
  return sendApplicationRequest(
    limit,
    fromUint8Array(new Uint8Array()),
    requestValue
  )
}

export function responseStatus(response: Response): Status {
  return response.status
}

export function responseHeaders(response: Response): Headers {
  return response.headers
}

export function responseBody(response: Response): Bytes {
  return fromUint8Array(toUint8Array(response.body))
}

/** Internal Provider Contract projection used by runtime conformance probes. */
export function send(
  request: HttpClientRequest
): Effect<HttpClientEnvironment, HttpClientError, HttpClientResponse> {
  return serviceEffect((environment, context) =>
    environment.httpClient.send(request, context)
  )
}

export function errorMessage(error: HttpError): string {
  if (error.tag === "HttpClientUnavailable") return error.tag
  if (error.tag === "HttpRequestLengthMismatch") {
    return `${error.tag}: declared ${error.value.declared}, actual ${error.value.actual}`
  }
  if (error.tag === "HttpResponseBodyLimitExceeded") {
    return `${error.tag}: ${error.value.limitBytes} bytes`
  }
  return `${error.tag}: ${error.value}`
}

export function httpClientSuccess(
  response: HttpClientResponse
): ServiceResult<never, HttpClientResponse> {
  return serviceSuccess(response)
}

export function httpClientFailure(
  error: HttpClientError
): ServiceResult<HttpClientError, never> {
  return serviceFailure(error)
}

function sendApplicationRequest(
  limit: HttpBodyLimit,
  body: Bytes,
  requestValue: Request
): Effect<HttpClientEnvironment, HttpError, Response> {
  return serviceEffect(async (environment, context) => {
    const requestBody = toUint8Array(body)
    const lengthValidation = validateContentLength(
      requestValue.headers,
      requestBody.length
    )
    if (lengthValidation !== undefined) return serviceFailure(lengthValidation)
    const result = await environment.httpClient.send(
      {
        method: requestValue.method,
        url: requestValue.url,
        headers: requestValue.headers,
        body: requestBody,
      },
      context
    )
    if (result.kind === "failure") return result
    if (result.value.body.length > limit) {
      return serviceFailure(
        HttpResponseBodyLimitExceeded({ limitBytes: limit })
      )
    }
    const response = Object.freeze({
      status: statusValue(result.value.status),
      headers: freezeHeaders(result.value.headers),
      body: fromUint8Array(result.value.body),
    }) as Response
    return serviceSuccess(response)
  })
}

function validateContentLength(
  headers: Headers,
  actual: number
): HttpError | undefined {
  const declared = parseContentLength(headers)
  if (declared === undefined) return undefined
  if (typeof declared !== "number") return declared
  return declared === actual
    ? undefined
    : HttpRequestLengthMismatch({ declared, actual })
}

function bodyValue<Environment, Failure>(
  stream: Stream<Environment, Failure, Bytes>,
  knownLength?: number
): Body<Environment, Failure> {
  return Object.freeze({
    stream,
    ...(knownLength === undefined ? {} : { knownLength }),
  }) as Body<Environment, Failure>
}

function parseContentLength(headers: Headers): number | HttpError | undefined {
  const values = headerValues("content-length", headers)
  if (values.length === 0) return undefined
  if (values.length !== 1 || !/^(0|[1-9][0-9]*)$/.test(values[0] ?? "")) {
    return HttpProtocolFailure("invalid Content-Length header")
  }
  const declared = Number(values[0])
  return Number.isSafeInteger(declared)
    ? declared
    : HttpProtocolFailure("Content-Length exceeds the safe integer range")
}

function publicEvent(event: HttpClientEvent): HttpEvent {
  switch (event.kind) {
    case "InformationalResponse":
      return InformationalResponse(publicHead(event.head))
    case "ResponseStarted":
      return ResponseStarted(publicHead(event.head))
    case "ResponseBodyChunk":
      if (
        event.bytes.length === 0 ||
        event.bytes.length > MAX_HTTP_CHUNK_BYTES
      ) {
        throw HttpProtocolFailure("response body chunk size is invalid")
      }
      return ResponseBodyChunk(fromUint8Array(event.bytes))
    case "ResponseTrailers":
      return ResponseTrailers(freezeHeaders(event.headers))
  }
}

function publicHead(head: HttpClientResponseHead): ResponseHead {
  return Object.freeze({
    version: versionValue(head.version),
    status: statusValue(head.status),
    headers: freezeHeaders(head.headers),
  })
}

function versionValue(version: HttpClientVersion): HttpVersion {
  switch (version) {
    case "HttpVersionUnknown":
      return HttpVersionUnknown
    case "Http1_0":
      return Http1_0
    case "Http1_1":
      return Http1_1
    case "Http2":
      return Http2
    case "Http3":
      return Http3
  }
}

function freezeHeaders(entries: ReadonlyArray<HttpClientHeader>): Headers {
  return Object.freeze(
    entries.map(({ name, value }) =>
      Object.freeze({ name: name.toLowerCase(), value })
    )
  ) as Headers
}

function validateHeader(
  name: string,
  value: string
): HttpBuildError | undefined {
  if (!tokenPattern.test(name)) return InvalidHeaderName(name)
  const normalized = name.toLowerCase()
  if (managedHeaders.has(normalized) || normalized.startsWith(":")) {
    return ManagedHttpHeader(normalized)
  }
  const offset = [...value].findIndex((character) => {
    const point = character.codePointAt(0)!
    return point === 0 || point === 10 || point === 13 || point > 255
  })
  return offset < 0
    ? undefined
    : InvalidHeaderValue({ name: normalized, offset })
}

function uppercasePercentEscapes(value: string): string {
  return value.replace(/%[0-9a-f]{2}/gi, (sequence) => sequence.toUpperCase())
}
