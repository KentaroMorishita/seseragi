import type { EffectContext } from "./effect"
import {
  type HttpClient,
  type HttpClientError,
  type HttpClientRequest,
  type HttpClientResponse,
  httpClientFailure,
  httpClientSuccess,
} from "./http-client"
import {
  invokeProviderOperation,
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
const sendContract: ProviderOperationContract = Object.freeze({
  identity: "std/http::HttpClient#send",
  kind: "one-shot",
  input: requestType,
  success: responseType,
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
  })
}

function snapshotRequest(value: unknown): HttpClientRequest {
  return snapshotMessage(value, true) as HttpClientRequest
}
function snapshotResponse(value: unknown): HttpClientResponse {
  return snapshotMessage(value, false) as HttpClientResponse
}
function snapshotMessage(value: unknown, request: boolean) {
  const message = dataRecord(
    value,
    request
      ? ["body", "headers", "method", "url"]
      : ["body", "headers", "status"]
  )
  const discriminator = request ? message.method : message.status
  if (
    (request
      ? typeof discriminator !== "string" || discriminator.length === 0
      : !Number.isSafeInteger(discriminator) ||
        (discriminator as number) < 100 ||
        (discriminator as number) > 599) ||
    (request &&
      (typeof message.url !== "string" || message.url.length === 0)) ||
    !Array.isArray(message.headers) ||
    !(message.body instanceof Uint8Array)
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
    body: new Uint8Array(message.body),
  })
}
function decodeError(value: unknown): HttpClientError {
  const error = dataRecord(value, ["message", "tag"])
  if (error.tag !== "HttpRequestFailed" || typeof error.message !== "string") {
    throw new TypeError("HTTP client failure is invalid")
  }
  return Object.freeze({
    tag: "HttpRequestFailed",
    message: error.message,
  })
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
