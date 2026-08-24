import { describe, expect, test } from "bun:test"
import { fromUint8Array } from "../../../runtime/ts/src/bytes"
import { run } from "../../../runtime/ts/src/effect"
import {
  appendHeader,
  bodyLimit,
  customMethod,
  defaultBodyLimit,
  emptyHeaders,
  get,
  type Headers,
  type HttpBodyLimit,
  type HttpClient,
  type HttpUrl,
  headerEntries,
  headerValues,
  isClientError,
  isSuccess,
  type Method,
  methodText,
  parseUrl,
  post,
  removeHeader,
  renderUrl,
  request,
  responseBody,
  responseHeaders,
  responseStatus,
  type Status,
  sendBytes,
  sendEmpty,
  setHeader,
  status,
  statusCode,
  withoutRequestHeader,
  withRequestHeader,
} from "../../../runtime/ts/src/http-client"

function right<Value>(result: {
  readonly tag: "Left" | "Right"
  readonly value: unknown
}): Value {
  if (result.tag !== "Right") throw new Error("expected Right")
  return result.value as Value
}

describe("std/http small-response application surface", () => {
  test("validates methods, status values, and normalized absolute URLs", () => {
    expect(methodText(get)).toBe("GET")
    expect(methodText(post)).toBe("POST")
    expect(methodText(right<Method>(customMethod("PURGE")))).toBe("PURGE")
    expect(customMethod("get")).toEqual({
      tag: "Left",
      value: { tag: "InvalidHttpMethod", value: "get" },
    })

    const created = right<Status>(status(299))
    expect(statusCode(created)).toBe(299)
    expect(isSuccess(created)).toBe(true)
    expect(isClientError(right(status(404)))).toBe(true)
    expect(status(99)).toEqual({
      tag: "Left",
      value: { tag: "InvalidHttpStatus", value: 99 },
    })

    const url = right<HttpUrl>(
      parseUrl("HTTPS://EXAMPLE.TEST:443/a/../b?x=1&x=2")
    )
    expect(renderUrl(url)).toBe("https://example.test/b?x=1&x=2")
    expect(parseUrl("ftp://example.test/file")).toEqual({
      tag: "Left",
      value: { tag: "UnsupportedHttpScheme", value: "ftp" },
    })
    expect(parseUrl("https://user@example.test/")).toEqual({
      tag: "Left",
      value: { tag: "HttpUrlContainsUserInfo" },
    })
    expect(parseUrl("https://example.test/#fragment")).toEqual({
      tag: "Left",
      value: { tag: "HttpUrlContainsFragment" },
    })
    expect(parseUrl("https://example.test/bad%escape")).toEqual({
      tag: "Left",
      value: { tag: "InvalidHttpUrl", value: { offset: 24 } },
    })
    expect(parseUrl("https://例.example.test/")).toEqual({
      tag: "Left",
      value: { tag: "InvalidHttpUrl", value: { offset: 8 } },
    })
  })

  test("keeps headers ordered, case-insensitive, immutable, and managed", () => {
    const first = right<Headers>(appendHeader("X-Trace", "one", emptyHeaders))
    const second = right<Headers>(appendHeader("x-trace", "two", first))
    const replaced = right<Headers>(setHeader("X-TRACE", "three", second))

    expect(headerValues("x-trace", second)).toEqual(["one", "two"])
    expect(headerEntries(replaced)).toEqual([["x-trace", "three"]])
    expect(headerEntries(first)).toEqual([["x-trace", "one"]])
    expect(headerEntries(removeHeader("X-Trace", second))).toEqual([])
    expect(appendHeader("connection", "close", emptyHeaders)).toEqual({
      tag: "Left",
      value: { tag: "ManagedHttpHeader", value: "connection" },
    })
    expect(appendHeader("bad name", "value", emptyHeaders)).toEqual({
      tag: "Left",
      value: { tag: "InvalidHeaderName", value: "bad name" },
    })
    expect(appendHeader("x-test", "bad\rvalue", emptyHeaders)).toEqual({
      tag: "Left",
      value: {
        tag: "InvalidHeaderValue",
        value: { name: "x-test", offset: 3 },
      },
    })
  })

  test("keeps small responses on HttpClient#send and enforces limits", async () => {
    const url = right<HttpUrl>(parseUrl("https://example.test/resource"))
    const base = request(post, url)
    const withHeader = right<ReturnType<typeof request>>(
      withRequestHeader("content-type", "application/octet-stream", base)
    )
    expect(base).not.toBe(withHeader)
    expect(withoutRequestHeader("content-type", withHeader)).toEqual(base)

    let calls = 0
    let observedBody: Uint8Array | undefined
    const client: HttpClient = Object.freeze({
      async send(value) {
        calls += 1
        observedBody = value.body
        return {
          kind: "success",
          value: {
            status: 201,
            headers: [{ name: "X-Result", value: "ready" }],
            body: new Uint8Array([4, 5, 6]),
          },
        }
      },
      exchange() {
        throw new Error("small-response wrapper must not use exchange")
      },
    })
    const sourceBody = fromUint8Array(new Uint8Array([1, 2, 3]))
    const effect = sendBytes(defaultBodyLimit(), sourceBody, withHeader)
    expect(calls).toBe(0)
    const result = await run(effect, { httpClient: client })
    expect(result.kind).toBe("success")
    expect(observedBody).toEqual(new Uint8Array([1, 2, 3]))
    expect(observedBody).not.toBe(sourceBody)
    if (result.kind !== "success") throw new Error("expected success")
    expect(statusCode(responseStatus(result.value))).toBe(201)
    expect(headerEntries(responseHeaders(result.value))).toEqual([
      ["x-result", "ready"],
    ])
    expect(Array.from(responseBody(result.value))).toEqual([4, 5, 6])

    const wrongLength = right<ReturnType<typeof request>>(
      withRequestHeader("content-length", "2", base)
    )
    const mismatch = await run(
      sendBytes(defaultBodyLimit(), sourceBody, wrongLength),
      { httpClient: client }
    )
    expect(mismatch).toEqual({
      kind: "failure",
      error: {
        tag: "HttpRequestLengthMismatch",
        value: { declared: 2, actual: 3 },
      },
    })
    expect(calls).toBe(1)

    const oneByte = right<HttpBodyLimit>(bodyLimit(1))
    const limited = await run(sendEmpty(oneByte, request(get, url)), {
      httpClient: client,
    })
    expect(limited).toEqual({
      kind: "failure",
      error: {
        tag: "HttpResponseBodyLimitExceeded",
        value: { limitBytes: 1 },
      },
    })
  })
})
