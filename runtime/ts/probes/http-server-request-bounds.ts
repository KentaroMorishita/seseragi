import assert from "node:assert/strict"
import type { IncomingMessage, Server, ServerResponse } from "node:http"
import { connect, type Socket } from "node:net"
import { createNodeHttpServerProvider } from "../../providers/node/http-server"
import type { ProviderHttpServerStreamBody } from "../src/http-server"
import type { ProviderResult } from "../src/provider"
import { ProviderPackageLoader } from "../src/provider-package"

type Token = {
  server: Server
  children: Set<Promise<void>>
  pendingBodies: Set<IncomingMessage>
}

const bodyLimit = 128

async function fixture(maxRequestBodyBytes?: number) {
  const provider = createNodeHttpServerProvider(
    maxRequestBodyBytes === undefined ? {} : { maxRequestBodyBytes }
  )
  const received: Uint8Array[] = []
  const result = (await provider.listen?.({
    hostname: "127.0.0.1",
    port: 0,
    async handler(request: { body: Uint8Array }) {
      received.push(request.body)
      return { status: 200, headers: [], body: request.body }
    },
  })) as ProviderResult
  assert(result?.kind === "success", "HTTP listener must start")
  const token = result.value as Token
  const address = token.server.address()
  assert(address !== null && typeof address !== "string")
  return {
    token,
    received,
    port: address.port,
    async close() {
      await deadline(provider.close?.(token), "provider close")
      assert.equal(token.server.listening, false)
      assert.equal(token.children.size, 0, "all request children must settle")
      assert.equal(token.pendingBodies.size, 0, "all body reads must settle")
    },
  }
}

function deadline<T>(value: T | PromiseLike<T>, message: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined
  return Promise.race([
    Promise.resolve(value),
    new Promise<never>((_resolve, reject) => {
      timer = setTimeout(() => reject(new Error(`${message} timed out`)), 2_000)
    }),
  ]).finally(() => clearTimeout(timer))
}

async function rawRequest(
  port: number,
  payload: string,
  followingChunk?: { server: Server; payload: string }
): Promise<string> {
  const socket = connect(port, "127.0.0.1")
  try {
    if (followingChunk !== undefined) {
      followingChunk.server.once("request", (request) => {
        request.once("data", () => socket.write(followingChunk.payload))
      })
    }
    return await deadline(
      new Promise<string>((resolve, reject) => {
        let response = ""
        socket.on("error", reject)
        socket.on("data", (chunk) => {
          response += chunk.toString("utf8")
          assert(response.length < 4_096, "rejection output must stay bounded")
        })
        socket.on("end", () => resolve(response))
        socket.once("connect", () => socket.write(payload))
      }),
      "HTTP response without a completed upload"
    )
  } finally {
    await destroySocket(socket)
  }
}

function requestHeaders(...headers: string[]): string {
  return ["POST /upload HTTP/1.1", "Host: 127.0.0.1", ...headers, "", ""].join(
    "\r\n"
  )
}

async function destroySocket(socket: Socket): Promise<void> {
  if (socket.closed) return
  await new Promise<void>((resolve) => {
    socket.once("close", () => resolve())
    socket.destroy()
  })
}

async function startedPartialUpload(
  selected: Awaited<ReturnType<typeof fixture>>
): Promise<Socket> {
  const socket = connect(selected.port, "127.0.0.1")
  // Keep the error listener until close; destruction during shutdown can
  // report a reset on either side of the connection.
  socket.on("error", () => undefined)
  try {
    await deadline(
      new Promise<void>((resolve, reject) => {
        socket.once("error", reject)
        socket.once("connect", () => {
          socket.write(`${requestHeaders(`Content-Length: ${bodyLimit}`)}x`)
        })
        selected.token.server.once("request", () => resolve())
      }),
      "partial upload start"
    )
    return socket
  } catch (cause) {
    await destroySocket(socket)
    throw cause
  }
}

