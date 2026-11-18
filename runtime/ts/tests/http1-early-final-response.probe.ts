import assert from "node:assert/strict"
import { createServer } from "node:net"
import { createBunHttp1Exchange } from "../../providers/http1-stream"

const chunk = Uint8Array.of(97, 98)
let pulls = 0
let cancelled = false
const events: Array<{ kind: string; status?: number; text?: string }> = []
let complete = false

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

const registration = createBunHttp1Exchange(
  {
    method: "POST",
    url: `http://127.0.0.1:${address.port}/`,
    headers: [],
  },
  {
    next(event: {
      kind: string
      head?: { status: number }
      bytes?: Uint8Array
    }) {
      events.push({
        kind: event.kind,
        ...(event.head === undefined ? {} : { status: event.head.status }),
        ...(event.bytes === undefined
          ? {}
          : { text: Buffer.from(event.bytes).toString("utf8") }),
      })
    },
    complete() {
      complete = true
    },
    failure(failure: unknown) {
      console.error(failure)
      process.exitCode = 1
    },
    defect(cause: unknown) {
      console.error(cause)
      process.exitCode = 1
    },
  },
  {
    async pull() {
      pulls += 1
      if (cancelled) return { done: true, value: undefined }
      return { done: false, value: chunk }
    },
    async cancel() {
      cancelled = true
    },
  }
)
registration.demand(4)

const deadline = Date.now() + 2_000
while (!complete && process.exitCode === undefined && Date.now() < deadline) {
  await new Promise<void>((resolve) => setTimeout(resolve, 10))
}
server.close()

const body = events
  .filter((event) => event.kind === "ResponseBodyChunk")
  .map((event) => event.text ?? "")
  .join("")
const started = events.find((event) => event.kind === "ResponseStarted")
if (
  !complete ||
  !cancelled ||
  pulls >= 1_000 ||
  started?.status !== 413 ||
  body !== "too large"
) {
  console.error(JSON.stringify({ complete, cancelled, pulls, events }))
  process.exit(1)
}

process.stdout.write(
  `${JSON.stringify({ complete, cancelled, pulls, status: started.status, body })}\nearly final response probe passed\n`
)
