import { describe, expect, test } from "bun:test"
import { provider as browserClientProvider } from "../../../runtime/ts/src/browser/provider-websocket"
import { createEffectExecution } from "../../../runtime/ts/src/effect"
import type { ProviderPackageEntry } from "../../../runtime/ts/src/provider-package"
import { createProviderWebSocketClient } from "../../../runtime/ts/src/provider-websocket"
import { createProviderWebSocketServer } from "../../../runtime/ts/src/provider-websocket-server"
import { bufferCapacity } from "../../../runtime/ts/src/stream"
import type {
  WebSocketConnection,
  WebSocketEvent,
  WebSocketMessage,
} from "../../../runtime/ts/src/websocket"
import {
  createWebSocketClientProvider,
  type WebSocketHost,
} from "../../../runtime/ts/src/websocket-host-provider"

const capacityResult = bufferCapacity(4)
if (capacityResult.tag === "Left") throw new Error("invalid test capacity")
const receiveBuffer = capacityResult.value

describe("portable WebSocket providers", () => {
  for (const name of ["Bun", "Node"] as const) {
    test(`keeps text, Bytes, subprotocol, close, and backpressure semantics on ${name}`, async () => {
      const serverEntry = await loadServerProvider(name)
      const execution = createEffectExecution()
      const port = 38_000 + Math.floor(Math.random() * 10_000)
      const server = createProviderWebSocketServer({
        provider: `test/${name.toLowerCase()}#websocket-server`,
        service: "std/websocket/server::WebSocketServer",
        entry: serverEntry,
      })
      const client = createProviderWebSocketClient({
        provider: "seseragi/runtime-browser#websocket-client",
        service: "std/websocket::WebSocketClient",
        entry: browserClientProvider,
      })
      const started = await server.listen(
        {
          hostname: "127.0.0.1",
          port,
          path: "/socket",
          protocols: ["seseragi.v1"],
          receiveBuffer,
          handler: echoThree,
        },
        execution.context
      )
      expect(started.kind).toBe("success")
      if (started.kind === "failure") throw new Error(started.error.message)

      try {
        const connected = await client.connect(
          {
            url: `ws://127.0.0.1:${port}/socket`,
            protocols: ["seseragi.v1"],
            receiveBuffer,
          },
          execution.context
        )
        expect(connected.kind).toBe("success")
        if (connected.kind === "failure") {
          throw new Error(connected.error.message)
        }
        const connection = connected.value
        expect(connection.protocol).toBe("seseragi.v1")
        const cursor = await connection.events.open({}, execution.context)
        const large = new Uint8Array(128 * 1024)
        large[0] = 7
        large[large.length - 1] = 9

        expect(
          await connection.send(
            { tag: "TextMessage", text: "hello" },
            execution.context
          )
        ).toEqual({ kind: "success", value: undefined })
        expect(
          await connection.send(
            { tag: "BytesMessage", bytes: large },
            execution.context
          )
        ).toEqual({ kind: "success", value: undefined })
        expect(
          await connection.send(
            { tag: "TextMessage", text: "close" },
            execution.context
          )
        ).toEqual({ kind: "success", value: undefined })

        expect(await nextEvent(cursor)).toEqual({
          tag: "TextMessage",
          text: "hello",
        })
        const binary = await nextEvent(cursor)
        expect(binary.tag).toBe("BytesMessage")
        if (binary.tag === "BytesMessage") {
          expect(binary.bytes.byteLength).toBe(large.byteLength)
          expect(binary.bytes[0]).toBe(7)
          expect(binary.bytes[binary.bytes.length - 1]).toBe(9)
        }
        expect(await nextEvent(cursor)).toEqual({
          tag: "TextMessage",
          text: "close",
        })
        const remoteClose = await nextEvent(cursor)
        expect(remoteClose).toEqual({
          tag: "RemoteClosed",
          close: { code: 4000, reason: "done", wasClean: false },
        })
        expect(await cursor.next()).toEqual({ done: true, value: undefined })
        await cursor.close()
      } finally {
        await server.close(started.value, execution.context)
        await execution.close()
      }
    })
  }

  test("cancels a client resource and completes the server receive stream", async () => {
    const bunServerProvider = await loadServerProvider("Bun")
    const serverExecution = createEffectExecution()
    const clientExecution = createEffectExecution()
    const port = 48_000 + Math.floor(Math.random() * 1_000)
    let resolveClose: (event: WebSocketEvent) => void = () => undefined
    const remoteClose = new Promise<WebSocketEvent>((resolve) => {
      resolveClose = resolve
    })
    const server = createProviderWebSocketServer({
      provider: "test/bun#websocket-server-cancellation",
      service: "std/websocket/server::WebSocketServer",
      entry: bunServerProvider,
    })
    const client = createProviderWebSocketClient({
      provider: "seseragi/runtime-browser#websocket-client",
      service: "std/websocket::WebSocketClient",
      entry: browserClientProvider,
    })
    const started = await server.listen(
      {
        hostname: "127.0.0.1",
        port,
        path: "/cancel",
        protocols: [],
        receiveBuffer,
        async handler(connection) {
          const cursor = await connection.events.open(
            {},
            serverExecution.context
          )
          try {
            resolveClose(await nextEvent(cursor))
          } finally {
            await cursor.close()
          }
        },
      },
      serverExecution.context
    )
    if (started.kind === "failure") throw new Error(started.error.message)

    try {
      const connected = await client.connect(
        {
          url: `ws://127.0.0.1:${port}/cancel`,
          protocols: [],
          receiveBuffer,
        },
        clientExecution.context
      )
      if (connected.kind === "failure") {
        throw new Error(connected.error.message)
      }
      await clientExecution.cancel()
      const event = await remoteClose
      expect(event.tag).toBe("RemoteClosed")
      if (event.tag === "RemoteClosed") expect(event.close.code).toBe(1000)
    } finally {
      await server.close(started.value, serverExecution.context)
      await serverExecution.close()
      await clientExecution.close()
    }
  })

  test("bounds pending client sends while host backpressure is active", async () => {
    const execution = createEffectExecution()
    const entry = createWebSocketClientProvider(
      "test/browser#websocket-client-backpressure",
      "browser",
      BackpressuredWebSocket
    )
    const client = createProviderWebSocketClient({
      provider: "test/browser#websocket-client-backpressure",
      service: "std/websocket::WebSocketClient",
      entry,
    })
    const connected = await client.connect(
      {
        url: "ws://example.test/socket",
        protocols: [],
        receiveBuffer,
      },
      execution.context
    )
    if (connected.kind === "failure") throw new Error(connected.error.message)

    try {
      const sends = Array.from({ length: 16 }, (_, index) =>
        connected.value.send(
          { tag: "TextMessage", text: `message-${index}` },
          execution.context
        )
      )
      await waitFor(() => BackpressuredWebSocket.latest.sendCount === 16)
      const overflow = await connected.value.send(
        { tag: "TextMessage", text: "overflow" },
        execution.context
      )
      expect(overflow).toEqual({
        kind: "failure",
        error: {
          tag: "WebSocketSendFailed",
          message: "WebSocket send backpressure queue exceeded 16 messages",
        },
      })

      BackpressuredWebSocket.latest.releaseBackpressure()
      expect(await Promise.all(sends)).toEqual(
        Array.from({ length: 16 }, () => ({
          kind: "success",
          value: undefined,
        }))
      )
    } finally {
      await execution.close()
    }
  })
})

