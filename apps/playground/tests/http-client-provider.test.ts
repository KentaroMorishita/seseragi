import { describe, expect, test } from "bun:test"
import { run } from "../../../runtime/ts/src/effect"
import {
  type HttpClientRequest,
  send,
} from "../../../runtime/ts/src/http-client"
import {
  ProviderBoundaryDefect,
  type ProviderEntry,
  providerRuntimeAbi,
} from "../../../runtime/ts/src/provider"
import { createProviderHttpClient } from "../../../runtime/ts/src/provider-http-client"
import {
  defineProviderPackage,
  ProviderPackageLoader,
} from "../../../runtime/ts/src/provider-package"

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

describe("HTTP client provider vertical slice", () => {
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
})
