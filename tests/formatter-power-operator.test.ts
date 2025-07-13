import { describe, test, expect } from "bun:test"
import { formatSeseragiCode } from "../src/formatter/relative-indent-formatter.js"

describe("Power Operator Formatting", () => {
  test("formats power operator correctly", () => {
    const input = "let x = 2**3"
    const expected = "let x = 2 ** 3\n"
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("formats power operator with spaces", () => {
    const input = "let y = a ** b"
    const expected = "let y = a ** b\n"
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("formats power operator with other operators", () => {
    const input = "let z = a*b**c"
    const expected = "let z = a * b ** c\n"
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("formats complex power expressions", () => {
    const input = "let result = (a+b)**(c-d)"
    const expected = "let result = (a + b) ** (c-d)\n"
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("formats chained power operators", () => {
    const input = "let chain = a**b**c"
    const expected = "let chain = a ** b ** c\n"
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("power operator does not interfere with string literals", () => {
    const input = 'let str = "**power**"'
    const expected = 'let str = "**power**"\n'
    expect(formatSeseragiCode(input)).toBe(expected)
  })
})
