import * as http from "node:http"
import * as https from "node:https"
import type { HttpClientRequestBody } from "@seseragi/runtime/http-client"
import {
  type ProviderSubscriptionObserver,
  type ProviderSubscriptionRegistration,
  providerRuntimeAbi,
  withProviderCancellation,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "@seseragi/runtime/provider-package"
import { createDecodedHttpExchange } from "./http-content-decoding"
import { createBunHttp1Exchange } from "./http1-stream"

type FetchHost = (input: string, init: RequestInit) => Promise<Response>

export function createFetchHttpClientProvider(
  provider: string,
  target: "bun-process" | "node-process",
  fetchHost: FetchHost = globalThis.fetch
): ProviderPackageEntry {
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/http::HttpClient",
    targets: [target],
    operations: {
      send(value) {
        const controller = new AbortController()
        const completion = sendRequest(fetchHost, value, controller.signal)
        return withProviderCancellation(completion, () => controller.abort())
      },
      exchange(value, observer, attachment) {
        return createDecodedHttpExchange(
          target === "bun-process"
            ? createBunHttp1Exchange
            : createNodeHttpExchange,
          value,
          observer,
          attachment
        )
      },
    },
  })
}

function createNodeHttpExchange(
  value: unknown,
  observerValue: unknown,
  attachment: unknown
): ProviderSubscriptionRegistration {
  const requestValue = validateStreamRequest(value)
  const observer = observerValue as ProviderSubscriptionObserver
  const body = attachment as HttpClientRequestBody
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

  const url = new URL(requestValue.url)
  const transport = url.protocol === "https:" ? https : http
  const queued: unknown[] = []
  let demand = 0
  let response: http.IncomingMessage | undefined
  let responseEnded = false
  let terminal = false
  let stopped = false
  let requestBodyDone = false
  let bodyCancelled = false
  let bodyCancellation: Promise<void> | undefined
  let stopping: Promise<void> | undefined

  const cancelBody = (): Promise<void> => {
    if (bodyCancellation !== undefined) return bodyCancellation
    bodyCancelled = true
    bodyCancellation = Promise.resolve().then(() => body.cancel())
    return bodyCancellation
  }
  const failure = (error: unknown): void => {
    if (terminal || stopped) return
    terminal = true
    observer.failure(httpFailure(error))
    void cancelBody().catch(() => undefined)
  }
  const completeIfDrained = (): void => {
    if (responseEnded && queued.length === 0 && !terminal && !stopped) {
      terminal = true
      observer.complete()
    }
  }
  const emit = (): void => {
    if (terminal || stopped) return
    while (demand > 0 && queued.length > 0) {
      demand -= 1
      observer.next(queued.shift())
    }
    if (demand > 0 && response !== undefined && !responseEnded) {
      const available = Math.min(response.readableLength, 64 * 1024)
      const chunk =
        available > 0
          ? (response.read(available) as Buffer | null)
          : (response.read() as Buffer | null)
      if (chunk !== null && chunk.length > 0) {
        demand -= 1
        observer.next({
          kind: "ResponseBodyChunk",
          bytes: new Uint8Array(chunk),
        })
      }
    }
    completeIfDrained()
  }
  const enqueueHead = (
    kind: "InformationalResponse" | "ResponseStarted",
    version: string,
    status: number,
    rawHeaders: ReadonlyArray<string>
  ): void => {
    queued.push({
      kind,
      head: {
        version: httpVersion(version),
        status,
        headers: rawHeaderEntries(rawHeaders),
      },
    })
    emit()
  }

  const headers = [
    "host",
    url.host,
    ...requestValue.headers
      .filter(({ name }) => name.toLowerCase() !== "host")
      .flatMap(({ name, value }) => [name, value]),
  ]
  const hasFraming = requestValue.headers.some(({ name }) => {
    const normalized = name.toLowerCase()
    return normalized === "content-length" || normalized === "transfer-encoding"
  })
  const knownLength = body.knownLength
  if (
    !hasFraming &&
    typeof knownLength === "number" &&
    Number.isSafeInteger(knownLength) &&
    knownLength >= 0
  ) {
    headers.push("content-length", String(knownLength))
  }

  const request = transport.request(
    url,
    {
      method: requestValue.method,
      // Node does not synthesize Host for the raw header-array form. Keep the
      // generated authority while preserving duplicate application headers.
      headers,
    },
    (incoming) => {
      response = incoming
      enqueueHead(
        "ResponseStarted",
        incoming.httpVersion,
        incoming.statusCode ?? 0,
        incoming.rawHeaders
      )
      if (!requestBodyDone) {
        void cancelBody().catch(observer.defect)
      }
      incoming.on("readable", emit)
      incoming.once("end", () => {
        responseEnded = true
        const trailers = rawHeaderEntries(incoming.rawTrailers)
        if (trailers.length > 0) {
          queued.push({ kind: "ResponseTrailers", headers: trailers })
        }
        emit()
      })
      incoming.once("error", failure)
      emit()
    }
  )
  request.on("information", (information) => {
    enqueueHead(
      "InformationalResponse",
      information.httpVersion,
      information.statusCode,
      information.rawHeaders
    )
  })
  request.once("error", failure)

  void (async () => {
    try {
      while (!stopped && !bodyCancelled) {
        const next = await body.pull()
        if (next.done) {
          requestBodyDone = true
          request.end()
          return
        }
        const chunk = new Uint8Array(next.value)
        if (chunk.length === 0) continue
        if (!request.write(chunk)) {
          await new Promise<void>((resolve, reject) => {
            request.once("drain", resolve)
            request.once("error", reject)
          })
        }
      }
      if (!requestBodyDone && !request.writableEnded) request.end()
    } catch (cause) {
      if (stopped || bodyCancelled) return
      failure({
        tag: "HttpRequestBodyFailure",
        value: cause instanceof Error ? cause.message : "request body failed",
      })
      request.destroy()
    }
  })()

  return Object.freeze({
    demand(count: number) {
      demand += count
      emit()
    },
    unsubscribe() {
      if (stopping !== undefined) return stopping
      stopped = true
      // Do not keep a peer alive while an asynchronous upload finalizer runs.
      response?.destroy()
      request.destroy()
      queued.length = 0
      stopping = cancelBody().finally(() => {
        response?.destroy()
        request.destroy()
        queued.length = 0
      })
      return stopping
    },
  })
}

