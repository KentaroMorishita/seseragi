import type { Effect } from "@seseragi/runtime/effect"
import type {
  HttpServerEnvironment,
  HttpServerHandle,
} from "@seseragi/runtime/http-server"
import { jsonResponse, listen } from "@seseragi/runtime/http-server"

/** The application depends on HttpServer only; provider identity stays wiring. */
export function startApplication(
  port: number
): Effect<HttpServerEnvironment, unknown, HttpServerHandle> {
  return listen({
    hostname: "127.0.0.1",
    port,
    async handler(request) {
      await Promise.resolve()
      const url = new URL(request.url)
      return jsonResponse({
        message: "Hello from Seseragi",
        method: request.method,
        path: url.pathname,
        body: new TextDecoder().decode(request.body),
      })
    },
  })
}
