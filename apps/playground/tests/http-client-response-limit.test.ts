import { describe, expect, test } from "bun:test"
import { createBrowserHttpClientProvider } from "../../../runtime/ts/src/browser/provider-http-client"
import { fromUint8Array } from "../../../runtime/ts/src/bytes"
import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "../../../runtime/ts/src/effect"
import {
  bodyLimit,
  get,
  type HttpBodyLimit,
  parseUrl,
  post,
  request,
  responseBody,
  responseStatus,
  sendBytes,
  sendEmpty,
  statusCode,
} from "../../../runtime/ts/src/http-client"
import { createProviderHttpClient } from "../../../runtime/ts/src/provider-http-client"
import { ProviderPackageLoader } from "../../../runtime/ts/src/provider-package"

function right<Value>(value: {
  readonly tag: "Left" | "Right"
  readonly value: unknown
}): Value {
  if (value.tag !== "Right") throw new Error("invalid HTTP fixture")
  return value.value as Value
}

function deferred<Value>() {
  let resolve: (value: Value) => void = () => undefined
  const promise = new Promise<Value>((complete) => {
    resolve = complete
  })
  return { promise, resolve }
}

const url = right<Parameters<typeof request>[1]>(
  parseUrl("https://example.test/bounded")
)
const limit = right<HttpBodyLimit>(bodyLimit(3))
const wrappers = [
  {
    name: "sendEmpty",
    effect: () => sendEmpty(limit, request(get, url)),
    upload: [],
  },
  {
    name: "sendBytes",
    effect: () =>
      sendBytes(
        limit,
        fromUint8Array(new Uint8Array([7, 8])),
        request(post, url)
      ),
    upload: [7, 8],
  },
]

async function browserEnvironment(
  fetchHost: (input: string, init: RequestInit) => Promise<Response>
) {
  const provider = "seseragi/runtime-browser#http-client"
  const loader = new ProviderPackageLoader("browser", [
    {
      provider,
      service: "std/http::HttpClient",
      target: "browser",
      module: "fixture/runtime-browser/http-client",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({
        provider: createBrowserHttpClientProvider(fetchHost),
      }),
    },
  ])
  return {
    loader,
    httpClient: createProviderHttpClient(await loader.load(provider)),
  }
}