for (const maxRequestBodyBytes of [0, -1, 1.5, NaN, Infinity]) {
  assert.throws(() => createNodeHttpServerProvider({ maxRequestBodyBytes }))
}

const selected = await fixture(bodyLimit)
try {
  for (const length of [0, bodyLimit]) {
    const response = await fetch(`http://127.0.0.1:${selected.port}/upload`, {
      method: "POST",
      headers: { connection: "close" },
      body: new Uint8Array(length).fill(97),
    })
    assert.equal(response.status, 200)
    assert.deepEqual(
      new Uint8Array(await response.arrayBuffer()),
      new Uint8Array(length).fill(97)
    )
  }
  assert.equal(selected.received.length, 2)

  const declared = await rawRequest(
    selected.port,
    requestHeaders(`Content-Length: ${bodyLimit + 1}`)
  )
  assert.match(declared, /^HTTP\/1\.1 413 /)
  assert.match(declared, /connection: close/iu)
  assert.equal(selected.received.length, 2, "oversize handler must not run")
  await deadline(
    Promise.allSettled([...selected.token.children]),
    "413 cleanup"
  )
  assert.equal(selected.token.children.size, 0)
  assert.equal(selected.token.pendingBodies.size, 0)

  const chunked = await rawRequest(
    selected.port,
    `${requestHeaders("Transfer-Encoding: chunked")}40\r\n${"a".repeat(64)}\r\n`,
    {
      server: selected.token.server,
      payload: `41\r\n${"b".repeat(65)}\r\n`,
    }
  )
  assert.match(chunked, /^HTTP\/1\.1 413 /)
  assert.match(chunked, /connection: close/iu)
  assert.equal(selected.received.length, 2, "chunked handler must not run")
  await deadline(
    Promise.allSettled([...selected.token.children]),
    "chunked cleanup"
  )
  assert.equal(selected.token.children.size, 0)
  assert.equal(selected.token.pendingBodies.size, 0)

  const exactChunked = await rawRequest(
    selected.port,
    `${requestHeaders("Transfer-Encoding: chunked", "Connection: close")}40\r\n${"a".repeat(64)}\r\n40\r\n${"b".repeat(64)}\r\n0\r\n\r\n`
  )
  assert.match(exactChunked, /^HTTP\/1\.1 200 /)
  assert.equal(selected.received.length, 3)
  assert.equal(selected.received[2]?.byteLength, bodyLimit)

  const abandoned = await startedPartialUpload(selected)
  await destroySocket(abandoned)
  await deadline(Promise.allSettled([...selected.token.children]), "disconnect")
  assert.equal(selected.token.pendingBodies.size, 0)
  assert.equal(selected.received.length, 3, "aborted handler must not run")
  const healthy = await fetch(`http://127.0.0.1:${selected.port}/healthy`, {
    headers: { connection: "close" },
  })
  assert.equal(healthy.status, 200, "server must survive rejected uploads")
  await healthy.arrayBuffer()

  const pending = await startedPartialUpload(selected)
  try {
    await selected.close()
    assert.equal(selected.received.length, 4, "shutdown must skip partial body")
  } finally {
    await destroySocket(pending)
  }
} finally {
  await selected.close()
}

const defaults = await fixture()
try {
  const response = await rawRequest(
    defaults.port,
    requestHeaders(`Content-Length: ${8 * 1024 * 1024 + 1}`)
  )
  assert.match(response, /^HTTP\/1\.1 413 /)
  assert.equal(defaults.received.length, 0)
} finally {
  await defaults.close()
}

await assertStreamingLifecycle("close")
await assertStreamingLifecycle("shutdown")

process.stdout.write("HTTP request body bounds probe passed\n")

