import type { HttpClientRequestBody } from "../http-client"
import {
  providerRuntimeAbi,
  type ProviderSubscriptionObserver,
  type ProviderSubscriptionRegistration,
  withProviderCancellation,
} from "../provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "../provider-package"

type FetchHost = (input: string, init: RequestInit) => Promise<Response>

export function createBrowserHttpClientProvider(
  fetchHost: FetchHost = globalThis.fetch
): ProviderPackageEntry {
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-browser#http-client",
    service: "std/http::HttpClient",
    targets: ["browser"],
    operations: {
      send(value) {
        const controller = new AbortController()
        const completion = sendRequest(fetchHost, value, controller.signal)
        return withProviderCancellation(completion, () => controller.abort())
      },
      exchange(value, observer, attachment) {
        return createFetchExchange(fetchHost, value, observer, attachment)
      },
    },
  })
}

type StreamingResponse = Response &
  Readonly<{
    httpVersion?: "Http1_0" | "Http1_1" | "Http2" | "Http3"
    trailers?: Promise<Headers>
  }>

function createFetchExchange(
  fetchHost: FetchHost,
  value: unknown,
  observerValue: unknown,
  attachment: unknown
): ProviderSubscriptionRegistration {
  const request = validateStreamRequest(value)
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
  const controller = new AbortController()
  let stopped = false
  let terminal = false
  let headPending = true
  let reader: ReadableStreamDefaultReader<Uint8Array> | undefined
  let response: StreamingResponse | undefined
  let trailersEmitted = false
  let pendingChunk = new Uint8Array()

  const upload =
    body.knownLength === 0
      ? undefined
      : new ReadableStream<Uint8Array>({
          async pull(streamController) {
            try {
              const next = await body.pull()
              if (next.done) streamController.close()
              else streamController.enqueue(new Uint8Array(next.value))
            } catch (cause) {
              streamController.error(cause)
            }
          },
          cancel() {
            return body.cancel()
          },
        })

  const ready = (async () => {
    try {
      response = (await fetchHost(request.url, {
        method: request.method,
        headers: request.headers.map(
          ({ name, value }) => [name, value] as [string, string]
        ),
        ...(upload === undefined ? {} : { body: upload }),
        redirect: "manual",
        signal: controller.signal,
      })) as StreamingResponse
      if (response.httpVersion === undefined) {
        observer.failure({
          tag: "HttpProtocolFailure",
          value: "browser Fetch does not expose the negotiated HTTP version",
        })
        terminal = true
        return
      }
      reader = response.body?.getReader()
    } catch (cause) {
      if (stopped) return
      terminal = true
      observer.failure({
        tag: "HttpConnectionFailure",
        value: cause instanceof Error ? cause.message : "HTTP request failed",
      })
    }
  })()

  return Object.freeze({
    async demand(count: number) {
      await ready
      if (stopped || terminal || response === undefined) return
      const activeResponse = response
      for (let remaining = count; remaining > 0; remaining -= 1) {
        if (headPending) {
          headPending = false
          observer.next({
            kind: "ResponseStarted",
            head: {
              version: activeResponse.httpVersion,
              status: activeResponse.status,
              headers: [...activeResponse.headers].map(([name, value]) => ({
                name,
                value,
              })),
            },
          })
          continue
        }
        while (pendingChunk.length === 0) {
          const next = await reader?.read()
          if (next === undefined || next.done) break
          pendingChunk = new Uint8Array(next.value)
        }
        if (pendingChunk.length > 0) {
          const chunk = pendingChunk.slice(0, 64 * 1024)
          pendingChunk = pendingChunk.slice(chunk.length)
          observer.next({
            kind: "ResponseBodyChunk",
            bytes: chunk,
          })
          continue
        }
        const trailers = await activeResponse.trailers
        if (
          !trailersEmitted &&
          trailers !== undefined &&
          [...trailers].length > 0
        ) {
          trailersEmitted = true
          observer.next({
            kind: "ResponseTrailers",
            headers: [...trailers].map(([name, value]) => ({ name, value })),
          })
          continue
        }
        terminal = true
        observer.complete()
        return
      }
    },
    async unsubscribe() {
      if (stopped) return
      stopped = true
      controller.abort()
      await reader?.cancel()
      await body.cancel()
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
  return request as Readonly<{
    method: string
    url: string
    headers: ReadonlyArray<{ name: string; value: string }>
  }>
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
  if (typeof value !== "object" || value === null) {
    throw new TypeError("HTTP request is invalid")
  }
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

export const provider = createBrowserHttpClientProvider()
