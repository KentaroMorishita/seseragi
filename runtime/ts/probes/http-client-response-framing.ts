import assert from "node:assert/strict"
import { ClientRequest, createServer as createHttpServer } from "node:http"
import { createServer, type Socket } from "node:net"
import { gzipSync } from "node:zlib"
import { createFetchHttpClientProvider } from "../../providers/http-client"
import { fromUint8Array } from "../src/bytes"
import { run } from "../src/effect"
import {
  bodyLimit,
  get,
  head,
  parseUrl,
  post,
  request,
  sendBytes,
  sendEmpty,
} from "../src/http-client"
import type { ProviderSubscriptionRegistration } from "../src/provider"
import { createProviderHttpClient } from "../src/provider-http-client"

const target = process.argv[2]
assert(target === "node-process" || target === "bun-process")
const identity = `response-framing/${target}#http-client`
const provider = createFetchHttpClientProvider(identity, target)
const environment = {
  httpClient: createProviderHttpClient({
    provider: identity,
    service: "std/http::HttpClient",
    entry: provider,
  }),
}
type Fixture = {
  name: string
  headers?: string
  status?: string
  method?: "GET" | "HEAD"
  body?: Buffer
  prefix?: string
  failure?: boolean
  bunOnly?: boolean
  keepOpen?: boolean
  expectedHeaders?: ReadonlyArray<{ name: string; value: string }>
  expectedTrailers?: ReadonlyArray<{ name: string; value: string }>
  expectedBody?: Buffer
  failureAfterHead?: boolean
}
type Event = {
  kind: string
  bytes?: Uint8Array
  head?: { status: number; headers: ReadonlyArray<unknown> }
  headers?: ReadonlyArray<unknown>
}
const chunked = Buffer.from("1\r\na\r\n0\r\n\r\n")
const compressed = gzipSync(Buffer.from("abc"))
const unsafeLength = "9007199254740992"
const fixtures: readonly Fixture[] = [
  { name: "fixed-zero", headers: "Content-Length: 0", keepOpen: true },
  {
    name: "fixed-body",
    headers: "Content-Length: 3",
    body: Buffer.from("abc"),
  },
  {
    name: "chunked-body",
    headers: "Transfer-Encoding: chunked",
    body: chunked,
  },
  {
    name: "final-chunked-after-gzip",
    headers: "Transfer-Encoding: gzip, chunked",
    body: Buffer.concat([
      Buffer.from(`${compressed.length.toString(16)}\r\n`),
      compressed,
      Buffer.from("\r\n0\r\n\r\n"),
    ]),
  },
  {
    name: "non-final-chunked-preserves-close-delimited-body",
    headers: "Transfer-Encoding: chunked, gzip",
    body: gzipSync(chunked),
  },
  {
    name: "unknown-transfer-is-close-delimited",
    headers: "Transfer-Encoding: future-coding",
    body: Buffer.from("abc"),
  },
  ...[
    'Transfer-Encoding: future; note="a,chunked,b", chunked',
    'Transfer-Encoding: future; note="a\\\",chunked,b", chunked',
    'Transfer-Encoding: future; note="a,chunked,b"\r\nTransfer-Encoding: chunked',
  ].map((headers, index) => ({
    name: `quoted-transfer-parameter-${index}`,
    headers,
    body: Buffer.from("3\r\nabc\r\n0\r\n\r\n"),
    expectedBody: Buffer.from("abc"),
  })),
  {
    name: "obs-text-and-horizontal-tab",
    headers: "X-Bytes:\t\xa0value\xff\t\r\nContent-Length: 0",
    expectedHeaders: [
      { name: "x-bytes", value: "\xa0value\xff" },
      { name: "content-length", value: "0" },
    ],
  },
  {
    name: "unknown-valid-status",
    status: "999 Future",
    headers: "Content-Length: 0",
  },
  {
    name: "obs-text-and-horizontal-tab-in-trailers",
    headers: "Transfer-Encoding: chunked",
    body: Buffer.from("0\r\nX-Bytes:\t\xa0value\xff\t\r\n\r\n", "latin1"),
    expectedBody: Buffer.alloc(0),
    expectedTrailers: [{ name: "x-bytes", value: "\xa0value\xff" }],
  },
  {
    name: "invalid-trailer-control",
    headers: "Transfer-Encoding: chunked",
    body: Buffer.from("0\r\nX-Bytes: \x00\r\n\r\n"),
    failure: true,
    failureAfterHead: true,
  },
  {
    name: "informational-then-unknown-final",
    prefix: "HTTP/1.1 199 Future\r\nX-Hint: finite\r\n\r\n",
    status: "299 Future",
    headers: "Content-Length: 3",
    body: Buffer.from("abc"),
  },
  {
    name: "continue-and-early-hints-before-final",
    prefix:
      "HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 103 Early Hints\r\nX-Hint: finite\r\n\r\n",
    headers: "Content-Length: 0",
  },
  {
    name: "sixteen-informational-responses",
    prefix: "HTTP/1.1 103 Early Hints\r\n\r\n".repeat(16),
    headers: "Content-Length: 0",
  },
  {
    name: "transfer-with-length",
    headers: "Transfer-Encoding: chunked\r\nContent-Length: 1",
    body: chunked,
    failure: true,
  },
  {
    name: "transfer-with-negative-length",
    headers: "Transfer-Encoding: chunked\r\nContent-Length: -1",
    body: chunked,
    failure: true,
  },
  {
    name: "transfer-with-conflicting-lengths",
    headers:
      "Transfer-Encoding: chunked\r\nContent-Length: 1\r\nContent-Length: 2",
    body: chunked,
    failure: true,
  },
  {
    name: "unknown-transfer-with-length",
    headers: "Transfer-Encoding: future-coding\r\nContent-Length: 1",
    body: Buffer.from("a"),
    failure: true,
  },
  ...["-1", "+1", "1, 1", unsafeLength, "9".repeat(400)].map((value) => ({
    name: `invalid-length-${value.slice(0, 20)}`,
    headers: `Content-Length: ${value}`,
    failure: true,
  })),
  {
    name: "duplicate-length",
    headers: "Content-Length: 1\r\nContent-Length: 1",
    failure: true,
  },
  {
    name: "head-with-unsafe-length",
    method: "HEAD",
    headers: `Content-Length: ${unsafeLength}`,
    failure: true,
  },
  {
    name: "no-content-with-unsafe-length",
    status: "204 No Content",
    headers: `Content-Length: ${unsafeLength}`,
    failure: true,
  },
  {
    name: "informational-with-unsafe-length",
    prefix: `HTTP/1.1 103 Early Hints\r\nContent-Length: ${unsafeLength}\r\n\r\n`,
    headers: "Content-Length: 0",
    failure: true,
  },
  ...["X Invalid", "X\x00Invalid"].map((name) => ({
    name: `invalid-header-name-${JSON.stringify(name)}`,
    headers: `${name}: value\r\nContent-Length: 0`,
    failure: true,
  })),
  ...["\x00", "\x01", "\x7f", "\r", "\n"].map((value) => ({
    name: `invalid-header-control-${JSON.stringify(value)}`,
    headers: `X-Value: before${value}after\r\nContent-Length: 0`,
    failure: true,
  })),
  ...["000", "099"].map((status) => ({
    name: `invalid-status-${status}`,
    status: `${status} Invalid`,
    headers: "Content-Length: 0",
    failure: true,
  })),
  {
    name: "repeated-chunked",
    headers: "Transfer-Encoding: chunked, chunked",
    body: chunked,
    failure: true,
    bunOnly: true,
  },
]
const results: unknown[] = []
const unhandled: unknown[] = []
const onUnhandled = (cause: unknown) => unhandled.push(cause)
process.on("unhandledRejection", onUnhandled)
try {
  for (const fixture of fixtures) {
    if (fixture.bunOnly && target !== "bun-process") continue
    await withPeer(fixture, async (url, peerClosed) => {
      const events: Event[] = []
      let failure: { tag: string } | undefined
      let completed = false
      let cancelled = 0
      let finish = (): void => undefined
      let reject = (_cause: unknown): void => undefined
      const finished = new Promise<void>((resolve, fail) => {
        finish = resolve
        reject = fail
      })
      const registration = provider.exchange!(
        { method: fixture.method ?? "GET", url, headers: [] },
        {
          next(event: unknown) {
            events.push(event as Event)
          },
          failure(cause: unknown) {
            failure = cause as { tag: string }
            finish()
          },
          complete() {
            completed = true
            finish()
          },
          defect: reject,
        },
        {
          knownLength: 0,
          async pull() {
            return { done: true, value: undefined }
          },
          async cancel() {
            cancelled += 1
          },
        }
      ) as ProviderSubscriptionRegistration
      try {
        registration.demand(100)
        await deadline(finished, fixture.name)
        if (fixture.failure) {
          assert.equal(failure?.tag, "HttpProtocolFailure", fixture.name)
          assert.equal(completed, false, fixture.name)
          assert.deepEqual(
            events.map((event) => event.kind),
            fixture.failureAfterHead ? ["ResponseStarted"] : [],
            fixture.name
          )
          // A finite invalid head must terminate without waiting for peer EOF.
          await waitUntil(peerClosed, `${fixture.name} closes the connection`)
        } else {
          assert.equal(failure, undefined, fixture.name)
          assert.equal(completed, true, fixture.name)
          const expected =
            fixture.expectedBody ??
            (fixture.name === "chunked-body"
              ? Buffer.from("a")
              : fixture.name === "final-chunked-after-gzip"
                ? compressed
                : (fixture.body ?? Buffer.alloc(0)))
          assert.deepEqual(
            Buffer.concat(events.flatMap((event) => event.bytes ?? [])),
            expected,
            fixture.name
          )
          if (fixture.expectedHeaders !== undefined) {
            assert.deepEqual(events[0]?.head?.headers, fixture.expectedHeaders)
          }
          if (fixture.expectedTrailers !== undefined) {
            assert.equal(events.at(-1)?.kind, "ResponseTrailers")
            assert.deepEqual(events.at(-1)?.headers, fixture.expectedTrailers)
          }
          if (fixture.name === "sixteen-informational-responses") {
            assert.equal(events.length, 17)
            assert.equal(events.at(-1)?.kind, "ResponseStarted")
          }
        }
        results.push({
          case: fixture.name,
          completed,
          failure,
          events: events.map((event) => event.kind),
        })
      } finally {
        await deadline(
          Promise.resolve(registration.unsubscribe()),
          `${fixture.name} cleanup`
        )
      }
      assert.equal(cancelled, 1, fixture.name)
    })
  }

  for (const initialDemand of [0, 1]) {
    await assertInformationalOverflow(initialDemand)
  }

  // Public helpers must observe malformed peer data as HttpError rather than a
  // provider result codec defect, including statuses accepted by native Node.
  for (const fixture of fixtures.filter((fixture) =>
    [
      "invalid-status-000",
      "head-with-unsafe-length",
      "unknown-valid-status",
    ].includes(fixture.name)
  )) {
    await withPeer(fixture, async (url) => {
      const parsed = parseUrl(url)
      const limit = bodyLimit(64)
      assert(parsed.tag === "Right" && limit.tag === "Right")
      const outcome = await deadline(
        run(
          sendEmpty(
            limit.value,
            request(fixture.method === "HEAD" ? head : get, parsed.value)
          ),
          environment
        ),
        `public ${fixture.name}`
      )
      if (fixture.failure) {
        assert.equal(outcome.kind, "failure", JSON.stringify(outcome))
        if (outcome.kind === "failure")
          assert.equal(outcome.error.tag, "HttpProtocolFailure")
      } else {
        assert.equal(outcome.kind, "success", JSON.stringify(outcome))
      }
    })
  }
  if (target === "node-process") await assertNodeUploadListeners()
  await delay(10)
  assert.deepEqual(unhandled, [])
  process.stdout.write(
    `${JSON.stringify({ target, runtime: process.versions.bun ?? process.versions.node, pid: process.pid, results, publicCases: 3, unhandled: unhandled.length })}\nHTTP client response framing probe passed\n`
  )
} finally {
  process.off("unhandledRejection", onUnhandled)
}

