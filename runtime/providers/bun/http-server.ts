import {
  isCancelledHttpServerResponse,
  onHttpServerForcedClose,
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
type ActiveStream = Readonly<{
  cancel: () => Promise<void>
  completion: Promise<void>
}>
type ServerLifecycle = {
  readonly streams: Set<ActiveStream>
  forced: boolean
}
type ServerToken = {
  server: BunServer
  children: Set<Promise<unknown>>
  handler: ListenRequest["handler"]
  lifecycle: ServerLifecycle
  forceClosed?: Promise<void>
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
          const lifecycle: ServerLifecycle = {
            streams: new Set(),
            forced: false,
          }
          const server = host.serve({
            port: request.port,
            ...(request.hostname === undefined
              ? {}
              : { hostname: request.hostname }),
            fetch(nativeRequest) {
              const child = handleRequest(
                request.handler,
                nativeRequest,
                lifecycle
              )
              children.add(child)
              void child.then(
                () => children.delete(child),
                () => children.delete(child)
              )
              return child
            },
          })
          const token: ServerToken = {
            server,
            children,
            handler: request.handler,
            lifecycle,
            closed: undefined,
          }
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
      for (const token of [...servers].reverse()) await closeServer(token, true)
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
  request: Request,
  lifecycle: ServerLifecycle
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
    // Bun rejects Response.error() because its status is zero. An already
    // errored body closes the connection before any HTTP headers are sent.
    return closedResponse()
  }
  const response = validateResponse(applicationResponse)
  const stream =
    response.body instanceof Uint8Array
      ? undefined
      : providerStreamBody(response.body)
  if (request.signal.aborted || lifecycle.forced) {
    await stream?.cancel()
    return closedResponse()
  }
  const nativeBody = responseBody(response.body, lifecycle.streams)
  try {
    return new Response(nativeBody.value, {
      status: response.status,
      headers: response.headers.map(
        ({ name, value }) => [name, value] as [string, string]
      ),
    })
  } catch (cause) {
    await nativeBody.cancel().catch(() => undefined)
    throw cause
  }
}

function closedResponse(): Response {
  return new Response(
    new ReadableStream<Uint8Array>({
      start(controller) {
        controller.error()
      },
    })
  )
}

function responseBody(
  body: AbiResponse["body"],
  active: Set<ActiveStream>
): Readonly<{
  value: ArrayBuffer | ReadableStream<Uint8Array<ArrayBuffer>>
  cancel: () => Promise<void>
}> {
  if (body instanceof Uint8Array) {
    return {
      value: Uint8Array.from(body).buffer,
      cancel: async () => undefined,
    }
  }
  const stream = providerStreamBody(body)
  let release = (): void => undefined
  const completion = new Promise<void>((resolve) => {
    release = resolve
  })
  let terminal: Promise<void> | undefined
  const finalize = (cancelled: boolean): Promise<void> => {
    terminal ??= Promise.resolve().then(async () => {
      try {
        if (cancelled) await stream.cancel()
        else await stream.complete()
      } finally {
        active.delete(owned)
        release()
      }
    })
    return terminal
  }
  const owned: ActiveStream = { cancel: () => finalize(true), completion }
  active.add(owned)
  const value = new ReadableStream<Uint8Array<ArrayBuffer>>({
    async pull(controller) {
      try {
        const next = await stream.pull()
        if (next.done) {
          await finalize(false)
          controller.close()
        } else controller.enqueue(Uint8Array.from(next.value))
      } catch (cause) {
        await owned.cancel().catch(() => undefined)
        controller.error(cause)
      }
    },
    async cancel() {
      await owned.cancel()
    },
  })
  return { value, cancel: owned.cancel }
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

function closeServer(token: ServerToken, force = false): Promise<void> {
  const forceClose = (): void => {
    token.lifecycle.forced = true
    token.forceClosed ??= Promise.resolve(token.server.stop(true))
    // The shared close completion joins this promise below, including when
    // shutdown escalates a close which is already draining.
    void token.forceClosed.catch(() => undefined)
    for (const stream of token.lifecycle.streams) {
      void stream.cancel().catch(() => undefined)
    }
  }
  if (force) forceClose()
  token.closed ??= (async () => {
    const stopped = token.server.stop(false)
    const unregister = onHttpServerForcedClose(token.handler, forceClose)
    try {
      await stopped
      await Promise.allSettled([...token.children])
      forceClose()
      await Promise.allSettled(
        [...token.lifecycle.streams].map((stream) => stream.completion)
      )
      await token.forceClosed
    } finally {
      unregister()
    }
  })()
  return token.closed
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : "HTTP server unavailable"
}
