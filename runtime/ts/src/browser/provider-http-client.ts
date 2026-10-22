import type { HttpClientRequestBody, HttpClientVersion } from "../http-client"
import {
  type ProviderSubscriptionObserver,
  type ProviderSubscriptionRegistration,
  providerRuntimeAbi,
  withProviderCancellation,
} from "../provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "../provider-package"

type FetchHost = (input: string, init: RequestInit) => Promise<Response>
type StreamingRequestInit = RequestInit & Readonly<{ duplex: "half" }>

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
  let responseVersion: HttpClientVersion = "HttpVersionUnknown"
  let trailersEmitted = false
  let pendingChunk = new Uint8Array()
  let stopping: Promise<void> | undefined
  let readFailure: unknown

  const fail = (cause: unknown): void => {
    if (stopped || terminal) return
    terminal = true
    observer.failure({
      tag: "HttpConnectionFailure",
      value: cause instanceof Error ? cause.message : "HTTP request failed",
    })
  }

  const upload =
    body.knownLength === undefined
      ? new ReadableStream<Uint8Array>({
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
      : undefined

  const ready = (async () => {
    try {
      let content: Uint8Array | ReadableStream<Uint8Array> | undefined = upload
      if (body.knownLength !== undefined && body.knownLength !== 0) {
        // bytesBody is already resident in memory. Pass those known bytes to
        // Fetch directly so small POSTs work without streaming-upload support.
        const bytes = new Uint8Array(body.knownLength)
        let offset = 0
        while (true) {
          const next = await body.pull()
          if (stopped) return
          if (next.done) break
          bytes.set(next.value, offset)
          offset += next.value.length
        }
        if (offset !== bytes.length) {
          throw new TypeError("HTTP request body length is invalid")
        }
        content = bytes
      }
      if (stopped) return
      const init: StreamingRequestInit = {
        method: request.method,
        headers: request.headers.map(
          ({ name, value }) => [name, value] as [string, string]
        ),
        ...(content === undefined ? {} : { body: content }),
        redirect: "manual",
        signal: controller.signal,
        duplex: "half",
      }
      const fetched = (await fetchHost(request.url, init)) as StreamingResponse
      if (stopped) {
        await fetched.body?.cancel()
        return
      }
      response = fetched
      responseVersion = response.httpVersion ?? "HttpVersionUnknown"
      reader = response.body?.getReader()
    } catch (cause) {
      fail(cause)
    }
  })()

  return Object.freeze({
    async demand(count: number) {
      try {
        await ready
        if (stopped || terminal || response === undefined) return
        const activeResponse = response
        for (let remaining = count; remaining > 0; remaining -= 1) {
          if (headPending) {
            headPending = false
            observer.next({
              kind: "ResponseStarted",
              head: {
                version: responseVersion,
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
            if (stopped) return
            if (next === undefined || next.done) break
            pendingChunk = new Uint8Array(next.value)
          }
          if (pendingChunk.length > 0) {
            const chunk = pendingChunk.slice(0, 64 * 1024)
            pendingChunk = pendingChunk.subarray(chunk.length)
            observer.next({
              kind: "ResponseBodyChunk",
              bytes: chunk,
            })
            continue
          }
          const trailers = await activeResponse.trailers
          if (stopped) return
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
      } catch (cause) {
        readFailure = cause
        fail(cause)
      }
    },
    unsubscribe() {
      if (stopping !== undefined) return stopping
      stopped = true
      pendingChunk = new Uint8Array()
      // Close the reader before aborting Fetch: abort would otherwise error an
      // open reader and turn a body-limit failure into a cleanup defect.
      const cancelling = reader?.cancel()
      controller.abort()
      stopping = (async () => {
        try {
          try {
            await cancelling
          } catch (cause) {
            // Cancelling an errored reader repeats its original read failure.
            // Keep that transport failure in its typed channel.
            if (cause !== readFailure) throw cause
          }
        } finally {
          try {
            reader?.releaseLock()
          } finally {
            await body.cancel()
          }
        }
      })()
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