async function assertNodeUploadListeners() {
  const warnings: Error[] = []
  const onWarning = (warning: Error) => warnings.push(warning)
  const originalWrite = ClientRequest.prototype.write
  const ownedWrite = Object.hasOwn(ClientRequest.prototype, "write")
  const observed = new Map<ClientRequest, number>()
  let backpressureWrites = 0
  ClientRequest.prototype.write = function (...args) {
    if (!observed.has(this)) observed.set(this, this.listenerCount("error"))
    const written = Reflect.apply(originalWrite, this, args) as boolean
    if (!written) backpressureWrites += 1
    return written
  }
  process.on("warning", onWarning)
  try {
    let received = 0
    const sockets = new Set<Socket>()
    const server = createHttpServer((incoming, response) => {
      incoming.on("data", (chunk: Buffer) => {
        received += chunk.length
      })
      incoming.once("end", () => response.end("ok"))
    })
    server.on("connection", (socket) => {
      sockets.add(socket)
      socket.on("error", () => undefined)
      socket.once("close", () => sockets.delete(socket))
    })
    try {
      await deadline(
        new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve)),
        "upload listen"
      )
      const address = server.address()
      assert(address !== null && typeof address === "object")
      const url = parseUrl(`http://127.0.0.1:${address.port}/upload`)
      const limit = bodyLimit(2)
      assert(url.tag === "Right" && limit.tag === "Right")
      const result = await deadline(
        run(
          sendBytes(
            limit.value,
            fromUint8Array(new Uint8Array(2 * 1024 * 1024)),
            request(post, url.value)
          ),
          environment
        ),
        "finite 2 MiB upload"
      )
      assert.equal(result.kind, "success", JSON.stringify(result))
      assert.equal(received, 2 * 1024 * 1024)
      assert(backpressureWrites > 10)
      await delay(10)
      assert.deepEqual(warnings, [])
      for (const [request, baselineErrors] of observed) {
        assert.equal(request.listenerCount("drain"), 0)
        assert(request.listenerCount("error") <= baselineErrors)
      }
      results.push({
        case: "finite-2MiB-upload",
        received,
        backpressureWrites,
        warnings: warnings.length,
      })
    } finally {
      server.closeAllConnections()
      for (const socket of sockets) socket.destroy()
      await deadline(
        new Promise<void>((resolve) => server.close(() => resolve())),
        "upload server close"
      )
      await waitUntil(() => sockets.size === 0, "upload sockets close")
    }

    observed.clear()
    let pulls = 0
    let cancellations = 0
    const stalledSockets = new Set<Socket>()
    const stalled = createServer((socket) => {
      stalledSockets.add(socket)
      socket.on("error", () => undefined)
      socket.once("close", () => stalledSockets.delete(socket))
      socket.pause()
    })
    let registration: ProviderSubscriptionRegistration | undefined
    try {
      await deadline(
        new Promise<void>((resolve) => stalled.listen(0, "127.0.0.1", resolve)),
        "stalled upload listen"
      )
      const address = stalled.address()
      assert(address !== null && typeof address === "object")
      const bytes = new Uint8Array(64 * 1024)
      registration = provider.exchange!(
        {
          method: "POST",
          url: `http://127.0.0.1:${address.port}/stalled`,
          headers: [],
        },
        {
          next() {},
          complete() {},
          failure() {},
          defect(cause: unknown) {
            throw cause
          },
        },
        {
          knownLength: 128 * bytes.length,
          async pull() {
            if (pulls === 128) return { done: true, value: undefined }
            pulls += 1
            return { done: false, value: bytes }
          },
          async cancel() {
            cancellations += 1
          },
        }
      ) as ProviderSubscriptionRegistration
      await waitUntil(
        () => observed.size === 1 && stalledSockets.size === 1,
        "stalled upload connected"
      )
      const request = [...observed.keys()][0]!
      let paused = false
      for (let attempt = 0; attempt < 40; attempt++) {
        const before = pulls
        await delay(10)
        if (pulls === before && request.writableNeedDrain && pulls < 128) {
          paused = true
          break
        }
      }
      assert(paused, "finite upload must encounter actual native backpressure")
      assert(request.listenerCount("drain") > 0)
      await deadline(
        Promise.resolve(registration.unsubscribe()),
        "cancel stalled upload"
      )
      await waitUntil(
        () => request.listenerCount("drain") === 0,
        "close releases drain listener"
      )
      assert.equal(request.listenerCount("close"), 0)
      assert(request.listenerCount("error") <= observed.get(request)!)
      assert.equal(cancellations, 1)
      assert.deepEqual(warnings, [])
      results.push({
        case: "stalled-upload-cancellation",
        pulls,
        cancellations,
        drainListeners: request.listenerCount("drain"),
        closeListeners: request.listenerCount("close"),
        warnings: warnings.length,
      })
    } finally {
      await deadline(
        Promise.resolve(registration?.unsubscribe()),
        "stalled upload cleanup"
      )
      for (const socket of stalledSockets) socket.destroy()
      await deadline(
        new Promise<void>((resolve) => stalled.close(() => resolve())),
        "stalled upload server close"
      )
      await waitUntil(
        () => stalledSockets.size === 0,
        "stalled upload sockets close"
      )
    }
  } finally {
    if (ownedWrite) ClientRequest.prototype.write = originalWrite
    else Reflect.deleteProperty(ClientRequest.prototype, "write")
    process.off("warning", onWarning)
  }
}

