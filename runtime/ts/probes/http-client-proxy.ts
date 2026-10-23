import assert from "node:assert/strict"
import { createServer, type Socket } from "node:net"
import { gzipSync } from "node:zlib"
import { createFetchHttpClientProvider } from "../../providers/http-client"
import { run } from "../src/effect"
import {
  bodyLimit,
  get,
  parseUrl,
  request,
  sendEmpty,
} from "../src/http-client"
import { createProviderHttpClient } from "../src/provider-http-client"

const sockets = new Set<Socket>()
const timers = new Set<ReturnType<typeof setTimeout>>()
const paths: string[] = []
let overflowReachedEof = false
const server = createServer((socket) => {
  sockets.add(socket)
  socket.on("error", () => undefined)
  socket.once("close", () => sockets.delete(socket))
  let input = ""
  socket.on("data", function onData(chunk) {
    input += chunk.toString("latin1")
    assert(input.length < 4096)
    if (!input.includes("\r\n\r\n")) return
    socket.off("data", onData)
    const target = input.split(" ")[1]!
    const url = new URL(target)
    assert.equal(url.hostname, "proxy-only.invalid")
    assert(
      input.includes(
        "proxy-authorization: Basic Zml4dHVyZS11c2VyOmZpeHR1cmUtcGFzcw=="
      )
    )
    paths.push(url.pathname)
    if (url.pathname === "/informational") {
      socket.write("HTTP/1.1 103 Early Hints\r\n\r\n".repeat(17))
    } else if (url.pathname === "/headers") {
      socket.write(`HTTP/1.1 200 OK\r\nX-Large: ${"x".repeat(17000)}`)
    } else if (url.pathname === "/trailers") {
      socket.write(
        `HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX-Large: ${"x".repeat(17000)}`
      )
    } else if (url.pathname === "/chunk-line") {
      socket.write(
        `HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1;${"x".repeat(1100)}`
      )
    } else {
      const overflow = url.pathname === "/overflow"
      const encoded = gzipSync(
        Buffer.from(overflow ? "x".repeat(256 * 1024) : "abc")
      )
      socket.write(
        `HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: ${encoded.length}\r\nConnection: close\r\n\r\n`
      )
      if (!overflow) socket.end(encoded)
      else {
        socket.write(encoded.subarray(0, -8))
        const timer = setTimeout(() => {
          timers.delete(timer)
          overflowReachedEof = true
          socket.end(encoded.subarray(-8))
        }, 700)
        timers.add(timer)
        socket.once("close", () => {
          clearTimeout(timer)
          timers.delete(timer)
        })
      }
    }
  })
})
const identity = "seseragi/runtime-bun#http-client"
const environment = {
  httpClient: createProviderHttpClient({
    provider: identity,
    service: "std/http::HttpClient",
    entry: createFetchHttpClientProvider(identity, "bun-process"),
  }),
}
async function send(path: string) {
  const parsed = parseUrl(`http://proxy-only.invalid${path}`)
  const limit = bodyLimit(3)
  assert.equal(parsed.tag, "Right")
  assert.equal(limit.tag, "Right")
  if (parsed.tag !== "Right" || limit.tag !== "Right")
    throw new Error("invalid fixture")
  return await run(
    sendEmpty(limit.value, request(get, parsed.value)),
    environment
  )
}
try {
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve))
  const address = server.address()
  assert(address && typeof address !== "string")
  for (const key of [
    "HTTP_PROXY",
    "http_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
    "NO_PROXY",
    "no_proxy",
  ])
    delete process.env[key]
  process.env.HTTP_PROXY = `http://fixture-user:fixture-pass@127.0.0.1:${address.port}`
  const normal = await send("/gzip")
  assert.equal(normal.kind, "success")
  if (normal.kind === "success")
    assert.equal(new TextDecoder().decode(normal.value.body), "abc")
  const overflow = await send("/overflow")
  assert.equal(overflow.kind, "failure")
  if (overflow.kind === "failure")
    assert.equal(overflow.error.tag, "HttpResponseBodyLimitExceeded")
  assert.equal(overflowReachedEof, false)
  for (const path of [
    "/informational",
    "/headers",
    "/trailers",
    "/chunk-line",
  ]) {
    const result = await send(path)
    assert.equal(result.kind, "failure")
    if (result.kind === "failure")
      assert.equal(result.error.tag, "HttpProtocolFailure")
  }
  assert.equal((await send("/recovery")).kind, "success")
  assert.equal(paths.length, 7)
  console.log("HTTP proxy helper probe passed")
} finally {
  for (const timer of timers) clearTimeout(timer)
  for (const socket of sockets) socket.destroy()
  await new Promise<void>((resolve) => server.close(() => resolve()))
}