class BackpressuredWebSocket implements WebSocketHost {
  static latest: BackpressuredWebSocket

  readyState = 0
  readonly protocol = ""
  bufferedAmount = 128 * 1024
  binaryType = "arraybuffer"
  sendCount = 0
  readonly #listeners = new Map<string, Set<(event: never) => void>>()

  constructor(_url: string, _protocols?: string | string[]) {
    BackpressuredWebSocket.latest = this
    queueMicrotask(() => {
      this.readyState = 1
      this.#emit("open")
    })
  }

  send(_data: string | Uint8Array): void {
    this.sendCount += 1
  }

  close(code = 1000, reason = ""): void {
    if (this.readyState >= 2) return
    this.readyState = 3
    this.#emit("close", { code, reason, wasClean: true })
  }

  addEventListener(type: string, listener: (event: never) => void): void {
    const listeners = this.#listeners.get(type) ?? new Set()
    listeners.add(listener)
    this.#listeners.set(type, listeners)
  }

  removeEventListener(type: string, listener: (event: never) => void): void {
    this.#listeners.get(type)?.delete(listener)
  }

  releaseBackpressure(): void {
    this.bufferedAmount = 0
  }

  #emit(type: string, event: unknown = undefined): void {
    for (const listener of this.#listeners.get(type) ?? []) {
      listener(event as never)
    }
  }
}

async function loadServerProvider(
  name: "Bun" | "Node"
): Promise<ProviderPackageEntry> {
  return name === "Bun"
    ? (await import("../../../runtime/providers/bun/websocket-server")).provider
    : (await import("../../../runtime/providers/node/websocket-server"))
        .provider
}

async function echoThree(connection: WebSocketConnection): Promise<void> {
  const execution = createEffectExecution()
  const cursor = await connection.events.open({}, execution.context)
  try {
    for (let index = 0; index < 3; index += 1) {
      const event = await nextEvent(cursor)
      if (event.tag === "RemoteClosed") return
      const sent = await connection.send(
        event as WebSocketMessage,
        execution.context
      )
      if (sent.kind === "failure") throw new Error(sent.error.message)
    }
    await connection.close(4000, "done", execution.context)
  } finally {
    await cursor.close()
    await execution.close()
  }
}

async function nextEvent(
  cursor: Readonly<{ next: () => Promise<IteratorResult<WebSocketEvent>> }>
): Promise<WebSocketEvent> {
  const result = await cursor.next()
  if (result.done) throw new Error("WebSocket event stream ended early")
  return result.value
}

async function waitFor(predicate: () => boolean): Promise<void> {
  const deadline = Date.now() + 1_000
  while (!predicate()) {
    if (Date.now() >= deadline) throw new Error("timed out waiting for state")
    await Bun.sleep(1)
  }
}
