import * as net from "node:net"
import * as tls from "node:tls"

type Header = Readonly<{ name: string; value: string }>
type ConnectedProxy = Readonly<{
  socket: net.Socket
  target: string
  headers: ReadonlyArray<Header>
}>

export type HttpProxyConnection = Readonly<{
  ready: Promise<ConnectedProxy>
  cancel(): void
}>

// Keep proxy negotiation separate from response parsing. The caller retains its
// bounded HTTP/1 parser after either a forward connection or a CONNECT tunnel.
export function createHttpProxyConnection(
  target: URL,
  proxy: URL,
  headers: ReadonlyArray<Header>
): HttpProxyConnection {
  const sockets = new Set<net.Socket>()
  let tlsParent: net.Socket | undefined
  let stopped = false
  let settled = false
  let resolveReady!: (value: ConnectedProxy) => void
  let rejectReady!: (cause: unknown) => void
  const ready = new Promise<ConnectedProxy>((resolve, reject) => {
    resolveReady = resolve
    rejectReady = reject
  })
  // Cancellation may precede the caller's ready handler.
  void ready.catch(() => undefined)
  const destroy = () => {
    for (const socket of sockets) socket.destroy()
  }
  const fail = (cause: unknown) => {
    if (settled) return
    settled = true
    stopped = true
    destroy()
    const code =
      typeof cause === "object" && cause !== null && "code" in cause
        ? String(cause.code)
        : ""
    rejectReady(
      code.startsWith("HPE_")
        ? {
            tag: "HttpProtocolFailure",
            value:
              cause instanceof Error ? cause.message : "invalid proxy response",
          }
        : cause
    )
  }
  const own = (socket: net.Socket) => {
    if (sockets.has(socket) || socket.destroyed) {
      if (stopped) socket.destroy()
      return socket
    }
    sockets.add(socket)
    socket.on("error", fail)
    socket.once("close", () => {
      sockets.delete(socket)
      if (!settled && socket !== tlsParent)
        fail(new Error("HTTP proxy connection closed early"))
    })
    if (stopped) socket.destroy()
    return socket
  }
  const accept = (
    socket: net.Socket,
    requestTarget: string,
    outgoingHeaders: ReadonlyArray<Header>
  ) => {
    if (stopped) {
      socket.destroy()
      return
    }
    socket.pause()
    settled = true
    resolveReady({ socket, target: requestTarget, headers: outgoingHeaders })
  }

  try {
    if (proxy.protocol !== "http:" && proxy.protocol !== "https:") {
      throw new TypeError("unsupported HTTP proxy protocol")
    }
    const authorization = headers.filter(
      (header) => header.name.toLowerCase() === "proxy-authorization"
    )
    if (
      authorization.length === 0 &&
      (proxy.username !== "" || proxy.password !== "")
    ) {
      authorization.push({
        name: "proxy-authorization",
        value: `Basic ${Buffer.from(
          `${decodeURIComponent(proxy.username)}:${decodeURIComponent(proxy.password)}`
        ).toString("base64")}`,
      })
    }
    const proxyHost = hostname(proxy)
    const proxyPort = Number(
      proxy.port || (proxy.protocol === "https:" ? 443 : 80)
    )
    if (target.protocol === "http:") {
      const outgoing = headers.some(
        (header) => header.name.toLowerCase() === "proxy-authorization"
      )
        ? headers
        : [...headers, ...authorization]
      const socket = own(
        proxy.protocol === "https:"
          ? tls.connect({
              host: proxyHost,
              port: proxyPort,
              servername: net.isIP(proxyHost) === 0 ? proxyHost : "",
              rejectUnauthorized: true,
            })
          : net.connect({ host: proxyHost, port: proxyPort })
      )
      socket.once(
        proxy.protocol === "https:" ? "secureConnect" : "connect",
        () => {
          accept(
            socket,
            `${target.origin}${target.pathname}${target.search}`,
            outgoing
          )
        }
      )
    } else if (target.protocol === "https:") {
      const targetHost = hostname(target)
      const authority = `${target.hostname}:${target.port || 443}`
      const socket = own(
        proxy.protocol === "https:"
          ? tls.connect({
              host: proxyHost,
              port: proxyPort,
              servername: net.isIP(proxyHost) === 0 ? proxyHost : "",
              rejectUnauthorized: true,
            })
          : net.connect({ host: proxyHost, port: proxyPort })
      )
      // Bun's node:http CONNECT path treats an interim response as the tunnel.
      // Read only bounded heads here, then hand the raw socket to verified TLS.
      const head = Buffer.alloc(16 * 1024)
      let used = 0
      let information = 0
      const protocolFailure = (message: string) =>
        fail({ tag: "HttpProtocolFailure", value: message })
      const onData = (chunk: Buffer) => {
        if (stopped) return
        for (let index = 0; index < chunk.length; index += 1) {
          if (used === head.length) {
            protocolFailure("HTTP proxy response headers exceed 16384 bytes")
            return
          }
          head[used++] = chunk[index]!
          if (
            used < 4 ||
            head[used - 4] !== 13 ||
            head[used - 3] !== 10 ||
            head[used - 2] !== 13 ||
            head[used - 1] !== 10
          )
            continue
          const lines = head
            .subarray(0, used - 4)
            .toString("latin1")
            .split("\r\n")
          const statusLine = lines.shift() ?? ""
          const matched =
            /^HTTP\/1\.[01] ([0-9]{3})(?: [\t\x20-\x7e\x80-\xff]*)?$/.exec(
              statusLine
            )
          if (
            matched === null ||
            lines.some((line) => !validProxyHeader(line))
          ) {
            protocolFailure("invalid HTTP proxy response headers")
            return
          }
          const status = Number(matched[1])
          if (status >= 100 && status < 200 && status !== 101) {
            information += 1
            if (information > 16) {
              protocolFailure(
                "HTTP proxy informational response limit exceeded"
              )
              return
            }
            used = 0
            continue
          }
          if (status === 101 || status < 100) {
            protocolFailure(
              "HTTP proxy sent an unexpected protocol upgrade or status"
            )
            return
          }
          if (status < 200 || status >= 300) {
            fail(new Error(`HTTP proxy CONNECT failed with status ${status}`))
            return
          }
          socket.pause()
          socket.off("data", onData)
          if (index + 1 < chunk.length)
            socket.unshift(chunk.subarray(index + 1))
          try {
            tlsParent = socket
            const inner = own(
              tls.connect({
                socket,
                host: targetHost,
                port: Number(target.port || 443),
                servername: net.isIP(targetHost) === 0 ? targetHost : "",
                rejectUnauthorized: true,
              })
            )
            inner.once("secureConnect", () =>
              accept(
                inner,
                `${target.pathname}${target.search}`,
                headers.filter(
                  (header) =>
                    header.name.toLowerCase() !== "proxy-authorization"
                )
              )
            )
          } catch (cause) {
            fail(cause)
          }
          return
        }
      }
      socket.on("data", onData)
      socket.once("end", () => {
        if (!settled && socket !== tlsParent)
          fail(new Error("HTTP proxy CONNECT ended early"))
      })
      socket.once(
        proxy.protocol === "https:" ? "secureConnect" : "connect",
        () => {
          if (stopped) return
          socket.write(
            [
              `CONNECT ${authority} HTTP/1.1`,
              `Host: ${authority}`,
              ...authorization.map(({ name, value }) => `${name}: ${value}`),
              "",
              "",
            ].join("\r\n"),
            (cause) => {
              if (cause) fail(cause)
            }
          )
        }
      )
    } else {
      throw new TypeError("unsupported HTTP target protocol")
    }
  } catch (cause) {
    fail(cause)
  }

  return Object.freeze({
    ready,
    cancel() {
      stopped = true
      if (!settled) {
        settled = true
        rejectReady(new Error("HTTP proxy connection cancelled"))
      }
      destroy()
    },
  })
}

function hostname(url: URL): string {
  return url.hostname.startsWith("[") ? url.hostname.slice(1, -1) : url.hostname
}

function validProxyHeader(line: string): boolean {
  const colon = line.indexOf(":")
  return (
    colon > 0 &&
    /^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/.test(line.slice(0, colon)) &&
    /^[\t\x20-\x7e\x80-\xff]*$/.test(line.slice(colon + 1))
  )
}
