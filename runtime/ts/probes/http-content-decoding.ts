import assert from "node:assert/strict"
import { createServer, type Socket } from "node:net"
import {
  brotliCompressSync,
  deflateRawSync,
  deflateSync,
  gzipSync,
  inflateRawSync,
} from "node:zlib"
import { createFetchHttpClientProvider } from "../../providers/http-client"
import { createDecodedHttpExchange } from "../../providers/http-content-decoding"
import { fromUint8Array } from "../src/bytes"
import { run } from "../src/effect"
import {
  bodyLimit,
  emptyBody,
  exchange,
  get,
  type HttpClientEvent,
  head,
  parseUrl,
  post,
  request,
  send,
  sendBytes,
  sendEmpty,
} from "../src/http-client"
import type { ProviderSubscriptionObserver } from "../src/provider"
import { createProviderHttpClient } from "../src/provider-http-client"
import { runCollect } from "../src/stream"

const target = process.argv[2]
assert(target === "node-process" || target === "bun-process")
const identity = `seseragi/runtime-${target === "node-process" ? "node" : "bun"}#http-client`
const entry = createFetchHttpClientProvider(identity, target)
const environment = {
  httpClient: createProviderHttpClient({
    provider: identity,
    service: "std/http::HttpClient",
    entry,
  }),
}
const plain = Buffer.from("abc")
const cases = [
  { name: "gzip", coding: "gzip", encoded: gzipSync(plain) },
  { name: "x-gzip", coding: "x-gzip", encoded: gzipSync(plain) },
  { name: "deflate", coding: "deflate", encoded: deflateSync(plain) },
  { name: "raw-deflate", coding: "deflate", encoded: deflateRawSync(plain) },
  { name: "br", coding: "br", encoded: brotliCompressSync(plain) },
  {
    name: "stacked",
    coding: "gzip, br",
    encoded: brotliCompressSync(gzipSync(plain)),
  },
  {
    name: "inner-deflate",
    coding: "deflate, gzip",
    encoded: gzipSync(deflateSync(plain)),
  },
  { name: "unknown", coding: "unknown, gzip", encoded: gzipSync(plain) },
]
const rawCmf8 = Buffer.from([
  0x08, 0x03, 0x00, 0xfc, 0xff, 0x61, 0x62, 0x63, 0x01, 0x00, 0x00, 0xff, 0xff,
])
const ambiguousRaw = Buffer.from([
  0x78, 0x01, 0x00, 0xfe, 0xff, 0x61, 0x01, 0x00, 0x00, 0xff, 0xff,
])
const dictionaryDeflate = deflateSync(plain, { dictionary: plain })
let eightLayers: Buffer = plain
for (let layer = 0; layer < 8; layer++) eightLayers = gzipSync(eightLayers)
const additionalCases = [
  {
    name: "raw-deflate-cmf8",
    coding: "deflate",
    encoded: rawCmf8,
    expected: plain,
  },
  ...["leading", "trailing", "repeated"].map((position) => ({
    name: `encoding-empty-${position}`,
    coding:
      position === "leading"
        ? ", gzip"
        : position === "trailing"
          ? "gzip,"
          : ", , gzip,, ,",
    encoded: gzipSync(plain),
    expected: plain,
  })),
  {
    name: "encoding-empty-chain",
    coding: ", gzip,, br, ",
    encoded: brotliCompressSync(gzipSync(plain)),
    expected: plain,
  },
  {
    name: "encoding-empty-unknown",
    coding: ", unknown,, gzip, ",
    encoded: gzipSync(plain),
    expected: gzipSync(plain),
  },
  {
    name: "encoding-empty-only",
    coding: ", ,",
    encoded: plain,
    expected: plain,
  },
  {
    name: "encoding-empty-eight-layers",
    coding: `, ${Array(8).fill("gzip").join(",, ")},`,
    encoded: eightLayers,
    expected: plain,
  },
]
const failureCases = [
  { name: "deflate-truncated-prefix", encoded: Buffer.from([0x78]) },
  { name: "deflate-dictionary", encoded: dictionaryDeflate },
  { name: "deflate-ambiguous-raw", encoded: ambiguousRaw },
]
const results: unknown[] = []
const unhandled: unknown[] = []
const onUnhandled = (cause: unknown) => unhandled.push(cause)
process.on("unhandledRejection", onUnhandled)

