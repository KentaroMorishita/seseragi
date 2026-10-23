import assert from "node:assert/strict"
import { createServer } from "node:http"
import type { Socket } from "node:net"
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
import { ProviderPackageLoader } from "../src/provider-package"

const target = process.argv[2]
assert(target === "node-process" || target === "bun-process")
const identity = `framing/runtime-${target}#http-client`
const provider = createFetchHttpClientProvider(identity, target)
const loader = new ProviderPackageLoader(target, [
  {
    provider: identity,
    service: "std/http::HttpClient",
    target,
    module: "fixture/http-client-framing",
    exportName: "provider",
    loadMode: "lazy",
    importModule: async () => ({ provider }),
  },
])
type Observation = {
  method: string | undefined
  rawHeaders: string[]
  body: number[]
}
const observations = new Map<string, Observation>()
const results: unknown[] = []
const sockets = new Set<Socket>()
const server = createServer((incoming, response) => {
  const chunks: Buffer[] = []
  incoming.on("data", (chunk: Buffer) => chunks.push(chunk))
  incoming.on("end", () => {
    observations.set(incoming.url!, {
      method: incoming.method,
      rawHeaders: incoming.rawHeaders,
      body: [...Buffer.concat(chunks)],
    })
    response.writeHead(200, { "content-length": "0" })
    response.end()
  })
})
server.on("connection", (socket) => {
  sockets.add(socket)
  socket.once("close", () => sockets.delete(socket))
})

