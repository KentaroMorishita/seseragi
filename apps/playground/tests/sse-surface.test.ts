import { describe, expect, test } from "bun:test"
import { fromUint8Array, toUint8Array } from "../../../runtime/ts/src/bytes"
import { fail, run } from "../../../runtime/ts/src/effect"
import {
  appendHeader,
  emptyHeaders,
  get,
  Http1_1,
  type HttpEvent,
  headerEntries,
  parseUrl,
  type Request,
  ResponseBodyChunk,
  ResponseStarted,
  request,
  status,
} from "../../../runtime/ts/src/http-client"
import {
  decodeLimit,
  defaultDecodeLimit,
  encode,
  event,
  eventData,
  eventId,
  eventName,
  eventRetryMillis,
  events,
  keepAlive,
  withEventName,
  withId,
  withLastEventId,
  withRetryMillis,
} from "../../../runtime/ts/src/sse"
import {
  fromArray,
  fromPull,
  runCollect,
  take,
} from "../../../runtime/ts/src/stream"
import type { Either } from "../../../runtime/ts/src/sum"

function right<Value>(result: Either<unknown, Value>): Value {
  if (result.tag !== "Right") throw new Error("expected Right")
  return result.value as Value
}

function responseHead(contentType = "text/event-stream; charset=utf-8") {
  return {
    version: Http1_1,
    status: right(status(200)),
    headers: right(appendHeader("content-type", contentType, emptyHeaders)),
  }
}

function bodyChunk(bytes: Uint8Array): HttpEvent {
  return ResponseBodyChunk(fromUint8Array(bytes))
}

