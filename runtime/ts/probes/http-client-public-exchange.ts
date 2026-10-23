import assert from "node:assert/strict"
import { createServer, get as rawGet, type Server } from "node:http"
import type { Socket } from "node:net"
import { gunzipSync, gzipSync } from "node:zlib"
import { createFetchHttpClientProvider } from "../../providers/http-client"
import { fromUint8Array } from "../src/bytes"
import { run } from "../src/effect"
import {
  bodyLimit,
  emptyBody,
  exchange,
  get,
  parseUrl,
  post,
  request,
  responseBody,
  sendBytes,
  sendEmpty,
} from "../src/http-client"
import { createProviderHttpClient } from "../src/provider-http-client"
import { ProviderPackageLoader } from "../src/provider-package"
import { runCollect } from "../src/stream"

const target = process.argv[2]
assert(target === "node-process" || target === "bun-process")
const observe = process.argv[3] === "observe"
const plain = Buffer.from("raw exchange: proxy payload\n")
const encoded = gzipSync(plain)
const limitedPlain = Buffer.alloc(256, 65)
const limitedEncoded = gzipSync(limitedPlain)
assert(limitedEncoded.length < 64 && limitedPlain.length > 64)
const sockets = new Set<Socket>()
const proxyTasks = new Set<Promise<void>>()
const providerName = `compatibility/${target}#http-client`
const provider = createFetchHttpClientProvider(providerName, target)
const loader = new ProviderPackageLoader(target, [
  {
    provider: providerName,
    service: "std/http::HttpClient",
    target,
    module: "candidate/public-http-exchange-raw",
    exportName: "provider",
    loadMode: "lazy",
    importModule: async () => ({ provider }),
  },
])
const environment = {
  httpClient: createProviderHttpClient(await loader.load(providerName)),
}
const origin = createServer((incoming, response) => {
  incoming.resume()
  incoming.once("end", () => {
    const bytes = incoming.url === "/limited" ? limitedEncoded : encoded
    response.writeHead(200, {
      "content-encoding": "gzip",
      "content-length": String(bytes.length),
      connection: "close",
    })
    response.end(bytes)
  })
})
let originBase = ""
type CollectedResponse = {
  status: number
  headers: ReadonlyArray<{ name: string; value: string }>
  bytes: Buffer
}
type RawResponse = {
  status: number
  headers: Readonly<Record<string, unknown>>
  bytes: Buffer
  aborted: boolean
  error?: string
}
let forwarded: CollectedResponse | undefined
const proxy = createServer((_incoming, response) => {
  const task = (async () => {
    forwarded = await collectPublic(`${originBase}/gzip`)
    response.writeHead(
      forwarded.status,
      forwarded.headers.flatMap(({ name, value }) => [name, value])
    )
    response.end(forwarded.bytes)
  })().catch((cause) => {
    response.destroy(cause instanceof Error ? cause : new Error(String(cause)))
  })
  proxyTasks.add(task)
  void task.finally(() => proxyTasks.delete(task))
})
for (const server of [origin, proxy]) {
  server.on("connection", (socket) => {
    sockets.add(socket)
    socket.once("close", () => sockets.delete(socket))
  })
}
let report: Record<string, unknown> = {}
let compatible = false
try {
  originBase = await listen(origin)
  const proxyBase = await listen(proxy)
  const direct = await collectPublic(`${originBase}/gzip`)
  const explicit = decode(direct.bytes)
  const proxied = await readRaw(`${proxyBase}/proxy`)
  assert(forwarded !== undefined)
  const proxyDecoded = decode(proxied.bytes)
  const exact = await run(
    sendEmpty(limit(256), buildRequest(`${originBase}/limited`)),
    environment
  )
  const helperExactDecoded =
    exact.kind === "success" &&
    Buffer.from(responseBody(exact.value)).equals(limitedPlain)
  const helperLimits: Array<{
    helper: string
    kind: string
    decodedLimitEnforced: boolean
    error?: unknown
    bodyBytes?: number
  }> = []
  for (const withUpload of [false, true]) {
    const selected = buildRequest(`${originBase}/limited`, withUpload)
    const result = await run(
      withUpload
        ? sendBytes(limit(64), fromUint8Array(new Uint8Array([7, 8])), selected)
        : sendEmpty(limit(64), selected),
      environment
    )
    helperLimits.push({
      helper: withUpload ? "sendBytes" : "sendEmpty",
      kind: result.kind,
      ...(result.kind === "failure"
        ? { error: result.error }
        : { bodyBytes: responseBody(result.value).length }),
      decodedLimitEnforced:
        result.kind === "failure" &&
        result.error.tag === "HttpResponseBodyLimitExceeded" &&
        result.error.value.limitBytes === 64,
    })
  }
  const rawMatches = direct.bytes.equals(encoded)
  const forwardedMatches = forwarded.bytes.equals(encoded)
  const proxyMatches = proxied.bytes.equals(encoded)
  compatible =
    rawMatches &&
    explicit.matches &&
    forwardedMatches &&
    proxyMatches &&
    proxyDecoded.matches &&
    !proxied.aborted &&
    helperExactDecoded &&
    helperLimits.every((value) => value.decodedLimitEnforced)
  report = {
    target,
    runtime:
      process.versions.bun === undefined
        ? `Node ${process.versions.node}`
        : `Bun ${process.versions.bun}`,
    pid: process.pid,
    finiteFixture: {
      plainBytes: plain.length,
      gzipBytes: encoded.length,
      limitedPlainBytes: limitedPlain.length,
      limitedGzipBytes: limitedEncoded.length,
      decodedLimit: 64,
    },
    publicExchange: {
      status: direct.status,
      headers: direct.headers,
      bytes: [...direct.bytes],
      rawMatches,
      plainMatches: direct.bytes.equals(plain),
      explicitDecode: explicit,
    },
    transparentProxy: {
      forwardedHeaders: forwarded.headers,
      forwardedBodyBytes: forwarded.bytes.length,
      forwardedMatches,
      status: proxied.status,
      receivedHeaders: proxied.headers,
      receivedBytes: [...proxied.bytes],
      rawMatches: proxyMatches,
      aborted: proxied.aborted,
      transferError: proxied.error,
      explicitDecode: proxyDecoded,
    },
    helpers: {
      exactDecoded: helperExactDecoded,
      exactResult:
        exact.kind === "failure"
          ? exact
          : { kind: exact.kind, bodyBytes: responseBody(exact.value).length },
      limits: helperLimits,
    },
    compatible,
  }
} finally {
  try {
    await deadline(loader.shutdown(), "provider cleanup")
  } finally {
    const closed = [...sockets].map(
      (socket) => new Promise<void>((resolve) => socket.once("close", resolve))
    )
    for (const socket of sockets) socket.destroy()
    await deadline(
      Promise.all(
        [origin, proxy]
          .filter((server) => server.listening)
          .map(
            (server) =>
              new Promise<void>((resolve, reject) =>
                server.close((error) =>
                  error === undefined ? resolve() : reject(error)
                )
              )
          )
      ),
      "server cleanup"
    )
    await deadline(Promise.all(closed), "socket cleanup")
    await deadline(Promise.allSettled([...proxyTasks]), "proxy task cleanup")
    assert.equal(sockets.size, 0)
    report = {
      ...report,
      remainingSockets: sockets.size,
      remainingProxyTasks: proxyTasks.size,
    }
  }
}
process.stdout.write(`${JSON.stringify(report)}\n`)
if (!observe)
  assert(
    compatible,
    "public native exchange must remain raw while convenience helpers decode and limit decoded bytes"
  )