try {
  await deadline(
    new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve)),
    "listen"
  )
  const address = server.address()
  assert(typeof address === "object" && address !== null)
  const base = `http://127.0.0.1:${address.port}`
  const environment = {
    httpClient: createProviderHttpClient(await loader.load(identity)),
  }
  const limit = bodyLimit(1)
  assert.equal(limit.tag, "Right")
  if (limit.tag !== "Right") throw new Error("invalid response limit")

  for (const [method, bytes] of [
    [post, [7, 8]],
    [get, []],
    [head, []],
  ] as const) {
    const path = `/convenience/${method}`
    const url = parseUrl(`${base}${path}`)
    if (url.tag !== "Right") throw new Error("invalid fixture URL")
    const selected = request(method, url.value)
    const effect: ReturnType<typeof sendEmpty> =
      method === post
        ? sendBytes(
            limit.value,
            fromUint8Array(new Uint8Array(bytes)),
            selected
          )
        : sendEmpty(limit.value, selected)
    const result = await deadline(run(effect, environment), path)
    assert.equal(result.kind, "success", JSON.stringify(result))
    const observed = observations.get(path)
    assert(observed !== undefined)
    assert.deepEqual(observed.body, [...bytes])
    assert.deepEqual(headerValues(observed, "content-length"), [
      String(bytes.length),
    ])
    assert.deepEqual(headerValues(observed, "transfer-encoding"), [])
    results.push({ case: path, ...observed })
  }

  const cases: FramingCase[] = [
    { name: "known-two-bytes", knownLength: 2, length: "2" },
    { name: "unknown-stream", transferEncoding: "chunked" },
    { name: "known-empty-post", knownLength: 0, bytes: [], length: "0" },
    {
      name: "explicit-content-length",
      knownLength: 99,
      headers: [{ name: "content-length", value: "02" }],
      length: "02",
    },
  ]
  if (target === "node-process") {
    cases.push(
      {
        name: "mixed-case-content-length",
        knownLength: 99,
        headers: [{ name: "cOnTeNt-LeNgTh", value: "02" }],
        length: "02",
      },
      {
        name: "explicit-transfer-encoding",
        knownLength: 2,
        headers: [{ name: "TrAnSfEr-EnCoDiNg", value: "chunked" }],
        transferEncoding: "chunked",
      },
      ...[
        -1,
        0.5,
        Number.NaN,
        Number.POSITIVE_INFINITY,
        Number.MAX_SAFE_INTEGER + 1,
        "2",
      ].map(
        (knownLength): FramingCase => ({
          name: `invalid-known-length-${String(knownLength)}-${typeof knownLength}`,
          knownLength: knownLength as number,
          transferEncoding: "chunked",
        })
      )
    )
  }
  for (const fixture of cases) {
    const path = `/exchange/${fixture.name}`
    let index = 0
    let cancellations = 0
    const bytes = fixture.bytes ?? [7, 8]
    let complete = (): void => undefined
    let fail = (_cause: unknown): void => undefined
    const finished = new Promise<void>((resolve, reject) => {
      complete = resolve
      fail = reject
    })
    const registration = provider.exchange!(
      {
        method: "POST",
        url: `${base}${path}`,
        headers: [
          { name: "X-Repeat", value: "first" },
          { name: "x-repeat", value: "second" },
          ...(fixture.headers ?? []),
        ],
      },
      { next() {}, complete, failure: fail, defect: fail },
      {
        ...(fixture.knownLength === undefined
          ? {}
          : { knownLength: fixture.knownLength }),
        async pull() {
          return index < bytes.length
            ? { done: false, value: new Uint8Array([bytes[index++]!]) }
            : { done: true, value: undefined }
        },
        async cancel() {
          cancellations += 1
        },
      }
    ) as ProviderSubscriptionRegistration
    try {
      registration.demand(4)
      await deadline(finished, path)
      const observed = observations.get(path)
      assert(observed !== undefined, path)
      assert.equal(observed.method, "POST")
      assert.deepEqual(observed.body, [...bytes])
      assert.deepEqual(
        headerValues(observed, "content-length"),
        fixture.length === undefined ? [] : [fixture.length],
        path
      )
      assert.deepEqual(
        headerValues(observed, "transfer-encoding"),
        fixture.transferEncoding === undefined
          ? []
          : [fixture.transferEncoding],
        path
      )
      assert.deepEqual(headerValues(observed, "x-repeat"), ["first", "second"])
      results.push({ case: path, ...observed })
    } finally {
      await deadline(
        Promise.resolve(registration.unsubscribe()),
        `${path} cleanup`
      )
      assert.equal(cancellations, 1)
    }
  }
} catch (cause) {
  process.stderr.write(
    `${JSON.stringify({ target, pid: process.pid, observations: [...observations] })}\n`
  )
  throw cause
} finally {
  try {
    await deadline(loader.shutdown(), "provider shutdown")
  } finally {
    const socketClosures = [...sockets].map(
      (socket) => new Promise<void>((resolve) => socket.once("close", resolve))
    )
    for (const socket of sockets) socket.destroy()
    if (server.listening) {
      await deadline(
        new Promise<void>((resolve, reject) =>
          server.close((error) =>
            error === undefined ? resolve() : reject(error)
          )
        ),
        "server close"
      )
    }
    await deadline(Promise.all(socketClosures), "socket close")
    assert.equal(sockets.size, 0)
  }
}
process.stdout.write(
  `${JSON.stringify({ target, runtime: process.versions.bun === undefined ? `Node ${process.versions.node}` : `Bun ${process.versions.bun}`, pid: process.pid, results, remainingSockets: sockets.size })}\nHTTP client request framing probe passed\n`
)

type FramingCase = {
  name: string
  knownLength?: number
  headers?: Array<{ name: string; value: string }>
  bytes?: number[]
  length?: string
  transferEncoding?: string
}

function headerValues(observation: Observation, name: string): string[] {
  const values: string[] = []
  for (let index = 0; index < observation.rawHeaders.length; index += 2) {
    if (observation.rawHeaders[index]!.toLowerCase() === name) {
      values.push(observation.rawHeaders[index + 1]!)
    }
  }
  return values
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
        timer = setTimeout(() => reject(new Error(`${label} timed out`)), 2000)
      }),
    ])
  } finally {
    if (timer !== undefined) clearTimeout(timer)
  }
}
