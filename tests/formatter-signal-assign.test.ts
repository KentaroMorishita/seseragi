import { describe, expect, test } from "bun:test"
import { formatSeseragiCode } from "../src/formatter"

describe("Signal assignment operator formatting", () => {
  test("should preserve := operator without spaces", () => {
    const input = `let s: Signal<Int> = Signal(0)
s:=42`
    const expected = `let s: Signal<Int> = Signal(0)
s := 42
`
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("should normalize spacing around := operator", () => {
    const input = `let s: Signal<Int> = Signal(0)
s  :=  42
result:=  "test"`
    const expected = `let s: Signal<Int> = Signal(0)
s := 42
result := "test"
`
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("should not affect other colon operators", () => {
    const input = `let x: Int = 5
let result = x > 5 ? "big" : "small"
let cons = 1 : 2 : []`
    const expected = `let x: Int = 5
let result = x > 5 ? "big" : "small"
let cons = 1 : 2 : []
`
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("should handle := in complex expressions", () => {
    const input = `fn updateSignal s: Signal<String> -> x: String -> Task<Signal<String>> = 
  Task (resolve (s := x))
let result = s:=(match value { Some x -> x | None -> "default" })`
    const expected = `fn updateSignal s: Signal<String> -> x: String -> Task<Signal<String>>  =
  Task (resolve (s := x))
let result = s := (match value { Some x -> x | None -> "default" })
`
    expect(formatSeseragiCode(input)).toBe(expected)
  })

  test("should preserve := in multiline expressions", () => {
    const input = `signal := (match x {
  Right val -> val
  Left err -> defaultValue
})`
    const expected = `signal := (match x {
  Right val -> val
  Left err -> defaultValue
  })
`
    expect(formatSeseragiCode(input)).toBe(expected)
  })
})
