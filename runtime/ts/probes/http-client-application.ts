import type { Effect } from "@seseragi/runtime/effect"
import {
  type HttpClientEnvironment,
  type HttpClientError,
  type HttpClientResponse,
  send,
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
