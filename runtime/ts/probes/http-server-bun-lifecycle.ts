import assert from "node:assert/strict"
import { connect, type Socket } from "node:net"
import { createBunHttpServerProvider } from "../../providers/bun/http-server"
import { fromUint8Array } from "../src/bytes"
import { createEffectExecution, isEffectCancellation, run } from "../src/effect"
import {
  close,
  type HttpServerHandle,
  serveOnce,
  streamResponse,
  textResponse,
} from "../src/http-server"
import { createProviderHttpServer } from "../src/provider-http-server"
import { ProviderPackageLoader } from "../src/provider-package"
import { fromPull } from "../src/stream"

function deferred() {
  let resolve = (): void => undefined
  const promise = new Promise<void>((done) => {
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
  const end = Date.now() + 2_000
  while (!predicate()) {
    assert(Date.now() < end, `${message} timed out`)
    await Bun.sleep(5)
  }
}

async function scenario(mode: string): Promise<void> {
  let native: ReturnType<typeof Bun.serve> | undefined
  const stopCalls: boolean[] = []
  const provider = createBunHttpServerProvider({
    serve(options) {
      native = Bun.serve(options)
      return {
        port: native.port,
        stop(force = false) {
          stopCalls.push(force)
          return native!.stop(force)
        },
      }
    },
  })
  const identity = "seseragi/runtime-bun#http-server"
  const loader = new ProviderPackageLoader("bun-process", [
    {
      provider: identity,
      service: "std/http/server::HttpServer",
      target: "bun-process",
      module: "http-server-bun-lifecycle",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider }),
    },
  ])
  const server = createProviderHttpServer(await loader.load(identity))
  let handle: HttpServerHandle | undefined
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
  const entered = deferred()
  const release = deferred()
  const finalizerEntered = deferred()
  const releaseFinalizer = deferred()
  const slowFinalizer = mode.endsWith("-slow-finalizer")
  const blocked = mode.endsWith("-drain") || slowFinalizer
  const maximumChunks = blocked ? 128 : 4
  const chunk = new Uint8Array(64 * 1024).fill(97)
  let produced = 0
  let cursorCloses = 0
  let cursorCloseFinished = 0
  let cancellations = 0
  let handled = 0
  const body = fromPull(() => ({
    async pull() {
      if (produced === maximumChunks) return { done: true, value: undefined }
      produced += 1
      return { done: false, value: fromUint8Array(chunk) }
    },
    async close() {
      cursorCloses += 1
      finalizerEntered.resolve()
      if (slowFinalizer) await releaseFinalizer.promise
      cursorCloseFinished += 1
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
          return streamResponse(200, [], body)(handlerEnvironment, context)
        }
      },
    }),
    environment,
    execution.context
  ).catch((cause: unknown) => cause)
  let peer: Socket | undefined
  let response: Promise<Response | Error> | undefined
  let shutdown: Promise<void> | undefined
  const abort = new AbortController()
  try {
    await until(() => native !== undefined, "listener start")
    if (blocked) {
      peer = connect(native!.port, "127.0.0.1")
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
      response = fetch(`http://127.0.0.1:${native!.port}/one`, {
        headers: { connection: "close" },
        signal: abort.signal,
      }).catch((cause: Error) => cause)
    }
    await deadline(entered.promise, "handler claim")
    await until(() => stopCalls.includes(false), "serveOnce graceful close")
    if (mode === "shutdown-late-stream") {
      shutdown = loader.shutdown()
      await until(() => stopCalls.includes(true), "force late transport close")
      assert((await deadline(response!, "closed fetch")) instanceof Error)
    }
    await Bun.sleep(50)
    release.resolve()
    if (blocked) {
      let previous = -1
      let stableSince = Date.now()
      await until(() => {
        if (produced !== previous) {
          previous = produced
          stableSince = Date.now()
        }
        return (
          produced > 0 &&
          produced < maximumChunks &&
          Date.now() - stableSince >= 50
        )
      }, "finite paused response backpressure")
      if (slowFinalizer) {
        let finished = false
        let operation: Promise<void>
        if (mode.startsWith("cancel-")) operation = execution.cancel()
        else {
          shutdown = loader.shutdown()
          operation = shutdown
        }
        const stopping = operation.then(() => {
          finished = true
        })
        await deadline(finalizerEntered.promise, "stream finalizer start")
        await Bun.sleep(25)
        assert.equal(
          finished,
          false,
          "close must join its pending stream finalizer"
        )
        releaseFinalizer.resolve()
        await deadline(stopping, "joined stream finalizer")
      } else if (mode === "cancel-drain")
        await deadline(execution.cancel(), "cancel during graceful drain")
      else if (mode === "close-drain") {
        assert(handle !== undefined)
        assert.deepEqual(
          await deadline(
            run(close(handle), environment, execution.context),
            "explicit close during graceful drain"
          ),
          { kind: "success", value: undefined }
        )
      } else {
        shutdown = loader.shutdown()
        await deadline(shutdown, "shutdown during graceful drain")
      }
      assert(stopCalls.includes(true), "force stop must precede peer cleanup")
      assert.equal(cursorCloses, 1)
      assert.equal(cursorCloseFinished, 1)
      assert.equal(cancellations, 1)
      const result = await deadline(serving, "stopped serveOnce")
      if (mode.startsWith("cancel-")) assert(isEffectCancellation(result))
      else assert.deepEqual(result, { kind: "success", value: undefined })
    } else if (mode === "shutdown-late-stream") {
      await deadline(shutdown!, "late stream shutdown")
      assert.equal(cursorCloses, 1)
      assert.equal(produced, 0, "late stream must not be pulled")
    } else {
      const value = await deadline(response!, "delayed response")
      assert(value instanceof Response)
      assert.equal(value.status, 200)
      if (mode === "delayed-buffer") assert.equal(await value.text(), "delayed")
      else {
        assert.deepEqual(
          new Uint8Array(await value.arrayBuffer()),
          new Uint8Array(maximumChunks * chunk.length).fill(97)
        )
        assert.equal(cursorCloses, 1)
      }
      assert.deepEqual(await deadline(serving, "completed serveOnce"), {
        kind: "success",
        value: undefined,
      })
      assert.equal(cancellations, 0)
    }
    assert.equal(handled, 1)
  } finally {
    release.resolve()
    releaseFinalizer.resolve()
    abort.abort()
    if (peer !== undefined && !peer.closed) {
      const closed = new Promise<void>((resolve) =>
        peer!.once("close", resolve)
      )
      peer.destroy()
      await deadline(closed, "peer cleanup")
    }
    await deadline(native?.stop(true) ?? Promise.resolve(), "native cleanup")
    await deadline(execution.cancel(), "execution cleanup")
    await deadline(shutdown ?? loader.shutdown(), "provider cleanup")
    await deadline(serving, "serveOnce cleanup")
    assert.equal(native?.pendingRequests ?? 0, 0)
  }
}

const mode = process.argv[2] ?? "delayed-buffer"
assert(
  [
    "delayed-buffer",
    "delayed-stream",
    "cancel-drain",
    "cancel-slow-finalizer",
    "shutdown-slow-finalizer",
    "close-drain",
    "shutdown-drain",
    "shutdown-late-stream",
  ].includes(mode)
)
await scenario(mode)
process.stdout.write(`Bun HTTP server lifecycle probe passed: ${mode}\n`)
