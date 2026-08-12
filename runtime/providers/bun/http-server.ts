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
type AbiResponse = Readonly<{
  status: number
  headers: ReadonlyArray<AbiHeader>
  body: Uint8Array
}>
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
              void child.finally(() => children.delete(child))
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
  const response = validateResponse(await handler(snapshot))
  return new Response(new Uint8Array(response.body), {
    status: response.status,
    headers: response.headers.map(({ name, value }) => [name, value]),
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
    !((value as AbiResponse).body instanceof Uint8Array)
  ) {
    throw new TypeError("HTTP handler response is invalid")
  }
  return value as AbiResponse
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
