import { fromUint8Array } from "@seseragi/runtime/bytes"
import {
  createEffectExecution,
  fail,
  registerResourceFinalizer,
  run,
} from "@seseragi/runtime/effect"
import {
  bytesResponse,
  close,
  emptyResponse,
  header,
  type HttpServerEnvironment,
  type HttpServerRequest,
  jsonResponse,
  listen,
  recoverHandler,
  requestBody,
  requestHeaders,
  requestHeaderValues,
  requestMethod,
  requestPath,
  requestQuery,
  requestUrl,
  textResponse,
} from "@seseragi/runtime/http-server"
import { assertProviderConformanceCase } from "@seseragi/runtime/provider-conformance"
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
const target = httpServerTarget(
  requiredEnvironment("SESERAGI_HTTP_SERVER_TARGET")
)
assertPublicSurface()
const loader = new ProviderPackageLoader(target, [
  {
    provider,
    service,
    target,
    module,
    exportName,
    loadMode: "lazy",
    importModule: () => importProviderModule(module, target, exportName),
    source: { path: "src/main.ssrg", start: 0, end: 10 },
  },
])

const selected = await loader.load(provider)
const environment: HttpServerEnvironment = Object.freeze({
  httpServer: createProviderHttpServer(selected),
})
const execution = createEffectExecution()
const started = await run(
  startApplication(port),
  environment,
  execution.context
)
assert(started.kind === "success", "HTTP server must start")
const conflictExecution = createEffectExecution()
const conflict = await run(
  startApplication(port),
  environment,
  conflictExecution.context
)
assert(
  conflict.kind === "failure" &&
    conflict.error.tag === "HttpServerUnavailable",
  "listener bind failure must stay a typed provider failure"
)
assertProviderConformanceCase({
  id: "typed-failure",
  terminal: "typed-failure",
})
await conflictExecution.close()
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
assertProviderConformanceCase({ id: "success", terminal: started.kind })
const closed = await run(close(started.value), environment, execution.context)
assert(closed.kind === "success", "HTTP server resource must close")
await execution.close()
await assertUnavailable(port, "explicit close")

const restartedExecution = createEffectExecution()
const restarted = await run(
  startApplication(port),
  environment,
  restartedExecution.context
)
assert(restarted.kind === "success", "HTTP server must restart after close")
const beforeShutdown = await fetch(`http://127.0.0.1:${port}/shutdown`, {
  headers: { connection: "close" },
})
assert(beforeShutdown.status === 200, "restarted HTTP server must respond")
await restartedExecution.close()
await assertUnavailable(port, "resource scope close")

await assertConcurrentRequests(environment, port)
await assertCloseCancelsInFlight(environment, port)
await loader.shutdown()
await assertUnavailable(port, "provider shutdown")
assertProviderConformanceCase({
  id: "cleanup",
  acquired: 4,
  released: 4,
  active: 0,
})
assertProviderConformanceCase({ id: "leak", activeAfterCleanup: 0 })

process.stdout.write(`HTTP server provider probe passed: ${target}\n`)

function assertPublicSurface(): void {
  const request: HttpServerRequest = Object.freeze({
    method: "POST",
    url: "http://127.0.0.1/users?role=admin#fragment",
    headers: Object.freeze([
      Object.freeze({ name: "x-role", value: "admin" }),
      Object.freeze({ name: "X-Role", value: "editor" }),
    ]),
    body: new Uint8Array([115, 101, 115, 101, 114, 97, 103, 105]),
  })
  assert(requestMethod(request) === "POST", "request method accessor")
  assert(
    requestUrl(request) === request.url,
    "request URL accessor must preserve the absolute URL"
  )
  assert(requestPath(request) === "/users", "request path accessor")
  const query = requestQuery(request)
  assert(
    query.tag === "Just" && query.value === "role=admin",
    "request query accessor"
  )
  assert(requestHeaders(request).length === 2, "request headers accessor")
  assert(
    JSON.stringify(requestHeaderValues("x-role", request)) ===
      JSON.stringify(["admin", "editor"]),
    "request header lookup must be case insensitive and preserve duplicates"
  )
  const body = requestBody(request)
  request.body[0] = 0
  assert(body[0] === 115, "request body accessor must return immutable Bytes")

  const custom = header("x-response", "yes")
  const empty = emptyResponse(204, [custom])
  assert(empty.status === 204 && empty.body.length === 0, "empty response")
  const bytes = bytesResponse(202, [], fromUint8Array(new Uint8Array([1, 2])))
  assert(
    bytes.status === 202 && bytes.body[1] === 2,
    "Bytes response must preserve status and body"
  )
  const text = textResponse(200, [custom], "hello")
  assert(
    new TextDecoder().decode(text.body) === "hello" &&
      text.headers.some(
        (entry) =>
          entry.name === "content-type" &&
          entry.value === "text/plain; charset=utf-8"
      ),
    "text response must encode UTF-8 and add its content type"
  )
  const json = jsonResponse(201, [], '{"ok":true}')
  assert(
    new TextDecoder().decode(json.body) === '{"ok":true}' &&
      json.headers[0]?.value === "application/json; charset=utf-8",
    "JSON response must preserve explicitly encoded JSON text"
  )
}

