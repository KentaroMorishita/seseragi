import { createServer } from "node:http"
import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "@seseragi/runtime/effect"
import { assertProviderConformanceCase } from "@seseragi/runtime/provider-conformance"
import { createProviderHttpClient } from "@seseragi/runtime/provider-http-client"
import { ProviderPackageLoader } from "@seseragi/runtime/provider-package"
import { provider as bunProvider } from "seseragi/runtime-bun/http-client"
import { provider as nodeProvider } from "seseragi/runtime-node/http-client"
import { postJson, readSlowBody } from "./http-client-application.ts"

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

let markSlowStarted = (): void => undefined
const slowStarted = new Promise<void>((resolve) => {
  markSlowStarted = resolve
})
const server = createServer((request, response) => {
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
await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve))
const address = server.address()
assert(typeof address === "object" && address !== null, "server must listen")

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
await loader.shutdown()
await new Promise<void>((resolve, reject) =>
  server.close((error) => (error === undefined ? resolve() : reject(error)))
)
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
