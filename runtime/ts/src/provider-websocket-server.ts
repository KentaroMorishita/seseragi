import {
  type EffectContext,
  registerResourceFinalizer,
  type Unit,
} from "./effect"
import {
  adoptProviderHandle,
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import { createProviderWebSocketConnection } from "./provider-websocket"
import { type ServiceResult, serviceSuccess } from "./service"
import {
  type ProviderWebSocketServerOptions,
  type WebSocketError,
  type WebSocketServer,
  type WebSocketServerHandle,
  webSocketFailure,
} from "./websocket"

const serverHandleType = Object.freeze({
  kind: "named",
  identity: "std/websocket/server::ServerHandle",
} as const)
const errorType = Object.freeze({
  kind: "named",
  identity: "std/websocket::WebSocketError",
} as const)
const unit = Object.freeze({ kind: "unit" } as const)
const never = Object.freeze({ kind: "never" } as const)

const listenContract: ProviderOperationContract = Object.freeze({
  identity: "std/websocket/server::WebSocketServer#listen",
  kind: "resource",
  input: {
    kind: "named" as const,
    identity: "std/websocket/server::ListenRequest",
  },
  success: serverHandleType,
  failure: errorType,
})
const closeContract: ProviderOperationContract = Object.freeze({
  identity: "std/websocket/server::WebSocketServer#closeServer",
  kind: "one-shot",
  input: serverHandleType,
  success: unit,
  failure: never,
})

type ConnectionAttachment = Readonly<{
  token: object
  protocol: string
}>

export function createProviderWebSocketServer(
  loaded: LoadedProviderEntry
): WebSocketServer {
  if (loaded.service !== "std/websocket/server::WebSocketServer") {
    throw new TypeError(
      "resolved provider does not implement std/websocket/server::WebSocketServer"
    )
  }
  const closeStates = new WeakMap<object, Promise<ServiceResult<never, Unit>>>()
  const closeServer = (
    handle: WebSocketServerHandle,
    context?: EffectContext
  ): Promise<ServiceResult<never, Unit>> => {
    const current = closeStates.get(handle)
    if (current !== undefined) return current
    const closing = closeResult(
      invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: closeContract,
        entry: loaded.entry,
        input: handle,
        codecs: codecsFor(loaded),
        ...(context === undefined ? {} : { context }),
      })
    )
    closeStates.set(handle, closing)
    return closing
  }
  return Object.freeze({
    async listen(
      options: ProviderWebSocketServerOptions,
      context: EffectContext
    ) {
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: listenContract,
        entry: loaded.entry,
        input: options,
        codecs: codecsFor(loaded),
        context,
      })
      if (outcome.kind === "defect") throw outcome.defect
      if (outcome.kind === "failure") {
        const error = validateError(outcome.failure)
        return webSocketFailure(error.tag, error.message)
      }
      const handle = outcome.value as WebSocketServerHandle
      const registration = registerResourceFinalizer(context, () =>
        closeServer(handle).then(() => undefined)
      )
      await registration.ready
      return serviceSuccess(handle)
    },
    close(server: WebSocketServerHandle, context?: EffectContext) {
      return closeServer(server, context)
    },
  })
}

function codecsFor(loaded: LoadedProviderEntry): ProviderCodecRegistry {
  return new ProviderCodecRegistry([
    {
      identity: "std/websocket/server::ListenRequest",
      encode(value) {
        return encodeListenRequest(loaded, value)
      },
      decode: (value) => value,
    },
    {
      identity: "std/websocket::WebSocketError",
      encode: (value) => value,
      decode: validateError,
    },
  ])
}

function encodeListenRequest(
  loaded: LoadedProviderEntry,
  value: unknown
): unknown {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("WebSocket server options are invalid")
  }
  const options = value as ProviderWebSocketServerOptions
  if (
    !Number.isSafeInteger(options.port) ||
    options.port < 0 ||
    options.port > 65_535 ||
    typeof options.path !== "string" ||
    !options.path.startsWith("/") ||
    options.path.includes("?") ||
    typeof options.handler !== "function" ||
    !Array.isArray(options.protocols) ||
    options.protocols.some((protocol) => !validProtocol(protocol)) ||
    new Set(options.protocols).size !== options.protocols.length ||
    typeof options.receiveBuffer !== "object" ||
    options.receiveBuffer === null ||
    !Number.isSafeInteger(options.receiveBuffer.value) ||
    options.receiveBuffer.value <= 0
  ) {
    throw new TypeError("WebSocket server options are invalid")
  }
  return Object.freeze({
    port: options.port,
    path: options.path,
    protocols: Object.freeze([...options.protocols]),
    receiveBuffer: options.receiveBuffer,
    ...(options.hostname === undefined ? {} : { hostname: options.hostname }),
    async handler(value: unknown) {
      const attachment = connectionAttachment(value)
      const handle = adoptProviderHandle(
        attachment.token,
        { provider: loaded.provider, service: loaded.service },
        "std/websocket::ConnectionHandle"
      )
      const connection = createProviderWebSocketConnection(
        loaded,
        handle,
        {
          url: "",
          protocols: options.protocols,
          receiveBuffer: options.receiveBuffer,
        },
        attachment.protocol
      )
      await options.handler(connection)
    },
  })
}

function connectionAttachment(value: unknown): ConnectionAttachment {
  if (
    typeof value !== "object" ||
    value === null ||
    typeof (value as ConnectionAttachment).token !== "object" ||
    (value as ConnectionAttachment).token === null ||
    typeof (value as ConnectionAttachment).protocol !== "string"
  ) {
    throw new TypeError("WebSocket server connection attachment is invalid")
  }
  return value as ConnectionAttachment
}

async function closeResult(
  completion: Promise<ProviderBridgeOutcome>
): Promise<ServiceResult<never, Unit>> {
  const outcome = await completion
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    throw new TypeError("WebSocket server close returned an impossible failure")
  }
  return serviceSuccess(undefined)
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
