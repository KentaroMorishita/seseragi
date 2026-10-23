import { fromUint8Array } from "@seseragi/runtime/bytes"
import type { Effect } from "@seseragi/runtime/effect"
import {
  bodyLimit,
  get,
  type HttpClientEnvironment,
  type HttpClientError,
  type HttpClientResponse,
  parseUrl,
  post,
  type Response,
  request,
  send,
  sendBytes,
  sendEmpty,
} from "@seseragi/runtime/http-client"

export function postJson(
  url: string
): Effect<HttpClientEnvironment, HttpClientError, HttpClientResponse> {
  return send({
    method: "POST",
    url,
    headers: [{ name: "content-type", value: "text/plain" }],
    body: new TextEncoder().encode("request-body"),
  })
}

export function readSlowBody(
  url: string
): Effect<HttpClientEnvironment, HttpClientError, HttpClientResponse> {
  return send({ method: "GET", url, headers: [], body: new Uint8Array() })
}

export function smallResponse(
  url: string,
  withBody: boolean
): Effect<HttpClientEnvironment, HttpClientError, Response> {
  const parsed = parseUrl(url)
  const limit = bodyLimit(3)
  if (parsed.tag !== "Right" || limit.tag !== "Right") {
    throw new Error("invalid bounded HTTP fixture")
  }
  return withBody
    ? sendBytes(
        limit.value,
        fromUint8Array(new Uint8Array([7, 8])),
        request(post, parsed.value)
      )
    : sendEmpty(limit.value, request(get, parsed.value))
}