describe("std/sse portable stream adapter", () => {
  test("encodes multiline events, explicit keepalive, and Last-Event-ID", () => {
    const named = right(withEventName("update", event("first\r\nsecond\n")))
    const identified = right(withId("event-42", named))
    const value = right(withRetryMillis(1500, identified))

    expect(new TextDecoder().decode(toUint8Array(encode(value)))).toBe(
      "event: update\nid: event-42\nretry: 1500\ndata: first\ndata: second\ndata: \n\n"
    )
    expect(
      new TextDecoder().decode(toUint8Array(right(keepAlive("still here"))))
    ).toBe(": still here\n\n")
    expect(withId("bad\nid", value)).toEqual({
      tag: "Left",
      value: { tag: "InvalidSseEventId", value: "bad\nid" },
    })

    const url = right(parseUrl("https://example.test/events"))
    const updated = right<Request>(
      withLastEventId("event-42", request(get, url))
    )
    expect(headerEntries(updated.headers)).toEqual([
      ["last-event-id", "event-42"],
    ])
  })

  test("parses split UTF-8, CRLF, comments, multiline data, id, and retry", async () => {
    const bytes = new TextEncoder().encode(
      "\ufeff: keepalive\r\ndata: first\r\ndata: 海\r\nevent: update\r\nid: event-42\r\nretry: 1500\r\n\r\n"
    )
    const split = bytes.indexOf(0xe6) + 1
    const source = fromArray<HttpEvent>([
      ResponseStarted(responseHead()),
      bodyChunk(bytes.slice(0, split)),
      bodyChunk(bytes.slice(split)),
    ])

    const result = await run(
      runCollect(events(defaultDecodeLimit(), source)),
      {}
    )

    expect(result.kind).toBe("success")
    if (result.kind !== "success") throw new Error("SSE parse failed")
    expect(result.value).toHaveLength(1)
    const parsed = result.value[0]
    if (parsed === undefined) throw new Error("missing parsed event")
    expect(eventData(parsed)).toBe("first\n海")
    expect(eventName(parsed)).toEqual({ tag: "Just", value: "update" })
    expect(eventId(parsed)).toEqual({ tag: "Just", value: "event-42" })
    expect(eventRetryMillis(parsed)).toEqual({ tag: "Just", value: 1500 })
  })

  test("keeps remote end normal and discards an incomplete final block", async () => {
    const source = fromArray<HttpEvent>([
      ResponseStarted(responseHead()),
      bodyChunk(new TextEncoder().encode("data: complete\n\ndata: partial")),
    ])

    const result = await run(
      runCollect(events(defaultDecodeLimit(), source)),
      {}
    )

    expect(result.kind).toBe("success")
    if (result.kind !== "success") throw new Error("SSE parse failed")
    expect(result.value.map(eventData)).toEqual(["complete"])
  })

  test("reports parser failures separately from the HTTP failure channel", async () => {
    const malformedRetry = fromArray<HttpEvent>([
      ResponseStarted(responseHead()),
      bodyChunk(new TextEncoder().encode("retry: later\n\n")),
    ])
    expect(
      await run(runCollect(events(defaultDecodeLimit(), malformedRetry)), {})
    ).toEqual({
      kind: "failure",
      error: {
        tag: "Right",
        value: { tag: "SseMalformedRetry", value: "later" },
      },
    })

    const wrongType = fromArray<HttpEvent>([
      ResponseStarted(responseHead("application/json")),
    ])
    expect(
      await run(runCollect(events(defaultDecodeLimit(), wrongType)), {})
    ).toEqual({
      kind: "failure",
      error: {
        tag: "Right",
        value: {
          tag: "SseInvalidContentType",
          value: "application/json",
        },
      },
    })

    const tenBytes = right<ReturnType<typeof defaultDecodeLimit>>(
      decodeLimit(10)
    )
    const oversized = fromArray<HttpEvent>([
      ResponseStarted(responseHead()),
      bodyChunk(new TextEncoder().encode("data: 海\n\n")),
    ])
    expect(await run(runCollect(events(tenBytes, oversized)), {})).toEqual({
      kind: "failure",
      error: {
        tag: "Right",
        value: { tag: "SseEventTooLarge", value: 10 },
      },
    })

    const malformedId = fromArray<HttpEvent>([
      ResponseStarted(responseHead()),
      bodyChunk(new Uint8Array([0x69, 0x64, 0x3a, 0x20, 0x00, 0x0a, 0x0a])),
    ])
    expect(
      await run(runCollect(events(defaultDecodeLimit(), malformedId)), {})
    ).toEqual({
      kind: "failure",
      error: { tag: "Right", value: { tag: "SseMalformedId" } },
    })

    const invalidUtf8 = fromArray<HttpEvent>([
      ResponseStarted(responseHead()),
      bodyChunk(new Uint8Array([0x64, 0x61, 0x74, 0x61, 0x3a, 0x20, 0xff])),
    ])
    expect(
      await run(runCollect(events(defaultDecodeLimit(), invalidUtf8)), {})
    ).toEqual({
      kind: "failure",
      error: { tag: "Right", value: { tag: "SseInvalidUtf8" } },
    })
  })

  test("preserves an underlying HTTP stream failure in Left", async () => {
    let index = 0
    const source = fromPull<unknown, string, HttpEvent>(() => ({
      async pull(context) {
        index += 1
        if (index === 1) {
          return { done: false, value: ResponseStarted(responseHead()) }
        }
        return await fail("network disconnected")({}, context)
      },
      close() {},
    }))

    expect(
      await run(runCollect(events(defaultDecodeLimit(), source)), {})
    ).toEqual({
      kind: "failure",
      error: { tag: "Left", value: "network disconnected" },
    })
  })

  test("closes the HTTP event cursor when downstream cancels early", async () => {
    let index = 0
    let closes = 0
    const values: ReadonlyArray<HttpEvent> = [
      ResponseStarted(responseHead()),
      bodyChunk(new TextEncoder().encode("data: first\n\ndata: second\n\n")),
    ]
    const source = fromPull<unknown, never, HttpEvent>(() => ({
      async pull() {
        return index < values.length
          ? { done: false as const, value: values[index++] as HttpEvent }
          : { done: true as const, value: undefined }
      },
      close() {
        closes += 1
      },
    }))

    const result = await run(
      runCollect(take(1, events(defaultDecodeLimit(), source))),
      {}
    )

    expect(result.kind).toBe("success")
    expect(closes).toBe(1)
  })
})
