import { describe, expect, test } from "bun:test"
import { run } from "../src/effect"
import {
  annotateForeignTask,
  createForeignTaskModule,
  type ForeignCodec,
  invokeForeignPure,
  invokeForeignTask,
  renderJsErrorDiagnostic,
} from "../src/foreign"
import { encodeUtf8, scalarAt } from "../src/text"

const invalidStrings = [
  "\ud800",
  "\udfff",
  "a\ud800b",
  "\ud800\ud800",
  "\udc00\ud800",
  "𠮷\udfff",
]
const validStrings = [
  "",
  "ASCII",
  "日本語",
  "\0",
  "\ufeff",
  "e\u0301",
  "é",
  "𠮷",
  "😀𠮷",
  "a😀b\0\ufeff",
]
const read = (value: unknown, codec: ForeignCodec = "string") =>
  invokeForeignPure({ read: () => value }, "read", "function", [], [], codec)

describe("foreign String scalar invariant", () => {
  test("rejects unpaired surrogates on ordinary String input and output", () => {
    for (const value of invalidStrings) {
      expect(() => read(value)).toThrow(
        "return value of read must be Unicode scalar string"
      )
      let called = false
      expect(() =>
        invokeForeignPure(
          {
            write() {
              called = true
            },
          },
          "write",
          "function",
          [value],
          ["string"],
          "unit"
        )
      ).toThrow("argument 1 must be Unicode scalar string")
      expect(called).toBe(false)
      expect(read(value, "js-string")).toBe(value)
    }
  })

  test("preserves valid UTF-16 bytes and scalar/UTF-8 observations", () => {
    for (const value of validStrings) {
      const converted = read(value) as string
      expect(converted).toBe(value)
      expect(
        Array.from({ length: converted.length }, (_, index) =>
          converted.charCodeAt(index)
        )
      ).toEqual(
        Array.from({ length: value.length }, (_, index) =>
          value.charCodeAt(index)
        )
      )
      expect(encodeUtf8(converted)).toEqual(new TextEncoder().encode(value))
      for (const [index, scalar] of [...value].entries()) {
        expect(scalarAt(index, converted)).toEqual({
          tag: "Just",
          value: scalar,
        })
      }
    }
  })

  test("retains nested record and array error paths", () => {
    const codec: ForeignCodec = { array: { record: { text: "string" } } }
    expect(() => read([{ text: "ok" }, { text: "\ud800" }], codec)).toThrow(
      "return value of read[1].text must be Unicode scalar string"
    )
    expect(read([{ text: "😀" }], codec)).toEqual([{ text: "😀" }])
    expect(() =>
      read(
        { tag: "Box", value: "\udc00" },
        { record: { tag: "string", value: "string" } }
      )
    ).toThrow("return value of read.value must be Unicode scalar string")
  })

  test("validates callback arguments and results through the shared codec", () => {
    const callback: ForeignCodec = {
      callback: { parameters: ["string"], result: "string" },
    }
    let called = false
    try {
      invokeForeignPure(
        { call: (f: (value: string) => string) => f("\ud800") },
        "call",
        "function",
        [
          () => {
            called = true
            return "ok"
          },
        ],
        [callback],
        "string"
      )
      throw new Error("expected callback conversion error")
    } catch (error) {
      expect((error as Error).cause).toBeInstanceOf(TypeError)
      expect(String((error as Error).cause)).toContain("Unicode scalar string")
    }
    expect(called).toBe(false)
    expect(() =>
      invokeForeignPure(
        { call: (f: (value: string) => string) => f("valid") },
        "call",
        "function",
        [() => "\udfff"],
        [callback],
        "string"
      )
    ).toThrow("foreign pure binding call threw")
    const returned = read((value: string) => value, callback) as (
      value: string
    ) => string
    expect(() => returned("\ud800")).toThrow("Unicode scalar string")
    expect(returned("😀")).toBe("😀")
  })

  test("rejects invalid task arguments before host invocation", async () => {
    let calls = 0
    const module = createForeignTaskModule(async () => ({
      write: (value: string) => {
        calls++
        return value
      },
    }))
    for (const value of invalidStrings) {
      const result = await run(
        invokeForeignTask(
          module,
          "write",
          "function",
          [value],
          ["string"],
          "string"
        ),
        {}
      )
      expect(result).toMatchObject({
        kind: "failure",
        error: {
          phase: "SynchronousThrow",
          message: "argument 1 must be Unicode scalar string",
        },
      })
    }
    expect(calls).toBe(0)
    for (const value of validStrings) {
      expect(
        await run(
          invokeForeignTask(
            module,
            "write",
            "function",
            [value],
            ["string"],
            "string"
          ),
          {}
        )
      ).toEqual({ kind: "success", value })
    }
  })

  test("keeps task conversion phase, member path and source provenance", async () => {
    const module = createForeignTaskModule(async () => ({
      api: { read: async () => [{ text: "\ud800" }] },
    }))
    const effect = annotateForeignTask(
      invokeForeignTask(module, ["api", "read"], "function", [], [], {
        array: { record: { text: "string" } },
      }),
      {
        language: "seseragi",
        function: "main",
        uri: "seseragi://fixture/foreign-string/main",
        range: { start: 10, end: 20 },
        generated: false,
      }
    )
    const result = await run(effect, {})
    expect(result).toMatchObject({
      kind: "failure",
      error: {
        phase: "SynchronousThrow",
        message:
          "return value of api.read[0].text must be Unicode scalar string",
      },
    })
    if (result.kind !== "failure") return
    const diagnostic = renderJsErrorDiagnostic(result.error, () => undefined)
    expect(diagnostic).toContain("seseragi://fixture/foreign-string/main")
    expect(diagnostic).toContain('"start":10')
  })
})
