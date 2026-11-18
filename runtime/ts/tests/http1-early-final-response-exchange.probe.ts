import assert from "node:assert/strict"
import { createServer } from "node:net"
import { createFetchHttpClientProvider } from "../../providers/http-client"
import { fromUint8Array } from "../src/bytes"
import { run } from "../src/effect"
import {
  exchange,
  parseUrl,
  post,
  request,
  streamBody,
} from "../src/http-client"
import { createProviderHttpClient } from "../src/provider-http-client"
import { ProviderPackageLoader } from "../src/provider-package"
import { fromPull, runCollect } from "../src/stream"

const target = process.argv[2]
assert(target === "node-process" || target === "bun-process")

const server = createServer((socket) => {
  socket.on("error", () => undefined)
  let pending = Buffer.alloc(0)
  const readHead = (incoming: Buffer) => {
    pending = Buffer.concat([pending, incoming])
    if (!pending.includes(Buffer.from("\r\n\r\n"))) return
    socket.off("data", readHead)
    socket.on("data", () => undefined)
    socket.write(
      "HTTP/1.1 413 Payload Too Large\r\nContent-Length: 9\r\nConnection: close\r\n\r\ntoo large"
    )
  }
  socket.on("data", readHead)
})

await new Promise<void>((resolve, reject) => {
  server.once("error", reject)
  server.listen(0, "127.0.0.1", resolve)
})
const address = server.address()
assert(address !== null && typeof address !== "string")

const providerName = `compatibility/${target}#http-client`
const provider = createFetchHttpClientProvider(providerName, target)
const loader = new ProviderPackageLoader(target, [
  {
    provider: providerName,
    service: "std/http::HttpClient",
    target,
    module: "candidate/early-final-exchange",
    exportName: "provider",
    loadMode: "lazy",
    importModule: async () => ({ provider }),
  },
])
const environment = {
  httpClient: createProviderHttpClient(await loader.load(providerName)),
}

const empty = fromUint8Array(new Uint8Array())
let closed = false
let pullsAfterCancel = 0
const content = fromPull(() => ({
  async pull() {
    if (closed) {
      pullsAfterCancel += 1
      return { done: true, value: undefined }
    }
    return { done: false, value: empty }
  },
  close() {
    closed = true
  },
}))

const parsed = parseUrl(`http://127.0.0.1:${address.port}/`)
assert.equal(parsed.tag, "Right")
if (parsed.tag !== "Right") throw new Error("invalid fixture URL")

const outcome = await run(
  runCollect(
    exchange(streamBody<unknown, never>(content), request(post, parsed.value))
  ),
  environment
)
server.close()

if (outcome.kind !== "success") {
  console.error(JSON.stringify({ outcome, closed, pullsAfterCancel }))
  process.exit(1)
}

let status = 0
const chunks: Buffer[] = []
for (const event of outcome.value) {
  if (event.tag === "ResponseStarted") status = event.value.status
  if (event.tag === "ResponseBodyChunk") chunks.push(Buffer.from(event.value))
}
const body = Buffer.concat(chunks).toString("utf8")
if (
  !closed ||
  pullsAfterCancel !== 0 ||
  status !== 413 ||
  body !== "too large"
) {
  console.error(
    JSON.stringify({ closed, pullsAfterCancel, status, body, outcome })
  )
  process.exit(1)
}

process.stdout.write(
  `${JSON.stringify({ closed, pullsAfterCancel, status, body })}\npublic exchange early final response probe passed\n`
)
