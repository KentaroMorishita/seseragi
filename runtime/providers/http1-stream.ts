import * as net from "node:net"
import * as tls from "node:tls"
import type { HttpClientRequestBody } from "@seseragi/runtime/http-client"
import type {
  ProviderSubscriptionObserver,
  ProviderSubscriptionRegistration,
} from "@seseragi/runtime/provider"

type Header = Readonly<{ name: string; value: string }>
type RequestValue = Readonly<{
  method: string
  url: string
  headers: ReadonlyArray<Header>
}>

type BodyMode =
  | Readonly<{ kind: "none" }>
  | { kind: "fixed"; remaining: number }
  | {
      kind: "chunked"
      remaining: number | undefined
      expectCrlf: boolean
      readingTrailers: boolean
    }
  | Readonly<{ kind: "until-close" }>

export function createBunHttp1Exchange(
  value: unknown,
  observerValue: unknown,
  attachment: unknown
): ProviderSubscriptionRegistration {
  const requestValue = validateRequest(value)
  const observer = observerValue as ProviderSubscriptionObserver
  const body = attachment as HttpClientRequestBody
  validateBridge(observer, body)

  const url = new URL(requestValue.url)
  const socket = connect(url)
  let buffer = Buffer.alloc(0)
  let demand = 0
  let bodyMode: BodyMode | undefined
  let responseComplete = false
  let socketEnded = false
  let terminal = false
  let stopped = false
  let requestBodyDone = false
  let requestBodyCancelled = false
  let draining = false

  const cancelBody = async (): Promise<void> => {
    if (requestBodyCancelled) return
    requestBodyCancelled = true
    await body.cancel()
  }
  const fail = (cause: unknown): void => {
    if (terminal || stopped) return
    terminal = true
    observer.failure(httpFailure(cause))
    socket.destroy()
    void cancelBody().catch(() => undefined)
  }
  const finish = (): void => {
    if (terminal || stopped) return
    terminal = true
    observer.complete()
    socket.destroy()
  }
  const stopRequestBody = (): void => {
    if (requestBodyDone || requestBodyCancelled) return
    void cancelBody().catch(observer.defect)
  }

  const drain = (): void => {
    if (draining || terminal || stopped) return
    draining = true
    try {
      while (demand > 0 && !terminal && !stopped) {
        const event = nextEvent()
        if (event === undefined) break
        demand -= 1
        observer.next(event)
      }
      if (responseComplete && buffer.length === 0) finish()
      if (demand > 0 && !terminal && !stopped) socket.resume()
      else socket.pause()
    } catch (cause) {
      fail({
        tag: "HttpProtocolFailure",
        value: cause instanceof Error ? cause.message : "invalid HTTP response",
      })
    } finally {
      draining = false
    }
  }

  const nextEvent = (): unknown | undefined => {
    if (bodyMode === undefined) return parseHead()
    switch (bodyMode.kind) {
      case "none":
        responseComplete = true
        return undefined
      case "fixed":
        if (bodyMode.remaining === 0) {
          responseComplete = true
          return undefined
        }
        if (buffer.length === 0) return undefined
        return bodyChunk(
          Math.min(buffer.length, bodyMode.remaining, 64 * 1024),
          bodyMode
        )
      case "until-close":
        if (buffer.length > 0) {
          return bodyChunk(Math.min(buffer.length, 64 * 1024))
        }
        if (socketEnded) responseComplete = true
        return undefined
      case "chunked":
        return parseChunked(bodyMode)
    }
  }

  const parseHead = (): unknown | undefined => {
    const end = buffer.indexOf("\r\n\r\n")
    if (end < 0) return undefined
    const lines = buffer.subarray(0, end).toString("latin1").split("\r\n")
    buffer = buffer.subarray(end + 4)
    const statusLine = lines.shift() ?? ""
    const matched = /^HTTP\/(1\.0|1\.1) ([0-9]{3})(?: |$)/.exec(statusLine)
    if (matched === null) throw new TypeError("invalid HTTP status line")
    const status = Number(matched[2])
    const headers = lines.map(parseHeader)
    const head = {
      version: matched[1] === "1.0" ? "Http1_0" : "Http1_1",
      status,
      headers,
    }
    if (status >= 100 && status < 200) {
      return { kind: "InformationalResponse", head }
    }

    stopRequestBody()
    bodyMode = responseBodyMode(requestValue.method, status, headers)
    return { kind: "ResponseStarted", head }
  }

  const parseChunked = (mode: Extract<BodyMode, { kind: "chunked" }>) => {
    if (mode.readingTrailers) return parseTrailers()
    if (mode.expectCrlf) {
      if (buffer.length < 2) return undefined
      if (buffer[0] !== 13 || buffer[1] !== 10) {
        throw new TypeError("invalid chunk terminator")
      }
      buffer = buffer.subarray(2)
      mode.expectCrlf = false
      mode.remaining = undefined
    }
    if (mode.remaining === undefined) {
      const lineEnd = buffer.indexOf("\r\n")
      if (lineEnd < 0) return undefined
      const line = buffer.subarray(0, lineEnd).toString("ascii")
      const sizeText = line.split(";", 1)[0] ?? ""
      if (!/^[0-9A-Fa-f]+$/.test(sizeText)) {
        throw new TypeError("invalid chunk size")
      }
      const size = Number.parseInt(sizeText, 16)
      if (!Number.isSafeInteger(size))
        throw new TypeError("chunk size is too large")
      buffer = buffer.subarray(lineEnd + 2)
      if (size === 0) {
        mode.readingTrailers = true
        return parseTrailers()
      }
      mode.remaining = size
    }
    if (mode.remaining === 0) {
      mode.expectCrlf = true
      return parseChunked(mode)
    }
    if (buffer.length === 0) return undefined
    const size = Math.min(buffer.length, mode.remaining, 64 * 1024)
    const event = bodyChunk(size)
    mode.remaining -= size
    if (mode.remaining === 0) mode.expectCrlf = true
    return event
  }

  const parseTrailers = (): unknown | undefined => {
    if (buffer.length >= 2 && buffer[0] === 13 && buffer[1] === 10) {
      buffer = buffer.subarray(2)
      responseComplete = true
      return undefined
    }
    const end = buffer.indexOf("\r\n\r\n")
    if (end < 0) return undefined
    const trailers = buffer
      .subarray(0, end)
      .toString("latin1")
      .split("\r\n")
      .map(parseHeader)
    buffer = buffer.subarray(end + 4)
    responseComplete = true
    return { kind: "ResponseTrailers", headers: trailers }
  }

  const bodyChunk = (
    size: number,
    fixed?: Extract<BodyMode, { kind: "fixed" }>
  ) => {
    const bytes = new Uint8Array(buffer.subarray(0, size))
    buffer = buffer.subarray(size)
    if (fixed !== undefined) {
      fixed.remaining -= size
      if (fixed.remaining === 0) responseComplete = true
    }
    return { kind: "ResponseBodyChunk", bytes }
  }

  socket.pause()
  socket.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, Buffer.from(chunk)])
    drain()
  })
  socket.once("end", () => {
    socketEnded = true
    if (bodyMode?.kind === "until-close") drain()
    else if (!responseComplete && !terminal && !stopped) {
      fail({ tag: "HttpProtocolFailure", value: "HTTP response ended early" })
    }
  })
  socket.once("error", fail)
  socket.once("connect", () => {
    void (async () => {
      const framing = requestFraming(requestValue.headers, body.knownLength)
      await writeSocket(socket, requestHead(url, requestValue, framing.headers))
      await pumpBody(socket, body, framing.chunked, () => {
        requestBodyDone = true
      })
      if (demand > 0) socket.resume()
    })().catch((cause) => {
      if (requestBodyCancelled || stopped) return
      fail({
        tag: "HttpRequestBodyFailure",
        value: cause instanceof Error ? cause.message : "request body failed",
      })
    })
  })

  return Object.freeze({
    demand(count: number) {
      demand += count
      drain()
    },
    async unsubscribe() {
      if (stopped) return
      stopped = true
      socket.destroy()
      buffer = Buffer.alloc(0)
      await cancelBody()
    },
  })
}

