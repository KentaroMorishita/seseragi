import { describe, expect, test } from "bun:test"
import { lex } from "../src/lexer"
import { parse } from "../src/parser"
import { infer } from "../src/type-inference"
import { generateTypeScript } from "../src/codegen"

describe("Simple Array↔List Conversion Test", () => {
  test("basic arrayToList and listToArray", () => {
    const code = `
let arr = [1, 2, 3]
let list = arrayToList arr
let back = listToArray list
`

    // Lexing
    const tokens = lex(code)
    console.log(
      "Tokens:",
      tokens.slice(0, 10).map((t) => ({ type: t.type, value: t.value }))
    )

    // Parsing
    const parseResult = parse(tokens)
    console.log("Parse errors:", parseResult.errors)
    console.log("Statements:", parseResult.statements?.length)

    expect(parseResult.errors).toEqual([])
    expect(parseResult.statements).toBeDefined()

    // Type inference
    const typeResult = infer(parseResult.statements!)
    console.log("Type errors:", typeResult.errors)

    expect(typeResult.errors).toEqual([])

    // Code generation
    const generated = generateTypeScript(parseResult.statements!, {
      runtimeMode: "embedded",
    })

    console.log("Generated code (first 500 chars):")
    console.log(generated.substring(0, 500))

    // Check that the generated code includes our functions
    expect(generated).toContain("arrayToList")
    expect(generated).toContain("listToArray")
  })
})
