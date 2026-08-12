import type { Effect, EffectContext, Unit } from "./effect"
import type { ProviderHandle } from "./provider"
import {
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"

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

export type HttpServerHandler = (
  request: HttpServerRequest
) => Promise<HttpServerResponse>

export type HttpServerOptions = Readonly<{
  hostname?: string
  port: number
  handler: HttpServerHandler
}>

export type HttpServerError = Readonly<{
  tag: "HttpServerUnavailable"
  message: string
}>

export type HttpServerHandle = ProviderHandle

export type HttpServer = Readonly<{
  listen: (
    options: HttpServerOptions,
    context: EffectContext
  ) => Promise<ServiceResult<HttpServerError, HttpServerHandle>>
  close: (
    server: HttpServerHandle,
    context: EffectContext
  ) => Promise<ServiceResult<never, Unit>>
}>

export type HttpServerEnvironment = Readonly<{ httpServer: HttpServer }>

export function jsonResponse(value: unknown, status = 200): HttpServerResponse {
  return response(
    status,
    [{ name: "content-type", value: "application/json; charset=utf-8" }],
    new TextEncoder().encode(JSON.stringify(value))
  )
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

export function listen(
  options: HttpServerOptions
): Effect<HttpServerEnvironment, HttpServerError, HttpServerHandle> {
  return serviceEffect((environment, context) =>
    environment.httpServer.listen(options, context)
  )
}

export function close(
  server: HttpServerHandle
): Effect<HttpServerEnvironment, never, Unit> {
  return serviceEffect((environment, context) =>
    environment.httpServer.close(server, context)
  )
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