function connect(url: URL): net.Socket | tls.TLSSocket {
  const port = Number(url.port || (url.protocol === "https:" ? 443 : 80))
  return url.protocol === "https:"
    ? tls.connect({ host: url.hostname, port, servername: url.hostname })
    : net.connect({ host: url.hostname, port })
}

function validateRequest(value: unknown): RequestValue {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("HTTP stream request is invalid")
  }
  const request = value as Partial<RequestValue>
  if (
    typeof request.method !== "string" ||
    typeof request.url !== "string" ||
    !Array.isArray(request.headers)
  ) {
    throw new TypeError("HTTP stream request is invalid")
  }
  return request as RequestValue
}

function validateBridge(
  observer: ProviderSubscriptionObserver,
  body: HttpClientRequestBody
): void {
  if (
    typeof observer?.next !== "function" ||
    typeof observer.complete !== "function" ||
    typeof observer.failure !== "function" ||
    typeof observer.defect !== "function" ||
    typeof body?.pull !== "function" ||
    typeof body.cancel !== "function"
  ) {
    throw new TypeError("HTTP exchange bridge is invalid")
  }
}

function requestFraming(headers: ReadonlyArray<Header>, knownLength?: number) {
  const output = [...headers]
  const hasLength = headers.some((header) => header.name === "content-length")
  const chunked = !hasLength && knownLength === undefined
  if (!hasLength && knownLength !== undefined) {
    output.push({ name: "content-length", value: String(knownLength) })
  }
  if (chunked) output.push({ name: "transfer-encoding", value: "chunked" })
  output.push({ name: "connection", value: "close" })
  return { headers: output, chunked }
}

