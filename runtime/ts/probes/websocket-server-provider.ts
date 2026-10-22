import assert from "node:assert/strict"
import { once } from "node:events"
import type { AddressInfo } from "node:net"
import { createConnection, type Socket } from "node:net"
import { WebSocket } from "ws"
import type { ServerConnectionToken } from "../../providers/websocket-server-common"
import type { ProviderResult } from "../src/provider"
import { ProviderPackageLoader } from "../src/provider-package"

const target = process.argv[2]
assert(target === "node" || target === "bun", "probe requires node or bun")
const scenario = process.argv[3] ?? "all"
const { provider } =
  target === "node"
    ? await import("../../providers/node/websocket-server")
    : await import("../../providers/bun/websocket-server")
const identity = `seseragi/runtime-${target}#websocket-server`
const loader = new ProviderPackageLoader(
  target === "node" ? "node-process" : "bun-process",
  [
    {
      provider: identity,
      service: "std/websocket/server::WebSocketServer",
      target: target === "node" ? "node-process" : "bun-process",
      module: "websocket-server-probe",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider }),
    },
  ]
)
await loader.load(identity)

type Attachment = Readonly<{ token: ServerConnectionToken }>
type Handle = Readonly<{
  server: Readonly<{ address?: () => AddressInfo; port?: number }>
  connections: Set<ServerConnectionToken>
  children: Set<Promise<void>>
  sockets?: Set<WebSocket>
}>
type Event = Readonly<{ tag: string; text?: string }>
const clients = new Set<WebSocket>()
const sockets = new Set<Socket>()
let acquired = 0
let released = 0

if (scenario === "all" || scenario === "handler") {
  await assertHandlerFailure("throw")
  await assertHandlerFailure("reject")
  await assertShutdownClosesReceiver()
}
if (target === "node" && (scenario === "all" || scenario === "frame")) {
  await assertMalformedFrame(false)
  await assertMalformedFrame(true)
}
if (target === "node" && (scenario === "all" || scenario === "upgrade")) {
  await assertMalformedUpgrade()
}
if (target === "bun" && scenario === "drain") await assertBunDrainClose()
if (scenario === "all") await assertProviderShutdown()
await loader.shutdown()
assert.equal(acquired, released, "every listener must be released")
assert.equal(clients.size, 0, "every test client must be released")
assert.equal(sockets.size, 0, "every raw test socket must be released")
process.stdout.write(
  `WebSocket server isolation probe passed: ${target}; listeners=${released}; pid=${process.pid}\n`
)

async function assertHandlerFailure(mode: "throw" | "reject"): Promise<void> {
  let accepted = 0
  await withServer(
    (attachment) => {
      accepted += 1
      if (accepted === 1) {
        if (mode === "throw") throw new Error("fixture synchronous failure")
        return Promise.reject(new Error("fixture asynchronous failure"))
      }
      return echo(attachment.token)
    },
    async (handle, port) => {
      const client = await openClient(port)
      const [code] = await bounded(
        once(client, "close"),
        "failed handler close"
      )
      assert.equal(code, 1011, "handler failure must close only its connection")
      await waitUntil(
        () => handle.children.size === 0 && handle.connections.size === 0,
        "failed handler cleanup"
      )
      await assertHealthy(port)
    }
  )
}

async function assertShutdownClosesReceiver(): Promise<void> {
  let observedClose = false
  await withServer(
    async ({ token }) => {
      const receiver = receive(token)
      try {
        await receiver.finished
        observedClose = receiver.events.some(
          (event) => event.tag === "RemoteClosed"
        )
      } finally {
        receiver.unsubscribe()
      }
    },
    async (handle, port, close) => {
      const client = await openClient(port)
      const closed = once(client, "close")
      await close()
      await bounded(closed, "shutdown client close")
      assert(observedClose, "server close must release an active receiver")
      assert.equal(handle.connections.size, 0)
      assert.equal(handle.children.size, 0)
    }
  )
}

async function assertProviderShutdown(): Promise<void> {
  await withServer(
    ({ token }) => echo(token),
    async (handle, port, close) => {
      const client = await openClient(port)
      const closed = once(client, "close")
      await close()
      await bounded(closed, "provider shutdown client close")
      assert.equal(handle.children.size, 0)
      assert.equal(handle.connections.size, 0)
      await assert.rejects(openSocket(port), /ECONNREFUSED/u)
    },
    undefined,
    true
  )
}