function validateStreamRequest(value: unknown) {
  if (typeof value !== "object" || value === null) {
    throw new TypeError("HTTP stream request is invalid")
  }
  const request = value as {
    method?: unknown
    url?: unknown
    headers?: unknown
  }
  if (
    typeof request.method !== "string" ||
    request.method.length === 0 ||
    typeof request.url !== "string" ||
    !Array.isArray(request.headers)
  ) {
    throw new TypeError("HTTP stream request is invalid")
  }
  for (const header of request.headers) {
    if (
      typeof header !== "object" ||
      header === null ||
      typeof header.name !== "string" ||
      typeof header.value !== "string"
    ) {
      throw new TypeError("HTTP request header is invalid")
    }
  }
  return request as Readonly<{
    method: string
    url: string
    headers: ReadonlyArray<{ name: string; value: string }>
  }>
}

function httpVersion(version: string) {
  switch (version) {
    case "1.0":
      return "Http1_0"
    case "1.1":
      return "Http1_1"
    case "2":
    case "2.0":
      return "Http2"
    case "3":
    case "3.0":
      return "Http3"
    default:
      throw new TypeError(`unsupported HTTP version: ${version}`)
  }
}

function rawHeaderEntries(raw: ReadonlyArray<string>) {
  const headers: Array<Readonly<{ name: string; value: string }>> = []
  for (let index = 0; index < raw.length; index += 2) {
    headers.push(
      Object.freeze({
        name: (raw[index] ?? "").toLowerCase(),
        value: raw[index + 1] ?? "",
      })
    )
  }
  return Object.freeze(headers)
}

function httpFailure(cause: unknown) {
  if (
    typeof cause === "object" &&
    cause !== null &&
    "tag" in cause &&
    typeof cause.tag === "string"
  ) {
    return cause
  }
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

async function sendRequest(
  fetchHost: FetchHost,
  value: unknown,
  signal: AbortSignal
) {
  const request = validateRequest(value)
  try {
    const response = await fetchHost(request.url, {
      method: request.method,
      headers: request.headers.map(
        ({ name, value }) => [name, value] as [string, string]
      ),
      ...(request.body.length === 0
        ? {}
        : { body: new Uint8Array(request.body) }),
      redirect: "manual",
      signal,
    })
    return {
      kind: "success" as const,
      value: Object.freeze({
        status: response.status,
        headers: Object.freeze(
          [...response.headers].map(([name, value]) =>
            Object.freeze({ name, value })
          )
        ),
        body: new Uint8Array(await response.arrayBuffer()),
      }),
    }
  } catch (cause) {
    if (signal.aborted) throw cause
    return {
      kind: "failure" as const,
      failure: Object.freeze({
        tag: "HttpConnectionFailure",
        value: cause instanceof Error ? cause.message : "HTTP request failed",
      }),
    }
  }
}

function validateRequest(value: unknown) {
  if (typeof value !== "object" || value === null)
    throw new TypeError("HTTP request is invalid")
  const request = value as {
    method?: unknown
    url?: unknown
    headers?: unknown
    body?: unknown
  }
  if (
    typeof request.method !== "string" ||
    request.method.length === 0 ||
    typeof request.url !== "string" ||
    !Array.isArray(request.headers) ||
    !(request.body instanceof Uint8Array)
  ) {
    throw new TypeError("HTTP request is invalid")
  }
  for (const header of request.headers) {
    if (
      typeof header !== "object" ||
      header === null ||
      typeof header.name !== "string" ||
      typeof header.value !== "string"
    ) {
      throw new TypeError("HTTP request header is invalid")
    }
  }
  return request as Readonly<{
    method: string
    url: string
    headers: ReadonlyArray<{ name: string; value: string }>
    body: Uint8Array
  }>
}
