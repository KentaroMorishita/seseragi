import {
  type EffectContext,
  registerResourceFinalizer,
  type Unit,
} from "./effect"
import {
  invokeProviderOperation,
  openProviderSubscription,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderHandle,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import { type ServiceResult, serviceSuccess } from "./service"
import { fromPull, type PullStreamSource } from "./stream"
import {
  type WebSocketClient,
  type WebSocketConnection,
  type WebSocketConnectOptions,
  type WebSocketError,
  type WebSocketEvent,
  type WebSocketMessage,
  webSocketFailure,
} from "./websocket"

const handleType = Object.freeze({
  kind: "named",
  identity: "std/websocket::ConnectionHandle",
} as const)
const errorType = Object.freeze({
  kind: "named",
  identity: "std/websocket::WebSocketError",
} as const)
const messageType = Object.freeze({
  kind: "named",
  identity: "std/websocket::WebSocketEvent",
} as const)
const unit = Object.freeze({ kind: "unit" } as const)
const never = Object.freeze({ kind: "never" } as const)
const string = Object.freeze({ kind: "primitive", name: "string" } as const)

const connectContract = contract(
  "std/websocket::WebSocketClient#connect",
  "resource",
  "std/websocket::ConnectRequest",
  handleType,
  errorType
)
const codecs = new ProviderCodecRegistry([
  namedCodec("std/websocket::ConnectRequest", validateConnect),
  namedCodec("std/websocket::WebSocketEvent", validateEvent),
  namedCodec("std/websocket::WebSocketError", validateError),
])

export function createProviderWebSocketClient(
  loaded: LoadedProviderEntry
): WebSocketClient {
  if (loaded.service !== "std/websocket::WebSocketClient") {
    throw new TypeError(
      "resolved provider does not implement std/websocket::WebSocketClient"
    )
  }
  return Object.freeze({
    async connect(options: WebSocketConnectOptions, context: EffectContext) {
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: connectContract,
        entry: loaded.entry,
        input: options,
        codecs,
        context,
      })
      if (outcome.kind === "defect") throw outcome.defect
      if (outcome.kind === "failure") {
        return webSocketFailureResult(outcome.failure)
      }
      const handle = outcome.value as ProviderHandle
      const connectionContracts = contractsFor(loaded.service)
      const protocol = await invoke(
        loaded,
        connectionContracts.protocol,
        handle,
        context
      )
      const connection = connectionFor(
        loaded,
        handle,
        options,
        protocol as string
      )
      const registration = registerResourceFinalizer(context, () =>
        connection.close(1000, "").then(() => undefined)
      )
      await registration.ready
      return serviceSuccess(connection)
    },
  })
}

function connectionFor(
  loaded: LoadedProviderEntry,
  handle: ProviderHandle,
  options: WebSocketConnectOptions,
  protocol: string
): WebSocketConnection {
  let closed = false
  const contracts = contractsFor(loaded.service)
  const events = fromPull<unknown, WebSocketError, WebSocketEvent>(
    () =>
      openProviderSubscription({
        provider: loaded.provider,
        service: loaded.service,
        operation: contracts.receive,
        entry: loaded.entry,
        input: handle,
        codecs,
        pushBuffer: {
          capacity: options.receiveBuffer.value,
          overflowFailure: Object.freeze({
            tag: "WebSocketBufferOverflow",
            message: `WebSocket receive buffer exceeded ${options.receiveBuffer.value} messages`,
          }),
        },
      }) as PullStreamSource<WebSocketEvent>
  )
  return Object.freeze({
    handle,
    protocol,
    events,
    async send(message: WebSocketMessage, context: EffectContext) {
      if (closed)
        return webSocketFailure("WebSocketClosed", "WebSocket is closed")
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: contracts.send,
        entry: loaded.entry,
        input: { connection: handle, message },
        codecs,
        context,
      })
      return unitResult(outcome)
    },
    async close(code: number, reason: string, context?: EffectContext) {
      if (closed) return serviceSuccess(undefined)
      validateCloseCode(code, reason)
      closed = true
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: contracts.close,
        entry: loaded.entry,
        input: { connection: handle, code, reason },
        codecs,
        ...(context === undefined ? {} : { context }),
      })
      if (outcome.kind === "defect") throw outcome.defect
      if (outcome.kind === "failure") {
        throw new TypeError("WebSocket close returned an impossible failure")
      }
      return serviceSuccess(undefined)
    },
  })
}

async function invoke(
  loaded: LoadedProviderEntry,
  operation: ProviderOperationContract,
  input: unknown,
  context: EffectContext
): Promise<unknown> {
  const outcome = await invokeProviderOperation({
    provider: loaded.provider,
    service: loaded.service,
    operation,
    entry: loaded.entry,
    input,
    codecs,
    context,
  })
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    throw new TypeError(`${operation.identity} returned an impossible failure`)
  }
  return outcome.value
}

function unitResult(
  outcome: ProviderBridgeOutcome
): ServiceResult<WebSocketError, Unit> {
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    return webSocketFailureResult(outcome.failure)
  }
  return serviceSuccess(undefined)
}

