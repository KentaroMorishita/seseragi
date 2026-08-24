import {
  createServer,
  type IncomingMessage,
  type Server,
  type ServerResponse,
} from "node:http"
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
type ServerToken = {
  readonly server: Server
  readonly children: Set<Promise<void>>
  closed?: Promise<void>
}

export function createNodeHttpServerProvider(): ProviderPackageEntry {
  const servers = new Set<ServerToken>()
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-node#http-server",
    service: "std/http/server::HttpServer",
    targets: ["node-process"],
    operations: {
      async listen(value) {
        try {
          const request = listenRequest(value)
          const children = new Set<Promise<void>>()
          const server = createServer((nativeRequest, nativeResponse) => {
            const child = handleRequest(
              request.handler,
              nativeRequest,
              nativeResponse
            )
            children.add(child)
            void child.then(
              () => children.delete(child),
              () => children.delete(child)
            )
          })
          await startServer(server, request)
          const token = { server, children }
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

export const provider = createNodeHttpServerProvider()

function startServer(server: Server, request: ListenRequest): Promise<void> {
  return new Promise((resolve, reject) => {
    const onError = (error: Error): void => {
      server.off("listening", onListening)
      reject(error)
    }
    const onListening = (): void => {
      server.off("error", onError)
      resolve()
    }
    server.once("error", onError)
    server.once("listening", onListening)
    server.listen(request.port, request.hostname)
  })
}

async function handleRequest(
  handler: ListenRequest["handler"],
  request: IncomingMessage,
  response: ServerResponse
): Promise<void> {
  let stream: ProviderHttpServerStreamBody | undefined
  try {
    const body = await readBody(request)
    const applicationResponse = await handler(
      Object.freeze({
        method: request.method ?? "GET",
        url: absoluteUrl(request),
        headers: requestHeaders(request),
        body,
      })
    )
    if (isCancelledHttpServerResponse(applicationResponse)) {
      response.destroy()
      return
    }
    const result = validateResponse(applicationResponse)
    for (const entry of result.headers) {
      response.appendHeader(entry.name, entry.value)
    }
    response.writeHead(result.status)
    if (result.body instanceof Uint8Array) {
      response.end(Buffer.from(result.body))
    } else {
      stream = providerStreamBody(result.body)
      await writeStreamBody(stream, response)
    }
  } catch (cause) {
    await stream?.cancel().catch(() => undefined)
    response.destroy(cause instanceof Error ? cause : new Error(String(cause)))
  }
}

async function writeStreamBody(
  body: ProviderHttpServerStreamBody,
  response: ServerResponse
): Promise<void> {
  let completed = false
  const disconnected = (): void => {
    if (!completed) void body.cancel().catch(() => undefined)
  }
  response.once("close", disconnected)
  try {
    while (true) {
      const next = await body.pull()
      if (next.done) break
      if (!response.write(Buffer.from(next.value))) await waitForDrain(response)
    }
    await endResponse(response)
    response.off("close", disconnected)
    await body.complete()
    completed = true
  } finally {
    response.off("close", disconnected)
    if (!completed) await body.cancel()
  }
}

function endResponse(response: ServerResponse): Promise<void> {
  return new Promise((resolve, reject) => {
    const cleanup = (): void => {
      response.off("error", onError)
      response.off("close", onClose)
    }
    const onError = (error: Error): void => {
      cleanup()
      reject(error)
    }
    const onClose = (): void => {
      cleanup()
      reject(new Error("HTTP response closed before completion"))
    }
    response.once("error", onError)
    response.once("close", onClose)
    response.end(() => {
      cleanup()
      resolve()
    })
  })
}

function waitForDrain(response: ServerResponse): Promise<void> {
  return new Promise((resolve, reject) => {
    const cleanup = (): void => {
      response.off("drain", onDrain)
      response.off("error", onError)
      response.off("close", onClose)
    }
    const onDrain = (): void => {
      cleanup()
      resolve()
    }
    const onError = (error: Error): void => {
      cleanup()
      reject(error)
    }
    const onClose = (): void => {
      cleanup()
      reject(new Error("HTTP response closed before drain"))
    }
    response.once("drain", onDrain)
    response.once("error", onError)
    response.once("close", onClose)
  })
}

async function readBody(request: IncomingMessage): Promise<Uint8Array> {
  const chunks: Buffer[] = []
  for await (const chunk of request) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk))
  }
  return new Uint8Array(Buffer.concat(chunks))
}

function absoluteUrl(request: IncomingMessage): string {
  const raw = request.url ?? "/"
  if (/^https?:\/\//u.test(raw)) return raw
  const host = request.headers.host ?? "localhost"
  return new URL(raw, `http://${host}`).toString()
}

function requestHeaders(request: IncomingMessage): ReadonlyArray<AbiHeader> {
  const headers: AbiHeader[] = []
  for (let index = 0; index < request.rawHeaders.length; index += 2) {
    const name = request.rawHeaders[index]
    const value = request.rawHeaders[index + 1]
    if (name !== undefined && value !== undefined) {
      headers.push(Object.freeze({ name, value }))
    }
  }
  return Object.freeze(headers)
}

function unavailable(message: string): ProviderResult {
  return {
    kind: "failure",
    failure: Object.freeze({ tag: "HttpServerUnavailable", message }),
  }
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
    const stopped = new Promise<void>((resolve, reject) => {
      token.server.close((error) => {
        if (error === undefined) resolve()
        else reject(error)
      })
    })
    token.server.closeIdleConnections()
    await Promise.allSettled([...token.children])
    token.server.closeAllConnections()
    await stopped
  })()
  return token.closed
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : "HTTP server unavailable"
}