async function assertBunDrainClose(): Promise<void> {
  type SocketHost = Readonly<{
    data: Readonly<{ protocol: string }>
    send: () => number
    close: () => void
  }>
  type Callbacks = Readonly<{
    open: (socket: SocketHost) => void
    close: (socket: SocketHost, code: number, reason: string) => void
  }>
  const host = (
    globalThis as unknown as {
      Bun: { serve: (options: unknown) => unknown }
    }
  ).Bun
  const original = host.serve
  let callbacks: Callbacks | undefined
  let writes = 0
  let outcome: ProviderResult | undefined
  host.serve = (options) => {
    callbacks = (options as { websocket: Callbacks }).websocket
    return { upgrade: () => false, stop: () => undefined }
  }
  let handle: Handle | undefined
  try {
    const started = (await provider.listen!({
      hostname: "127.0.0.1",
      port: 0,
      path: "/socket",
      protocols: [],
      receiveBuffer: { value: 4 },
      async handler({ token }: Attachment) {
        outcome = await token.send({ tag: "TextMessage", text: "small" })
      },
    })) as ProviderResult
    assert.equal(started.kind, "success")
    if (started.kind !== "success") throw new Error("fixture listener failed")
    acquired += 1
    handle = started.value as Handle
    const socket: SocketHost = {
      data: { protocol: "" },
      send() {
        writes += 1
        return -1
      },
      close() {},
    }
    assert(callbacks)
    callbacks.open(socket)
    await waitUntil(() => writes === 1, "Bun backpressure write")
    callbacks.close(socket, 1006, "fixture remote close")
    await waitUntil(
      () => handle?.children.size === 0,
      "Bun pending send cleanup"
    )
    assert.equal(outcome?.kind, "failure")
    assert.equal(handle.connections.size, 0)
  } finally {
    if (handle !== undefined) {
      await bounded(
        Promise.resolve(provider.closeServer!(handle)),
        "fixture close"
      )
      released += 1
    }
    host.serve = original
  }
}

async function assertMalformedFrame(delayedReceiver: boolean): Promise<void> {
  let attach = (): void => undefined
  const attached = new Promise<void>((resolve) => {
    attach = resolve
  })
  let allowReceive = (): void => undefined
  const receiveGate = new Promise<void>((resolve) => {
    allowReceive = resolve
  })
  let recorded: ReturnType<typeof receive> | undefined
  let accepted = 0
  await withServer(
    async ({ token }) => {
      accepted += 1
      if (accepted > 1) return echo(token)
      attach()
      if (delayedReceiver) await receiveGate
      recorded = receive(token)
      try {
        await recorded.finished
      } finally {
        recorded.unsubscribe()
      }
    },
    async (handle, port) => {
      const socket = await rawUpgrade(port)
      await bounded(attached, "malformed connection handler")
      // One small masked message followed by an illegal unmasked frame. A
      // delayed subscriber must see the failure, not replay stale queued data.
      socket.write(
        Buffer.concat([
          ...(delayedReceiver ? [maskedText("queued")] : []),
          Buffer.from([0x81, 0x01, 0x61]),
        ])
      )
      await waitUntil(() => handle.sockets?.size === 0, "failed socket close")
      allowReceive()
      await waitUntil(
        () => handle.connections.size === 0 && handle.children.size === 0,
        "failed connection handler cleanup"
      )
      assert(recorded, "connection receiver must start")
      assert.equal(recorded.failures.length, 1)
      assert.equal(recorded.failures[0]?.tag, "WebSocketConnectionFailed")
      assert.equal(recorded.events.length, 0)
      assert.equal(recorded.completions, 0)
      assert.equal(recorded.defects.length, 0)
      await assertHealthy(port)
    },
    allowReceive
  )
}

async function assertMalformedUpgrade(): Promise<void> {
  await withServer(
    ({ token }) => echo(token),
    async (handle, port) => {
      const socket = await openSocket(port)
      const response = readHeaders(socket)
      // Node's HTTP parser accepts this request target; WHATWG URL parsing
      // rejects its invalid authority. It must remain a request-local 400.
      socket.write(upgradeRequest("//["))
      assert.match(await response, /^HTTP\/1\.1 400 /u)
      socket.destroy()
      assert.equal(handle.connections.size, 0)
      assert.equal(handle.children.size, 0)
      await assertHealthy(port)
    }
  )
}

async function echo(token: ServerConnectionToken): Promise<void> {
  const writes: Promise<ProviderResult>[] = []
  const receiver = receive(token, (event) => {
    if (event.tag === "TextMessage") {
      writes.push(token.send({ tag: "TextMessage", text: event.text ?? "" }))
    }
  })
  try {
    await receiver.finished
    for (const result of await Promise.all(writes)) {
      assert.equal(result.kind, "success")
    }
  } finally {
    receiver.unsubscribe()
  }
}

function receive(
  token: ServerConnectionToken,
  onEvent?: (event: Event) => void
) {
  const events: Event[] = []
  const failures: { tag?: string }[] = []
  const defects: unknown[] = []
  let completions = 0
  let finish = (): void => undefined
  const finished = new Promise<void>((resolve) => {
    finish = resolve
  })
  const subscription = token.subscribe({
    next(value) {
      const event = value as Event
      events.push(event)
      onEvent?.(event)
    },
    complete() {
      completions += 1
      finish()
    },
    failure(value) {
      failures.push(value as { tag?: string })
      finish()
    },
    defect(cause) {
      defects.push(cause)
      finish()
    },
  })
  return {
    events,
    failures,
    defects,
    get completions() {
      return completions
    },
    finished: bounded(finished, "connection receiver terminal"),
    unsubscribe: () => subscription.unsubscribe(),
  }
}

