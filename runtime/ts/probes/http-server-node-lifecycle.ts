import assert from "node:assert/strict"
import {
  type ClientRequest,
  request as httpRequest,
  type IncomingMessage,
  type Server,
  type ServerResponse,
} from "node:http"
import { connect, type Socket } from "node:net"
import { createNodeHttpServerProvider } from "../../providers/node/http-server"
import { type Bytes, fromUint8Array } from "../src/bytes"
import { createEffectExecution, isEffectCancellation, run } from "../src/effect"
import {
  response as bufferedResponse,
  close,
  type HttpServerHandle,
  serveOnce,
  streamResponse,
  textResponse,
} from "../src/http-server"
import type { ProviderResult } from "../src/provider"
import { createProviderHttpServer } from "../src/provider-http-server"
import { ProviderPackageLoader } from "../src/provider-package"
import { fromPull } from "../src/stream"

type Token = {
  server: Server
  children: Set<Promise<void>>
  pendingBodies: Set<IncomingMessage>
}

function deferred<Value>() {
  let resolve = (_value: Value): void => undefined
  const promise = new Promise<Value>((done) => {
    resolve = done
  })
  return { promise, resolve }
}

async function deadline<Value>(
  promise: PromiseLike<Value>,
  message: string
): Promise<Value> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(
          () => reject(new Error(`${message} timed out`)),
          2_000
        )
      }),
    ])
  } finally {
    clearTimeout(timer)
  }
}

async function until(predicate: () => boolean, message: string): Promise<void> {
  const endsAt = Date.now() + 2_000
  while (!predicate()) {
    assert(Date.now() < endsAt, `${message} timed out`)
    await new Promise((resolve) => setTimeout(resolve, 5))
  }
}

async function destroySocket(socket: Socket): Promise<void> {
  if (socket.closed) return
  await deadline(
    new Promise<void>((resolve) => {
      socket.once("close", resolve)
      socket.destroy()
    }),
    "peer cleanup"
  )
}

