import { expect, test, describe } from "bun:test"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import { TypeInferenceSystem } from "../src/type-inference"
import * as AST from "../src/ast"

describe("List Syntax Sugar", () => {
  function parseAndGenerate(source: string): string {
    const parser = new Parser(source)
    const parseResult = parser.parse()

    // Type inference
    const typeInference = new TypeInferenceSystem()
    const program = new AST.Program(parseResult.statements || [])
    typeInference.infer(program)

    return generateTypeScript(parseResult.statements || [], {
      indent: "  ",
      runtimeMode: "embedded",
    })
  }

  function parseExpression(source: string): AST.Expression {
    const parser = new Parser(source)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements || [])
    if (program.statements.length === 0) {
      throw new Error("No statements found")
    }
    const stmt = program.statements[0]
    if (!(stmt instanceof AST.ExpressionStatement)) {
      throw new Error("First statement is not an expression statement")
    }
    return stmt.expression
  }

  test("parses empty list sugar `[]", () => {
    const expr = parseExpression("`[]")
    expect(expr).toBeInstanceOf(AST.ListSugar)
    const listSugar = expr as AST.ListSugar
    expect(listSugar.elements).toHaveLength(0)
  })

  test("parses list sugar with elements `[1, 2, 3]", () => {
    const expr = parseExpression("`[1, 2, 3]")
    expect(expr).toBeInstanceOf(AST.ListSugar)
    const listSugar = expr as AST.ListSugar
    expect(listSugar.elements).toHaveLength(3)

    // Check elements are literals
    listSugar.elements.forEach((elem, i) => {
      expect(elem).toBeInstanceOf(AST.Literal)
      const literal = elem as AST.Literal
      expect(literal.value).toBe(i + 1)
    })
  })

  test("parses cons expression 1 : 2", () => {
    const expr = parseExpression("1 : 2")
    expect(expr).toBeInstanceOf(AST.BinaryOperation)
    const binOp = expr as AST.BinaryOperation
    expect(binOp.operator).toBe(":")

    expect(binOp.left).toBeInstanceOf(AST.Literal)
    expect((binOp.left as AST.Literal).value).toBe(1)

    expect(binOp.right).toBeInstanceOf(AST.Literal)
    expect((binOp.right as AST.Literal).value).toBe(2)
  })

  test("parses right-associative cons expression 1 : 2 : 3", () => {
    const expr = parseExpression("1 : 2 : 3")
    expect(expr).toBeInstanceOf(AST.BinaryOperation)
    const binOp = expr as AST.BinaryOperation
    expect(binOp.operator).toBe(":")

    // Should be parsed as 1 : (2 : 3)
    expect(binOp.left).toBeInstanceOf(AST.Literal)
    expect((binOp.left as AST.Literal).value).toBe(1)

    expect(binOp.right).toBeInstanceOf(AST.BinaryOperation)
    const rightBinOp = binOp.right as AST.BinaryOperation
    expect(rightBinOp.operator).toBe(":")
    expect((rightBinOp.left as AST.Literal).value).toBe(2)
    expect((rightBinOp.right as AST.Literal).value).toBe(3)
  })

  test("parses mixed cons and list sugar 1 : `[2, 3]", () => {
    const expr = parseExpression("1 : `[2, 3]")
    expect(expr).toBeInstanceOf(AST.BinaryOperation)
    const binOp = expr as AST.BinaryOperation
    expect(binOp.operator).toBe(":")

    expect(binOp.left).toBeInstanceOf(AST.Literal)
    expect((binOp.left as AST.Literal).value).toBe(1)

    expect(binOp.right).toBeInstanceOf(AST.ListSugar)
    const listSugar = binOp.right as AST.ListSugar
    expect(listSugar.elements).toHaveLength(2)
  })

  test("generates code for empty list sugar", () => {
    const code = parseAndGenerate("`[]")
    expect(code).toContain("Empty")
  })

  test("generates code for list sugar with elements", () => {
    const code = parseAndGenerate("`[1, 2, 3]")
    expect(code).toContain("Cons(1, Cons(2, Cons(3, Empty)))")
  })

  test("generates code for cons expression", () => {
    const code = parseAndGenerate("1 : 2")
    expect(code).toContain("Cons(1, 2)")
  })

  test("generates code for right-associative cons", () => {
    const code = parseAndGenerate("1 : 2 : 3")
    expect(code).toContain("Cons(1, Cons(2, 3))")
  })

  test("generates code for mixed cons and list sugar", () => {
    const code = parseAndGenerate("1 : `[2, 3]")
    expect(code).toContain("Cons(1, Cons(2, Cons(3, Empty)))")
  })

  test("distinguishes arrays from lists", () => {
    const arrayCode = parseAndGenerate("[1, 2, 3]")
    const listCode = parseAndGenerate("`[1, 2, 3]")

    // Arrays should generate JavaScript arrays
    expect(arrayCode).toContain("[1, 2, 3]")

    // Lists should generate Cons calls
    expect(listCode).toContain("Cons(1, Cons(2, Cons(3, Empty)))")

    // They should be different
    expect(arrayCode).not.toEqual(listCode)
  })

  test("handles nested list sugar", () => {
    const code = parseAndGenerate("`[`[1, 2], `[3, 4]]")
    expect(code).toContain(
      "Cons(Cons(1, Cons(2, Empty)), Cons(Cons(3, Cons(4, Empty)), Empty))"
    )
  })

  test("handles string elements in list sugar", () => {
    const code = parseAndGenerate('`["hello", "world"]')
    expect(code).toContain('Cons("hello", Cons("world", Empty))')
  })

  test("handles variable declaration with list sugar", () => {
    const code = parseAndGenerate("let myList = `[1, 2, 3]")
    expect(code).toContain("const myList = Cons(1, Cons(2, Cons(3, Empty)))")
  })

  test("handles function parameter with list sugar", () => {
    const code = parseAndGenerate(
      "fn process list = length list\nprocess `[1, 2, 3]"
    )
    expect(code).toContain("Cons(1, Cons(2, Cons(3, Empty)))")
  })
})