async function assertInformationalOverflow(initialDemand: number) {
  await withPeer(
    {
      name: "informational-overflow",
      prefix: "HTTP/1.1 103 Early Hints\r\n\r\n".repeat(17),
      headers: "Content-Length: 0",
      keepOpen: true,
    },
    async (url, peerClosed) => {
      let failure: { tag: string; value: string } | undefined
      const events: unknown[] = []
      let cancelled = 0
      const registration = provider.exchange!(
        { method: "GET", url, headers: [] },
        {
          next(event: unknown) {
            events.push(event)
          },
          failure(cause: unknown) {
            failure = cause as typeof failure
          },
          complete() {
            throw new Error("overflow completed")
          },
          defect(cause: unknown) {
            throw cause
          },
        },
        {
          knownLength: 0,
          async pull() {
            return { done: true, value: undefined }
          },
          async cancel() {
            cancelled += 1
          },
        }
      ) as ProviderSubscriptionRegistration
      try {
        if (initialDemand > 0) registration.demand(initialDemand)
        // Native Node parses informational heads even without downstream demand.
        // Bun pauses its socket, and must fail after bounded demand resumes it.
        if (target === "bun-process") registration.demand(100)
        await waitUntil(() => failure !== undefined, "informational cap")
        assert.equal(failure?.tag, "HttpProtocolFailure")
        assert(failure?.value.includes("informational response limit"))
        if (target === "node-process")
          assert.equal(events.length, initialDemand)
        const before = events.length
        registration.demand(1000)
        await delay(10)
        assert.equal(events.length, before, "failure releases queued heads")
        await waitUntil(peerClosed, "informational cap closes the peer")
        results.push({
          case: "informational-overflow",
          initialDemand,
          events: before,
          failure,
        })
      } finally {
        await deadline(
          Promise.resolve(registration.unsubscribe()),
          "informational cap cleanup"
        )
      }
      assert.equal(cancelled, 1)
    }
  )
}

