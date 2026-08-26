import { describe, expect, test } from "bun:test"
import {
  type Bytes,
  fromUint8Array,
  toUint8Array,
} from "../../../runtime/ts/src/bytes"
import { run, unit } from "../../../runtime/ts/src/effect"
import { bytesBody, streamBody } from "../../../runtime/ts/src/http-client"
import {
  appendBody,
  appendBytes,
  appendText,
  contentType,
  empty,
  body as multipartBody,
} from "../../../runtime/ts/src/multipart"
import {
  fromPull,
  runCollect,
  type Stream,
  take,
} from "../../../runtime/ts/src/stream"
import type { Either } from "../../../runtime/ts/src/sum"
import { Just, Nothing } from "../../../runtime/ts/src/sum"
import {
  asBlob,
  body as blobBody,
  fromBytes,
  lastModifiedMillis,
  mimeType,
  name,
  readBytes,
  readChunks,
  sizeBytes,
  wrapFile,
} from "../../../runtime/ts/src/web-file"

function right<Value>(result: Either<unknown, Value>): Value {
  if (result.tag !== "Right") throw new Error("expected Right")
  return result.value
}

async function collectText<Environment, Failure>(
  stream: Stream<Environment, Failure, Bytes>,
  environment: Environment
): Promise<string> {
  const result = await run(runCollect(stream), environment)
  if (result.kind !== "success") throw new Error("expected stream success")
  const chunks = result.value.map(toUint8Array)
  const length = chunks.reduce((total, chunk) => total + chunk.length, 0)
  const output = new Uint8Array(length)
  let offset = 0
  for (const chunk of chunks) {
    output.set(chunk, offset)
    offset += chunk.length
  }
  return new TextDecoder().decode(output)
}

describe("std/http/multipart portable streaming encoder", () => {
  test("owns its boundary and streams text, bytes, and Body parts in order", async () => {
    const first = right(appendText("title", "Seseragi", empty(unit)))
    const second = right(
      appendBytes(
        "small",
        Just("small.txt"),
        Just("text/plain"),
        fromUint8Array(new TextEncoder().encode("bytes")),
        first
      )
    )
    const value = right(
      appendBody(
        "large",
        Just("large.bin"),
        Just("application/octet-stream"),
        bytesBody(fromUint8Array(new Uint8Array([0, 1, 2]))),
        second
      )
    )
    const type = contentType(value)
    const boundary = type.slice("multipart/form-data; boundary=".length)
    const wire = await collectText(multipartBody(value).stream, {})

    expect(boundary).toMatch(/^seseragi-[0-9a-f]{36}$/)
    expect(wire).toBe(
      [
        `--${boundary}\r\n`,
        'Content-Disposition: form-data; name="title"\r\n',
        "Content-Type: text/plain; charset=utf-8\r\n\r\n",
        "Seseragi\r\n",
        `--${boundary}\r\n`,
        'Content-Disposition: form-data; name="small"; filename="small.txt"\r\n',
        "Content-Type: text/plain\r\n\r\n",
        "bytes\r\n",
        `--${boundary}\r\n`,
        'Content-Disposition: form-data; name="large"; filename="large.bin"\r\n',
        "Content-Type: application/octet-stream\r\n\r\n",
        "\u0000\u0001\u0002\r\n",
        `--${boundary}--\r\n`,
      ].join("")
    )
  })

  test("rejects unsafe disposition values and implicit MIME guesses", () => {
    const value = empty(unit)
    expect(appendText("bad\r\nname", "x", value)).toEqual({
      tag: "Left",
      value: { tag: "InvalidMultipartFieldName", value: "bad\r\nname" },
    })
    expect(
      appendBytes(
        "file",
        Just("bad\nname"),
        Nothing,
        fromUint8Array(new Uint8Array()),
        value
      )
    ).toEqual({
      tag: "Left",
      value: { tag: "InvalidMultipartFileName", value: "bad\nname" },
    })
    expect(
      appendBytes(
        "file",
        Nothing,
        Just("not mime"),
        fromUint8Array(new Uint8Array()),
        value
      )
    ).toEqual({
      tag: "Left",
      value: { tag: "InvalidMultipartMimeType", value: "not mime" },
    })
  })

  test("pulls one Body chunk at a time and closes the active part on cancellation", async () => {
    let pulls = 0
    let closes = 0
    const source = fromPull<unknown, never, Bytes>(() => ({
      async pull() {
        pulls += 1
        return {
          done: false,
          value: fromUint8Array(new Uint8Array([pulls])),
        }
      },
      close() {
        closes += 1
      },
    }))
    const value = right(
      appendBody("upload", Nothing, Nothing, streamBody(source), empty(unit))
    )

    const result = await run(
      runCollect(take(2, multipartBody(value).stream)),
      {}
    )
    expect(result.kind).toBe("success")
    expect(pulls).toBe(1)
    expect(closes).toBe(1)
  })
})

describe("std/web/file opaque browser handles", () => {
  test("snapshots metadata and supports bounded and streaming reads", async () => {
    const native = new File(["hello"], "hello.txt", {
      type: "text/plain",
      lastModified: 1234,
    })
    const file = wrapFile(native)
    const blob = asBlob(file)

    expect(name(file)).toBe("hello.txt")
    expect(mimeType(blob)).toEqual(Just(native.type))
    expect(sizeBytes(blob)).toBe(5)
    expect(lastModifiedMillis(file)).toBe(1234)

    const bounded = await run(readBytes(5, blob), {})
    expect(bounded.kind).toBe("success")
    if (bounded.kind !== "success") throw new Error("expected bounded read")
    expect(new TextDecoder().decode(toUint8Array(bounded.value))).toBe("hello")
    expect(await run(readBytes(4, blob), {})).toEqual({
      kind: "failure",
      error: {
        tag: "BlobReadLimitExceeded",
        value: { limitBytes: 4, sizeBytes: 5 },
      },
    })
    expect(await collectText(readChunks(blob), {})).toBe("hello")
    expect(await collectText(blobBody(blob).stream, {})).toBe("hello")
  })

  test("creates checked blobs only with an explicit valid MIME type", () => {
    const content = fromUint8Array(new TextEncoder().encode("data"))
    const plain = right(fromBytes(Nothing, content))
    expect(mimeType(plain)).toEqual(Nothing)
    expect(fromBytes(Just(""), content)).toEqual({
      tag: "Left",
      value: { tag: "InvalidBlobMimeType", value: "" },
    })
    expect(fromBytes(Just("text/plain"), content).tag).toBe("Right")
    expect(fromBytes(Just("text/plain; charset=utf-8"), content).tag).toBe(
      "Right"
    )
  })

  test("bounds every large-file stream chunk to 64 KiB", async () => {
    const native = new File([new Uint8Array(130 * 1024)], "large.bin")
    const result = await run(
      runCollect(readChunks(asBlob(wrapFile(native)))),
      {}
    )
    expect(result.kind).toBe("success")
    if (result.kind !== "success") throw new Error("expected streamed read")
    const lengths = result.value.map((chunk) => chunk.length)
    expect(lengths.every((length) => length <= 64 * 1024)).toBe(true)
    expect(lengths.reduce((total, length) => total + length, 0)).toBe(
      130 * 1024
    )
  })
})
