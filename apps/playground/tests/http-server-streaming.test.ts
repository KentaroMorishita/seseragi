import { describe, expect, test } from "bun:test"
import { type Bytes, fromUint8Array } from "../../../runtime/ts/src/bytes"
import { createEffectExecution, run } from "../../../runtime/ts/src/effect"
import {
  type HttpServer,
  type HttpServerHandle,
  type HttpServerOptions,
  httpServerSuccess,
  listen,
  type ProviderHttpServerOptions,
  type ProviderHttpServerStreamBody,
  streamResponse,
} from "../../../runtime/ts/src/http-server"
import { fromPull } from "../../../runtime/ts/src/stream"

type Fixture = Readonly<{
  execution: ReturnType<typeof createEffectExecution>
  options: ProviderHttpServerOptions
  providerCloses: () => number
}>

async function fixture(
  handler: HttpServerOptions<unknown>["handler"]
): Promise<Fixture> {
  let options: ProviderHttpServerOptions | undefined
  let closes = 0
  const handle = Object.freeze({}) as HttpServerHandle
  const httpServer: HttpServer = Object.freeze({
    async listen(value) {
      options = value
      return httpServerSuccess(handle)
    },
    async close() {
      closes += 1
      return httpServerSuccess(undefined)
    },
  })
  const execution = createEffectExecution()
  const started = await run(
    listen({
      port: 41290,
      handler,
    }),
    { httpServer },
    execution.context
  )
  expect(started.kind).toBe("success")
  if (options === undefined) throw new Error("server options were not captured")
  return { execution, options, providerCloses: () => closes }
}

function providerBody(value: unknown): ProviderHttpServerStreamBody {
  if (
    typeof value !== "object" ||
    value === null ||
    !("kind" in value) ||
    value.kind !== "stream"
  ) {
    throw new Error("expected provider streaming body")
  }
  return value as ProviderHttpServerStreamBody
}

describe("HTTP server streaming response bridge", () => {
  test("keeps the request scope open until the provider completes transfer", async () => {
    let pulls = 0
    let cursorCloses = 0
    const source = fromPull<unknown, never, Bytes>(() => ({
      async pull() {
        pulls += 1
        return pulls === 1
          ? {
              done: false as const,
              value: fromUint8Array(new Uint8Array([1, 2, 3])),
            }
          : { done: true as const, value: undefined }
      },
      close() {
        cursorCloses += 1
      },
    }))
    const selected = await fixture(() => streamResponse(200, [], source))
    const response = await selected.options.handler({
      method: "GET",
      url: "http://127.0.0.1/events",
      headers: [],
      body: new Uint8Array(),
    })
    const body = providerBody(response.body)

    expect(await body.pull()).toEqual({
      done: false,
      value: new Uint8Array([1, 2, 3]),
    })
    expect(await body.pull()).toEqual({ done: true, value: undefined })
    expect(cursorCloses).toBe(0)
    await body.complete()
    expect(cursorCloses).toBe(1)

    await selected.execution.close()
    expect(selected.providerCloses()).toBe(1)
  })

  test("cancels and closes the body cursor on client disconnect", async () => {
    let cursorCloses = 0
    const source = fromPull<unknown, never, Bytes>(() => ({
      async pull() {
        return {
          done: false as const,
          value: fromUint8Array(new Uint8Array([9])),
        }
      },
      close() {
        cursorCloses += 1
      },
    }))
    const selected = await fixture(() => streamResponse(200, [], source))
    const response = await selected.options.handler({
      method: "GET",
      url: "http://127.0.0.1/events",
      headers: [],
      body: new Uint8Array(),
    })
    const body = providerBody(response.body)

    expect((await body.pull()).done).toBe(false)
    await body.cancel()
    await body.cancel()
    expect(cursorCloses).toBe(1)

    await selected.execution.close()
    expect(selected.providerCloses()).toBe(1)
  })
})