async function withPeer(
  fixture: Fixture,
  action: (url: string, peerClosed: () => boolean) => Promise<void>
) {
  const sockets = new Set<Socket>()
  let closed = false
  const server = createServer((socket) => {
    sockets.add(socket)
    socket.on("error", () => undefined)
    socket.once("close", () => {
      sockets.delete(socket)
      closed = true
    })
    socket.once("data", () => {
      const wire = Buffer.concat([
        Buffer.from(
          `${fixture.prefix ?? ""}HTTP/1.1 ${fixture.status ?? "200 OK"}\r\n${fixture.headers ?? ""}\r\n\r\n`,
          "latin1"
        ),
        fixture.body ?? Buffer.alloc(0),
      ])
      if (fixture.failure || fixture.keepOpen) socket.write(wire)
      else socket.end(wire)
    })
  })
  try {
    await deadline(
      new Promise<void>((resolve, reject) => {
        server.once("error", reject)
        server.listen(0, "127.0.0.1", resolve)
      }),
      "fixture listen"
    )
    const address = server.address()
    assert(address !== null && typeof address === "object")
    await action(`http://127.0.0.1:${address.port}/`, () => closed)
  } finally {
    for (const socket of sockets) socket.destroy()
    await deadline(
      new Promise<void>((resolve) => server.close(() => resolve())),
      "fixture close"
    )
    await waitUntil(() => sockets.size === 0, "all fixture sockets closed")
    assert.equal(server.listening, false)
  }
}

function delay(milliseconds: number) {
  return new Promise<void>((resolve) => setTimeout(resolve, milliseconds))
}

async function waitUntil(predicate: () => boolean, message: string) {
  const until = Date.now() + 1500
  while (!predicate()) {
    assert(Date.now() < until, message)
    await delay(2)
  }
}

async function deadline<Value>(promise: Promise<Value>, message: string) {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(message)), 2500)
      }),
    ])
  } finally {
    clearTimeout(timer)
  }
}