async function assertStreamingLifecycle(operation: "close" | "shutdown") {
  const provider = createNodeHttpServerProvider({
    maxRequestBodyBytes: bodyLimit,
  })
  const loader = new ProviderPackageLoader("node-process", [
    {
      provider: "seseragi/runtime-node#http-server",
      service: "std/http/server::HttpServer",
      target: "node-process",
      module: "http-server-request-bounds",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider }),
    },
  ])
  await loader.load("seseragi/runtime-node#http-server")
  const healthy = streamBody(new Uint8Array([97, 98]), 3)
  // Cap this fixture at 8 MiB, with only one 1 KiB chunk retained by the
  // producer. A paused loopback peer fills its socket long before that cap.
  const blocked = streamBody(new Uint8Array(1024).fill(97), 8192)
  let blockedResponse: ServerResponse | undefined
  const started = (await provider.listen!({
    hostname: "127.0.0.1",
    port: 0,
    async handler(request: { url: string }) {
      return {
        status: 200,
        headers: [],
        body: request.url.endsWith("/healthy-stream")
          ? healthy.body
          : blocked.body,
      }
    },
  })) as ProviderResult
  assert(started.kind === "success")
  const token = started.value as Token
  const address = token.server.address()
  assert(address !== null && typeof address !== "string")
  token.server.on("request", (request, response) => {
    if (request.url === "/blocked-stream") blockedResponse = response
  })
  let client: Socket | undefined
  let stopping: Promise<unknown> | undefined
  try {
    const response = await fetch(
      `http://127.0.0.1:${address.port}/healthy-stream`,
      {
        headers: { connection: "close" },
      }
    )
    assert.equal(response.status, 200)
    assert.equal(await response.text(), "ababab")
    await deadline(
      Promise.allSettled([...token.children]),
      "healthy stream cleanup"
    )
    assert.equal(token.children.size, 0)
    assert.equal(healthy.completed, 1, "normal stream must complete")
    assert.equal(healthy.cancelled, 0, "normal finish is not a disconnect")

    client = connect(address.port, "127.0.0.1")
    client.on("error", () => undefined)
    client.pause()
    await deadline(
      new Promise<void>((resolve, reject) => {
        client!.once("connect", resolve)
        client!.once("error", reject)
      }),
      "paused stream peer connect"
    )
    client.write(
      "GET /blocked-stream HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    let previous = -1
    let stableSince = Date.now()
    const backpressureDeadline = Date.now() + 2_000
    await (async () => {
      while (true) {
        assert(
          Date.now() < backpressureDeadline,
          "response backpressure fixture timed out"
        )
        if (blocked.produced !== previous) {
          previous = blocked.produced
          stableSince = Date.now()
        }
        if (
          blockedResponse?.writableNeedDrain &&
          blocked.produced < 8192 &&
          Date.now() - stableSince >= 25
        )
          return
        await new Promise((resolve) => setTimeout(resolve, 5))
      }
    })()
    assert.equal(token.children.size, 1)
    assert.equal(token.pendingBodies.size, 0, "request body already completed")
    stopping = Promise.resolve(
      operation === "close" ? provider.close!(token) : loader.shutdown()
    )
    await deadline(stopping, `${operation} during response backpressure`)
    assert.equal(token.server.listening, false)
    assert.equal(token.children.size, 0)
    assert.equal(blocked.cancelled, 1, "shutdown must release the stream")
    assert.equal(blocked.completed, 0)
  } finally {
    if (client !== undefined) await destroySocket(client)
    await deadline(
      stopping ?? Promise.resolve(provider.close!(token)),
      "stream server cleanup"
    )
    await loader.shutdown()
  }
}

function streamBody(chunk: Uint8Array, maximumChunks: number) {
  let produced = 0
  let completed = 0
  let cancelled = 0
  let finalized = false
  const body: ProviderHttpServerStreamBody = {
    kind: "stream",
    async pull() {
      if (produced >= maximumChunks) return { done: true, value: undefined }
      produced += 1
      return { done: false, value: chunk }
    },
    async complete() {
      if (finalized) return
      finalized = true
      completed += 1
    },
    async cancel() {
      if (finalized) return
      finalized = true
      cancelled += 1
    },
  }
  return {
    body,
    get produced() {
      return produced
    },
    get completed() {
      return completed
    },
    get cancelled() {
      return cancelled
    },
  }
}
