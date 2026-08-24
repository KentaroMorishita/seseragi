import { type Bytes, fromUint8Array } from "./bytes"
import {
  createEffectExecution,
  type Effect,
  type EffectContext,
  registerResourceFinalizer,
  run,
  type Unit,
} from "./effect"
import type { ProviderHandle } from "./provider"
import {
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"
import type { BufferCapacity, Stream } from "./stream"

export type WebSocketMessage =
  | Readonly<{ tag: "TextMessage"; text: string }>
  | Readonly<{ tag: "BytesMessage"; bytes: Uint8Array }>

export type WebSocketClose = Readonly<{
  code: number
  reason: string
  wasClean: boolean
}>

export type WebSocketEvent =
  | WebSocketMessage
  | Readonly<{ tag: "RemoteClosed"; close: WebSocketClose }>

export type WebSocketError = Readonly<{
  tag:
    | "InvalidWebSocketUrl"
    | "WebSocketConnectionFailed"
    | "WebSocketProtocolMismatch"
    | "WebSocketSendFailed"
    | "WebSocketBufferOverflow"
    | "WebSocketClosed"
  message: string
}>

export type WebSocketConnectOptions = Readonly<{
  url: string
  protocols: ReadonlyArray<string>
  receiveBuffer: BufferCapacity
}>

export type WebSocketConnection = Readonly<{
  handle: ProviderHandle
  protocol: string
  events: Stream<unknown, WebSocketError, WebSocketEvent>
  send: (
    message: WebSocketMessage,
    context: EffectContext
  ) => Promise<ServiceResult<WebSocketError, Unit>>
  close: (
    code: number,
    reason: string,
    context?: EffectContext
  ) => Promise<ServiceResult<WebSocketError, Unit>>
}>

export type WebSocketClient = Readonly<{
  connect: (
    options: WebSocketConnectOptions,
    context: EffectContext
  ) => Promise<ServiceResult<WebSocketError, WebSocketConnection>>
}>

export type WebSocketClientEnvironment = Readonly<{
  webSocketClient: WebSocketClient
}>

export type WebSocketServerOptions<Environment> = Readonly<{
  hostname?: string
  port: number
  path: string
  protocols: ReadonlyArray<string>
  receiveBuffer: BufferCapacity
  handler: (connection: WebSocketConnection) => Effect<Environment, never, Unit>
}>

export type ProviderWebSocketServerOptions = Readonly<{
  hostname?: string
  port: number
  path: string
  protocols: ReadonlyArray<string>
  receiveBuffer: BufferCapacity
  handler: (connection: WebSocketConnection) => Promise<void>
}>

export type WebSocketServerHandle = ProviderHandle

export type WebSocketServer = Readonly<{
  listen: (
    options: ProviderWebSocketServerOptions,
    context: EffectContext
  ) => Promise<ServiceResult<WebSocketError, WebSocketServerHandle>>
  close: (
    server: WebSocketServerHandle,
    context?: EffectContext
  ) => Promise<ServiceResult<never, Unit>>
}>

export type WebSocketServerEnvironment = Readonly<{
  webSocketServer: WebSocketServer
}>

export function TextMessage(text: string): WebSocketMessage {
  return Object.freeze({ tag: "TextMessage", text })
}

export function BytesMessage(bytes: Bytes): WebSocketMessage {
  return Object.freeze({ tag: "BytesMessage", bytes: new Uint8Array(bytes) })
}

export function foldEvent<Value>(
  onText: (text: string) => Value,
  onBytes: (bytes: Bytes) => Value,
  onClose: (close: WebSocketClose) => Value,
  event: WebSocketEvent
): Value {
  switch (event.tag) {
    case "TextMessage":
      return onText(event.text)
    case "BytesMessage":
      return onBytes(fromUint8Array(event.bytes))
    case "RemoteClosed":
      return onClose(event.close)
  }
}

export function closeCode(close: WebSocketClose): number {
  return close.code
}

export function closeReason(close: WebSocketClose): string {
  return close.reason
}

export function closeWasClean(close: WebSocketClose): boolean {
  return close.wasClean
}

export function errorMessage(error: WebSocketError): string {
  return error.message
}

export function selectedProtocol(connection: WebSocketConnection): string {
  return connection.protocol
}

export function messages(
  connection: WebSocketConnection
): Stream<unknown, WebSocketError, WebSocketEvent> {
  return connection.events
}

export function connect(
  options: WebSocketConnectOptions
): Effect<WebSocketClientEnvironment, WebSocketError, WebSocketConnection> {
  return serviceEffect(async (environment, context) => {
    const result = await environment.webSocketClient.connect(options, context)
    if (result.kind === "failure") return result
    const registration = registerResourceFinalizer(context, () =>
      result.value.close(1000, "").then(() => undefined)
    )
    await registration.ready
    return serviceSuccess(result.value)
  })
}

export function sendText(
  text: string,
  connection: WebSocketConnection
): Effect<unknown, WebSocketError, Unit> {
  return serviceEffect((_environment, context) =>
    connection.send(TextMessage(text), context)
  )
}

export function sendBytes(
  bytes: Bytes,
  connection: WebSocketConnection
): Effect<unknown, WebSocketError, Unit> {
  return serviceEffect((_environment, context) =>
    connection.send(BytesMessage(bytes), context)
  )
}

export function closeConnection(
  code: number,
  reason: string,
  connection: WebSocketConnection
): Effect<unknown, WebSocketError, Unit> {
  return serviceEffect((_environment, context) =>
    connection.close(code, reason, context)
  )
}

export function listen<Environment>(
  options: WebSocketServerOptions<Environment>
): Effect<
  Environment & WebSocketServerEnvironment,
  WebSocketError,
  WebSocketServerHandle
> {
  return serviceEffect(async (environment, context) => {
    const result = await environment.webSocketServer.listen(
      {
        ...options,
        async handler(connection) {
          const execution = createEffectExecution(context)
          try {
            const handled = await run(
              options.handler(connection),
              environment,
              execution.context
            )
            if (handled.kind === "failure") {
              throw new TypeError(
                "WebSocket Handler<R, Never> produced an impossible failure"
              )
            }
          } finally {
            await execution.close()
          }
        },
      },
      context
    )
    if (result.kind === "failure") return result
    const registration = registerResourceFinalizer(context, () =>
      environment.webSocketServer.close(result.value).then(() => undefined)
    )
    await registration.ready
    return serviceSuccess(result.value)
  })
}

export function closeServer(
  server: WebSocketServerHandle
): Effect<WebSocketServerEnvironment, never, Unit> {
  return serviceEffect((environment, context) =>
    environment.webSocketServer.close(server, context)
  )
}

export function webSocketFailure(
  tag: WebSocketError["tag"],
  message: string
): ServiceResult<WebSocketError, never> {
  return serviceFailure(Object.freeze({ tag, message }))
}