async function withServer(
  handler: (attachment: Attachment) => Promise<void>,
  body: (
    handle: Handle,
    port: number,
    close: () => Promise<void>
  ) => Promise<void>,
  releaseHandler: () => void = () => undefined,
  shutdownProvider = false
): Promise<void> {
  const started = (await provider.listen!({
    hostname: "127.0.0.1",
    port: 0,
    path: "/socket",
    protocols: [],
    receiveBuffer: { value: 4 },
    handler,
  })) as ProviderResult
  assert.equal(started.kind, "success")
  if (started.kind !== "success") throw new Error("listener did not start")
  acquired += 1
  const handle = started.value as Handle
  const port = handle.server.port ?? handle.server.address?.().port
  assert(port !== undefined && port > 0)
  let closing: Promise<void> | undefined
  const close = (): Promise<void> => {
    closing ??= bounded(
      (shutdownProvider
        ? loader.shutdown()
        : Promise.resolve(provider.closeServer!(handle)).then((value) => {
            assert.equal((value as ProviderResult).kind, "success")
          })
      ).then(() => {
        released += 1
      }),
      "server close"
    )
    return closing
  }
  try {
    await body(handle, port, close)
  } finally {
    releaseHandler()
    const remainingClients = [...clients]
    const remainingSockets = [...sockets]
    for (const client of remainingClients) client.terminate()
    for (const socket of remainingSockets) socket.destroy()
    await close()
    await waitUntil(
      () => clients.size === 0 && sockets.size === 0,
      "owned client/socket cleanup"
    )
    assert.equal(handle.children.size, 0)
    assert.equal(handle.connections.size, 0)
    assert.equal(handle.sockets?.size ?? 0, 0)
  }
}

async function assertHealthy(port: number): Promise<void> {
  const client = await openClient(port)
  const message = once(client, "message")
  client.send("healthy")
  const [data] = await bounded(message, "healthy echo")
  assert.equal(String(data), "healthy", "a new connection must still work")
  const closed = once(client, "close")
  client.close(1000, "done")
  await bounded(closed, "healthy client close")
}

async function openClient(port: number): Promise<WebSocket> {
  const client = new WebSocket(`ws://127.0.0.1:${port}/socket`)
  clients.add(client)
  client.once("close", () => clients.delete(client))
  await bounded(once(client, "open"), "client open")
  return client
}

async function openSocket(port: number): Promise<Socket> {
  const socket = createConnection({ host: "127.0.0.1", port })
  sockets.add(socket)
  socket.on("error", () => undefined)
  socket.once("close", () => sockets.delete(socket))
  await bounded(once(socket, "connect"), "raw socket connect")
  return socket
}

async function rawUpgrade(port: number): Promise<Socket> {
  const socket = await openSocket(port)
  const headers = readHeaders(socket)
  socket.write(upgradeRequest("/socket"))
  assert.match(await headers, /^HTTP\/1\.1 101 /u)
  return socket
}

function upgradeRequest(path: string): string {
  return [
    `GET ${path} HTTP/1.1`,
    "Host: 127.0.0.1",
    "Connection: Upgrade",
    "Upgrade: websocket",
    "Sec-WebSocket-Version: 13",
    "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==",
    "\r\n",
  ].join("\r\n")
}

async function readHeaders(socket: Socket): Promise<string> {
  let contents = ""
  let resolveHeaders = (_value: string): void => undefined
  let rejectHeaders = (_cause: Error): void => undefined
  const headers = new Promise<string>((resolve, reject) => {
    resolveHeaders = resolve
    rejectHeaders = reject
  })
  const onData = (chunk: Buffer): void => {
    contents += chunk.toString("latin1")
    if (contents.includes("\r\n\r\n")) resolveHeaders(contents)
  }
  const onClose = (): void => rejectHeaders(new Error("upgrade socket closed"))
  socket.on("data", onData)
  socket.once("close", onClose)
  try {
    return await bounded(headers, "upgrade response")
  } finally {
    socket.off("data", onData)
    socket.off("close", onClose)
  }
}

function maskedText(text: string): Buffer {
  const bytes = Buffer.from(text)
  const mask = Buffer.from([1, 2, 3, 4])
  assert(bytes.length < 126)
  return Buffer.concat([
    Buffer.from([0x81, 0x80 | bytes.length]),
    mask,
    Buffer.from(bytes.map((byte, index) => byte ^ mask[index % 4]!)),
  ])
}

async function waitUntil(
  condition: () => boolean,
  label: string
): Promise<void> {
  const started = Date.now()
  while (!condition()) {
    assert(Date.now() - started < 2000, `${label} timed out`)
    await new Promise((resolve) => setTimeout(resolve, 5))
  }
}

async function bounded<T>(completion: Promise<T>, label: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      completion,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(`${label} timed out`)), 2500)
      }),
    ])
  } finally {
    if (timer !== undefined) clearTimeout(timer)
  }
}