try {
  await withServer(async (port, state) => {
    for (const fixture of cases) {
      const url = `http://127.0.0.1:${port}/${fixture.name}`
      const legacy = await deadline(
        run(
          send({ method: "GET", url, headers: [], body: new Uint8Array() }),
          environment
        ),
        `legacy ${fixture.name}`
      )
      const expected = fixture.name === "unknown" ? fixture.encoded : plain
      assert.equal(legacy.kind, "success", JSON.stringify(legacy))
      if (legacy.kind !== "success") throw new Error("legacy send failed")
      const actual = await deadline(
        run(sendEmpty(limit(64), buildRequest(url)), environment),
        `bounded ${fixture.name}`
      )
      assert.equal(actual.kind, "success", JSON.stringify(actual))
      if (actual.kind !== "success") throw new Error("bounded send failed")
      assert.deepEqual(Buffer.from(actual.value.body), expected)
      if (fixture.name !== "unknown" && !fixture.coding.includes(","))
        assert.deepEqual(actual.value.body, legacy.value.body)
      results.push({
        case: fixture.name,
        legacyBytes: legacy.value.body.length,
        boundedBytes: actual.value.body.length,
        matchesLegacy: Buffer.from(actual.value.body).equals(
          Buffer.from(legacy.value.body)
        ),
      })
    }

    assert.deepEqual(inflateRawSync(rawCmf8), plain)
    assert.deepEqual(inflateRawSync(ambiguousRaw), Buffer.from("a"))
    for (const fixture of additionalCases) {
      const result = await deadline(
        run(
          sendEmpty(
            limit(64),
            buildRequest(`http://127.0.0.1:${port}/${fixture.name}`)
          ),
          environment
        ),
        fixture.name
      )
      assert.equal(result.kind, "success", JSON.stringify(result))
      if (result.kind !== "success") throw new Error(fixture.name)
      assert.deepEqual(Buffer.from(result.value.body), fixture.expected)
      results.push({
        case: fixture.name,
        wireBytes: fixture.encoded.length,
        responseBytes: result.value.body.length,
      })
    }

    const postResult = await deadline(
      run(
        sendBytes(
          limit(3),
          fromUint8Array(new Uint8Array([7, 8])),
          buildRequest(`http://127.0.0.1:${port}/post`, post)
        ),
        environment
      ),
      "compressed POST"
    )
    assert.equal(postResult.kind, "success", JSON.stringify(postResult))
    assert.deepEqual(state.postBody, Buffer.from([7, 8]))

    for (const [path, method] of [
      ["head", head],
      ["status-204", get],
      ["status-205", get],
      ["status-304", get],
      ["empty-200", get],
      ["empty-gzip", get],
    ] as const) {
      const result = await deadline(
        run(
          sendEmpty(
            limit(3),
            buildRequest(`http://127.0.0.1:${port}/${path}`, method)
          ),
          environment
        ),
        path
      )
      assert.equal(result.kind, "success", JSON.stringify(result))
      if (result.kind === "success") assert.equal(result.value.body.length, 0)
    }

    const events = await deadline(
      run(
        runCollect(
          exchange(
            emptyBody(),
            buildRequest(`http://127.0.0.1:${port}/trailers`)
          )
        ),
        environment
      ),
      "raw body before trailers"
    )
    assert.equal(events.kind, "success", JSON.stringify(events))
    if (events.kind !== "success") throw new Error("trailers failed")
    assert.deepEqual(
      events.value.map((event) => event.tag),
      [
        "InformationalResponse",
        "ResponseStarted",
        "ResponseBodyChunk",
        "ResponseTrailers",
      ]
    )
    assert.equal(events.value[2]?.tag, "ResponseBodyChunk")
    if (events.value[2]?.tag === "ResponseBodyChunk")
      assert.deepEqual(Buffer.from(events.value[2].value), gzipSync(plain))

    const large = await deadline(
      run(
        runCollect(
          exchange(
            emptyBody(),
            buildRequest(`http://127.0.0.1:${port}/large-trailers`)
          )
        ),
        environment
      ),
      "draining encoded response before trailers"
    )
    assert.equal(large.kind, "success", JSON.stringify(large))
    if (large.kind !== "success") throw new Error("large response failed")
    let rawBytes = 0
    for (const event of large.value) {
      if (event.tag !== "ResponseBodyChunk") continue
      assert(event.value.length <= 64 * 1024)
      rawBytes += event.value.length
    }
    assert.equal(rawBytes, gzipSync(Buffer.alloc(256 * 1024, 65)).length)
    assert.equal(large.value.at(-1)?.tag, "ResponseTrailers")

    const decodedLarge = await deadline(
      run(
        sendEmpty(
          limit(256 * 1024),
          buildRequest(`http://127.0.0.1:${port}/large-trailers`)
        ),
        environment
      ),
      "helper drains expanded response"
    )
    assert.equal(decodedLarge.kind, "success", JSON.stringify(decodedLarge))
    if (decodedLarge.kind !== "success")
      throw new Error("decoded helper failed")
    assert.deepEqual(
      Buffer.from(decodedLarge.value.body),
      Buffer.alloc(256 * 1024, 65)
    )

    for (const path of [
      "truncated",
      "bad-crc",
      "too-deep",
      ...failureCases.map(({ name }) => name),
    ]) {
      const result = await deadline(
        run(
          sendEmpty(
            limit(1024),
            buildRequest(`http://127.0.0.1:${port}/${path}`)
          ),
          environment
        ),
        path
      )
      assert.equal(result.kind, "failure", JSON.stringify(result))
      if (result.kind === "failure")
        assert.equal(result.error.tag, "HttpProtocolFailure")
    }

    const bomb = await deadline(
      run(
        sendEmpty(limit(1024), buildRequest(`http://127.0.0.1:${port}/bomb`)),
        environment
      ),
      "decoded byte limit before peer EOF"
    )
    assert.equal(bomb.kind, "failure", JSON.stringify(bomb))
    if (bomb.kind === "failure")
      assert.deepEqual(bomb.error, {
        tag: "HttpResponseBodyLimitExceeded",
        value: { limitBytes: 1024 },
      })
    await waitUntil(
      () => state.bombClosed,
      "body limit closes compressed connection"
    )
    assert.equal(state.bombFullySent, false)
    const recovered = await deadline(
      run(
        sendEmpty(limit(3), buildRequest(`http://127.0.0.1:${port}/gzip`)),
        environment
      ),
      "request after compressed cancellation"
    )
    assert.equal(recovered.kind, "success")
    results.push({
      case: "native lifecycle",
      postBody: [...state.postBody],
      emptyResponses: 6,
      rawExchangeBytes: rawBytes,
      fullyDrainedDecodedBytes: decodedLarge.value.body.length,
      publicExchangeTrailersAfterRawBody: true,
      protocolFailures: 3 + failureCases.length,
      bombWireBytes: state.bombWireBytes,
      bombDecodedBytes: 256 * 1024,
      cancelledBeforeEof: !state.bombFullySent,
      recovery: true,
    })
  })
  await assertDeflatePrefixHandling()
  await assertDecodedTrailerOrder()
  await assertBoundedDemandAndCleanup()
  await delay(10)
  assert.deepEqual(unhandled, [])
  process.stdout.write(
    `${JSON.stringify({ target, runtime: process.versions.bun === undefined ? `Node ${process.versions.node}` : `Bun ${process.versions.bun}`, executable: process.execPath, pid: process.pid, results })}\nHTTP content decoding probe passed\n`
  )
} finally {
  process.off("unhandledRejection", onUnhandled)
}

