import {
  isCancelledHttpServerResponse,
  type ProviderHttpServerResponse,
  type ProviderHttpServerStreamBody,
} from "@seseragi/runtime/http-server"
import {
  type ProviderResult,
  providerRuntimeAbi,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "@seseragi/runtime/provider-package"

type AbiHeader = Readonly<{ name: string; value: string }>
type AbiRequest = Readonly<{
  method: string
  url: string
  headers: ReadonlyArray<AbiHeader>
  body: Uint8Array
}>
type AbiResponse = ProviderHttpServerResponse
type ListenRequest = Readonly<{
  hostname?: string
  port: number
  handler: (request: AbiRequest) => Promise<AbiResponse>
}>
type BunServer = Readonly<{
  port: number
  stop: (closeActiveConnections?: boolean) => void | Promise<void>
}>
type BunHost = Readonly<{
  serve: (
    options: Readonly<{
      hostname?: string
      port: number
      fetch: (request: Request) => Promise<Response>
    }>
  ) => BunServer
}>
type ServerToken = {
  server: BunServer
  children: Set<Promise<unknown>>
  closed: Promise<void> | undefined
}

const liveHost = (globalThis as typeof globalThis & { Bun?: BunHost }).Bun

export function createBunHttpServerProvider(
  host: BunHost | undefined = liveHost
): ProviderPackageEntry {
  const servers = new Set<ServerToken>()
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-bun#http-server",
    service: "std/http/server::HttpServer",
    targets: ["bun-process"],
    operations: {
      async listen(value) {
        if (host === undefined) {
          return unavailable("Bun.serve is unavailable")
        }
        try {
          const request = listenRequest(value)
          const children = new Set<Promise<unknown>>()
          const server = host.serve({
            port: request.port,
            ...(request.hostname === undefined
              ? {}
              : { hostname: request.hostname }),
            fetch(nativeRequest) {
              const child = handleRequest(request.handler, nativeRequest)
              children.add(child)
              void child.then(
                () => children.delete(child),
                () => children.delete(child)
              )
              return child
            },
          })
          const token = { server, children, closed: undefined }
          servers.add(token)
          return { kind: "success", value: token }
        } catch (cause) {
          return unavailable(errorMessage(cause))
        }
      },
      async close(value) {
        const token = serverToken(value)
        await closeServer(token)
        servers.delete(token)
        return { kind: "success", value: undefined }
      },
    },
    shutdown: async () => {
      for (const token of [...servers].reverse()) await closeServer(token)
      servers.clear()
    },
  })
}

export const provider = createBunHttpServerProvider()

function unavailable(message: string): ProviderResult {
  return {
    kind: "failure",
    failure: Object.freeze({ tag: "HttpServerUnavailable", message }),
  }
}

async function handleRequest(
  handler: ListenRequest["handler"],
  request: Request
): Promise<Response> {
  const body = new Uint8Array(await request.arrayBuffer())
  const snapshot: AbiRequest = Object.freeze({
    method: request.method,
    url: request.url,
    headers: Object.freeze(
      [...request.headers.entries()].map(([name, value]) =>
        Object.freeze({ name, value })
      )
    ),
    body,
  })
  const applicationResponse = await handler(snapshot)
  if (isCancelledHttpServerResponse(applicationResponse)) {
    return Response.error()
  }
  const response = validateResponse(applicationResponse)
  const stream =
    response.body instanceof Uint8Array
      ? undefined
      : providerStreamBody(response.body)
  try {
    return new Response(responseBody(response.body), {
      status: response.status,
      headers: response.headers.map(
        ({ name, value }) => [name, value] as [string, string]
      ),
    })
  } catch (cause) {
    await stream?.cancel().catch(() => undefined)
    throw cause
  }
}

function responseBody(
  body: AbiResponse["body"]
): ArrayBuffer | ReadableStream<Uint8Array<ArrayBuffer>> {
  if (body instanceof Uint8Array) return Uint8Array.from(body).buffer
  const stream = providerStreamBody(body)
  return new ReadableStream<Uint8Array<ArrayBuffer>>({
    async pull(controller) {
      try {
        const next = await stream.pull()
        if (next.done) {
          await stream.complete()
          controller.close()
        } else controller.enqueue(Uint8Array.from(next.value))
      } catch (cause) {
        await stream.cancel().catch(() => undefined)
        controller.error(cause)
      }
    },
    async cancel() {
      await stream.cancel()
    },
  })
}

function listenRequest(value: unknown): ListenRequest {
  if (
    typeof value !== "object" ||
    value === null ||
    typeof (value as ListenRequest).handler !== "function"
  ) {
    throw new TypeError("HTTP listen request is invalid")
  }
  return value as ListenRequest
}

function validateResponse(value: unknown): AbiResponse {
  if (
    typeof value !== "object" ||
    value === null ||
    !Number.isSafeInteger((value as AbiResponse).status) ||
    !Array.isArray((value as AbiResponse).headers) ||
    !(
      (value as AbiResponse).body instanceof Uint8Array ||
      isProviderStreamBody((value as AbiResponse).body)
    )
  ) {
    throw new TypeError("HTTP handler response is invalid")
  }
  return value as AbiResponse
}

function isProviderStreamBody(
  value: unknown
): value is ProviderHttpServerStreamBody {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as { kind?: unknown }).kind === "stream" &&
    typeof (value as { pull?: unknown }).pull === "function" &&
    typeof (value as { complete?: unknown }).complete === "function" &&
    typeof (value as { cancel?: unknown }).cancel === "function"
  )
}

function providerStreamBody(value: unknown): ProviderHttpServerStreamBody {
  if (!isProviderStreamBody(value)) {
    throw new TypeError("HTTP streaming response body is invalid")
  }
  return value
}

function serverToken(value: unknown): ServerToken {
  if (
    typeof value !== "object" ||
    value === null ||
    !("server" in value) ||
    !("children" in value)
  ) {
    throw new TypeError("HTTP server handle is invalid")
  }
  return value as ServerToken
}

function closeServer(token: ServerToken): Promise<void> {
  token.closed ??= (async () => {
    await token.server.stop(false)
    await Promise.allSettled([...token.children])
    await token.server.stop(true)
  })()
  return token.closed
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : "HTTP server unavailable"
}
