import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"
import { infer } from "../src/inference/engine/infer"
import { TypeChecker } from "../src/typechecker"

describe("Lambda Expression Tests", () => {
  test("should parse simple lambda expression", () => {
    const parser = new Parser("\\x -> x + 1")
    const parseResult = parser.parse()

    expect(parseResult.statements).toHaveLength(1)
    const stmt = parseResult.statements![0] as AST.ExpressionStatement
    const lambda = stmt.expression as AST.LambdaExpression

    expect(lambda.kind).toBe("LambdaExpression")
    expect(lambda.parameters).toHaveLength(1)
    expect(lambda.parameters[0].name).toBe("x")
    expect(lambda.body.kind).toBe("BinaryOperation")
  })

  test("should parse lambda with type annotation", () => {
    const parser = new Parser("\\x: Int -> x * 2")
    const program = parser.parse()

    const stmt = program.statements[0] as AST.ExpressionStatement
    const lambda = stmt.expression as AST.LambdaExpression

    expect(lambda.parameters[0].type.kind).toBe("PrimitiveType")
    expect((lambda.parameters[0].type as AST.PrimitiveType).name).toBe("Int")
  })

  test("should parse curried lambda", () => {
    const parser = new Parser("\\x -> \\y -> x + y")
    const program = parser.parse()

    const stmt = program.statements[0] as AST.ExpressionStatement
    const outerLambda = stmt.expression as AST.LambdaExpression

    expect(outerLambda.parameters).toHaveLength(1)
    expect(outerLambda.body.kind).toBe("LambdaExpression")

    const innerLambda = outerLambda.body as AST.LambdaExpression
    expect(innerLambda.parameters).toHaveLength(1)
    expect(innerLambda.body.kind).toBe("BinaryOperation")
  })

  test("should type check lambda expressions", () => {
    const parser = new Parser("let f = \\x: Int -> x + 1")
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements!)
    const typeChecker = new TypeChecker()
    const errors = typeChecker.check(program)

    expect(errors).toHaveLength(0)
  })

  test("should infer lambda types", () => {
    const parser = new Parser("let f = \\x -> x + 1")
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements!)
    const result = infer(program)

    expect(result.errors).toHaveLength(0)
  })

  test("should handle lambda application", () => {
    const parser = new Parser("let result = (\\x: Int -> x + 1) 5")
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements!)
    const typeChecker = new TypeChecker()
    const errors = typeChecker.check(program)

    expect(errors).toHaveLength(0)
  })

  test("should generate correct TypeScript for lambda", () => {
    const parser = new Parser("let f = \\x -> x + 1")
    const parseResult = parser.parse()

    const code = generateTypeScript(parseResult.statements!)

    expect(code).toContain('(x: any) => __dispatchOperator(x, "+", 1)')
  })

  test("should generate correct TypeScript for lambda application", () => {
    const parser = new Parser("let result = (\\x -> x + 1) 5")
    const parseResult = parser.parse()

    const code = generateTypeScript(parseResult.statements!)

    expect(code).toContain('((x: any) => __dispatchOperator(x, "+", 1))(5)')
  })

  test("should generate correct TypeScript for curried lambda", () => {
    const parser = new Parser("let add = \\x -> \\y -> x + y")
    const parseResult = parser.parse()

    const code = generateTypeScript(parseResult.statements!)

    expect(code).toContain(
      '(x: any) => (y: any) => __dispatchOperator(x, "+", y)'
    )
  })

  test("should handle lambda with explicit types", () => {
    const parser = new Parser("let typed = \\x: Int -> x + 100")
    const parseResult = parser.parse()

    const code = generateTypeScript(parseResult.statements!)

    expect(code).toContain('(x: number) => __dispatchOperator(x, "+", 100)')
  })
})
