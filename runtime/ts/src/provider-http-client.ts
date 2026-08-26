import type { EffectContext } from "./effect"
import {
  type HttpClient,
  type HttpClientError,
  type HttpClientEvent,
  type HttpClientRequest,
  type HttpClientRequestBody,
  type HttpClientResponse,
  httpClientFailure,
  httpClientSuccess,
} from "./http-client"
import {
  invokeProviderOperation,
  openProviderSubscription,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"

const requestType = Object.freeze({
  kind: "named",
  identity: "std/http::ClientRequest",
} as const)
const responseType = Object.freeze({
  kind: "named",
  identity: "std/http::ClientResponse",
} as const)
const errorType = Object.freeze({
  kind: "named",
  identity: "std/http::HttpError",
} as const)
const streamRequestType = Object.freeze({
  kind: "named",
  identity: "std/http::ClientStreamRequest",
} as const)
const eventType = Object.freeze({
  kind: "named",
  identity: "std/http::HttpEvent",
} as const)
const sendContract: ProviderOperationContract = Object.freeze({
  identity: "std/http::HttpClient#send",
  kind: "one-shot",
  input: requestType,
  success: responseType,
  failure: errorType,
})
const exchangeContract: ProviderOperationContract = Object.freeze({
  identity: "std/http::HttpClient#exchange",
  kind: "subscription",
  input: streamRequestType,
  success: eventType,
  failure: errorType,
})
const codecs = new ProviderCodecRegistry([
  { identity: requestType.identity, encode: snapshotRequest, decode: (v) => v },
  {
    identity: responseType.identity,
    encode: (v) => v,
    decode: snapshotResponse,
  },
  { identity: errorType.identity, encode: (v) => v, decode: decodeError },
  {
    identity: streamRequestType.identity,
    encode: snapshotStreamRequest,
    decode: (value) => value,
  },
  {
    identity: eventType.identity,
    encode: (value) => value,
    decode: snapshotEvent,
  },
])

export function createProviderHttpClient(
  loaded: LoadedProviderEntry
): HttpClient {
  if (loaded.service !== "std/http::HttpClient") {
    throw new TypeError(
      "resolved provider does not implement std/http::HttpClient"
    )
  }
  return Object.freeze({
    async send(request: HttpClientRequest, context: EffectContext) {
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: sendContract,
        entry: loaded.entry,
        input: request,
        codecs,
        context,
      })
      if (outcome.kind === "defect") throw outcome.defect
      return outcome.kind === "failure"
        ? httpClientFailure(outcome.failure as HttpClientError)
        : httpClientSuccess(outcome.value as HttpClientResponse)
    },
    exchange(
      request: Omit<HttpClientRequest, "body">,
      body: HttpClientRequestBody,
      context: EffectContext
    ) {
      const source = openProviderSubscription({
        provider: loaded.provider,
        service: loaded.service,
        operation: exchangeContract,
        entry: loaded.entry,
        input: request,
        codecs,
        context,
        attachment: body,
      })
      return Object.freeze({
        async pull(pullContext: EffectContext) {
          return (await source.pull(
            pullContext
          )) as IteratorResult<HttpClientEvent>
        },
        close: source.close,
      })
    },
  })
}

function snapshotRequest(value: unknown): HttpClientRequest {
  return snapshotMessage(value, "request") as HttpClientRequest
}
function snapshotStreamRequest(
  value: unknown
): Omit<HttpClientRequest, "body"> {
  return snapshotMessage(value, "stream-request") as Omit<
    HttpClientRequest,
    "body"
  >
}
function snapshotResponse(value: unknown): HttpClientResponse {
  return snapshotMessage(value, "response") as HttpClientResponse
}
function snapshotMessage(
  value: unknown,
  messageKind: "request" | "stream-request" | "response"
) {
  const request = messageKind !== "response"
  const streamRequest = messageKind === "stream-request"
  const message = dataRecord(
    value,
    request
      ? streamRequest
        ? ["headers", "method", "url"]
        : ["body", "headers", "method", "url"]
      : ["body", "headers", "status"]
  )
  const discriminator = request ? message.method : message.status
  if (
    (request
      ? typeof discriminator !== "string" || discriminator.length === 0
      : !Number.isSafeInteger(discriminator) ||
        (discriminator as number) < 100 ||
        (discriminator as number) > 999) ||
    (request &&
      (typeof message.url !== "string" || message.url.length === 0)) ||
    !Array.isArray(message.headers) ||
    (!streamRequest && !(message.body instanceof Uint8Array))
  )
    throw new TypeError("HTTP message is invalid")
  const headers = message.headers.map((header) => {
    const item = dataRecord(header, ["name", "value"])
    const { name, value } = item
    if (
      typeof name !== "string" ||
      typeof value !== "string" ||
      name.length === 0 ||
      name.includes("\r") ||
      name.includes("\n") ||
      value.includes("\r") ||
      value.includes("\n")
    )
      throw new TypeError("HTTP header is invalid")
    return Object.freeze({ name: name.toLowerCase(), value })
  })
  return Object.freeze({
    ...(request
      ? { method: discriminator, url: message.url }
      : { status: discriminator }),
    headers: Object.freeze(headers),
    ...(streamRequest
      ? {}
      : { body: new Uint8Array(message.body as Uint8Array) }),
  })
}
function snapshotEvent(value: unknown): HttpClientEvent {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("HTTP event is invalid")
  }
  const kind = (value as { kind?: unknown }).kind
  if (kind === "ResponseBodyChunk") {
    const event = dataRecord(value, ["bytes", "kind"])
    if (!(event.bytes instanceof Uint8Array) || event.bytes.length === 0) {
      throw new TypeError("HTTP response body chunk is invalid")
    }
    return Object.freeze({
      kind,
      bytes: new Uint8Array(event.bytes),
    })
  }
  if (kind === "ResponseTrailers") {
    const event = dataRecord(value, ["headers", "kind"])
    return Object.freeze({ kind, headers: snapshotHeaders(event.headers) })
  }
  if (kind === "InformationalResponse" || kind === "ResponseStarted") {
    const event = dataRecord(value, ["head", "kind"])
    const head = dataRecord(event.head, ["headers", "status", "version"])
    if (
      !["HttpVersionUnknown", "Http1_0", "Http1_1", "Http2", "Http3"].includes(
        head.version as string
      ) ||
      !Number.isSafeInteger(head.status) ||
      (head.status as number) < 100 ||
      (head.status as number) > 999
    ) {
      throw new TypeError("HTTP response head is invalid")
    }
    return Object.freeze({
      kind,
      head: Object.freeze({
        version: head.version,
        status: head.status,
        headers: snapshotHeaders(head.headers),
      }),
    }) as HttpClientEvent
  }
  throw new TypeError("HTTP event kind is invalid")
}