function requestHead(
  url: URL,
  request: RequestValue,
  headers: ReadonlyArray<Header>
) {
  const target = `${url.pathname}${url.search}`
  const defaultPort = url.protocol === "https:" ? "443" : "80"
  const host =
    url.port !== "" && url.port !== defaultPort
      ? `${url.hostname}:${url.port}`
      : url.hostname
  const rendered = [
    `${request.method} ${target} HTTP/1.1`,
    `Host: ${host}`,
    ...headers.map(({ name, value }) => `${name}: ${value}`),
    "",
    "",
  ]
  return rendered.join("\r\n")
}

async function pumpBody(
  socket: net.Socket | tls.TLSSocket,
  body: HttpClientRequestBody,
  chunked: boolean,
  completed: () => void
): Promise<void> {
  while (true) {
    const next = await body.pull()
    if (next.done) {
      if (chunked) await writeSocket(socket, "0\r\n\r\n")
      completed()
      return
    }
    const bytes = Buffer.from(next.value)
    if (bytes.length === 0) continue
    if (chunked) {
      await writeSocket(
        socket,
        Buffer.concat([
          Buffer.from(`${bytes.length.toString(16)}\r\n`, "ascii"),
          bytes,
          Buffer.from("\r\n", "ascii"),
        ])
      )
    } else {
      await writeSocket(socket, bytes)
    }
  }
}

function writeSocket(
  socket: net.Socket | tls.TLSSocket,
  value: string | Uint8Array
): Promise<void> {
  if (socket.destroyed)
    return Promise.reject(new Error("HTTP connection is closed"))
  if (socket.write(value)) return Promise.resolve()
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      socket.off("drain", onDrain)
      socket.off("error", onError)
      socket.off("close", onClose)
    }
    const onDrain = () => {
      cleanup()
      resolve()
    }
    const onError = (cause: Error) => {
      cleanup()
      reject(cause)
    }
    const onClose = () => {
      cleanup()
      reject(new Error("HTTP connection closed before the request was written"))
    }
    socket.once("drain", onDrain)
    socket.once("error", onError)
    socket.once("close", onClose)
  })
}

function parseHeader(line: string): Header {
  const separator = line.indexOf(":")
  if (separator <= 0) throw new TypeError("invalid HTTP header")
  return Object.freeze({
    name: line.slice(0, separator).toLowerCase(),
    value: line.slice(separator + 1).trim(),
  })
}

function responseBodyMode(
  method: string,
  status: number,
  headers: ReadonlyArray<Header>
): BodyMode {
  if (method === "HEAD" || status === 204 || status === 304)
    return { kind: "none" }
  const transfer = headers
    .filter((header) => header.name === "transfer-encoding")
    .map((header) => header.value.toLowerCase())
  if (
    transfer.some((value) =>
      value
        .split(",")
        .map((part) => part.trim())
        .includes("chunked")
    )
  ) {
    return {
      kind: "chunked",
      remaining: undefined,
      expectCrlf: false,
      readingTrailers: false,
    }
  }
  const lengths = headers.filter((header) => header.name === "content-length")
  if (lengths.length > 0) {
    if (
      lengths.length !== 1 ||
      !/^(0|[1-9][0-9]*)$/.test(lengths[0]?.value ?? "")
    ) {
      throw new TypeError("invalid response Content-Length")
    }
    return { kind: "fixed", remaining: Number(lengths[0]?.value) }
  }
  return { kind: "until-close" }
}

function httpFailure(cause: unknown) {
  if (typeof cause === "object" && cause !== null && "tag" in cause)
    return cause
  const code =
    typeof cause === "object" && cause !== null && "code" in cause
      ? String(cause.code)
      : ""
  const message = cause instanceof Error ? cause.message : "HTTP request failed"
  if (["ENOTFOUND", "EAI_AGAIN"].includes(code)) {
    return { tag: "HttpDnsFailure", value: message }
  }
  if (code.startsWith("ERR_TLS") || code.includes("CERT")) {
    return { tag: "HttpTlsFailure", value: message }
  }
  return { tag: "HttpConnectionFailure", value: message }
}