describe("bounded HTTP convenience responses", () => {
  for (const wrapper of wrappers) {
    for (const contentLength of [undefined, "1"]) {
      test(`${wrapper.name} cancels on the first overflowing chunk (${contentLength ?? "no length"})`, async () => {
        let pulls = 0
        let cancels = 0
        let aborts = 0
        let responseController:
          | ReadableStreamDefaultController<Uint8Array>
          | undefined
        const chunks = [
          [1, 2],
          [3, 4],
          [5, 6],
        ]
        const response = new Response(
          new ReadableStream<Uint8Array>(
            {
              start(controller) {
                responseController = controller
              },
              pull(controller) {
                const chunk = chunks[pulls++]
                if (chunk === undefined) controller.close()
                else controller.enqueue(new Uint8Array(chunk))
              },
              cancel() {
                cancels += 1
              },
            },
            { highWaterMark: 0 }
          ),
          {
            ...(contentLength === undefined
              ? {}
              : { headers: { "content-length": contentLength } }),
          }
        )
        response.arrayBuffer = async () => {
          throw new Error("bounded responses must not buffer the full body")
        }
        const selected = await browserEnvironment(async (_url, init) => {
          init.signal?.addEventListener("abort", () => {
            aborts += 1
            responseController?.error(new Error("fixture fetch aborted"))
          })
          const upload: number[] = []
          if (init.body instanceof Uint8Array) {
            for (let offset = 0; offset < init.body.length; offset += 1) {
              upload.push(init.body[offset]!)
            }
          } else if (init.body instanceof ReadableStream) {
            const reader = init.body.getReader()
            try {
              while (true) {
                const next = await reader.read()
                if (next.done) break
                upload.push(...next.value)
              }
            } finally {
              reader.releaseLock()
            }
          }
          expect(upload).toEqual(wrapper.upload)
          expect(init.body instanceof ReadableStream).toBe(false)
          return response
        })
        try {
          expect(await run(wrapper.effect(), selected)).toEqual({
            kind: "failure",
            error: {
              tag: "HttpResponseBodyLimitExceeded",
              value: { limitBytes: 3 },
            },
          })
          expect(pulls).toBe(2)
          expect(cancels).toBe(1)
          expect(aborts).toBe(1)
          expect(response.body?.locked).toBe(false)
        } finally {
          await selected.loader.shutdown()
        }
      })
    }

    test(`${wrapper.name} accepts the exact limit and remains cold and reusable`, async () => {
      let calls = 0
      const responses: Response[] = []
      const selected = await browserEnvironment(async () => {
        calls += 1
        let chunk = 0
        const response = new Response(
          new ReadableStream<Uint8Array>(
            {
              pull(controller) {
                if (chunk++ === 0) controller.enqueue(new Uint8Array([1, 2]))
                else if (chunk === 2) controller.enqueue(new Uint8Array([3]))
                else controller.close()
              },
            },
            { highWaterMark: 0 }
          ),
          { status: 404 }
        )
        responses.push(response)
        return response
      })
      try {
        const effect = wrapper.effect()
        expect(calls).toBe(0)
        for (let repetition = 0; repetition < 2; repetition += 1) {
          const result = await run(effect, selected)
          expect(result.kind).toBe("success")
          if (result.kind !== "success") throw new Error("HTTP fixture failed")
          expect(statusCode(responseStatus(result.value))).toBe(404)
          expect(Array.from(responseBody(result.value))).toEqual([1, 2, 3])
        }
        expect(calls).toBe(2)
        expect(responses.every((response) => !response.body?.locked)).toBe(true)
      } finally {
        await selected.loader.shutdown()
      }
    })
  }

  test("keeps a response read failure typed and releases its reader", async () => {
    const response = new Response(
      new ReadableStream<Uint8Array>(
        {
          pull(controller) {
            controller.error(new Error("fixture response failed"))
          },
        },
        { highWaterMark: 0 }
      )
    )
    const selected = await browserEnvironment(async () => response)
    try {
      expect(await run(sendEmpty(limit, request(get, url)), selected)).toEqual({
        kind: "failure",
        error: {
          tag: "HttpConnectionFailure",
          value: "fixture response failed",
        },
      })
      expect(response.body?.locked).toBe(false)
    } finally {
      await selected.loader.shutdown()
    }
  })

  test("cancels a pending response read once outside typed failures", async () => {
    const reading = deferred<void>()
    const releasePull = deferred<void>()
    let cancels = 0
    let aborts = 0
    const response = new Response(
      new ReadableStream<Uint8Array>(
        {
          pull() {
            reading.resolve()
            return releasePull.promise
          },
          cancel() {
            cancels += 1
          },
        },
        { highWaterMark: 0 }
      )
    )
    const selected = await browserEnvironment(async (_url, init) => {
      init.signal?.addEventListener("abort", () => {
        aborts += 1
      })
      return response
    })
    const execution = createEffectExecution()
    try {
      const pending = run(
        sendEmpty(limit, request(get, url)),
        selected,
        execution.context
      ).catch((error: unknown) => error)
      await reading.promise
      await Promise.all([execution.cancel(), execution.cancel()])
      expect(isEffectCancellation(await pending)).toBe(true)
      expect(cancels).toBe(1)
      expect(aborts).toBe(1)
      expect(response.body?.locked).toBe(false)
    } finally {
      releasePull.resolve()
      await execution.cancel()
      await selected.loader.shutdown()
    }
  })

  test("disposes a response returned after fetch cancellation", async () => {
    const started = deferred<void>()
    const fetchResult = deferred<Response>()
    const cancelled = deferred<void>()
    let cancels = 0
    let aborts = 0
    const response = new Response(
      new ReadableStream<Uint8Array>(
        {
          cancel() {
            cancels += 1
            cancelled.resolve()
          },
        },
        { highWaterMark: 0 }
      )
    )
    const selected = await browserEnvironment(async (_url, init) => {
      init.signal?.addEventListener("abort", () => {
        aborts += 1
      })
      started.resolve()
      return await fetchResult.promise
    })
    const execution = createEffectExecution()
    try {
      const pending = run(
        sendEmpty(limit, request(get, url)),
        selected,
        execution.context
      ).catch((error: unknown) => error)
      await started.promise
      await execution.cancel()
      expect(isEffectCancellation(await pending)).toBe(true)
      fetchResult.resolve(response)
      await cancelled.promise
      expect(cancels).toBe(1)
      expect(aborts).toBe(1)
      expect(response.body?.locked).toBe(false)
    } finally {
      fetchResult.resolve(response)
      await execution.cancel()
      await selected.loader.shutdown()
    }
  })
})
