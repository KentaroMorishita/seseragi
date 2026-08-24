import {
  type ProviderResult,
  type ProviderSubscriptionObserver,
  type ProviderSubscriptionRegistration,
  providerRuntimeAbi,
  withProviderCancellation,
} from "./provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
  type ProviderRuntimeTarget,
} from "./provider-package"

type HostMessageEvent = Readonly<{ data: unknown }>
type HostCloseEvent = Readonly<{
  code: number
  reason: string
  wasClean: boolean
}>
export type WebSocketHost = {
  readonly readyState: number
  readonly protocol: string
  readonly bufferedAmount: number
  binaryType: string
  send(data: string | Uint8Array): void
  close(code?: number, reason?: string): void
  addEventListener(type: string, listener: (event: never) => void): void
  removeEventListener(type: string, listener: (event: never) => void): void
}

export type WebSocketHostConstructor = new (
  url: string,
  protocols?: string | string[]
) => WebSocketHost

class ConnectionToken {
  readonly socket: WebSocketHost
  readonly #capacity: number
  readonly #queue: unknown[] = []
  #observer: ProviderSubscriptionObserver | undefined
  #failure: unknown
  #terminal = false
  #delivery = Promise.resolve()
  #pendingSends = 0

  constructor(socket: WebSocketHost, capacity: number) {
    this.socket = socket
    this.#capacity = capacity
    socket.addEventListener("message", this.#onMessage)
    socket.addEventListener("close", this.#onClose)
    socket.addEventListener("error", this.#onError)
  }

  subscribe(
    observer: ProviderSubscriptionObserver
  ): ProviderSubscriptionRegistration {
    if (this.#observer !== undefined) {
      throw new TypeError("WebSocket connection already has an active receiver")
    }
    this.#observer = observer
    if (this.#failure !== undefined) {
      observer.failure(this.#failure)
    } else {
      for (const event of this.#queue.splice(0)) observer.next(event)
      if (this.#terminal) observer.complete()
    }
    return Object.freeze({
      demand() {},
      unsubscribe: () => {
        if (this.#observer === observer) this.#observer = undefined
      },
    })
  }

  reserveSend(): boolean {
    if (this.#pendingSends >= 16) return false
    this.#pendingSends += 1
    return true
  }

  releaseSend(): void {
    this.#pendingSends = Math.max(0, this.#pendingSends - 1)
  }

  readonly #onMessage = (eventValue: never): void => {
    const event = eventValue as HostMessageEvent
    this.#delivery = this.#delivery.then(async () => {
      if (this.#terminal) return
      try {
        this.#emit(await decodeMessage(event.data))
      } catch (cause) {
        this.#fail({
          tag: "WebSocketConnectionFailed",
          message: errorMessage(cause),
        })
      }
    })
  }

  readonly #onClose = (eventValue: never): void => {
    const event = eventValue as HostCloseEvent
    this.#delivery = this.#delivery.then(() => {
      if (this.#terminal) return
      this.#emit({
        tag: "RemoteClosed",
        close: {
          code: event.code,
          reason: event.reason,
          wasClean: event.wasClean,
        },
      })
      this.#terminal = true
      this.#observer?.complete()
      this.#detach()
    })
  }