function buildRequest(url: string, upload = false) {
  const parsed = parseUrl(url)
  if (parsed.tag !== "Right") throw new Error("invalid fixture URL")
  return request(upload ? post : get, parsed.value)
}
function limit(bytes: number) {
  const result = bodyLimit(bytes)
  if (result.tag !== "Right") throw new Error("invalid fixture limit")
  return result.value
}
async function collectPublic(url: string): Promise<CollectedResponse> {
  const outcome = await deadline(
    run(
      runCollect(exchange(emptyBody<unknown, never>(), buildRequest(url))),
      environment
    ),
    "public exchange"
  )
  assert.equal(outcome.kind, "success", JSON.stringify(outcome))
  if (outcome.kind !== "success") throw new Error("public exchange failed")
  let status = 0
  let headers: ReadonlyArray<{ name: string; value: string }> = []
  const chunks: Buffer[] = []
  for (const event of outcome.value) {
    if (event.tag === "ResponseStarted") {
      status = event.value.status
      headers = event.value.headers
    }
    if (event.tag === "ResponseBodyChunk") chunks.push(Buffer.from(event.value))
  }
  return { status, headers, bytes: Buffer.concat(chunks) }
}
function decode(bytes: Buffer) {
  try {
    return { success: true, matches: gunzipSync(bytes).equals(plain) }
  } catch (cause) {
    return {
      success: false,
      matches: false,
      code:
        typeof cause === "object" && cause !== null && "code" in cause
          ? String(cause.code)
          : undefined,
      message: cause instanceof Error ? cause.message : String(cause),
    }
  }
}
async function listen(server: Server): Promise<string> {
  await deadline(
    new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve)),
    "listen"
  )
  const address = server.address()
  assert(address !== null && typeof address !== "string")
  return `http://127.0.0.1:${address.port}`
}
async function readRaw(url: string): Promise<RawResponse> {
  let client: ReturnType<typeof rawGet> | undefined
  try {
    return await deadline(
      new Promise<RawResponse>((resolve) => {
        let settled = false
        let status = 0
        let headers: Readonly<Record<string, unknown>> = {}
        const chunks: Buffer[] = []
        const finish = (aborted: boolean, error?: string) => {
          if (settled) return
          settled = true
          resolve({
            status,
            headers,
            bytes: Buffer.concat(chunks),
            aborted,
            error,
          })
        }
        client = rawGet(url, { agent: false }, (response) => {
          status = response.statusCode ?? 0
          headers = response.headers
          response.on("data", (chunk: Buffer) => chunks.push(chunk))
          response.once("end", () => finish(false))
          response.once("aborted", () => finish(true, "response aborted"))
          response.once("error", (cause) => finish(true, cause.message))
        })
        client.once("error", (cause) => finish(true, cause.message))
      }),
      "transparent proxy response"
    )
  } finally {
    client?.destroy()
  }
}
async function deadline<Value>(
  promise: Promise<Value>,
  label: string
): Promise<Value> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(`${label} timed out`)), 3000)
      }),
    ])
  } finally {
    clearTimeout(timer)
  }
}
