import { fromUint8Array, type Bytes } from "./bytes"
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
  body: Uint8Array
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
  handler: (request: HttpServerRequest) => Promise<HttpServerResponse>
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
export function isCancelledHttpServerResponse(
  response: HttpServerResponse
): boolean {
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
  body: Uint8Array
): HttpServerResponse {
  validateStatus(status)
  const snapshotHeaders = headers.map(snapshotHeader)
  return Object.freeze({
    status,
    headers: Object.freeze(snapshotHeaders),
    body: new Uint8Array(body),
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
        return result.value
      } catch (error) {
        if (isEffectCancellation(error)) return cancelledResponse()
        throw error
      } finally {
        await execution.close()
        state.requests.delete(requestExecution)
      }
    },
  })
}

function cancelledResponse(): HttpServerResponse {
  const cancelled = response(204, [], new Uint8Array())
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