async function assertConcurrentRequests(
  environment: HttpServerEnvironment,
  port: number
): Promise<void> {
  let active = 0
  let maximum = 0
  let release = (): void => undefined
  const gate = new Promise<void>((resolve) => {
    release = resolve
  })
  const execution = createEffectExecution()
  const started = await run(
    listen({
      hostname: "127.0.0.1",
      port,
      handler() {
        return async () => {
          active += 1
          maximum = Math.max(maximum, active)
          await gate
          active -= 1
          return textResponse(200, [], "done")
        }
      },
    }),
    environment,
    execution.context
  )
  assert(started.kind === "success", "concurrency server must start")
  const first = fetch(`http://127.0.0.1:${port}/first`, {
    headers: { connection: "close" },
  })
  const second = fetch(`http://127.0.0.1:${port}/second`, {
    headers: { connection: "close" },
  })
  await waitUntil(() => active === 2, "concurrent handlers must overlap")
  release()
  const responses = await Promise.all([first, second])
  assert(
    maximum === 2 && responses.every((response) => response.status === 200),
    "concurrent request outcomes must stay independent"
  )
  assertProviderConformanceCase({
    id: "concurrency",
    started: 2,
    settled: responses.length,
    maximumActive: maximum,
  })
  const closed = await run(close(started.value), environment, execution.context)
  assert(closed.kind === "success", "concurrency server must close")
  await execution.close()
}

async function assertCloseCancelsInFlight(
  environment: HttpServerEnvironment,
  port: number
): Promise<void> {
  let startHandler = (): void => undefined
  const handlerStarted = new Promise<void>((resolve) => {
    startHandler = resolve
  })
  let observeCancellation = (): void => undefined
  const cancellationObserved = new Promise<void>((resolve) => {
    observeCancellation = resolve
  })
  let finalized = 0
  const execution = createEffectExecution()
  const recovered = recoverHandler(
    (error: string) => textResponse(418, [], error),
    (_request: HttpServerRequest) => fail("recovered")
  )
  const recoveredResult = await run(
    recovered(
      Object.freeze({ method: "GET", url: "http://local/", headers: [], body: new Uint8Array() })
    ),
    {},
    execution.context
  )
  assert(
    recoveredResult.kind === "success" && recoveredResult.value.status === 418,
    "typed handler failure must require explicit recovery"
  )
  const started = await run(
    listen({
      hostname: "127.0.0.1",
      port,
      handler() {
        return (_environment, context) => {
          if (context === undefined) throw new Error("missing request context")
          registerResourceFinalizer(context, () => {
            finalized += 1
          })
          context.onCancel(() => observeCancellation())
          startHandler()
          return new Promise(() => undefined)
        }
      },
    }),
    environment,
    execution.context
  )
  assert(started.kind === "success", "cancellation server must start")
  const request = fetch(`http://127.0.0.1:${port}/slow`, {
    headers: { connection: "close" },
  }).catch((error: unknown) => error)
  await handlerStarted
  const closed = await run(close(started.value), environment, execution.context)
  assert(closed.kind === "success", "close must settle after request cancellation")
  await cancellationObserved
  await request
  assert(finalized === 1, "cancelled request resources must finalize exactly once")
  assertProviderConformanceCase({
    id: "cancellation",
    terminal: "cancellation",
    notifications: 1,
    lateCompletion: "discarded",
  })
  await execution.close()
}

async function waitUntil(
  predicate: () => boolean,
  message: string
): Promise<void> {
  for (let attempt = 0; attempt < 200; attempt += 1) {
    if (predicate()) return
    await new Promise((resolve) => setTimeout(resolve, 5))
  }
  throw new Error(message)
}

function requiredEnvironment(name: string): string {
  const value = process.env[name]
  if (value === undefined || value.length === 0) {
    throw new Error(`missing ${name}`)
  }
  return value
}

function httpServerTarget(
  value: string
): "bun-process" | "node-process" {
  if (value === "bun-process" || value === "node-process") return value
  throw new Error(`unsupported HTTP server probe target: ${value}`)
}

async function importProviderModule(
  module: string,
  target: "bun-process" | "node-process",
  exportName: string
): Promise<unknown> {
  assert(exportName === "provider", "HTTP server provider export selection")
  const expected =
    target === "bun-process"
      ? "seseragi/runtime-bun/http-server"
      : "seseragi/runtime-node/http-server"
  assert(module === expected, `${target} HTTP server module selection`)
  const loaded =
    target === "bun-process"
      ? await import("seseragi/runtime-bun/http-server")
      : await import("seseragi/runtime-node/http-server")
  return Object.freeze({ provider: loaded.provider })
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
