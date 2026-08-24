import type {
  ProviderResult,
  ProviderSubscriptionObserver,
  ProviderSubscriptionRegistration,
} from "@seseragi/runtime/provider"

type Event =
  | Readonly<{ tag: "TextMessage"; text: string }>
  | Readonly<{ tag: "BytesMessage"; bytes: Uint8Array }>
  | Readonly<{
      tag: "RemoteClosed"
      close: Readonly<{ code: number; reason: string; wasClean: boolean }>
    }>

export type ServerConnectionHost = Readonly<{
  protocol: string
  write: (message: Exclude<Event, { tag: "RemoteClosed" }>) => Promise<void>
  close: (code: number, reason: string) => void
}>

export class ServerConnectionToken {
  readonly #host: ServerConnectionHost
  readonly #capacity: number
  readonly #queued: Event[] = []
  #observer: ProviderSubscriptionObserver | undefined
  #overflow = false
  #terminal = false
  #pendingSends = 0
  #sendTail = Promise.resolve()

  constructor(host: ServerConnectionHost, capacity: number) {
    this.#host = host
    this.#capacity = capacity
  }

  get protocol(): string {
    return this.#host.protocol
  }

  emit(event: Event): void {
    if (this.#terminal) return
    if (this.#observer !== undefined) {
      this.#observer.next(event)
      if (event.tag === "RemoteClosed") {
        this.#terminal = true
        this.#observer.complete()
      }
      return
    }
    if (this.#queued.length >= this.#capacity) {
      this.#overflow = true
      this.#terminal = true
      this.#host.close(1008, "receive buffer overflow")
      return
    }
    this.#queued.push(event)
    if (event.tag === "RemoteClosed") this.#terminal = true
  }

  subscribe(
    observer: ProviderSubscriptionObserver
  ): ProviderSubscriptionRegistration {
    if (this.#observer !== undefined) {
      throw new TypeError("WebSocket connection already has an active receiver")
    }
    this.#observer = observer
    if (this.#overflow) {
      observer.failure({
        tag: "WebSocketBufferOverflow",
        message: `WebSocket receive buffer exceeded ${this.#capacity} messages`,
      })
    } else {
      for (const event of this.#queued.splice(0)) observer.next(event)
      if (this.#terminal) observer.complete()
    }
    return Object.freeze({
      demand() {},
      unsubscribe: () => {
        if (this.#observer === observer) this.#observer = undefined
      },
    })
  }

  send(
    message: Exclude<Event, { tag: "RemoteClosed" }>
  ): Promise<ProviderResult> {
    if (this.#terminal) {
      return Promise.resolve(failure("WebSocketClosed", "WebSocket is closed"))
    }
    if (this.#pendingSends >= 16) {
      return Promise.resolve(
        failure(
          "WebSocketSendFailed",
          "WebSocket send backpressure queue exceeded 16 messages"
        )
      )
    }
    this.#pendingSends += 1
    const completion = this.#sendTail.then(() => this.#host.write(message))
    this.#sendTail = completion.catch(() => undefined)
    return completion
      .then(
        () => success(undefined),
        (cause) => failure("WebSocketSendFailed", errorMessage(cause))
      )
      .finally(() => {
        this.#pendingSends -= 1
      })
  }

  close(code: number, reason: string): void {
    if (this.#terminal) return
    this.#host.close(code, reason)
  }
}

export const connectionOperations = Object.freeze({
  receive(value: unknown, observerValue: unknown) {
    const observer = observerValue as ProviderSubscriptionObserver
    if (
      typeof observer?.next !== "function" ||
      typeof observer.complete !== "function" ||
      typeof observer.failure !== "function" ||
      typeof observer.defect !== "function"
    ) {
      throw new TypeError("WebSocket receive bridge is invalid")
    }
    return connection(value).subscribe(observer)
  },
  send(value: unknown) {
    const request = value as {
      connection?: unknown
      message?: Exclude<Event, { tag: "RemoteClosed" }>
    }
    const token = connection(request.connection)
    if (
      request.message?.tag !== "TextMessage" &&
      request.message?.tag !== "BytesMessage"
    ) {
      throw new TypeError("WebSocket send message is invalid")
    }
    return token.send(request.message)
  },
  async closeConnection(value: unknown) {
    const request = value as {
      connection?: unknown
      code?: unknown
      reason?: unknown
    }
    const token = connection(request.connection)
    if (
      typeof request.code !== "number" ||
      typeof request.reason !== "string"
    ) {
      throw new TypeError("WebSocket close request is invalid")
    }
    token.close(request.code, request.reason)
    return success(undefined)
  },
  async protocol(value: unknown) {
    return success(connection(value).protocol)
  },
})

export function listenRequest(value: unknown): Readonly<{
  hostname?: string
  port: number
  path: string
  protocols: ReadonlyArray<string>
  receiveBuffer: Readonly<{ value: number }>
  handler: (attachment: unknown) => Promise<void>
}> {
  if (
    typeof value !== "object" ||
    value === null ||
    typeof (value as { handler?: unknown }).handler !== "function"
  ) {
    throw new TypeError("WebSocket listen request is invalid")
  }
  return value as ReturnType<typeof listenRequest>
}

export function success(value: unknown): ProviderResult {
  return { kind: "success", value }
}

export function unavailable(cause: unknown): ProviderResult {
  return failure("WebSocketConnectionFailed", errorMessage(cause))
}

function connection(value: unknown): ServerConnectionToken {
  if (!(value instanceof ServerConnectionToken)) {
    throw new TypeError("WebSocket server connection handle is invalid")
  }
  return value
}

function failure(tag: string, message: string): ProviderResult {
  return { kind: "failure", failure: Object.freeze({ tag, message }) }
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : "WebSocket host failure"
}
