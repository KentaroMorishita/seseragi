import { run } from "@seseragi/runtime/effect"
import {
  close,
  type HttpServerEnvironment,
} from "@seseragi/runtime/http-server"
import { createProviderHttpServer } from "@seseragi/runtime/provider-http-server"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import { startApplication } from "./http-server-application"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

const provider = requiredEnvironment("SESERAGI_HTTP_SERVER_PROVIDER")
const service = requiredEnvironment("SESERAGI_HTTP_SERVER_SERVICE")
const module = requiredEnvironment("SESERAGI_HTTP_SERVER_MODULE")
const exportName = requiredEnvironment("SESERAGI_HTTP_SERVER_EXPORT")
const port = Number(requiredEnvironment("SESERAGI_HTTP_SERVER_PORT"))
const loader = new ProviderPackageLoader("bun-process", [
  {
    provider,
    service,
    target: "bun-process",
    module,
    exportName,
    loadMode: "lazy",
    importModule: () => import(module),
    source: { path: "src/main.ssrg", start: 0, end: 10 },
  },
])

const selected = await loader.load(provider)
const environment: HttpServerEnvironment = Object.freeze({
  httpServer: createProviderHttpServer(selected),
})
const started = await run(startApplication(port), environment)
assert(started.kind === "success", "HTTP server must start")
const response = await fetch(`http://127.0.0.1:${port}/hello`, {
  method: "POST",
  headers: { connection: "close" },
  body: "request-body",
})
assert(response.status === 200, "HTTP server must return status 200")
assert(
  response.headers.get("content-type") === "application/json; charset=utf-8",
  "HTTP server must preserve response headers"
)
assert(
  JSON.stringify(await response.json()) ===
    JSON.stringify({
      message: "Hello from Seseragi",
      method: "POST",
      path: "/hello",
      body: "request-body",
    }),
  "HTTP server must cross the request and JSON response boundary"
)
const closed = await run(close(started.value), environment)
assert(closed.kind === "success", "HTTP server resource must close")
await assertUnavailable(port, "explicit close")

const restarted = await run(startApplication(port), environment)
assert(restarted.kind === "success", "HTTP server must restart after close")
const beforeShutdown = await fetch(`http://127.0.0.1:${port}/shutdown`, {
  headers: { connection: "close" },
})
assert(beforeShutdown.status === 200, "restarted HTTP server must respond")
await loader.shutdown()
await assertUnavailable(port, "provider shutdown")

process.stdout.write("HTTP server provider probe passed\n")

function requiredEnvironment(name: string): string {
  const value = process.env[name]
  if (value === undefined || value.length === 0) {
    throw new Error(`missing ${name}`)
  }
  return value
}

async function assertUnavailable(
  port: number,
  boundary: string
): Promise<void> {
  const result = await fetch(`http://127.0.0.1:${port}/closed`, {
    headers: { connection: "close" },
    signal: AbortSignal.timeout(1_000),
  }).catch((error: unknown) => error)
  assert(result instanceof Error, `${boundary} must stop accepting`)
}
