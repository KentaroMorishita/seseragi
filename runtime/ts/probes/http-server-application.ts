import { succeed, type Effect } from "@seseragi/runtime/effect"
import type {
  HttpServerEnvironment,
  HttpServerHandle,
} from "@seseragi/runtime/http-server"
import {
  jsonResponse,
  listen,
  requestBody,
  requestMethod,
  requestPath,
} from "@seseragi/runtime/http-server"
import { decodeUtf8Lossy } from "@seseragi/runtime/text"

/** The application depends on HttpServer only; provider identity stays wiring. */
export function startApplication(
  port: number
): Effect<HttpServerEnvironment, unknown, HttpServerHandle> {
  return listen({
    hostname: "127.0.0.1",
    port,
    handler(request) {
      return succeed(
        jsonResponse(
          200,
          [],
          JSON.stringify({
            message: "Hello from Seseragi",
            method: requestMethod(request),
            path: requestPath(request),
            body: decodeUtf8Lossy(requestBody(request)),
          })
        )
      )
    },
  })
}
