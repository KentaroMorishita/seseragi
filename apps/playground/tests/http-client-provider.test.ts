import { describe, expect, test } from "bun:test"
import { createBrowserHttpClientProvider } from "../../../runtime/ts/src/browser/provider-http-client"
import { fromUint8Array } from "../../../runtime/ts/src/bytes"
import {
  createEffectExecution,
  isEffectCancellation,
  run,
} from "../../../runtime/ts/src/effect"
import {
  emptyBody,
  exchange,
  get,
  type HttpClientRequest,
  parseUrl,
  post,
  request,
  send,
  streamBody,
} from "../../../runtime/ts/src/http-client"
import {
  ProviderBoundaryDefect,
  type ProviderEntry,
  type ProviderSubscriptionObserver,
  providerRuntimeAbi,
} from "../../../runtime/ts/src/provider"
import { createProviderHttpClient } from "../../../runtime/ts/src/provider-http-client"
import {
  defineProviderPackage,
  ProviderPackageLoader,
} from "../../../runtime/ts/src/provider-package"
import { fromPull, runCollect, take } from "../../../runtime/ts/src/stream"

let fixture = 0

async function environment(operations: ProviderEntry) {
  fixture += 1
  const provider = `fixture/runtime-bun#http-client-${fixture}`
  const entry = defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/http::HttpClient",
    targets: ["bun-process"],
    operations,
  })
  const loader = new ProviderPackageLoader("bun-process", [
    {
      provider,
      service: "std/http::HttpClient",
      target: "bun-process",
      module: "fixture/runtime-bun/http-client",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider: entry }),
    },
  ])
  return {
    loader,
    httpClient: createProviderHttpClient(await loader.load(provider)),
  }
}

async function browserEnvironment(
  fetchHost: (input: string, init: RequestInit) => Promise<Response>
) {
  const provider = "seseragi/runtime-browser#http-client"
  const entry = createBrowserHttpClientProvider(fetchHost)
  const loader = new ProviderPackageLoader("browser", [
    {
      provider,
      service: "std/http::HttpClient",
      target: "browser",
      module: "fixture/runtime-browser/http-client",
      exportName: "provider",
      loadMode: "lazy",
      importModule: async () => ({ provider: entry }),
    },
  ])
  return {
    loader,
    httpClient: createProviderHttpClient(await loader.load(provider)),
  }
}