  readonly #onError = (): void => {
    this.#fail({
      tag: "WebSocketConnectionFailed",
      message: "WebSocket transport failed",
    })
  }

  #emit(event: unknown): void {
    if (this.#observer !== undefined) {
      this.#observer.next(event)
      return
    }
    if (this.#queue.length >= this.#capacity) {
      this.#fail({
        tag: "WebSocketBufferOverflow",
        message: `WebSocket receive buffer exceeded ${this.#capacity} messages`,
      })
      this.socket.close(1008, "receive buffer overflow")
      return
    }
    this.#queue.push(event)
  }

  #fail(failureValue: unknown): void {
    if (this.#terminal) return
    this.#terminal = true
    this.#queue.length = 0
    this.#failure = failureValue
    this.#observer?.failure(failureValue)
    this.#detach()
  }

  #detach(): void {
    this.socket.removeEventListener("message", this.#onMessage)
    this.socket.removeEventListener("close", this.#onClose)
    this.socket.removeEventListener("error", this.#onError)
  }
}

export function createWebSocketClientProvider(
  provider: string,
  target: ProviderRuntimeTarget,
  Host: WebSocketHostConstructor | undefined
): ProviderPackageEntry {
  const connections = new Set<ConnectionToken>()
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/websocket::WebSocketClient",
    targets: [target],
    operations: {
      connect(value) {
        const controller = new AbortController()
        const completion = connect(Host, value, controller.signal).then(
          (result) => {
            if (result.kind === "success") {
              connections.add(result.value as ConnectionToken)
            }
            return result
          }
        )
        return withProviderCancellation(completion, () => controller.abort())
      },
      receive(value, observer) {
        const target = observer as ProviderSubscriptionObserver
        if (
          typeof target?.next !== "function" ||
          typeof target.complete !== "function" ||
          typeof target.failure !== "function" ||
          typeof target.defect !== "function"
        ) {
          throw new TypeError("WebSocket receive bridge is invalid")
        }
        return connectionToken(value).subscribe(target)
      },
      send(value) {
        return send(value)
      },
      async closeConnection(value) {
        const request = closeRequest(value)
        request.connection.socket.close(request.code, request.reason)
        return success(undefined)
      },
      async protocol(value) {
        return success(connectionToken(value).socket.protocol)
      },
    },
    shutdown: async () => {
      for (const token of connections) {
        if (token.socket.readyState < 2) token.socket.close(1001, "shutdown")
      }
      connections.clear()
    },
  })
}

async function connect(
  Host: WebSocketHostConstructor | undefined,
  value: unknown,
  signal: AbortSignal
): Promise<ProviderResult> {
  if (Host === undefined) {
    return failure(
      "WebSocketConnectionFailed",
      "host WebSocket client is unavailable"
    )
  }
  const request = connectRequest(value)
  let url: URL
  try {
    url = new URL(request.url)
  } catch {
    return failure("InvalidWebSocketUrl", "WebSocket URL is invalid")
  }
  if (url.protocol !== "ws:" && url.protocol !== "wss:") {
    return failure("InvalidWebSocketUrl", "WebSocket URL must use ws or wss")
  }
  return await new Promise<ProviderResult>((resolve) => {
    let socket: WebSocketHost
    try {
      socket = new Host(url.toString(), [...request.protocols])
      socket.binaryType = "arraybuffer"
    } catch (cause) {
      resolve(failure("WebSocketConnectionFailed", errorMessage(cause)))
      return
    }
    let settled = false
    const cleanup = (): void => {
      signal.removeEventListener("abort", onAbort)
      socket.removeEventListener("open", onOpen)
      socket.removeEventListener("error", onError)
    }
    const finish = (result: ProviderResult): void => {
      if (settled) return
      settled = true
      cleanup()
      resolve(result)
    }
    const onOpen = (): void =>
      finish(success(new ConnectionToken(socket, request.receiveBuffer.value)))
    const onError = (): void =>
      finish(failure("WebSocketConnectionFailed", "WebSocket handshake failed"))
    const onAbort = (): void => {
      socket.close(1000, "cancelled")
      finish(
        failure("WebSocketConnectionFailed", "WebSocket connect was cancelled")
      )
    }
    socket.addEventListener("open", onOpen)
    socket.addEventListener("error", onError)
    signal.addEventListener("abort", onAbort, { once: true })
    if (signal.aborted) onAbort()
  })
}

