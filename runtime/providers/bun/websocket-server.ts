import { providerRuntimeAbi } from "@seseragi/runtime/provider"
import { defineProviderPackage } from "@seseragi/runtime/provider-package"
import {
  connectionOperations,
  listenRequest,
  ServerConnectionToken,
  success,
  unavailable,
} from "../websocket-server-common"

type BunSocket = Readonly<{
  data: Readonly<{ protocol: string }>
  send(value: string | Uint8Array): number
  close(code?: number, reason?: string): void
}>
type BunServer = Readonly<{
  upgrade(
    request: Request,
    options?: Readonly<{
      data?: Readonly<{ protocol: string }>
      headers?: Readonly<Record<string, string>>
    }>
  ): boolean
  stop(closeActiveConnections?: boolean): void | Promise<void>
}>
type BunHost = Readonly<{
  serve(options: Readonly<Record<string, unknown>>): BunServer
}>
type ServerToken = Readonly<{
  server: BunServer
  connections: Set<ServerConnectionToken>
  children: Set<Promise<void>>
}>

const host = (globalThis as typeof globalThis & { Bun?: BunHost }).Bun
const servers = new Set<ServerToken>()

export const provider = defineProviderPackage({
  abi: providerRuntimeAbi,
  provider: "seseragi/runtime-bun#websocket-server",
  service: "std/websocket/server::WebSocketServer",
  targets: ["bun-process"],
  operations: {
    async listen(value) {
      if (host === undefined) return unavailable("Bun.serve is unavailable")
      try {
        const request = listenRequest(value)
        const connections = new Set<ServerConnectionToken>()
        const children = new Set<Promise<void>>()
        const sockets = new WeakMap<
          object,
          Readonly<{ token: ServerConnectionToken; driver: BunSocketDriver }>
        >()
        const server = host.serve({
          port: request.port,
          ...(request.hostname === undefined
            ? {}
            : { hostname: request.hostname }),
          fetch(nativeRequest: Request, hostServer: BunServer) {
            if (new URL(nativeRequest.url).pathname !== request.path) {
              return new Response(undefined, { status: 404 })
            }
            const protocol = selectProtocol(
              request.protocols,
              nativeRequest.headers.get("sec-websocket-protocol")
            )
            if (request.protocols.length > 0 && protocol === undefined) {
              return new Response(undefined, { status: 400 })
            }
            const upgraded = hostServer.upgrade(nativeRequest, {
              data: { protocol: protocol ?? "" },
              ...(protocol === undefined
                ? {}
                : { headers: { "Sec-WebSocket-Protocol": protocol } }),
            })
            return upgraded
              ? undefined
              : new Response(undefined, { status: 426 })
          },
          websocket: {
            backpressureLimit: 1024 * 1024,
            closeOnBackpressureLimit: true,
            open(webSocket: BunSocket) {
              const driver = new BunSocketDriver(webSocket)
              const connection = new ServerConnectionToken(
                driver,
                request.receiveBuffer.value
              )
              sockets.set(webSocket, { token: connection, driver })
              connections.add(connection)
              const child = request
                .handler({
                  token: connection,
                  protocol: webSocket.data.protocol,
                })
                .finally(() => {
                  connection.close(1000, "handler completed")
                  connections.delete(connection)
                })
              children.add(child)
              void child.finally(() => children.delete(child))
            },
            message(webSocket: BunSocket, message: string | Uint8Array) {
              const connection = sockets.get(webSocket)?.token
              if (connection === undefined) return
              connection.emit(
                typeof message === "string"
                  ? { tag: "TextMessage", text: message }
                  : { tag: "BytesMessage", bytes: new Uint8Array(message) }
              )
            },
            close(webSocket: BunSocket, code: number, reason: string) {
              sockets.get(webSocket)?.token.emit({
                tag: "RemoteClosed",
                close: { code, reason, wasClean: code !== 1006 },
              })
            },
            drain(webSocket: BunSocket) {
              sockets.get(webSocket)?.driver.drain()
            },
          },
        })
        const token: ServerToken = { server, connections, children }
        servers.add(token)
        return success(token)
      } catch (cause) {
        return unavailable(cause)
      }
    },
    async closeServer(value) {
      const token = serverToken(value)
      for (const connection of token.connections) {
        connection.close(1001, "server shutdown")
      }
      await Promise.allSettled([...token.children])
      await boundedStop(token.server.stop(true))
      servers.delete(token)
      return success(undefined)
    },
    ...connectionOperations,
  },
  shutdown: async () => {
    for (const token of servers) await token.server.stop(true)
    servers.clear()
  },
})

class BunSocketDriver {
  readonly protocol: string
  readonly #socket: BunSocket
  #drained:
    | Readonly<{ resolve: () => void; reject: (cause: Error) => void }>
    | undefined

  constructor(socket: BunSocket) {
    this.#socket = socket
    this.protocol = socket.data.protocol
  }

  write(
    message:
      | Readonly<{ tag: "TextMessage"; text: string }>
      | Readonly<{ tag: "BytesMessage"; bytes: Uint8Array }>
  ): Promise<void> {
    const value = message.tag === "TextMessage" ? message.text : message.bytes
    return new Promise((resolve, reject) => {
      const written = this.#socket.send(value)
      if (written > 0) {
        resolve()
      } else if (written === -1) {
        this.#drained = { resolve, reject }
      } else {
        reject(new Error("Bun WebSocket message was dropped"))
      }
    })
  }

  close(code: number, reason: string): void {
    this.#drained?.reject(new Error("Bun WebSocket closed while draining"))
    this.#drained = undefined
    this.#socket.close(code, reason)
  }

  drain(): void {
    const completion = this.#drained
    this.#drained = undefined
    completion?.resolve()
  }
}

function selectProtocol(
  supported: ReadonlyArray<string>,
  header: string | null
): string | undefined {
  if (supported.length === 0) return undefined
  const offered = new Set(
    header?.split(",").map((protocol) => protocol.trim()) ?? []
  )
  return supported.find((protocol) => offered.has(protocol))
}

function serverToken(value: unknown): ServerToken {
  if (
    typeof value !== "object" ||
    value === null ||
    !("server" in value) ||
    !("connections" in value)
  ) {
    throw new TypeError("WebSocket server handle is invalid")
  }
  return value as ServerToken
}

async function boundedStop(completion: void | Promise<void>): Promise<void> {
  if (completion === undefined) return
  await Promise.race([
    completion,
    new Promise<void>((resolve) => setTimeout(resolve, 250)),
  ])
}
