import { createServer } from "node:http"
import type { Socket } from "node:net"
import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "@seseragi/runtime/effect"
import { responseBody } from "@seseragi/runtime/http-client"
import { assertProviderConformanceCase } from "@seseragi/runtime/provider-conformance"
import { createProviderHttpClient } from "@seseragi/runtime/provider-http-client"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import { provider as bunProvider } from "seseragi/runtime-bun/http-client"
import { provider as nodeProvider } from "seseragi/runtime-node/http-client"
import {
  postJson,
  readSlowBody,
  smallResponse,
} from "./http-client-application.ts"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

let markSlowStarted = (): void => undefined
const slowStarted = new Promise<void>((resolve) => {
  markSlowStarted = resolve
})
const sockets = new Set<Socket>()
const timers = new Set<ReturnType<typeof setTimeout>>()
const oversized = new Map<
  string,
  { fullyWritten: boolean; closed: Promise<void> }
>()
const server = createServer((request, response) => {
  if (request.url?.startsWith("/bounded/")) {
    const path = request.url
    request.resume()
    request.on("end", () => {
      response.writeHead(200, { "content-type": "application/octet-stream" })
      if (path.endsWith("/exact")) {
        response.end(new Uint8Array([1, 2, 3]))
        return
      }
      let markClosed = (): void => undefined
      const observation = {
        fullyWritten: false,
        closed: new Promise<void>((resolve) => {
          markClosed = resolve
        }),
      }
      oversized.set(path, observation)
      // The response is finite even if a regressing client buffers it all.
      // The bounded client must close before this final part is produced.
      const timer = setTimeout(() => {
        timers.delete(timer)
        observation.fullyWritten = true
        response.end(new Uint8Array([5, 6]))
      }, 2000)
      timers.add(timer)
      response.once("close", () => {
        clearTimeout(timer)
        timers.delete(timer)
        markClosed()
      })
      response.write(new Uint8Array([1, 2, 3, 4]))
    })
    return
  }
  if (request.url === "/slow") {
    response.writeHead(200, { "content-type": "text/plain" })
    response.write("started")
    markSlowStarted()
    return
  }
  const chunks: Uint8Array[] = []
  request.on("data", (chunk: Uint8Array) => chunks.push(chunk))
  request.on("end", () => {
    response.writeHead(200, { "content-type": "application/json" })
    response.end(
      JSON.stringify({
        method: request.method,
        path: request.url,
        body: new TextDecoder().decode(Buffer.concat(chunks)),
      })
    )
  })
})
server.on("connection", (socket) => {
  sockets.add(socket)
  socket.once("close", () => sockets.delete(socket))
})

const provider = requiredEnvironment("SESERAGI_HTTP_CLIENT_PROVIDER")
const service = requiredEnvironment("SESERAGI_HTTP_CLIENT_SERVICE")
const module = requiredEnvironment("SESERAGI_HTTP_CLIENT_MODULE")
const exportName = requiredEnvironment("SESERAGI_HTTP_CLIENT_EXPORT")
const target = requiredEnvironment("SESERAGI_HTTP_CLIENT_TARGET") as
  | "bun-process"
  | "node-process"
const loader = new ProviderPackageLoader(target, [
  {
    provider,
    service,
    target,
    module,
    exportName,
    loadMode: "lazy",
    importModule: selectedImport(module),
    source: { path: "src/main.ssrg", start: 0, end: 10 },
  },
])
await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve))
try {
  const address = server.address()
  assert(typeof address === "object" && address !== null, "server must listen")
  const environment = Object.freeze({
    httpClient: createProviderHttpClient(await loader.load(provider)),
  })
  const base = `http://127.0.0.1:${address.port}`
  const result = await run(postJson(`${base}/json`), environment)
  assert(result.kind === "success", "HTTP client request must succeed")
  assert(result.value.status === 200, "HTTP client must preserve status")
  assert(
    JSON.stringify(JSON.parse(new TextDecoder().decode(result.value.body))) ===
      JSON.stringify({ method: "POST", path: "/json", body: "request-body" }),
    "HTTP client must cross the same request and response boundary"
  )
  assertProviderConformanceCase({ id: "success", terminal: result.kind })

  for (const withBody of [false, true]) {
    const route = `/bounded/${withBody ? "bytes" : "empty"}`
    const exact = await run(
      smallResponse(`${base}${route}/exact`, withBody),
      environment
    )
    assert(
      exact.kind === "success",
      "the exact response body limit must succeed"
    )
    assert(
      Array.from(responseBody(exact.value)).join(",") === "1,2,3",
      `the bounded response must preserve every byte at the limit (${route}: ${Array.from(responseBody(exact.value)).join(",")})`
    )
    const failure = await run(
      smallResponse(`${base}${route}/overflow`, withBody),
      environment
    )
    assert(
      failure.kind === "failure" &&
        failure.error.tag === "HttpResponseBodyLimitExceeded" &&
        failure.error.value.limitBytes === 3,
      "the first overflowing response chunk must be a typed body limit failure"
    )
    const observation = oversized.get(`${route}/overflow`)
    assert(observation !== undefined, "the bounded response must start")
    await observation.closed
    assert(
      !observation.fullyWritten,
      "the response must be cancelled before EOF"
    )
  }

  const execution = createEffectExecution()
  const pending = run(
    readSlowBody(`${base}/slow`),
    environment,
    execution.context
  ).catch((error: unknown) => error)
  await slowStarted
  await execution.cancel()
  assert(
    isEffectCancellation(await pending),
    "response body cancellation must stay outside typed failure"
  )
  assertProviderConformanceCase({
    id: "cancellation",
    terminal: "cancellation",
    notifications: 1,
    lateCompletion: "discarded",
  })
} finally {
  try {
    await loader.shutdown()
  } finally {
    for (const timer of timers) clearTimeout(timer)
    for (const socket of sockets) socket.destroy()
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error === undefined ? resolve() : reject(error)))
    )
  }
}
process.stdout.write(`HTTP client provider probe passed: ${target}\n`)

function requiredEnvironment(name: string): string {
  const value = process.env[name]
  if (value === undefined || value.length === 0)
    throw new Error(`missing ${name}`)
  return value
}

function selectedImport(module: string): () => Promise<unknown> {
  if (module === "seseragi/runtime-bun/http-client") {
    return async () => Object.freeze({ provider: bunProvider })
  }
  if (module === "seseragi/runtime-node/http-client") {
    return async () => Object.freeze({ provider: nodeProvider })
  }
  throw new Error(`unexpected HTTP client provider module: ${module}`)
}
