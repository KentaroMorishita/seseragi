import { expect, test } from "bun:test"
import { createServer, type Socket } from "node:net"
import { createBunHttp1Exchange } from "../../providers/http1-stream"

type ExchangeResult = {
  events: Array<{ kind: string; bytes?: Uint8Array }>
  failure?: { tag: string; value: string }
  complete: boolean
  cancelled: number
}

async function receive(parts: readonly string[]): Promise<ExchangeResult> {
  const sockets = new Set<Socket>()
  const server = createServer((socket) => {
    sockets.add(socket)
    socket.on("error", () => undefined)
    socket.once("close", () => sockets.delete(socket))
    socket.once("data", () => {
      void (async () => {
        for (const part of parts) {
          if (socket.destroyed) return
          socket.write(part, "latin1")
          await new Promise<void>((resolve) => setImmediate(resolve))
        }
        // Keep the peer open: an overflow must fail before EOF or a timeout.
      })().catch(() => socket.destroy())
    })
  })
  let registration: ReturnType<typeof createBunHttp1Exchange> | undefined
  let timer: ReturnType<typeof setTimeout> | undefined
  const result: ExchangeResult = { events: [], complete: false, cancelled: 0 }
  try {
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject)
      server.listen(0, "127.0.0.1", resolve)
    })
    const address = server.address()
    if (address === null || typeof address === "string") {
      throw new Error("fixture did not listen")
    }
    await new Promise<void>((resolve, reject) => {
      timer = setTimeout(
        () => reject(new Error("HTTP fixture timed out")),
        2000
      )
      registration = createBunHttp1Exchange(
        {
          method: "GET",
          url: `http://127.0.0.1:${address.port}/`,
          headers: [],
        },
        {
          next(event: ExchangeResult["events"][number]) {
            result.events.push(event)
          },
          failure(failure: ExchangeResult["failure"]) {
            result.failure = failure
            resolve()
          },
          complete() {
            result.complete = true
            resolve()
          },
          defect: reject,
        },
        {
          knownLength: 0,
          async pull() {
            return { done: true }
          },
          async cancel() {
            result.cancelled += 1
          },
        }
      )
      registration.demand(100)
    })
    return result
  } finally {
    clearTimeout(timer)
    await registration?.unsubscribe()
    for (const socket of sockets) socket.destroy()
    await new Promise<void>((resolve) => server.close(() => resolve()))
  }
}

function padded(prefix: string, suffix: string, total: number): string {
  return prefix + "a".repeat(total - prefix.length - suffix.length) + suffix
}

const fixedHead = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n"
const chunkedHead = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n"

test("HTTP/1 accepts an exact-limit head split across the delimiter", async () => {
  const head = padded(
    "HTTP/1.1 200 OK\r\nX-Fill: ",
    "\r\nContent-Length: 2\r\n\r\n",
    16 * 1024
  )
  const result = await receive([head.slice(0, -3), `${head.slice(-3)}ok`])
  expect(result.failure).toBeUndefined()
  expect(result.complete).toBe(true)
  expect(
    new TextDecoder().decode(
      Buffer.concat(result.events.flatMap((event) => event.bytes ?? []))
    )
  ).toBe("ok")
  expect(result.cancelled).toBe(1)
})

test.each([
  [
    "unfinished headers",
    ["HTTP/1.1 200 OK\r\nX-Fill: ", "a".repeat(16 * 1024)],
    "headers",
  ],
  [
    "complete oversized headers",
    [padded("HTTP/1.1 200 OK\r\nX-Fill: ", "\r\n\r\n", 16 * 1024 + 1)],
    "headers",
  ],
  [
    "unfinished chunk size",
    [chunkedHead, `1;x=${"a".repeat(1024)}`],
    "chunk size line",
  ],
  [
    "complete oversized chunk size",
    [chunkedHead, padded("1;x=", "\r\n", 1025)],
    "chunk size line",
  ],
  [
    "unfinished trailers",
    [chunkedHead, "0\r\nX-Fill: ", "a".repeat(16 * 1024)],
    "trailers",
  ],
  [
    "complete oversized trailers",
    [chunkedHead, `0\r\n${padded("X-Fill: ", "\r\n\r\n", 16 * 1024 + 1)}`],
    "trailers",
  ],
] as const)(
  "HTTP/1 rejects %s before the peer finishes",
  async (_name, parts, section) => {
    const result = await receive(parts)
    expect(result.complete).toBe(false)
    expect(result.failure?.tag).toBe("HttpProtocolFailure")
    expect(result.failure?.value).toContain(section)
    expect(result.cancelled).toBe(1)
  }
)

test("HTTP/1 accepts exact-limit chunk metadata and trailers", async () => {
  const sizeLine = padded("1;x=", "\r\n", 1024)
  const trailers = padded("X-Fill: ", "\r\n\r\n", 16 * 1024)
  const result = await receive([
    chunkedHead,
    `${sizeLine}a\r\n0\r\n`,
    trailers.slice(0, -1),
    trailers.slice(-1),
  ])
  expect(result.failure).toBeUndefined()
  expect(result.complete).toBe(true)
  expect(result.events.map((event) => event.kind)).toEqual([
    "ResponseStarted",
    "ResponseBodyChunk",
    "ResponseTrailers",
  ])
})

test("HTTP/1 accepts empty trailers split across reads", async () => {
  const result = await receive([chunkedHead, "1\r\na\r\n0\r\n", "\r", "\n"])
  expect(result.failure).toBeUndefined()
  expect(result.complete).toBe(true)
})

test("HTTP/1 bounds informational responses and accepts the boundary", async () => {
  const informational = "HTTP/1.1 103 Early Hints\r\n\r\n"
  const accepted = await receive([informational.repeat(16), `${fixedHead}ok`])
  expect(accepted.failure).toBeUndefined()
  expect(accepted.complete).toBe(true)
  expect(
    accepted.events.filter((event) => event.kind === "InformationalResponse")
  ).toHaveLength(16)
  const rejected = await receive([informational.repeat(17)])
  expect(rejected.failure?.tag).toBe("HttpProtocolFailure")
  expect(rejected.failure?.value).toContain("informational response limit")
  expect(rejected.cancelled).toBe(1)
})

test.each([
  [`${fixedHead}okextra`],
  [`${chunkedHead}0\r\n\r\nextra`],
  ["HTTP/1.1 204 No Content\r\n\r\nextra"],
])("HTTP/1 rejects bytes left beyond the response body", async (wire) => {
  const result = await receive([wire])
  expect(result.failure?.tag).toBe("HttpProtocolFailure")
  expect(result.failure?.value).toContain("after its body")
  expect(result.complete).toBe(false)
})
