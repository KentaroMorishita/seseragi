import { expect, test } from "bun:test"
import {
  EmptyInt,
  IntOutsideRange,
  InvalidIntDigit,
  InvalidIntRadix,
  parse,
} from "../src/int"
import { intParseErrorShow, renderShow } from "../src/show"

test("IntParseError Show keeps constructor payloads and document layout", () => {
  for (const [error, expected] of [
    [EmptyInt, "EmptyInt"],
    [InvalidIntRadix(1), "InvalidIntRadix 1"],
    [
      InvalidIntDigit({ offset: 1, radix: 10 }),
      "InvalidIntDigit { offset: 1, radix: 10 }",
    ],
    [IntOutsideRange, "IntOutsideRange"],
  ] as const) {
    expect(intParseErrorShow.show(error)).toBe(expected)
    expect(renderShow(intParseErrorShow, error, { layout: "compact" })).toBe(
      expected
    )
  }
  const parsed = parse("1🙂")
  expect(parsed.tag).toBe("Left")
  if (parsed.tag !== "Left") throw new Error("expected a typed parse failure")
  expect(renderShow(intParseErrorShow, parsed.value)).toBe(
    "InvalidIntDigit { offset: 1, radix: 10 }"
  )
  expect(
    renderShow(intParseErrorShow, parsed.value, { layout: "multiline" })
  ).toBe("InvalidIntDigit {\n  offset: 1,\n  radix: 10\n}")
})
