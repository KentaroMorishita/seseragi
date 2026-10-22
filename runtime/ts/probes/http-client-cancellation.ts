import assert from "node:assert/strict"
import { createServer } from "node:http"
import type { Socket } from "node:net"
import { createFetchHttpClientProvider } from "../../providers/http-client"
import type { ProviderSubscriptionRegistration } from "../src/provider"

const target = process.argv[2]
assert(target === "node-process" || target === "bun-process")
const provider = createFetchHttpClientProvider(
  `seseragi/runtime-${target === "node-process" ? "node" : "bun"}#http-client`,
  target
)
const results: unknown[] = []

await assertPendingCancellation()
await assertRejectedCancellation()
process.stdout.write(
  `${JSON.stringify({ target, pid: process.pid, results })}\nHTTP client cancellation probe passed\n`
)

async function assertPendingCancellation(): Promise<void> {
  let releaseCancellation = (): void => undefined
  const cancellationGate = new Promise<void>((resolve) => {
    releaseCancellation = resolve
  })
  let releasePull = (): void => undefined
  const pullGate = new Promise<IteratorResult<Uint8Array>>((resolve) => {
    releasePull = () => resolve({ done: true, value: undefined })
  })
  let pulls = 0
  let cancellations = 0
  let cancellationFinished = false
  let unsubscribed = 0
  const completions: Promise<void>[] = []
  await withServer(
    false,
    async (port, state, register) => {
      const registration = register(
        provider.exchange!(
          {
            method: "POST",
            url: `http://127.0.0.1:${port}/early`,
            headers: [],
          },
          observer(),
          {
            async pull() {
              pulls += 1
              if (pulls === 1)
                return { done: false, value: new Uint8Array([7]) }
              return await pullGate
            },
            async cancel() {
              cancellations += 1
              await cancellationGate
              cancellationFinished = true
              releasePull()
            },
          }
        )
      )
      registration.demand(1)
      await waitUntil(
        () => cancellations === 1,
        "early response must start upload cancellation"
      )
      for (let call = 0; call < 2; call++) {
        completions.push(
          Promise.resolve(registration.unsubscribe()).then(() => {
            unsubscribed += 1
          })
        )
      }
      await waitUntil(
        () => state.closed,
        "unsubscribe must close transport before upload cleanup"
      )
      await delay(5)
      assert.equal(cancellations, 1)
      assert.equal(cancellationFinished, false)
      assert.equal(
        unsubscribed,
        0,
        "both unsubscribes must join in-flight cancellation"
      )
      assert.equal(state.fullyWritten, false)
      releaseCancellation()
      await deadline(Promise.all(completions), "shared cancellation completion")
      assert.equal(cancellationFinished, true)
      assert.equal(unsubscribed, 2)
      assert.equal(cancellations, 1)
      results.push({
        case: "pending",
        cancellations,
        cancellationFinished,
        unsubscribed,
        cancelledBeforeEof: !state.fullyWritten,
      })
    },
    async () => {
      releaseCancellation()
      releasePull()
      await Promise.allSettled(completions)
    }
  )
}

async function assertRejectedCancellation(): Promise<void> {
  let headReceived = false
  let cancellations = 0
  const cleanupFailure = new Error("fixture upload cleanup failure")
  await withServer(true, async (port, state, register) => {
    const registration = register(
      provider.exchange!(
        { method: "GET", url: `http://127.0.0.1:${port}/finite`, headers: [] },
        observer(() => {
          headReceived = true
        }),
        {
          knownLength: 0,
          async pull() {
            return { done: true, value: undefined }
          },
          async cancel() {
            cancellations += 1
            throw cleanupFailure
          },
        }
      )
    )
    registration.demand(1)
    await waitUntil(() => headReceived, "response head must arrive")
    assert.equal(cancellations, 0)
    const completions = await Promise.allSettled([
      registration.unsubscribe(),
      registration.unsubscribe(),
    ])
    assert(
      completions.every(
        (result) =>
          result.status === "rejected" && result.reason === cleanupFailure
      ),
      "all unsubscribe callers must observe the same cleanup failure"
    )
    await waitUntil(
      () => state.closed,
      "cleanup rejection must still close transport"
    )
    assert.equal(state.fullyWritten, false)
    assert.equal(cancellations, 1)
    results.push({
      case: "rejected",
      cancellations,
      rejectedCallers: completions.length,
      cancelledBeforeEof: !state.fullyWritten,
    })
  })
}

async function withServer(
  waitForUpload: boolean,
  body: (
    port: number,
    state: { closed: boolean; fullyWritten: boolean },
    register: (value: unknown) => ProviderSubscriptionRegistration
  ) => Promise<void>,
  release: () => Promise<void> = async () => undefined
): Promise<void> {
  const sockets = new Set<Socket>()
  const timers = new Set<ReturnType<typeof setTimeout>>()
  const registrations: ProviderSubscriptionRegistration[] = []
  const state = { closed: false, fullyWritten: false }
  const server = createServer((request, response) => {
    const respond = (): void => {
      response.writeHead(200, { "content-type": "application/octet-stream" })
      response.write(new Uint8Array([1]))
      const timer = setTimeout(() => {
        timers.delete(timer)
        state.fullyWritten = true
        response.end(new Uint8Array([2]))
      }, 2000)
      timers.add(timer)
      response.once("close", () => {
        state.closed = true
        clearTimeout(timer)
        timers.delete(timer)
      })
    }
    if (waitForUpload) {
      request.resume()
      request.once("end", respond)
    } else respond()
  })
  server.on("connection", (socket) => {
    sockets.add(socket)
    socket.on("error", () => undefined)
    socket.once("close", () => sockets.delete(socket))
  })
  try {
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject)
      server.listen(0, "127.0.0.1", resolve)
    })
    const address = server.address()
    assert(address !== null && typeof address !== "string")
    await body(address.port, state, (value) => {
      const registration = value as ProviderSubscriptionRegistration
      registrations.push(registration)
      return registration
    })
  } finally {
    await release()
    await Promise.allSettled(
      registrations.map((registration) => registration.unsubscribe())
    )
    for (const timer of timers) clearTimeout(timer)
    timers.clear()
    const stopped = new Promise<void>((resolve, reject) =>
      server.close((error) => (error === undefined ? resolve() : reject(error)))
    )
    for (const socket of sockets) socket.destroy()
    server.closeAllConnections()
    await deadline(stopped, "fixture listener close")
    await waitUntil(() => sockets.size === 0, "fixture socket cleanup")
    assert.equal(server.listening, false)
  }
}

function observer(next: () => void = () => undefined) {
  return {
    next,
    complete() {},
    failure(cause: unknown) {
      throw new Error(`unexpected typed failure: ${JSON.stringify(cause)}`)
    },
    defect(cause: unknown) {
      throw cause
    },
  }
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}
async function waitUntil(
  condition: () => boolean,
  label: string
): Promise<void> {
  const limit = Date.now() + 2000
  while (!condition()) {
    assert(Date.now() < limit, `${label} timed out`)
    await delay(5)
  }
}
async function deadline<T>(completion: Promise<T>, label: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      completion,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(`${label} timed out`)), 2000)
      }),
    ])
  } finally {
    if (timer !== undefined) clearTimeout(timer)
  }
}
