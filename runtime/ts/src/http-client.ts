import type { Effect, EffectContext } from "./effect"
import type { ServiceResult } from "./service"
import { serviceEffect, serviceFailure, serviceSuccess } from "./service"

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
export type HttpClientError = Readonly<{
  tag: "HttpRequestFailed"
  message: string
}>
export type HttpClient = Readonly<{
  send: (
    request: HttpClientRequest,
    context: EffectContext
  ) => Promise<ServiceResult<HttpClientError, HttpClientResponse>>
}>
export type HttpClientEnvironment = Readonly<{ httpClient: HttpClient }>

export function send(
  request: HttpClientRequest
): Effect<HttpClientEnvironment, HttpClientError, HttpClientResponse> {
  return serviceEffect((environment, context) =>
    environment.httpClient.send(request, context)
  )
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
