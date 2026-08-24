import { type Bytes, fromUint8Array, toUint8Array } from "./bytes"
import {
  createEffectExecution,
  type Effect,
  EffectCancellation,
  type EffectContext,
  isEffectCancellation,
  recover,
  registerResourceFinalizer,
  run,
  succeed,
  type Unit,
} from "./effect"
import type { ProviderHandle } from "./provider"
import {
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"
import type { Stream, StreamCursor } from "./stream"
import { Just, type Maybe, Nothing } from "./sum"
import { encodeUtf8 } from "./text"

export type HttpHeader = Readonly<{ name: string; value: string }>

export type HttpServerRequest = Readonly<{
  method: string
  url: string
  headers: ReadonlyArray<HttpHeader>
  body: Uint8Array
}>

export type HttpServerResponse = Readonly<{
  status: number
  headers: ReadonlyArray<HttpHeader>
  body: Uint8Array | HttpServerStreamBody
}>

type HttpServerStreamBody = Readonly<{
  kind: "stream"
  cursor: StreamCursor<Bytes>
}>

export type ProviderHttpServerStreamBody = Readonly<{
  kind: "stream"
  pull: () => Promise<IteratorResult<Uint8Array>>
  complete: () => Promise<void>
  cancel: () => Promise<void>
}>

export type ProviderHttpServerResponse = Readonly<{
  status: number
  headers: ReadonlyArray<HttpHeader>
  body: Uint8Array | ProviderHttpServerStreamBody
}>

export type HttpServerHandler<Environment, Failure> = (
  request: HttpServerRequest
) => Effect<Environment, Failure, HttpServerResponse>

export type HttpServerOptions<Environment> = Readonly<{
  hostname?: string
  port: number
  handler: HttpServerHandler<Environment, never>
}>

export type ProviderHttpServerOptions = Readonly<{
  hostname?: string
  port: number
  handler: (request: HttpServerRequest) => Promise<ProviderHttpServerResponse>
}>

export type HttpServerError = Readonly<{
  tag: "HttpServerUnavailable"
  message: string
}>

export type HttpServerHandle = ProviderHandle

export type HttpServer = Readonly<{
  listen: (
    options: ProviderHttpServerOptions,
    context: EffectContext
  ) => Promise<ServiceResult<HttpServerError, HttpServerHandle>>
  close: (
    server: HttpServerHandle,
    context: EffectContext
  ) => Promise<ServiceResult<never, Unit>>
}>

export type HttpServerEnvironment = Readonly<{ httpServer: HttpServer }>

type RequestExecution = Readonly<{
  cancel: () => Promise<void>
}>

type ServerExecutionState = {
  handle?: HttpServerHandle
  readonly server: HttpServer
  readonly context: EffectContext
  readonly requests: Set<RequestExecution>
  unregisterCleanup: () => void
  closeCompletion?: Promise<ServiceResult<never, Unit>>
}

const serverExecutions = new WeakMap<object, ServerExecutionState>()
const cancelledResponses = new WeakSet<object>()

/** Runtime-provider bridge marker; not part of std/http/server. */
export function isCancelledHttpServerResponse(response: object): boolean {
  return cancelledResponses.has(response)
}

export function requestMethod(request: HttpServerRequest): string {
  return request.method
}

export function requestUrl(request: HttpServerRequest): string {
  return request.url
}

export function requestPath(request: HttpServerRequest): string {
  return new URL(request.url).pathname
}

export function requestQuery(request: HttpServerRequest): Maybe<string> {
  const queryStart = request.url.indexOf("?")
  if (queryStart < 0) return Nothing
  const fragmentStart = request.url.indexOf("#", queryStart + 1)
  return Just(
    request.url.slice(
      queryStart + 1,
      fragmentStart < 0 ? request.url.length : fragmentStart
    )
  )
}

export function requestHeaders(
  request: HttpServerRequest
): ReadonlyArray<HttpHeader> {
  return Object.freeze(request.headers.map(snapshotHeader))
}

export function requestHeaderValues(
  name: string,
  request: HttpServerRequest
): ReadonlyArray<string> {
  const normalized = name.toLowerCase()
  return Object.freeze(
    request.headers
      .filter((entry) => entry.name.toLowerCase() === normalized)
      .map((entry) => entry.value)
  )
}

export function requestBody(request: HttpServerRequest): Bytes {
  return fromUint8Array(request.body)
}

export function header(name: string, value: string): HttpHeader {
  return snapshotHeader({ name, value })
}

export function emptyResponse(
  status: number,
  headers: ReadonlyArray<HttpHeader>
): HttpServerResponse {
  return response(status, headers, new Uint8Array())
}

export function bytesResponse(
  status: number,
  headers: ReadonlyArray<HttpHeader>,
  body: Bytes
): HttpServerResponse {
  return response(status, headers, body)
}

/**
 * Opens a cold response body inside the current request Effect scope. The
 * request remains owned by that scope until the provider drains or cancels the
 * returned body.
 */
export function streamResponse<Environment>(
  status: number,
  headers: ReadonlyArray<HttpHeader>,
  body: Stream<Environment, never, Bytes>
): Effect<Environment, never, HttpServerResponse> {
  return async (environment, context) => {
    if (context === undefined) {
      throw new TypeError("streamResponse requires an active request scope")
    }
    const cursor = await body.open(environment, context)
    return response(status, headers, Object.freeze({ kind: "stream", cursor }))
  }
}

export function textResponse(
  status: number,
  headers: ReadonlyArray<HttpHeader>,
  body: string
): HttpServerResponse {
  return response(
    status,
    withDefaultContentType(headers, "text/plain; charset=utf-8"),
    encodeUtf8(body)
  )
}

/** Builds a response from JSON text produced explicitly by std/json. */
export function jsonResponse(
  status: number,
  headers: ReadonlyArray<HttpHeader>,
  json: string
): HttpServerResponse {
  return response(
    status,
    withDefaultContentType(headers, "application/json; charset=utf-8"),
    encodeUtf8(json)
  )
}

export function pureHandler(
  handler: (request: HttpServerRequest) => HttpServerResponse
): HttpServerHandler<unknown, never> {
  return (request) => succeed(handler(request))
}

export function recoverHandler<Environment, Failure>(
  render: (error: Failure) => HttpServerResponse,
  handler: HttpServerHandler<Environment, Failure>
): HttpServerHandler<Environment, never> {
  return (request) =>
    recover((error: Failure) => succeed(render(error)), handler(request))
}

export function errorMessage(error: HttpServerError): string {
  return error.message
}

export function response(
  status: number,
  headers: ReadonlyArray<HttpHeader>,
  body: Uint8Array | HttpServerStreamBody
): HttpServerResponse {
  validateStatus(status)
  const snapshotHeaders = headers.map(snapshotHeader)
  return Object.freeze({
    status,
    headers: Object.freeze(snapshotHeaders),
    body: body instanceof Uint8Array ? new Uint8Array(body) : body,
  })
}

export function listen<Environment>(
  options: HttpServerOptions<Environment>
): Effect<
  Environment & HttpServerEnvironment,
  HttpServerError,
  HttpServerHandle
> {
  return serviceEffect(async (environment, context) => {
    const state = createServerState(environment.httpServer, context)
    const started = await environment.httpServer.listen(
      bridgeOptions(options, environment, state),
      context
    )
    if (started.kind === "failure") return started
    state.handle = started.value
    serverExecutions.set(started.value, state)
    const registration = registerResourceFinalizer(context, () =>
      closeServerState(state, true).then(() => undefined)
    )
    state.unregisterCleanup = registration.unregister
    await registration.ready
    return httpServerSuccess(started.value)
  })
}

/** Serves one request and then closes the selected provider-backed server. */
export function serveOnce<Environment>(
  options: HttpServerOptions<Environment>
): Effect<Environment & HttpServerEnvironment, HttpServerError, Unit> {
  return serviceEffect(async (environment, context) => {
    let claimRequest: () => void = () => undefined
    const requestClaimed = new Promise<void>((resolve) => {
      claimRequest = resolve
    })
    let claimed = false
    const state = createServerState(environment.httpServer, context)
    const started = await environment.httpServer.listen(
      bridgeOptions(
        {
          ...options,
          handler(request) {
            if (claimed) {
              return () => {
                throw new EffectCancellation()
              }
            }
            claimed = true
            claimRequest()
            return options.handler(request)
          },
        },
        environment,
        state
      ),
      context
    )
    if (started.kind === "failure") return started
    state.handle = started.value
    serverExecutions.set(started.value, state)
    const registration = registerResourceFinalizer(context, () =>
      closeServerState(state, true).then(() => undefined)
    )
    state.unregisterCleanup = registration.unregister
    await registration.ready
    await waitForRequestClaim(requestClaimed, context)
    await closeServerState(state, false)
    return httpServerSuccess(undefined)
  })
}

export function close(
  server: HttpServerHandle
): Effect<HttpServerEnvironment, never, Unit> {
  return serviceEffect((environment, context) => {
    const state = serverExecutions.get(server)
    return state === undefined
      ? environment.httpServer.close(server, context)
      : closeServerState(state, true)
  })
}

export function httpServerSuccess<Success>(
  value: Success
): ServiceResult<never, Success> {
  return serviceSuccess(value)
}

export function httpServerFailure(
  error: HttpServerError
): ServiceResult<HttpServerError, never> {
  return serviceFailure(error)
}

function snapshotHeader(header: HttpHeader): HttpHeader {
  if (
    header.name.length === 0 ||
    header.name.includes("\r") ||
    header.name.includes("\n") ||
    header.value.includes("\r") ||
    header.value.includes("\n")
  ) {
    throw new TypeError("HTTP header is invalid")
  }
  return Object.freeze({ name: header.name.toLowerCase(), value: header.value })
}

function validateStatus(status: number): void {
  if (!Number.isSafeInteger(status) || status < 200 || status > 599) {
    throw new RangeError("HTTP server response status must be 200 through 599")
  }
}

function withDefaultContentType(
  headers: ReadonlyArray<HttpHeader>,
  value: string
): ReadonlyArray<HttpHeader> {
  return headers.some((entry) => entry.name.toLowerCase() === "content-type")
    ? headers
    : [...headers, { name: "content-type", value }]
}

function createServerState(
  server: HttpServer,
  context: EffectContext
): ServerExecutionState {
  return {
    server,
    context,
    requests: new Set(),
    unregisterCleanup: () => undefined,
  }
}

function bridgeOptions<Environment>(
  options: HttpServerOptions<Environment>,
  environment: Environment & HttpServerEnvironment,
  state: ServerExecutionState
): ProviderHttpServerOptions {
  return Object.freeze({
    port: options.port,
    ...(options.hostname === undefined ? {} : { hostname: options.hostname }),
    async handler(request) {
      const execution = createEffectExecution(state.context)
      const requestExecution: RequestExecution = {
        cancel: execution.cancel,
      }
      state.requests.add(requestExecution)
      let bodyOwnsExecution = false
      let requestCancelled = false
      const releaseRequest = onceAsync(async (cancelled: boolean) => {
        state.requests.delete(requestExecution)
        if (cancelled) await execution.cancel()
        else await execution.close()
      })
      try {
        const result = await run(
          options.handler(request),
          environment,
          execution.context
        )
        if (result.kind === "failure") {
          throw new TypeError(
            "HTTP server Handler<R, Never> produced an impossible typed failure",
            { cause: result.error }
          )
        }
        if (isStreamResponse(result.value)) {
          bodyOwnsExecution = true
          return bridgeStreamingResponse(result.value, releaseRequest)
        }
        return result.value as ProviderHttpServerResponse
      } catch (error) {
        if (isEffectCancellation(error)) {
          requestCancelled = true
          return cancelledResponse()
        }
        throw error
      } finally {
        if (!bodyOwnsExecution) await releaseRequest(requestCancelled)
      }
    },
  })
}

function isStreamResponse(response: HttpServerResponse): boolean {
  return !(response.body instanceof Uint8Array)
}

function bridgeStreamingResponse(
  response: HttpServerResponse,
  releaseRequest: (cancelled: boolean) => Promise<void>
): ProviderHttpServerResponse {
  const body = response.body as HttpServerStreamBody
  let ended = false
  const finalize = onceAsync(async (cancelled: boolean) => {
    if (cancelled) {
      try {
        await releaseRequest(true)
      } finally {
        await body.cursor.close()
      }
      return
    }
    try {
      await body.cursor.close()
    } finally {
      await releaseRequest(false)
    }
  })
  const cancel = () => finalize(true)
  const providerBody: ProviderHttpServerStreamBody = Object.freeze({
    kind: "stream",
    async pull() {
      try {
        const next = await body.cursor.next()
        if (next.done) {
          ended = true
          return { done: true as const, value: undefined }
        }
        const bytes = toUint8Array(next.value)
        if (bytes.length === 0 || bytes.length > 64 * 1024) {
          throw new TypeError(
            "HTTP server response body chunks must contain 1 through 65536 bytes"
          )
        }
        return {
          done: false as const,
          value: Uint8Array.from(bytes),
        }
      } catch (error) {
        await cancel()
        throw error
      }
    },
    async complete() {
      if (!ended) {
        await cancel()
        throw new TypeError(
          "HTTP streaming response cannot complete before the body ends"
        )
      }
      await finalize(false)
    },
    cancel,
  })
  return Object.freeze({
    status: response.status,
    headers: response.headers,
    body: providerBody,
  })
}

function onceAsync<Arguments extends ReadonlyArray<unknown>>(
  action: (...arguments_: Arguments) => Promise<void>
): (...arguments_: Arguments) => Promise<void> {
  let completion: Promise<void> | undefined
  return (...arguments_) => {
    completion ??= action(...arguments_)
    return completion
  }
}

function cancelledResponse(): ProviderHttpServerResponse {
  const cancelled: ProviderHttpServerResponse = Object.freeze({
    status: 204,
    headers: Object.freeze([]),
    body: new Uint8Array(),
  })
  cancelledResponses.add(cancelled)
  return cancelled
}

function closeServerState(
  state: ServerExecutionState,
  cancelRequests: boolean
): Promise<ServiceResult<never, Unit>> {
  state.unregisterCleanup()
  state.closeCompletion ??= (async () => {
    const handle = state.handle
    if (handle === undefined) return httpServerSuccess(undefined)
    const closing = state.server.close(handle, state.context)
    if (cancelRequests) {
      await Promise.allSettled(
        [...state.requests].map((request) => request.cancel())
      )
    }
    const result = await closing
    serverExecutions.delete(handle)
    return result
  })()
  return state.closeCompletion
}

function waitForRequestClaim(
  claimed: Promise<void>,
  context: EffectContext
): Promise<void> {
  if (context.signal.aborted) return Promise.reject(new EffectCancellation())
  return new Promise((resolve, reject) => {
    const abort = (): void => reject(new EffectCancellation())
    context.signal.addEventListener("abort", abort, { once: true })
    void claimed.then(
      () => {
        context.signal.removeEventListener("abort", abort)
        resolve()
      },
      (error) => {
        context.signal.removeEventListener("abort", abort)
        reject(error)
      }
    )
  })
}
