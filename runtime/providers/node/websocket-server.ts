import { createServer, type Server } from "node:http"
import { providerRuntimeAbi } from "@seseragi/runtime/provider"
import { defineProviderPackage } from "@seseragi/runtime/provider-package"
import { type WebSocket, WebSocketServer } from "ws"
import {
  connectionOperations,
  listenRequest,
  ServerConnectionToken,
  success,
  unavailable,
} from "../websocket-server-common"

type ServerToken = Readonly<{
  server: Server
  webSockets: WebSocketServer
  connections: Set<ServerConnectionToken>
  sockets: Set<WebSocket>
  children: Set<Promise<void>>
}>

const servers = new Set<ServerToken>()

export const provider = defineProviderPackage({
  abi: providerRuntimeAbi,
  provider: "seseragi/runtime-node#websocket-server",
  service: "std/websocket/server::WebSocketServer",
  targets: ["node-process"],
  operations: {
    async listen(value) {
      try {
        const request = listenRequest(value)
        const server = createServer((_request, response) => {
          response.writeHead(426, { Connection: "Upgrade" })
          response.end()
        })
        const protocolOrder = new Map(
          request.protocols.map((protocol, index) => [protocol, index])
        )
        const webSockets = new WebSocketServer({
          noServer: true,
          ...(protocolOrder.size === 0
            ? {}
            : {
                handleProtocols(protocols: Set<string>) {
                  return (
                    [...protocols]
                      .filter((protocol) => protocolOrder.has(protocol))
                      .sort(
                        (left, right) =>
                          (protocolOrder.get(left) ?? 0) -
                          (protocolOrder.get(right) ?? 0)
                      )[0] ?? false
                  )
                },
              }),
        })
        const token: ServerToken = {
          server,
          webSockets,
          connections: new Set(),
          sockets: new Set(),
          children: new Set(),
        }
        server.on("upgrade", (incoming, socket, head) => {
          const path = new URL(incoming.url ?? "/", "http://localhost").pathname
          if (path !== request.path) {
            socket.end("HTTP/1.1 404 Not Found\r\n\r\n")
            return
          }
          if (
            protocolOrder.size > 0 &&
            !offeredProtocols(incoming.headers["sec-websocket-protocol"]).some(
              (protocol) => protocolOrder.has(protocol)
            )
          ) {
            socket.end("HTTP/1.1 400 Bad Request\r\n\r\n")
            return
          }
          webSockets.handleUpgrade(incoming, socket, head, (webSocket) => {
            webSockets.emit("connection", webSocket, incoming)
          })
        })
        webSockets.on("connection", (webSocket) => {
          token.sockets.add(webSocket)
          webSocket.once("close", () => token.sockets.delete(webSocket))
          const connection = nodeConnection(
            webSocket,
            request.receiveBuffer.value
          )
          token.connections.add(connection)
          const child = request
            .handler({ token: connection, protocol: webSocket.protocol })
            .finally(() => {
              connection.close(1000, "handler completed")
              token.connections.delete(connection)
            })
          token.children.add(child)
          void child.finally(() => token.children.delete(child))
        })
        await start(server, request.port, request.hostname)
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
      for (const socket of token.sockets) socket.terminate()
      await new Promise<void>((resolve) =>
        token.webSockets.close(() => resolve())
      )
      await Promise.race([
        new Promise<void>((resolve, reject) => {
          token.server.close((error) =>
            error === undefined ? resolve() : reject(error)
          )
          token.server.closeAllConnections()
        }),
        new Promise<void>((resolve) => setTimeout(resolve, 250)),
      ])
      servers.delete(token)
      return success(undefined)
    },
    ...connectionOperations,
  },
  shutdown: async () => {
    for (const token of servers) {
      for (const connection of token.connections) {
        connection.close(1001, "server shutdown")
      }
      token.server.closeAllConnections()
    }
    servers.clear()
  },
})

function nodeConnection(
  webSocket: WebSocket,
  capacity: number
): ServerConnectionToken {
  const token = new ServerConnectionToken(
    {
      protocol: webSocket.protocol,
      write(message) {
        return new Promise((resolve, reject) => {
          const value =
            message.tag === "TextMessage"
              ? message.text
              : Buffer.from(message.bytes)
          webSocket.send(value, (error) =>
            error === undefined ? resolve() : reject(error)
          )
        })
      },
      close(code, reason) {
        webSocket.close(code, reason)
      },
    },
    capacity
  )
  webSocket.on("message", (data, isBinary) => {
    token.emit(
      isBinary
        ? { tag: "BytesMessage", bytes: new Uint8Array(data as Buffer) }
        : { tag: "TextMessage", text: data.toString() }
    )
  })
  webSocket.on("close", (code, reason) => {
    token.emit({
      tag: "RemoteClosed",
      close: { code, reason: reason.toString(), wasClean: code !== 1006 },
    })
  })
  return token
}

function start(
  server: Server,
  port: number,
  hostname: string | undefined
): Promise<void> {
  return new Promise((resolve, reject) => {
    server.once("error", reject)
    server.listen(port, hostname, () => {
      server.off("error", reject)
      resolve()
    })
  })
}

function offeredProtocols(value: string | undefined): string[] {
  return value?.split(",").map((protocol) => protocol.trim()) ?? []
}

function serverToken(value: unknown): ServerToken {
  if (
    typeof value !== "object" ||
    value === null ||
    !("server" in value) ||
    !("webSockets" in value)
  ) {
    throw new TypeError("WebSocket server handle is invalid")
  }
  return value as ServerToken
}
