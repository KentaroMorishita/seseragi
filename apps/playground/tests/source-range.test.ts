import { describe, expect, test } from "bun:test"
import {
  describeSourceLocation,
  formatSourceLocation,
  utf16OffsetToUtf8Byte,
  utf16OffsetToSourcePosition,
  utf8ByteOffsetToUtf16,
  utf8RangeToUtf16,
  utf8RangeToSourceLocation,
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
    expect(utf16OffsetToUtf8Byte(source, 1)).toBe(1)
    expect(utf16OffsetToUtf8Byte(source, 2)).toBe(4)
    expect(utf16OffsetToUtf8Byte(source, 4)).toBe(8)
  })

  test("clamps offsets beyond the document", () => {
    expect(utf8ByteOffsetToUtf16("瀬", 99)).toBe(1)
  })

  test("reports one-based human positions for ASCII, Japanese, and emoji", () => {
    const source = "first\n瀬🌊 value\nlast"
    expect(
      utf16OffsetToSourcePosition(source, source.indexOf("value"))
    ).toEqual({
      line: 2,
      column: 4,
    })

    const start = new TextEncoder().encode("first\n瀬🌊 ").length
    const end = start + new TextEncoder().encode("value").length
    const location = utf8RangeToSourceLocation(source, { start, end })
    expect(formatSourceLocation("main.ssrg", location)).toBe("main.ssrg:2:4–9")
    expect(describeSourceLocation(location)).toBe("Line 2, columns 4–9")
  })

  test("formats multi-line locations without exposing byte offsets", () => {
    const source = "one\ntwo\nthree"
    const location = utf8RangeToSourceLocation(source, { start: 2, end: 9 })
    expect(formatSourceLocation("main.ssrg", location)).toBe(
      "main.ssrg:1:3–3:2"
    )
    expect(describeSourceLocation(location)).toBe(
      "Line 1, column 3 to line 3, column 2"
    )
  })
})