function send(value: unknown): Promise<ProviderResult> {
  const request = sendRequest(value)
  if (request.connection.socket.readyState !== 1) {
    return Promise.resolve(failure("WebSocketClosed", "WebSocket is not open"))
  }
  if (!request.connection.reserveSend()) {
    return Promise.resolve(
      failure(
        "WebSocketSendFailed",
        "WebSocket send backpressure queue exceeded 16 messages"
      )
    )
  }
  try {
    request.connection.socket.send(
      request.message.tag === "TextMessage"
        ? request.message.text
        : request.message.bytes
    )
  } catch (cause) {
    request.connection.releaseSend()
    return Promise.resolve(failure("WebSocketSendFailed", errorMessage(cause)))
  }
  let timer: ReturnType<typeof setTimeout> | undefined
  let cancel: (() => void) | undefined
  const completion = new Promise<ProviderResult>((resolve) => {
    const poll = (): void => {
      if (request.connection.socket.readyState > 1) {
        resolve(failure("WebSocketClosed", "WebSocket closed while sending"))
      } else if (request.connection.socket.bufferedAmount <= 64 * 1024) {
        resolve(success(undefined))
      } else {
        timer = setTimeout(poll, 1)
      }
    }
    cancel = () => {
      if (timer !== undefined) clearTimeout(timer)
      request.connection.socket.close(1000, "cancelled")
      resolve(failure("WebSocketSendFailed", "WebSocket send was cancelled"))
    }
    poll()
  })
  return withProviderCancellation(
    completion.finally(() => request.connection.releaseSend()),
    () => cancel?.()
  )
}

async function decodeMessage(data: unknown): Promise<unknown> {
  if (typeof data === "string") return { tag: "TextMessage", text: data }
  if (data instanceof ArrayBuffer) {
    return { tag: "BytesMessage", bytes: new Uint8Array(data) }
  }
  if (ArrayBuffer.isView(data)) {
    return {
      tag: "BytesMessage",
      bytes: new Uint8Array(data.buffer, data.byteOffset, data.byteLength),
    }
  }
  if (typeof Blob !== "undefined" && data instanceof Blob) {
    return {
      tag: "BytesMessage",
      bytes: new Uint8Array(await data.arrayBuffer()),
    }
  }
  throw new TypeError("WebSocket host delivered an unsupported message")
}

type ConnectRequest = Readonly<{
  url: string
  protocols: ReadonlyArray<string>
  receiveBuffer: Readonly<{ value: number }>
}>
type SendRequest = Readonly<{
  connection: ConnectionToken
  message:
    | Readonly<{ tag: "TextMessage"; text: string }>
    | Readonly<{ tag: "BytesMessage"; bytes: Uint8Array }>
}>
type CloseRequest = Readonly<{
  connection: ConnectionToken
  code: number
  reason: string
}>

function connectRequest(value: unknown): ConnectRequest {
  const request = value as Partial<ConnectRequest>
  if (
    typeof value !== "object" ||
    value === null ||
    typeof request.url !== "string" ||
    !Array.isArray(request.protocols) ||
    request.protocols.some(
      (protocol) =>
        typeof protocol !== "string" ||
        !/^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/u.test(protocol)
    ) ||
    new Set(request.protocols).size !== request.protocols.length ||
    typeof request.receiveBuffer !== "object" ||
    request.receiveBuffer === null ||
    !Number.isSafeInteger(request.receiveBuffer.value) ||
    request.receiveBuffer.value <= 0
  ) {
    throw new TypeError("WebSocket connect request is invalid")
  }
  return request as ConnectRequest
}

function sendRequest(value: unknown): SendRequest {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("WebSocket send request is invalid")
  }
  const request = value as SendRequest
  connectionToken(request.connection)
  if (
    (request.message?.tag === "TextMessage" &&
      typeof request.message.text === "string") ||
    (request.message?.tag === "BytesMessage" &&
      request.message.bytes instanceof Uint8Array)
  ) {
    return request
  }
  throw new TypeError("WebSocket send message is invalid")
}

function closeRequest(value: unknown): CloseRequest {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("WebSocket close request is invalid")
  }
  const request = value as CloseRequest
  connectionToken(request.connection)
  return request
}

function connectionToken(value: unknown): ConnectionToken {
  if (
    typeof value !== "object" ||
    value === null ||
    !("socket" in value) ||
    typeof (value as ConnectionToken).socket?.send !== "function"
  ) {
    throw new TypeError("WebSocket connection handle is invalid")
  }
  return value as ConnectionToken
}

function success(value: unknown): ProviderResult {
  return { kind: "success", value }
}

function failure(tag: string, message: string): ProviderResult {
  return { kind: "failure", failure: Object.freeze({ tag, message }) }
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : "WebSocket host failure"
}
