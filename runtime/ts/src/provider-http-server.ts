import {
  type EffectContext,
  registerResourceFinalizer,
  throwIfCancelled,
  type Unit,
} from "./effect"
import {
  type HttpServer,
  type HttpServerError,
  type HttpServerHandle,
  type HttpServerOptions,
  httpServerFailure,
  httpServerSuccess,
} from "./http-server"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import type { ServiceResult } from "./service"

const listenRequest = Object.freeze({
  kind: "named",
  identity: "std/http/server::ListenRequest",
} as const)
const serverHandle = Object.freeze({
  kind: "named",
  identity: "std/http/server::ServerHandle",
} as const)
const serverError = Object.freeze({
  kind: "named",
  identity: "std/http/server::ServerError",
} as const)
const unit = Object.freeze({ kind: "unit" } as const)
const never = Object.freeze({ kind: "never" } as const)

const listenContract: ProviderOperationContract = Object.freeze({
  identity: "std/http/server::HttpServer#listen",
  kind: "resource",
  input: listenRequest,
  success: serverHandle,
  failure: serverError,
})

const closeContract: ProviderOperationContract = Object.freeze({
  identity: "std/http/server::HttpServer#close",
  kind: "one-shot",
  input: serverHandle,
  success: unit,
  failure: never,
})

const codecs = new ProviderCodecRegistry([
  {
    identity: listenRequest.identity,
    encode: validateOptions,
    decode: (value) => value,
  },
  {
    identity: serverError.identity,
    encode: (value) => value,
    decode: decodeServerError,
  },
])

export function createProviderHttpServer(
  loaded: LoadedProviderEntry
): HttpServer {
  if (loaded.service !== "std/http/server::HttpServer") {
    throw new TypeError(
      "resolved provider does not implement std/http/server::HttpServer"
    )
  }
  type ServerState = {
    readonly handle: HttpServerHandle
    unregisterCleanup: () => void
    closeCompletion?: Promise<ServiceResult<never, Unit>>
  }
  const servers = new WeakMap<object, ServerState>()
  const closeState = (
    state: ServerState
  ): Promise<ServiceResult<never, Unit>> => {
    state.unregisterCleanup()
    state.closeCompletion ??= (async () =>
      closeResult(
        await invokeProviderOperation({
          provider: loaded.provider,
          service: loaded.service,
          operation: closeContract,
          entry: loaded.entry,
          input: state.handle,
          codecs,
        })
      ))()
    return state.closeCompletion
  }
  return Object.freeze({
    async listen(options: HttpServerOptions, context: EffectContext) {
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: listenContract,
        entry: loaded.entry,
        input: options,
        codecs,
        context,
      })
      if (outcome.kind === "defect") throw outcome.defect
      if (outcome.kind === "failure") {
        return httpServerFailure(outcome.failure as HttpServerError)
      }
      const handle = outcome.value as HttpServerHandle
      const state: ServerState = {
        handle,
        unregisterCleanup: () => undefined,
      }
      servers.set(handle, state)
      const registration = registerResourceFinalizer(context, () =>
        closeState(state).then(() => undefined)
      )
      state.unregisterCleanup = registration.unregister
      await registration.ready
      throwIfCancelled(context)
      return httpServerSuccess(handle)
    },
    async close(server: HttpServerHandle, context: EffectContext) {
      const state = servers.get(server)
      if (state !== undefined) return closeState(state)
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: closeContract,
        entry: loaded.entry,
        input: server,
        codecs,
        context,
      })
      return closeResult(outcome)
    },
  })
}

function closeResult(outcome: ProviderBridgeOutcome) {
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    throw new TypeError("HttpServer close returned an impossible failure")
  }
  return httpServerSuccess(outcome.value as Unit)
}

function validateOptions(value: unknown): HttpServerOptions {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("HTTP server options must be an object")
  }
  const options = value as HttpServerOptions
  if (
    !Number.isSafeInteger(options.port) ||
    options.port < 0 ||
    options.port > 65_535 ||
    typeof options.handler !== "function" ||
    (options.hostname !== undefined &&
      (typeof options.hostname !== "string" ||
        options.hostname.length === 0 ||
        options.hostname.trim() !== options.hostname))
  ) {
    throw new TypeError("HTTP server options are invalid")
  }
  return Object.freeze({
    port: options.port,
    handler: options.handler,
    ...(options.hostname === undefined ? {} : { hostname: options.hostname }),
  })
}

function decodeServerError(value: unknown): HttpServerError {
  if (
    typeof value !== "object" ||
    value === null ||
    (value as { tag?: unknown }).tag !== "HttpServerUnavailable" ||
    typeof (value as { message?: unknown }).message !== "string"
  ) {
    throw new TypeError("HTTP server failure is invalid")
  }
  return Object.freeze({
    tag: "HttpServerUnavailable",
    message: (value as { message: string }).message,
  })
}
