import { expect, test } from "bun:test"
import { createServer, type Socket } from "node:net"
import { createBunHttp1Exchange } from "../../providers/http1-stream"

type Event = { kind: string; bytes?: Uint8Array }
type Observation = {
  events: Event[]
  failure?: { tag: string; value: string }
  complete: boolean
}

const fixedHead = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n"
const chunkedHead = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n"

async function waitUntil(predicate: () => boolean): Promise<void> {
  const deadline = Date.now() + 2_000
  while (!predicate()) {
    if (Date.now() > deadline) throw new Error("HTTP fixture timed out")
    await Bun.sleep(1)
  }
}

async function receive(wire: string, endPeer = true) {
  const sockets = new Set<Socket>()
  let peerFinished = false
  const server = createServer((socket) => {
    sockets.add(socket)
    socket.on("error", () => undefined)
    socket.once("close", () => sockets.delete(socket))
    socket.once("data", () => {
      if (endPeer)
        socket.end(wire, () => {
          peerFinished = true
        })
      else socket.write(wire)
    })
  })
  const result: Observation = { events: [], complete: false }
  let afterHead: Observation | undefined
  let cancelled = 0
  let registration: ReturnType<typeof createBunHttp1Exchange> | undefined
  try {
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject)
      server.listen(0, "127.0.0.1", resolve)
    })
    const address = server.address()
    if (address === null || typeof address === "string") {
      throw new Error("HTTP fixture did not listen")
    }
    registration = createBunHttp1Exchange(
      { method: "GET", url: `http://127.0.0.1:${address.port}/`, headers: [] },
      {
        next(event: Event) {
          result.events.push(event)
        },
        failure(failure: NonNullable<Observation["failure"]>) {
          result.failure = failure
        },
        complete() {
          result.complete = true
        },
        defect(cause: unknown) {
          throw cause
        },
      },
      {
        knownLength: 0,
        async pull() {
          return { done: true }
        },
        async cancel() {
          cancelled += 1
        },
      }
    )
    registration.demand(1)
    await waitUntil(
      () => result.events.length > 0 || result.failure !== undefined
    )
    if (endPeer) {
      await waitUntil(() => peerFinished)
      // Let the already-sent FIN reach the client while downstream has no
      // further demand. Its private response buffer may still hold the body.
      await Bun.sleep(25)
    }
    afterHead = { ...result, events: [...result.events] }
    if (endPeer) {
      for (
        let pull = 0;
        pull < 8 && !result.complete && !result.failure;
        pull++
      ) {
        registration.demand(1)
        await Bun.sleep(1)
      }
      await waitUntil(() => result.complete || result.failure !== undefined)
    } else {
      await registration.unsubscribe()
      await waitUntil(() => sockets.size === 0)
    }
  } finally {
    await registration?.unsubscribe()
    for (const socket of sockets) socket.destroy()
    await new Promise<void>((resolve) => server.close(() => resolve()))
    expect(server.listening).toBe(false)
  }
  return { ...result, afterHead, cancelled }
}

test.each([
  ["an empty fixed body", "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n", ""],
  ["a fixed body", `${fixedHead}ok`, "ok"],
  ["the final chunk", `${chunkedHead}1\r\na\r\n0\r\n\r\n`, "a"],
  ["a no-content status", "HTTP/1.1 204 No Content\r\n\r\n", ""],
] as const)(
  "HTTP/1 retains %s across downstream pause and peer FIN",
  async (_name, wire, body) => {
    const result = await receive(wire)
    expect(result.afterHead?.events.map((event) => event.kind)).toEqual([
      "ResponseStarted",
    ])
    expect(result.afterHead?.failure).toBeUndefined()
    expect(result.failure).toBeUndefined()
    expect(result.complete).toBe(true)
    expect(
      new TextDecoder().decode(
        Buffer.concat(result.events.flatMap((event) => event.bytes ?? []))
      )
    ).toBe(body)
    expect(result.cancelled).toBe(1)
  }
)