describe("HTTP client provider vertical slice", () => {
  test("keeps exchange cold and closes upload and provider on early stop", async () => {
    let registrations = 0
    let demands = 0
    let bodyPulls = 0
    let bodyCloses = 0
    let unsubscribes = 0
    const selected = await environment({
      exchange(_value, observerValue, attachment) {
        registrations += 1
        const observer = observerValue as ProviderSubscriptionObserver
        const body = attachment as {
          pull(): Promise<IteratorResult<Uint8Array>>
        }
        return {
          async demand(count: number) {
            demands += count
            await body.pull()
            observer.next({
              kind: "ResponseStarted",
              head: { version: "Http1_1", status: 200, headers: [] },
            })
          },
          unsubscribe() {
            unsubscribes += 1
          },
        }
      },
    })
    const upload = streamBody(
      fromPull(() => ({
        async pull() {
          bodyPulls += 1
          return {
            done: false,
            value: fromUint8Array(new Uint8Array([1, 2, 3])),
          }
        },
        close() {
          bodyCloses += 1
        },
      }))
    )
    const parsed = parseUrl("https://example.test/upload")
    if (parsed.tag === "Left") throw new Error("fixture URL is invalid")
    const stream = exchange(upload, request(post, parsed.value))

    expect(registrations).toBe(0)
    const result = await run(runCollect(take(1, stream)), {
      httpClient: selected.httpClient,
    })

    expect(result.kind).toBe("success")
    if (result.kind !== "success") throw new Error("exchange fixture failed")
    const events = result.value
    expect(events).toHaveLength(1)
    expect(events[0]?.tag).toBe("ResponseStarted")
    expect(registrations).toBe(1)
    expect(demands).toBe(1)
    expect(bodyPulls).toBe(1)
    expect(unsubscribes).toBe(1)
    expect(bodyCloses).toBe(1)
    await selected.loader.shutdown()
  })

  test("cancels a pending exchange and discards late provider events", async () => {
    let observer: ProviderSubscriptionObserver | undefined
    let bodyCloses = 0
    let unsubscribes = 0
    let demandReady: () => void = () => undefined
    const demanded = new Promise<void>((resolve) => {
      demandReady = resolve
    })
    const selected = await environment({
      exchange(_value, observerValue) {
        observer = observerValue as ProviderSubscriptionObserver
        return {
          demand() {
            demandReady()
          },
          unsubscribe() {
            unsubscribes += 1
          },
        }
      },
    })
    const upload = streamBody(
      fromPull(() => ({
        async pull() {
          return await new Promise<IteratorResult<never>>(() => undefined)
        },
        close() {
          bodyCloses += 1
        },
      }))
    )
    const parsed = parseUrl("https://example.test/pending")
    if (parsed.tag === "Left") throw new Error("fixture URL is invalid")
    const execution = createEffectExecution()
    const pending = run(
      runCollect(exchange(upload, request(post, parsed.value))),
      { httpClient: selected.httpClient },
      execution.context
    ).catch((error: unknown) => error)

    await demanded
    await execution.cancel()
    expect(isEffectCancellation(await pending)).toBe(true)
    expect(unsubscribes).toBe(1)
    expect(bodyCloses).toBe(1)
    observer?.next({
      kind: "ResponseStarted",
      head: { version: "Http1_1", status: 200, headers: [] },
    })
    await selected.loader.shutdown()
  })

  test("streams browser response chunks and trailers only with an exposed version", async () => {
    const response = new Response(
      new ReadableStream<Uint8Array>({
        start(controller) {
          controller.enqueue(new Uint8Array(64 * 1024 + 1))
          controller.close()
        },
      }),
      {
        status: 200,
        headers: { "content-type": "application/octet-stream" },
      }
    ) as Response & {
      httpVersion?: "Http2"
      trailers?: Promise<Headers>
    }
    response.httpVersion = "Http2"
    response.trailers = Promise.resolve(new Headers({ "x-finished": "yes" }))
    const selected = await browserEnvironment(async () => response)
    const parsed = parseUrl("https://example.test/stream")
    if (parsed.tag === "Left") throw new Error("fixture URL is invalid")

    const result = await run(
      runCollect(exchange(emptyBody(), request(get, parsed.value))),
      { httpClient: selected.httpClient }
    )

    expect(result.kind).toBe("success")
    if (result.kind !== "success") throw new Error("browser exchange failed")
    const events = result.value
    expect(events.map((event) => event.tag)).toEqual([
      "ResponseStarted",
      "ResponseBodyChunk",
      "ResponseBodyChunk",
      "ResponseTrailers",
    ])
    expect(events[0]).toMatchObject({
      value: { version: { tag: "Http2" } },
    })
    await selected.loader.shutdown()
  })

  test("does not fabricate an HTTP version when browser Fetch omits it", async () => {
    const selected = await browserEnvironment(async () => new Response("ok"))
    const parsed = parseUrl("https://example.test/version")
    if (parsed.tag === "Left") throw new Error("fixture URL is invalid")

    const result = await run(
      runCollect(exchange(emptyBody(), request(get, parsed.value))),
      { httpClient: selected.httpClient }
    )

    expect(result).toEqual({
      kind: "failure",
      error: {
        tag: "Right",
        value: {
          tag: "HttpProtocolFailure",
          value: "browser Fetch does not expose the negotiated HTTP version",
        },
      },
    })
    await selected.loader.shutdown()
  })

  test("stays cold and copies request and response bytes at the ABI boundary", async () => {
    let calls = 0
    let observedBody: Uint8Array | undefined
    const providerBody = new Uint8Array([4, 5, 6])
    const selected = await environment({
      async send(value) {
        calls += 1
        observedBody = (value as HttpClientRequest).body
        return {
          kind: "success",
          value: {
            status: 200,
            headers: [{ name: "content-type", value: "text/plain" }],
            body: providerBody,
          },
        }
      },
    })
    const requestBody = new Uint8Array([1, 2, 3])
    const effect = send({
      method: "POST",
      url: "https://example.test/",
      headers: [],
      body: requestBody,
    })
    expect(calls).toBe(0)

    const result = await run(effect, { httpClient: selected.httpClient })
    expect(result.kind).toBe("success")
    expect(calls).toBe(1)
    expect(observedBody).toEqual(new Uint8Array([1, 2, 3]))
    expect(observedBody).not.toBe(requestBody)
    providerBody[0] = 99
    if (result.kind === "success") {
      expect(result.value.body).toEqual(new Uint8Array([4, 5, 6]))
    }
    await selected.loader.shutdown()
  })

  test("rejects accessor fields before calling the provider", async () => {
    let calls = 0
    const selected = await environment({
      async send() {
        calls += 1
        return { kind: "success", value: undefined }
      },
    })
    const request = {
      get method() {
        return "GET"
      },
      url: "https://example.test/",
      headers: [],
      body: new Uint8Array(),
    } as HttpClientRequest
    const defect = await run(send(request), {
      httpClient: selected.httpClient,
    }).catch((error: unknown) => error)

    expect(defect).toBeInstanceOf(ProviderBoundaryDefect)
    if (!(defect instanceof ProviderBoundaryDefect)) {
      throw new Error("expected an HTTP client provider boundary defect")
    }
    expect(defect.stage).toBe("input")
    expect(calls).toBe(0)
    await selected.loader.shutdown()
  })

  test("preserves typed network and protocol failures", async () => {
    for (const failure of [
      { tag: "HttpDnsFailure", value: "host not found" },
      { tag: "HttpProtocolFailure", value: "invalid response framing" },
    ] as const) {
      const selected = await environment({
        async send() {
          return { kind: "failure", failure }
        },
      })
      const result = await run(
        send({
          method: "GET",
          url: "https://example.test/",
          headers: [],
          body: new Uint8Array(),
        }),
        { httpClient: selected.httpClient }
      )

      expect(result).toEqual({ kind: "failure", error: failure })
      await selected.loader.shutdown()
    }
  })

  test("keeps redirects manual at the browser provider boundary", async () => {
    let observed: RequestInit | undefined
    const selected = await browserEnvironment(async (_input, init) => {
      observed = init
      return new Response("redirect", {
        status: 302,
        headers: { location: "https://example.test/next" },
      })
    })

    const result = await run(
      send({
        method: "GET",
        url: "https://example.test/start",
        headers: [],
        body: new Uint8Array(),
      }),
      { httpClient: selected.httpClient }
    )

    expect(result.kind).toBe("success")
    expect(observed?.redirect).toBe("manual")
    await selected.loader.shutdown()
  })

  test("aborts the browser fetch once when Effect cancellation wins", async () => {
    let aborts = 0
    const selected = await browserEnvironment(
      (_input, init) =>
        new Promise((_resolve, reject) => {
          init.signal?.addEventListener(
            "abort",
            () => {
              aborts += 1
              reject(new Error("cancelled HTTP request"))
            },
            { once: true }
          )
        })
    )
    const execution = createEffectExecution()
    const pending = run(
      send({
        method: "GET",
        url: "https://example.test/pending",
        headers: [],
        body: new Uint8Array(),
      }),
      { httpClient: selected.httpClient },
      execution.context
    ).catch((error: unknown) => error)

    const first = execution.cancel()
    const second = execution.cancel()
    expect(first).toBe(second)
    expect(isEffectCancellation(await pending)).toBe(true)
    await first
    expect(aborts).toBe(1)
    await selected.loader.shutdown()
  })
})