function snapshotHeaders(
  value: unknown
): ReadonlyArray<{ name: string; value: string }> {
  if (!Array.isArray(value)) throw new TypeError("HTTP headers are invalid")
  return Object.freeze(
    value.map((header) => {
      const item = dataRecord(header, ["name", "value"])
      if (
        typeof item.name !== "string" ||
        typeof item.value !== "string" ||
        item.name.length === 0 ||
        item.name.includes("\r") ||
        item.name.includes("\n") ||
        item.value.includes("\r") ||
        item.value.includes("\n")
      ) {
        throw new TypeError("HTTP header is invalid")
      }
      return Object.freeze({
        name: item.name.toLowerCase(),
        value: item.value,
      })
    })
  )
}
function decodeError(value: unknown): HttpClientError {
  if (
    typeof value === "object" &&
    value !== null &&
    (value as { tag?: unknown }).tag === "HttpClientUnavailable"
  ) {
    dataRecord(value, ["tag"])
    return Object.freeze({ tag: "HttpClientUnavailable" })
  }
  const error = dataRecord(value, ["tag", "value"])
  if (
    ![
      "HttpDnsFailure",
      "HttpConnectionFailure",
      "HttpTlsFailure",
      "HttpProtocolFailure",
      "HttpRequestBodyFailure",
    ].includes(error.tag as string) ||
    typeof error.value !== "string"
  ) {
    if (
      error.tag === "HttpRequestLengthMismatch" &&
      typeof error.value === "object" &&
      error.value !== null
    ) {
      const detail = dataRecord(error.value, ["actual", "declared"])
      if (
        Number.isSafeInteger(detail.actual) &&
        Number.isSafeInteger(detail.declared)
      ) {
        return Object.freeze({
          tag: error.tag,
          value: detail,
        }) as HttpClientError
      }
    }
    if (
      error.tag === "HttpResponseBodyLimitExceeded" &&
      typeof error.value === "object" &&
      error.value !== null
    ) {
      const detail = dataRecord(error.value, ["limitBytes"])
      if (Number.isSafeInteger(detail.limitBytes)) {
        return Object.freeze({
          tag: error.tag,
          value: detail,
        }) as HttpClientError
      }
    }
    throw new TypeError("HTTP client failure is invalid")
  }
  return Object.freeze({
    tag: error.tag,
    value: error.value,
  }) as HttpClientError
}

function dataRecord(
  value: unknown,
  keys: ReadonlyArray<string>
): Record<string, unknown> {
  if (
    typeof value !== "object" ||
    value === null ||
    ![Object.prototype, null].includes(Object.getPrototypeOf(value))
  ) {
    throw new TypeError("HTTP boundary value must be a plain record")
  }
  const actual = Reflect.ownKeys(value)
  if (
    actual.length !== keys.length ||
    actual.some((key) => typeof key !== "string" || !keys.includes(key))
  ) {
    throw new TypeError("HTTP boundary record shape is invalid")
  }
  const record: Record<string, unknown> = {}
  for (const key of keys) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (
      descriptor === undefined ||
      !("value" in descriptor) ||
      !descriptor.enumerable
    ) {
      throw new TypeError("HTTP boundary fields must be enumerable data values")
    }
    record[key] = descriptor.value
  }
  return record
}