function limit(bytes: number) {
  const result = bodyLimit(bytes)
  assert.equal(result.tag, "Right")
  if (result.tag !== "Right") throw new Error("invalid fixture body limit")
  return result.value
}

function buildRequest(url: string, method = get) {
  const parsed = parseUrl(url)
  assert.equal(parsed.tag, "Right")
  if (parsed.tag !== "Right") throw new Error("invalid fixture URL")
  return request(method, parsed.value)
}

async function withServer(
  action: (
    port: number,
    state: {
      postBody: Buffer
      bombClosed: boolean
      bombFullySent: boolean
      bombWireBytes: number
    }
  ) => Promise<void>
) {
  const sockets = new Set<Socket>()
  const timers = new Set<ReturnType<typeof setTimeout>>()
  const state = {
    postBody: Buffer.alloc(0),
    bombClosed: false,
    bombFullySent: false,
    bombWireBytes: 0,
  }
  const server = createServer((socket) => {
    sockets.add(socket)
    socket.on("error", () => undefined)
    socket.once("close", () => sockets.delete(socket))
    socket.setTimeout(3000, () => socket.destroy())
    let input = Buffer.alloc(0)
    socket.on("data", function onData(chunk) {
      input = Buffer.concat([input, chunk])
      const end = input.indexOf("\r\n\r\n")
      if (end < 0) return
      const head = input.subarray(0, end).toString("latin1")
      const declared = /\r\ncontent-length: ([0-9]+)/i.exec(head)?.[1]
      const bodyLength = declared === undefined ? 0 : Number(declared)
      if (input.length < end + 4 + bodyLength) return
      socket.off("data", onData)
      const path = head.split(" ")[1]?.slice(1) ?? ""
      const emptyStatus = /^status-(204|205|304)$/.exec(path)
      if (path === "head" || path === "empty-200" || emptyStatus !== null) {
        const status = emptyStatus?.[1] ?? "200"
        socket.end(
          `HTTP/1.1 ${status} OK\r\nContent-Encoding: gzip\r\nContent-Length: 0\r\nConnection: close\r\n\r\n`
        )
        return
      }
      const fixture = [...cases, ...additionalCases].find(
        ({ name }) => name === path
      )
      let encoded = fixture?.encoded ?? gzipSync(plain)
      let coding = fixture?.coding ?? "gzip"
      const failureFixture = failureCases.find(({ name }) => name === path)
      if (failureFixture !== undefined) {
        encoded = failureFixture.encoded
        coding = "deflate"
      }
      if (path === "empty-gzip") encoded = gzipSync(Buffer.alloc(0))
      if (path === "large-trailers")
        encoded = gzipSync(Buffer.alloc(256 * 1024, 65))
      if (path === "post")
        state.postBody = input.subarray(end + 4, end + 4 + bodyLength)
      if (path === "truncated")
        encoded = encoded.subarray(0, encoded.length - 4)
      if (path === "bad-crc") {
        encoded = Buffer.from(encoded)
        encoded[encoded.length - 8]! ^= 1
      }
      if (path === "too-deep") coding = Array(9).fill("gzip").join(", ")
      if (path === "bomb") {
        encoded = gzipSync(Buffer.alloc(256 * 1024, 65))
        state.bombWireBytes = encoded.length
        assert(encoded.length < 1024)
        socket.once("close", () => {
          state.bombClosed = true
        })
        socket.write(
          "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
        )
        const prefix = encoded.subarray(0, encoded.length - 8)
        socket.write(
          Buffer.concat([
            Buffer.from(`${prefix.length.toString(16)}\r\n`),
            prefix,
            Buffer.from("\r\n"),
          ])
        )
        const timer = setTimeout(() => {
          timers.delete(timer)
          if (socket.destroyed) return
          state.bombFullySent = true
          const footer = encoded.subarray(encoded.length - 8)
          socket.end(
            Buffer.concat([
              Buffer.from("8\r\n"),
              footer,
              Buffer.from("\r\n0\r\n\r\n"),
            ])
          )
        }, 1500)
        timers.add(timer)
        return
      }
      if (path === "trailers" || path === "large-trailers") {
        socket.write(
          "HTTP/1.1 103 Early Hints\r\nLink: </finite>\r\n\r\nHTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
        )
        socket.end(
          Buffer.concat([
            Buffer.from(`${encoded.length.toString(16)}\r\n`),
            encoded,
            Buffer.from("\r\n0\r\nx-proof: finite\r\n\r\n"),
          ])
        )
        return
      }
      socket.write(
        `HTTP/1.1 200 OK\r\nContent-Encoding: ${coding}\r\nContent-Length: ${encoded.length}\r\nConnection: close\r\n\r\n`
      )
      socket.end(encoded)
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
    await action(address.port, state)
  } finally {
    for (const timer of timers) clearTimeout(timer)
    for (const socket of sockets) socket.destroy()
    await deadline(
      new Promise<void>((resolve) => server.close(() => resolve())),
      "fixture close"
    )
    await waitUntil(() => sockets.size === 0, "all fixture sockets closed")
    assert.equal(server.listening, false)
  }
}

async function assertDeflatePrefixHandling() {
  // Raw stored blocks can contain arbitrary padding before LEN. 0x88 has a
  // deflate method nibble, and 0x1c satisfies FCHECK, but CINFO=8 is not zlib.
  const invalidWindowRaw = Buffer.concat([
    Buffer.from([0x88, 0x1c, 0x00, 0xe3, 0xff]),
    Buffer.alloc(28, 97),
    Buffer.from([0x01, 0x00, 0x00, 0xff, 0xff]),
  ])
  assert.deepEqual(inflateRawSync(invalidWindowRaw), Buffer.alloc(28, 97))
  for (const fixture of [
    { name: "zlib", encoded: deflateSync(plain), expected: plain },
    { name: "raw-cmf8", encoded: rawCmf8, expected: plain },
    {
      name: "raw-invalid-window",
      encoded: invalidWindowRaw,
      expected: Buffer.alloc(28, 97),
    },
    { name: "short-zlib", encoded: Buffer.from([0x78]) },
    { name: "short-raw", encoded: Buffer.from([0x03]) },
    { name: "dictionary", encoded: dictionaryDeflate },
    // A valid raw stream can share a complete zlib header. Prefer zlib, as
    // Fetch does here; do not replay an unbounded body after decoding starts.
    { name: "ambiguous-raw", encoded: ambiguousRaw },
  ]) {
    const events: HttpClientEvent[] = [
      {
        kind: "ResponseStarted",
        head: {
          version: "Http1_1",
          status: 200,
          headers: [{ name: "content-encoding", value: "deflate" }],
        },
      },
      ...[...fixture.encoded].map((byte) => ({
        kind: "ResponseBodyChunk" as const,
        bytes: new Uint8Array([byte]),
      })),
    ]
    const chunks: Uint8Array[] = []
    let unsubscribes = 0
    let finish = (_cause?: unknown): void => undefined
    const completion = new Promise<unknown>((resolve) => {
      finish = resolve
    })
    const registration = createDecodedHttpExchange(
      (_value, observerValue) => ({
        demand() {
          const observer = observerValue as ProviderSubscriptionObserver
          const event = events.shift()
          if (event !== undefined) observer.next(event)
          if (events.length === 0) observer.complete()
        },
        unsubscribe() {
          unsubscribes += 1
        },
      }),
      { method: "GET" },
      {
        next(event: HttpClientEvent) {
          if (event.kind === "ResponseBodyChunk") chunks.push(event.bytes)
        },
        complete() {
          finish()
        },
        failure: finish,
        defect: finish,
      },
      undefined
    )
    try {
      registration.demand(100)
      const failure = await deadline(completion, `split ${fixture.name}`)
      if (fixture.expected !== undefined) {
        assert.equal(failure, undefined)
        assert.deepEqual(Buffer.concat(chunks), fixture.expected)
      } else {
        assert.equal((failure as { tag: string }).tag, "HttpProtocolFailure")
      }
    } finally {
      await registration.unsubscribe()
    }
    assert.equal(unsubscribes, 1)
    results.push({
      case: `one-byte fragments ${fixture.name}`,
      expected: fixture.expected === undefined ? "typed failure" : "success",
      rawUnsubscribes: unsubscribes,
    })
  }

  let rawDemands = 0
  let rawUnsubscribes = 0
  const pendingPrefix = createDecodedHttpExchange(
    (_value, observerValue) => ({
      demand() {
        const observer = observerValue as ProviderSubscriptionObserver
        rawDemands += 1
        if (rawDemands === 1)
          observer.next({
            kind: "ResponseStarted",
            head: {
              version: "Http1_1",
              status: 200,
              headers: [{ name: "content-encoding", value: "deflate" }],
            },
          })
        else if (rawDemands === 2)
          observer.next({
            kind: "ResponseBodyChunk",
            bytes: new Uint8Array([0x78]),
          })
      },
      unsubscribe() {
        rawUnsubscribes += 1
      },
    }),
    { method: "GET" },
    {
      next() {},
      complete() {
        assert.fail("incomplete prefix must not complete")
      },
      failure(cause: unknown) {
        throw cause
      },
      defect(cause: unknown) {
        throw cause
      },
    },
    undefined
  )
  try {
    pendingPrefix.demand(2)
    await waitUntil(() => rawDemands === 3, "awaiting second prefix byte")
    await deadline(
      Promise.all([pendingPrefix.unsubscribe(), pendingPrefix.unsubscribe()]),
      "cancel incomplete deflate prefix"
    )
    assert.equal(rawUnsubscribes, 1)
  } finally {
    await pendingPrefix.unsubscribe()
  }
  results.push({
    case: "cancel incomplete deflate prefix",
    rawDemands,
    rawUnsubscribes,
  })
}

async function assertDecodedTrailerOrder() {
  const content = Buffer.alloc(256 * 1024, 65)
  const events: HttpClientEvent[] = [
    {
      kind: "InformationalResponse",
      head: { version: "Http1_1", status: 103, headers: [] },
    },
    {
      kind: "ResponseStarted",
      head: {
        version: "Http1_1",
        status: 200,
        headers: [{ name: "content-encoding", value: "gzip" }],
      },
    },
    { kind: "ResponseBodyChunk", bytes: gzipSync(content) },
    {
      kind: "ResponseTrailers",
      headers: [{ name: "x-proof", value: "finite" }],
    },
  ]
  const observed: HttpClientEvent[] = []
  let finish = (): void => undefined
  let fail = (_cause: unknown): void => undefined
  const completion = new Promise<void>((resolve, reject) => {
    finish = resolve
    fail = reject
  })
  let unsubscribes = 0
  const registration = createDecodedHttpExchange(
    (_value, observerValue) => ({
      demand() {
        const observer = observerValue as ProviderSubscriptionObserver
        const event = events.shift()
        if (event !== undefined) observer.next(event)
        if (events.length === 0) observer.complete()
      },
      unsubscribe() {
        unsubscribes += 1
      },
    }),
    { method: "GET" },
    {
      next(value: unknown) {
        observed.push(value as HttpClientEvent)
      },
      complete: finish,
      failure: fail,
      defect: fail,
    },
    undefined
  )
  try {
    registration.demand(100)
    await deadline(
      completion,
      "decoder drains all output before trailers/completion"
    )
    assert.equal(observed[0]?.kind, "InformationalResponse")
    assert.equal(observed[1]?.kind, "ResponseStarted")
    assert.equal(observed.at(-1)?.kind, "ResponseTrailers")
    const chunks = observed.flatMap((event) =>
      event.kind === "ResponseBodyChunk" ? [event.bytes] : []
    )
    assert(chunks.every((bytes) => bytes.length <= 64 * 1024))
    assert.deepEqual(Buffer.concat(chunks), content)
  } finally {
    await registration.unsubscribe()
  }
  assert.equal(unsubscribes, 1)
  results.push({
    case: "direct decoder trailer ordering",
    decodedBytes: content.length,
    rawUnsubscribes: unsubscribes,
  })
}

async function assertBoundedDemandAndCleanup() {
  const encoded = gzipSync(Buffer.alloc(256 * 1024, 65))
  let rawDemands = 0
  let rawUnsubscribes = 0
  let releaseCancellation = (): void => undefined
  const cancellation = new Promise<void>((resolve) => {
    releaseCancellation = resolve
  })
  const events: HttpClientEvent[] = [
    {
      kind: "ResponseStarted",
      head: {
        version: "Http1_1",
        status: 200,
        headers: [{ name: "content-encoding", value: "gzip" }],
      },
    },
    { kind: "ResponseBodyChunk", bytes: encoded },
  ]
  const observed: HttpClientEvent[] = []
  let complete = false
  const registration = createDecodedHttpExchange(
    (_value, observerValue) => {
      const observer = observerValue as ProviderSubscriptionObserver
      return {
        demand(count) {
          rawDemands += count
          const event = events.shift()
          if (event !== undefined) observer.next(event)
          else observer.complete()
        },
        async unsubscribe() {
          rawUnsubscribes += 1
          await cancellation
        },
      }
    },
    { method: "GET" },
    {
      next(event: unknown) {
        observed.push(event as HttpClientEvent)
      },
      complete() {
        complete = true
      },
      failure(cause: unknown) {
        throw new Error(JSON.stringify(cause))
      },
      defect(cause: unknown) {
        throw cause
      },
    },
    undefined
  )
  const completions: Promise<void>[] = []
  try {
    assert.equal(rawDemands, 0)
    registration.demand(1)
    assert.equal(observed.length, 1)
    assert.equal(rawDemands, 1)
    registration.demand(1)
    await waitUntil(
      () => observed.length === 2,
      "decoded chunk without waiting for write completion"
    )
    const chunk = observed[1]
    assert.equal(chunk?.kind, "ResponseBodyChunk")
    if (chunk?.kind === "ResponseBodyChunk")
      assert(chunk.bytes.length <= 64 * 1024)
    const paused = rawDemands
    await delay(15)
    assert.equal(rawDemands, paused)
    assert.equal(complete, false)
    let finished = 0
    for (let call = 0; call < 2; call++) {
      completions.push(
        Promise.resolve(registration.unsubscribe()).then(() => {
          finished += 1
        })
      )
    }
    await delay(10)
    assert.equal(finished, 0)
    assert.equal(rawUnsubscribes, 1)
    releaseCancellation()
    await deadline(Promise.all(completions), "shared decoder cancellation")
    assert.equal(finished, 2)
    results.push({
      case: "bounded demand and cancellation",
      chunkBytes: chunk?.kind === "ResponseBodyChunk" ? chunk.bytes.length : 0,
      pausedRawDemands: paused,
      rawUnsubscribes,
      joinedUnsubscribes: finished,
    })
  } finally {
    releaseCancellation()
    await Promise.allSettled(completions)
    await registration.unsubscribe()
  }

  const cleanupFailure = new Error("fixture cleanup failed")
  let cancelled = 0
  let releaseFailure = (): void => undefined
  const failureGate = new Promise<void>((resolve) => {
    releaseFailure = resolve
  })
  const malformed: HttpClientEvent[] = [
    {
      kind: "ResponseStarted",
      head: {
        version: "Http1_1",
        status: 200,
        headers: [{ name: "content-encoding", value: "gzip" }],
      },
    },
    { kind: "ResponseBodyChunk", bytes: new Uint8Array([1, 2, 3]) },
  ]
  let decodingFailure: unknown
  const rejected = createDecodedHttpExchange(
    (_value, observerValue) => ({
      demand() {
        const observer = observerValue as ProviderSubscriptionObserver
        const event = malformed.shift()
        if (event !== undefined) observer.next(event)
        else observer.complete()
      },
      async unsubscribe() {
        cancelled += 1
        await failureGate
        throw cleanupFailure
      },
    }),
    { method: "GET" },
    {
      next() {},
      complete() {},
      failure(cause: unknown) {
        decodingFailure = cause
      },
      defect(cause: unknown) {
        throw cause
      },
    },
    undefined
  )
  let closing: Promise<PromiseSettledResult<void>[]> | undefined
  try {
    rejected.demand(2)
    await waitUntil(
      () => decodingFailure !== undefined,
      "decoder failure is typed before cleanup finishes"
    )
    assert.equal(
      (decodingFailure as { tag: string }).tag,
      "HttpProtocolFailure"
    )
    await waitUntil(
      () => cancelled === 1,
      "decoder failure cancels raw exchange"
    )
    closing = Promise.allSettled([
      Promise.resolve(rejected.unsubscribe()),
      Promise.resolve(rejected.unsubscribe()),
    ])
    releaseFailure()
    const closed = await deadline(closing, "decoder failure cleanup joins")
    assert(
      closed.every(
        (outcome) =>
          outcome.status === "rejected" && outcome.reason === cleanupFailure
      )
    )
    assert.equal(cancelled, 1)
  } finally {
    releaseFailure()
    await closing
    await Promise.allSettled([rejected.unsubscribe()])
  }
}

function delay(milliseconds: number) {
  return new Promise<void>((resolve) => setTimeout(resolve, milliseconds))
}

async function waitUntil(predicate: () => boolean, message: string) {
  const deadline = Date.now() + 1500
  while (!predicate()) {
    assert(Date.now() < deadline, message)
    await delay(2)
  }
}

async function deadline<Value>(
  promise: Promise<Value>,
  message: string
): Promise<Value> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(message)), 3000)
      }),
    ])
  } finally {
    if (timer !== undefined) clearTimeout(timer)
  }
}