function webSocketFailureResult(
  value: unknown
): ServiceResult<WebSocketError, never> {
  const error = validateError(value)
  return webSocketFailure(error.tag, error.message)
}

function contract(
  identity: string,
  kind: ProviderOperationContract["kind"],
  input: ProviderOperationContract["input"] | string,
  success: ProviderOperationContract["success"],
  failure: ProviderOperationContract["failure"]
): ProviderOperationContract {
  return Object.freeze({
    identity,
    kind,
    input:
      typeof input === "string"
        ? ({ kind: "named", identity: input } as const)
        : input,
    success,
    failure,
  })
}

function namedCodec(identity: string, validate: (value: unknown) => unknown) {
  return { identity, encode: validate, decode: validate }
}

function validateConnect(value: unknown): WebSocketConnectOptions {
  const options = value as Partial<WebSocketConnectOptions>
  if (
    typeof value !== "object" ||
    value === null ||
    typeof options.url !== "string" ||
    !Array.isArray(options.protocols) ||
    options.protocols.some((protocol) => !validProtocol(protocol)) ||
    new Set(options.protocols).size !== options.protocols.length ||
    typeof options.receiveBuffer !== "object" ||
    options.receiveBuffer === null ||
    !Number.isSafeInteger(options.receiveBuffer.value) ||
    options.receiveBuffer.value <= 0
  ) {
    throw new TypeError("WebSocket connect options are invalid")
  }
  return Object.freeze({
    url: options.url,
    protocols: Object.freeze([...options.protocols]),
    receiveBuffer: options.receiveBuffer,
  }) as WebSocketConnectOptions
}

function validateEvent(value: unknown, allowClose = true): WebSocketEvent {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("WebSocket event is invalid")
  }
  const event = value as WebSocketEvent
  if (event.tag === "TextMessage" && typeof event.text === "string") {
    return Object.freeze({ tag: event.tag, text: event.text })
  }
  if (event.tag === "BytesMessage" && event.bytes instanceof Uint8Array) {
    return Object.freeze({ tag: event.tag, bytes: new Uint8Array(event.bytes) })
  }
  if (
    allowClose &&
    event.tag === "RemoteClosed" &&
    Number.isSafeInteger(event.close?.code) &&
    typeof event.close?.reason === "string" &&
    typeof event.close?.wasClean === "boolean"
  ) {
    return Object.freeze({
      tag: event.tag,
      close: Object.freeze({ ...event.close }),
    })
  }
  throw new TypeError("WebSocket event is invalid")
}

function validateError(value: unknown): WebSocketError {
  if (
    typeof value !== "object" ||
    value === null ||
    !isWebSocketErrorTag((value as WebSocketError).tag) ||
    typeof (value as WebSocketError).message !== "string"
  ) {
    throw new TypeError("WebSocket failure is invalid")
  }
  return Object.freeze({
    tag: (value as WebSocketError).tag,
    message: (value as WebSocketError).message,
  })
}

function isWebSocketErrorTag(value: unknown): value is WebSocketError["tag"] {
  return (
    value === "InvalidWebSocketUrl" ||
    value === "WebSocketConnectionFailed" ||
    value === "WebSocketProtocolMismatch" ||
    value === "WebSocketSendFailed" ||
    value === "WebSocketBufferOverflow" ||
    value === "WebSocketClosed"
  )
}

function validProtocol(value: unknown): value is string {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/u.test(value)
  )
}

function validateCloseCode(code: unknown, reason: unknown): void {
  if (
    typeof code !== "number" ||
    !Number.isSafeInteger(code) ||
    (code !== 1000 && (code < 3000 || code > 4999)) ||
    typeof reason !== "string" ||
    new TextEncoder().encode(reason).byteLength > 123
  ) {
    throw new TypeError("WebSocket close code or reason is invalid")
  }
}

export { connectionFor as createProviderWebSocketConnection }

function contractsFor(service: string): Readonly<{
  receive: ProviderOperationContract
  send: ProviderOperationContract
  close: ProviderOperationContract
  protocol: ProviderOperationContract
}> {
  return Object.freeze({
    receive: Object.freeze({
      identity: `${service}#receive`,
      kind: "subscription",
      input: handleType,
      success: messageType,
      failure: errorType,
    }),
    send: contract(
      `${service}#send`,
      "one-shot",
      {
        kind: "record",
        fields: [
          { name: "connection", type: handleType },
          { name: "message", type: messageType },
        ],
      },
      unit,
      errorType
    ),
    close: contract(
      `${service}#closeConnection`,
      "one-shot",
      {
        kind: "record",
        fields: [
          { name: "connection", type: handleType },
          { name: "code", type: { kind: "primitive", name: "int" } },
          { name: "reason", type: string },
        ],
      },
      unit,
      never
    ),
    protocol: Object.freeze({
      identity: `${service}#protocol`,
      kind: "one-shot",
      input: handleType,
      success: string,
      failure: never,
    }),
  })
}