async function scenario(mode: string): Promise<void> {
  const provider = createNodeHttpServerProvider()
  const loader = new ProviderPackageLoader("node-process", [
    {
      provider: "seseragi/runtime-node#http-server",
      service: "std/http/server::HttpServer",
      target: "node-process",
      module: "http-server-node-lifecycle",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider }),
    },
  ])
  const loaded = await loader.load("seseragi/runtime-node#http-server")
  const listening = deferred<Token>()
  let handle: HttpServerHandle | undefined
  const server = createProviderHttpServer({
    ...loaded,
    entry: {
      ...loaded.entry,
      async listen(value) {
        const result = (await provider.listen!(value)) as ProviderResult
        if (result.kind === "success") listening.resolve(result.value as Token)
        return result
      },
    },
  })
  const environment = {
    httpServer: {
      async listen(...arguments_: Parameters<typeof server.listen>) {
        const result = await server.listen(...arguments_)
        if (result.kind === "success") handle = result.value
        return result
      },
      close: server.close,
    },
  }
  const execution = createEffectExecution()
  const entered = deferred<void>()
  const release = deferred<void>()
  const chunk = new Uint8Array(64 * 1024).fill(97)
  const isBlockedStream = mode === "cancel-drain" || mode === "close-drain"
  const maximumChunks = isBlockedStream ? 128 : 4
  let produced = 0
  let cursorCloses = 0
  let cancellations = 0
  let handled = 0
  const body = fromPull<unknown, never, Bytes>(() => ({
    async pull() {
      if (produced === maximumChunks) return { done: true, value: undefined }
      produced += 1
      return { done: false, value: fromUint8Array(chunk) }
    },
    close() {
      cursorCloses += 1
    },
  }))
  const serving = run(
    serveOnce({
      hostname: "127.0.0.1",
      port: 0,
      handler() {
        handled += 1
        return async (handlerEnvironment, context) => {
          assert(context !== undefined)
          context.onCancel(() => {
            cancellations += 1
          })
          entered.resolve()
          await release.promise
          if (mode === "delayed-buffer") return textResponse(200, [], "delayed")
          if (mode === "buffered-backpressure") {
            return bufferedResponse(
              200,
              [],
              new Uint8Array(8 * 1024 * 1024).fill(97)
            )
          }
          return streamResponse(200, [], body)(handlerEnvironment, context)
        }
      },
    }),
    environment,
    execution.context
  ).catch((cause: unknown) => cause)
  let token: Token | undefined
  let peer: Socket | undefined
  let response: Promise<Response | Error> | undefined
  let stop: Promise<void> | undefined
  let clientRequest: ClientRequest | undefined
  let incoming: IncomingMessage | undefined
  let clientBody: Promise<Buffer | Error> | undefined
  const headersReady = deferred<void>()
  const abort = new AbortController()
  try {
    token = await deadline(listening.promise, "listener start")
    const address = token.server.address()
    assert(address !== null && typeof address !== "string")
    let nativeResponse: ServerResponse | undefined
    token.server.once("request", (_request, value) => {
      nativeResponse = value
    })
    if (mode === "buffered-backpressure") {
      clientBody = new Promise<Buffer>((resolve, reject) => {
        clientRequest = httpRequest(
          {
            hostname: "127.0.0.1",
            port: address.port,
            path: "/one",
            headers: { connection: "close" },
          },
          (response) => {
            incoming = response
            response.pause()
            const chunks: Buffer[] = []
            let length = 0
            response.on("data", (chunk: Buffer) => {
              length += chunk.length
              if (length > 8 * 1024 * 1024) {
                reject(
                  new Error("buffered response exceeded its finite fixture")
                )
                response.destroy()
              } else chunks.push(chunk)
            })
            response.once("error", reject)
            response.once("end", () => resolve(Buffer.concat(chunks)))
            headersReady.resolve()
          }
        )
        clientRequest.once("error", reject)
        clientRequest.end()
      }).catch((cause: Error) => cause)
    } else if (isBlockedStream) {
      peer = connect(address.port, "127.0.0.1")
      peer.on("error", () => undefined)
      peer.pause()
      await deadline(
        new Promise<void>((resolve, reject) => {
          peer!.once("connect", resolve)
          peer!.once("error", reject)
        }),
        "peer connect"
      )
      peer.write(
        "GET /one HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
      )
    } else {
      response = fetch(`http://127.0.0.1:${address.port}/one`, {
        headers: { connection: "close" },
        signal: abort.signal,
      }).catch((cause: Error) => cause)
    }
    await deadline(entered.promise, "handler claim")
    await until(() => !token!.server.listening, "graceful listener close")
    if (mode === "shutdown-late-stream") {
      stop = loader.shutdown()
      await until(
        () => nativeResponse?.destroyed === true,
        "shutdown transport close"
      )
    }
    // Hold a real handler beyond the close call; the old unconditional
    // closeAllConnections loses this response before the gate opens.
    await new Promise((resolve) => setTimeout(resolve, 50))
    release.resolve()
    if (isBlockedStream) {
      let previous = -1
      let stableSince = Date.now()
      await until(() => {
        if (produced !== previous) {
          previous = produced
          stableSince = Date.now()
        }
        return (
          nativeResponse?.writableNeedDrain === true &&
          produced < maximumChunks &&
          Date.now() - stableSince >= 25
        )
      }, "real response backpressure")
      if (mode === "cancel-drain") {
        await deadline(execution.cancel(), "cancel during graceful drain")
      } else {
        assert(handle !== undefined)
        const closed = await deadline(
          run(close(handle), environment, execution.context),
          "explicit close during graceful drain"
        )
        assert.deepEqual(closed, { kind: "success", value: undefined })
      }
      assert.equal(nativeResponse?.destroyed, true)
      const result = await deadline(serving, "cancelled serveOnce")
      if (mode === "cancel-drain")
        assert(
          isEffectCancellation(result),
          "serveOnce must report cancellation"
        )
      else assert.deepEqual(result, { kind: "success", value: undefined })
      assert.equal(cursorCloses, 1)
      assert.equal(cancellations, 1)
    } else if (mode === "buffered-backpressure") {
      await deadline(headersReady.promise, "paused buffered response headers")
      await until(
        () =>
          nativeResponse!.writableLength > 0 &&
          !nativeResponse!.writableFinished,
        "buffered native write backpressure"
      )
      assert.equal(
        token.children.size,
        1,
        "native buffer write must remain an owned child"
      )
      incoming!.resume()
      const bytes = await deadline(clientBody!, "complete buffered response")
      assert(Buffer.isBuffer(bytes))
      assert.equal(bytes.length, 8 * 1024 * 1024)
      assert(bytes.every((byte) => byte === 97))
      assert.deepEqual(
        await deadline(serving, "buffered serveOnce completion"),
        { kind: "success", value: undefined }
      )
      assert.equal(cancellations, 0)
    } else if (mode === "shutdown-late-stream") {
      await deadline(stop!, "shutdown after late stream")
      assert((await deadline(response!, "closed fetch")) instanceof Error)
      assert.equal(cursorCloses, 1, "late stream must release its cursor")
      assert.equal(produced, 0, "closed transport must not pull a late stream")
    } else {
      const result = await deadline(response!, "delayed response")
      assert(result instanceof Response, `${mode} must retain its connection`)
      assert.equal(result.status, 200)
      if (mode === "delayed-buffer")
        assert.equal(await result.text(), "delayed")
      else {
        assert.deepEqual(
          new Uint8Array(await result.arrayBuffer()),
          new Uint8Array(maximumChunks * chunk.length).fill(97)
        )
        assert.equal(cursorCloses, 1)
      }
      const completed = await deadline(serving, "completed serveOnce")
      assert.deepEqual(completed, { kind: "success", value: undefined })
      assert.equal(
        cancellations,
        0,
        "normal completion must not cancel its handler"
      )
    }
    assert.equal(handled, 1)
    await deadline(Promise.allSettled([...token.children]), "request cleanup")
    assert.equal(token.children.size, 0)
    assert.equal(token.pendingBodies.size, 0)
  } finally {
    release.resolve()
    abort.abort()
    incoming?.destroy()
    clientRequest?.destroy()
    if (clientRequest?.socket !== undefined && clientRequest.socket !== null) {
      await destroySocket(clientRequest.socket)
    }
    if (peer !== undefined) await destroySocket(peer)
    await deadline(execution.cancel(), "execution cleanup")
    if (token !== undefined)
      await deadline(provider.close!(token), "provider cleanup")
    await deadline(stop ?? loader.shutdown(), "package cleanup")
    await deadline(serving, "serveOnce cleanup")
  }
}

const mode = process.argv[2] ?? "delayed-buffer"
assert(
  [
    "delayed-buffer",
    "delayed-stream",
    "buffered-backpressure",
    "cancel-drain",
    "close-drain",
    "shutdown-late-stream",
  ].includes(mode)
)
await scenario(mode)
process.stdout.write(`Node HTTP server lifecycle probe passed: ${mode}\n`)
