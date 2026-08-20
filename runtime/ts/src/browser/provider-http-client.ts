import { providerRuntimeAbi, withProviderCancellation } from "../provider"
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
    },
  })
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
