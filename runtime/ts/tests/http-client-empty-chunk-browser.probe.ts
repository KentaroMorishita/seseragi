import assert from "node:assert/strict"
import { createBrowserHttpClientProvider } from "../src/browser/provider-http-client"
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

delete globalThis.setImmediate
assert.equal(typeof setImmediate, "undefined")

const empty = fromUint8Array(new Uint8Array())
const payload = fromUint8Array(Uint8Array.of(7))
let remaining = 128
const content = fromPull(() => ({
  async pull() {
    if (remaining < 0) return { done: true, value: undefined }
    if (remaining === 0) {
      remaining = -1
      return { done: false, value: payload }
    }
    remaining -= 1
    return { done: false, value: empty }
  },
  close() {
    remaining = -1
  },
}))

const received: number[] = []
const fetchHost = async (
  _input: string,
  init: RequestInit
): Promise<Response> => {
  if (!(init.body instanceof ReadableStream)) {
    throw new Error("browser upload did not stream")
  }
  const reader = init.body.getReader()
  while (true) {
    const next = await reader.read()
    if (next.done) break
    received.push(...next.value)
  }
  return new Response("ok", { status: 200 })
}

const providerName = "seseragi/runtime-browser#http-client"
const provider = createBrowserHttpClientProvider(fetchHost)
const loader = new ProviderPackageLoader("browser", [
  {
    provider: providerName,
    service: "std/http::HttpClient",
    target: "browser",
    module: "candidate/empty-chunk-browser",
    exportName: "provider",
    loadMode: "lazy",
    importModule: async () => ({ provider }),
  },
])
const environment = {
  httpClient: createProviderHttpClient(await loader.load(providerName)),
}
const parsed = parseUrl("https://example.test/empty-chunks")
assert.equal(parsed.tag, "Right")
if (parsed.tag !== "Right") throw new Error("invalid fixture URL")

const outcome = await run(
  runCollect(
    exchange(streamBody<unknown, never>(content), request(post, parsed.value))
  ),
  environment
)
await loader.shutdown()

if (outcome.kind !== "success") {
  console.error(JSON.stringify({ outcome, received }))
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
  status !== 200 ||
  body !== "ok" ||
  received.length !== 1 ||
  received[0] !== 7
) {
  console.error(JSON.stringify({ status, body, received }))
  process.exit(1)
}

process.stdout.write(
  `${JSON.stringify({ status, body, received })}\nbrowser empty chunk upload probe passed\n`
)
