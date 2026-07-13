import { describe, expect, test } from "bun:test"
import {
  utf8ByteOffsetToUtf16,
  utf8RangeToUtf16,
} from "../src/diagnostics/source-range"

describe("UTF-8 driver ranges", () => {
  test("maps ASCII byte offsets directly", () => {
    expect(utf8RangeToUtf16("hello", { start: 1, end: 4 })).toEqual({
      from: 1,
      to: 4,
    })
  })

  test("maps multibyte scalars to CodeMirror UTF-16 offsets", () => {
    const source = "a瀬🌊z"
    expect(utf8ByteOffsetToUtf16(source, 1)).toBe(1)
    expect(utf8ByteOffsetToUtf16(source, 4)).toBe(2)
    expect(utf8ByteOffsetToUtf16(source, 8)).toBe(4)
    expect(utf8ByteOffsetToUtf16(source, 9)).toBe(5)
  })

  test("clamps offsets beyond the document", () => {
    expect(utf8ByteOffsetToUtf16("瀬", 99)).toBe(1)
  })
})
