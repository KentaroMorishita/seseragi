import { describe, test, expect } from "bun:test"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"
import { CodeGenerator } from "../src/codegen"
import { Program } from "../src/ast"

describe("Unit value () literal", () => {
  describe("Basic Unit value parsing", () => {
    test("should parse () as Unit value literal", () => {
      const parser = new Parser("()")
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const expr = parseResult.statements?.[0] as any
      expect(expr.kind).toBe("ExpressionStatement")
      expect(expr.expression.kind).toBe("Literal")
      expect(expr.expression.literalType).toBe("unit")
      expect(expr.expression.value).toBe(null)
    })

    test("should parse let unit = ()", () => {
      const parser = new Parser("let unit = ()")
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const letStmt = parseResult.statements?.[0] as any
      expect(letStmt.kind).toBe("VariableDeclaration")
      expect(letStmt.name).toBe("unit")
      expect(letStmt.initializer.kind).toBe("Literal")
      expect(letStmt.initializer.literalType).toBe("unit")
    })
  })

  describe("Parameter-less lambda expressions", () => {
    test('should parse \\() -> "hello"', () => {
      const parser = new Parser('\\() -> "hello"')
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const expr = parseResult.statements?.[0] as any
      expect(expr.kind).toBe("ExpressionStatement")
      expect(expr.expression.kind).toBe("LambdaExpression")
      expect(expr.expression.parameters.length).toBe(1)
      expect(expr.expression.parameters[0].name).toBe("_unit")
      expect(expr.expression.parameters[0].type.name).toBe("Unit")
    })

    test('should parse let f = \\() -> "test"', () => {
      const parser = new Parser('let f = \\() -> "test"')
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const letStmt = parseResult.statements?.[0] as any
      expect(letStmt.kind).toBe("VariableDeclaration")
      expect(letStmt.name).toBe("f")
      expect(letStmt.initializer.kind).toBe("LambdaExpression")
      expect(letStmt.initializer.parameters[0].type.name).toBe("Unit")
    })
  })

  describe("Type inference", () => {
    test("should infer Unit type for ()", () => {
      const parser = new Parser("let unit = ()")
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)

      const program = new Program(parseResult.statements || [])
      const typeInference = new TypeInferenceSystem()
      const result = typeInference.infer(program)

      expect(result.errors.length).toBe(0)

      const letStmt = program.statements[0] as any
      const unitType = result.nodeTypeMap.get(letStmt.initializer)
      expect(unitType?.name).toBe("Unit")
    })

    test("should infer correct function type for parameter-less lambda", () => {
      const parser = new Parser('let f = \\() -> "hello"')
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)

      const program = new Program(parseResult.statements || [])
      const typeInference = new TypeInferenceSystem()
      const result = typeInference.infer(program)

      expect(result.errors.length).toBe(0)

      const letStmt = program.statements[0] as any
      const lambdaType = result.nodeTypeMap.get(letStmt.initializer)
      expect(lambdaType?.kind).toBe("FunctionType")
      expect((lambdaType as any).paramType.name).toBe("Unit")
      expect((lambdaType as any).returnType.name).toBe("String")
    })
  })

  describe("Code generation", () => {
    test("should generate Unit object for Unit value", () => {
      const parser = new Parser("let unit = ()")
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)

      const program = new Program(parseResult.statements || [])
      const typeInference = new TypeInferenceSystem()
      const typeResult = typeInference.infer(program)
      expect(typeResult.errors.length).toBe(0)

      const codegen = new CodeGenerator({ typeInferenceResult: typeResult })
      const tsCode = codegen.generateProgram(program.statements)

      expect(tsCode).toContain("const unit = Unit")
    })

    test("should generate correct TypeScript for parameter-less lambda", () => {
      const parser = new Parser('let f = \\() -> "hello"')
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)

      const program = new Program(parseResult.statements || [])
      const typeInference = new TypeInferenceSystem()
      const typeResult = typeInference.infer(program)
      expect(typeResult.errors.length).toBe(0)

      const codegen = new CodeGenerator({ typeInferenceResult: typeResult })
      const tsCode = codegen.generateProgram(program.statements)

      // Parameter-less lambda should generate function with Unit parameter
      expect(tsCode).toContain('const f = (_unit: void) => "hello"')
    })
  })

  describe("Function calls with Unit", () => {
    test("should parse function call with no arguments f()", () => {
      const parser = new Parser("f()")
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const expr = parseResult.statements?.[0] as any
      expect(expr.kind).toBe("ExpressionStatement")
      expect(expr.expression.kind).toBe("FunctionCall")
      expect(expr.expression.function.name).toBe("f")
      expect(expr.expression.arguments.length).toBe(0) // No arguments
    })

    test("should parse function call with Unit argument f(())", () => {
      const parser = new Parser("f(())")
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const expr = parseResult.statements?.[0] as any
      expect(expr.kind).toBe("ExpressionStatement")
      expect(expr.expression.kind).toBe("FunctionCall")
      expect(expr.expression.function.name).toBe("f")
      expect(expr.expression.arguments.length).toBe(1)
      expect(expr.expression.arguments[0].literalType).toBe("unit")
    })
  })

  describe("Function definition equivalence", () => {
    test("should parse fn name -> Type", () => {
      const parser = new Parser('fn greet -> String = "hello"')
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const fnDecl = parseResult.statements?.[0] as any
      expect(fnDecl.kind).toBe("FunctionDeclaration")
      expect(fnDecl.name).toBe("greet")
      expect(fnDecl.parameters.length).toBe(0) // No explicit parameters
    })

    test("should parse fn name () -> Type", () => {
      const parser = new Parser('fn greet () -> String = "hello"')
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const fnDecl = parseResult.statements?.[0] as any
      expect(fnDecl.kind).toBe("FunctionDeclaration")
      expect(fnDecl.name).toBe("greet")
      expect(fnDecl.parameters.length).toBe(0) // Unit parameter handled implicitly
    })

    test("should parse fn name (): Unit -> Type", () => {
      const parser = new Parser('fn greet (): Unit -> String = "hello"')
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const fnDecl = parseResult.statements?.[0] as any
      expect(fnDecl.kind).toBe("FunctionDeclaration")
      expect(fnDecl.name).toBe("greet")
      expect(fnDecl.parameters.length).toBe(0) // Unit parameter handled implicitly
    })
  })

  describe("Multiple parameters with Unit", () => {
    test("should parse fn name param: Type -> () -> ReturnType", () => {
      const parser = new Parser(
        'fn greet name: String -> () -> String = "hello"'
      )
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const fnDecl = parseResult.statements?.[0] as any
      expect(fnDecl.kind).toBe("FunctionDeclaration")
      expect(fnDecl.name).toBe("greet")
      expect(fnDecl.parameters.length).toBe(2) // String param + Unit param
      expect(fnDecl.parameters[0].name).toBe("name")
      expect(fnDecl.parameters[0].type.name).toBe("String")
      expect(fnDecl.parameters[1].name).toBe("_unit")
      expect(fnDecl.parameters[1].type.name).toBe("Unit")
    })

    test("should parse fn name param: Type -> (): Unit -> ReturnType", () => {
      const parser = new Parser(
        'fn greet name: String -> (): Unit -> String = "hello"'
      )
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)
      expect(parseResult.statements?.length).toBe(1)

      const fnDecl = parseResult.statements?.[0] as any
      expect(fnDecl.kind).toBe("FunctionDeclaration")
      expect(fnDecl.name).toBe("greet")
      expect(fnDecl.parameters.length).toBe(2) // String param + Unit param
      expect(fnDecl.parameters[1].name).toBe("_unit")
      expect(fnDecl.parameters[1].type.name).toBe("Unit")
    })
  })

  describe("Runtime behavior", () => {
    test("should display Unit value as ()", () => {
      const parser = new Parser("let unit = ()")
      const parseResult = parser.parse()

      expect(parseResult.errors.length).toBe(0)

      const program = new Program(parseResult.statements || [])
      const typeInference = new TypeInferenceSystem()
      const typeResult = typeInference.infer(program)
      expect(typeResult.errors.length).toBe(0)

      const codegen = new CodeGenerator({ typeInferenceResult: typeResult })
      const tsCode = codegen.generateProgram(program.statements)

      // Should contain Unit type definition
      expect(tsCode).toContain("type Unit = { tag: 'Unit', value: undefined }")
      // Should contain Unit constant
      expect(tsCode).toContain(
        "const Unit: Unit = { tag: 'Unit', value: undefined }"
      )
      // Should contain Unit display function
      expect(tsCode).toContain("return '()'")
    })
  })

  describe("Error cases", () => {
    test("should handle malformed unit expressions gracefully", () => {
      const parser = new Parser("let broken = (")
      const parseResult = parser.parse()

      // Should have parse errors but not crash
      expect(parseResult.errors.length).toBeGreaterThan(0)
    })

    test("should handle incomplete unit parameter gracefully", () => {
      const parser = new Parser('fn test ( -> String = "test"')
      const parseResult = parser.parse()

      // Should have parse errors but not crash
      expect(parseResult.errors.length).toBeGreaterThan(0)
    })
  })
})