test.each([
  ["unfinished headers", "HTTP/1.1 200 OK\r\nContent-Length:"],
  ["a short fixed body", `${fixedHead}o`],
  ["a missing final chunk", `${chunkedHead}1\r\na\r\n`],
  ["a short chunk body", `${chunkedHead}2\r\na`],
  ["a short chunk terminator", `${chunkedHead}1\r\na\r`],
  ["unfinished trailers", `${chunkedHead}0\r\nX-Test: value\r\n`],
] as const)(
  "HTTP/1 reports real EOF for %s with one event of demand",
  async (_name, wire) => {
    const result = await receive(wire)
    expect(result.complete).toBe(false)
    expect(result.failure?.tag).toBe("HttpProtocolFailure")
    expect(result.failure?.value).toContain("ended early")
    expect(result.cancelled).toBe(1)
  }
)

test("HTTP/1 cancellation closes a paused socket before peer EOF", async () => {
  const result = await receive(`${fixedHead}o`, false)
  expect(result.afterHead?.events.map((event) => event.kind)).toEqual([
    "ResponseStarted",
  ])
  expect(result.failure).toBeUndefined()
  expect(result.complete).toBe(false)
  expect(result.cancelled).toBe(1)
})

test("HTTP/1 unsubscribe waits for an already-started cancellation once", async () => {
  const sockets = new Set<Socket>()
  const server = createServer((socket) => {
    sockets.add(socket)
    socket.on("error", () => undefined)
    socket.once("close", () => sockets.delete(socket))
    socket.once("data", () => {
      socket.write(`HTTP/1.1 200 OK\r\nX-Fill: ${"a".repeat(16 * 1024)}`)
    })
  })
  let releaseCancellation = (): void => undefined
  const cancellationGate = new Promise<void>((resolve) => {
    releaseCancellation = resolve
  })
  let cancelled = 0
  let cancellationFinished = false
  let unsubscribed = 0
  let failure: Observation["failure"]
  let registration: ReturnType<typeof createBunHttp1Exchange> | undefined
  const unsubscriptions: Promise<void>[] = []
  try {
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject)
      server.listen(0, "127.0.0.1", resolve)
    })
    const address = server.address()
    if (address === null || typeof address === "string") {
      throw new Error("HTTP fixture did not listen")
    }
    registration = createBunHttp1Exchange(
      { method: "GET", url: `http://127.0.0.1:${address.port}/`, headers: [] },
      {
        next() {},
        complete() {},
        failure(value: NonNullable<Observation["failure"]>) {
          failure = value
        },
        defect(cause: unknown) {
          throw cause
        },
      },
      {
        knownLength: 0,
        async pull() {
          return { done: true }
        },
        async cancel() {
          cancelled += 1
          await cancellationGate
          cancellationFinished = true
        },
      }
    )
    registration.demand(1)
    await waitUntil(() => failure !== undefined && cancelled === 1)
    expect(failure?.tag).toBe("HttpProtocolFailure")
    for (let call = 0; call < 2; call++) {
      unsubscriptions.push(
        Promise.resolve(registration.unsubscribe()).then(() => {
          unsubscribed += 1
        })
      )
    }
    // Both calls must join the cancellation started by failure, even though
    // the socket has already been destroyed and the subscription is stopped.
    await Bun.sleep(5)
    expect(cancelled).toBe(1)
    expect(cancellationFinished).toBe(false)
    expect(unsubscribed).toBe(0)
    releaseCancellation()
    await Promise.all(unsubscriptions)
    expect(cancellationFinished).toBe(true)
    expect(unsubscribed).toBe(2)
    expect(cancelled).toBe(1)
  } finally {
    releaseCancellation()
    await Promise.allSettled(unsubscriptions)
    await registration?.unsubscribe()
    for (const socket of sockets) socket.destroy()
    await new Promise<void>((resolve) => server.close(() => resolve()))
    expect(server.listening).toBe(false)
  }
})
