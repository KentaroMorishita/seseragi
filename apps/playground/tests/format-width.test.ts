import { describe, expect, test } from "bun:test"
import {
  maximumAutoFormatWidth,
  minimumAutoFormatWidth,
  resolveAutoFormatWidth,
} from "../src/editor/format-width"

describe("adaptive formatter width", () => {
  test("derives columns from the editor content box and character width", () => {
    expect(
      resolveAutoFormatWidth({
        contentWidth: 640,
        characterWidth: 8,
        paddingInline: 16,
      })
    ).toBe(78)
  })

  test("clamps narrow and wide editors to the Auto contract", () => {
    expect(
      resolveAutoFormatWidth({
        contentWidth: 220,
        characterWidth: 8,
        paddingInline: 16,
      })
    ).toBe(minimumAutoFormatWidth)
    expect(
      resolveAutoFormatWidth({
        contentWidth: 1200,
        characterWidth: 8,
        paddingInline: 16,
      })
    ).toBe(maximumAutoFormatWidth)
  })

  test("falls back to canonical width before a hidden editor is measurable", () => {
    expect(
      resolveAutoFormatWidth({
        contentWidth: 0,
        characterWidth: 0,
        paddingInline: 0,
      })
    ).toBe(88)
  })
})
