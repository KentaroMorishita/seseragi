import { describe, expect, test } from "bun:test"
import type * as AST from "../src/ast"
import { Lexer } from "../src/lexer"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

describe("ApplicativeApply type inference", () => {
  test("ApplicativeApply with explicit Maybe types", () => {
    const code = `
      let func = Just (\\x -> x + 1)
      let result = func <*> Just 10
    `
    const lexer = new Lexer(code)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const parseResult = parser.parse()
    expect(parseResult.errors).toHaveLength(0)

    const program = {
      statements: parseResult.statements,
      kind: "Program" as const,
      line: 0,
      column: 0,
    }
    const typeInference = new TypeInferenceSystem()
    const inferResult = typeInference.infer(program)

    expect(inferResult.errors).toHaveLength(0)

    // result should be Maybe<Int>
    const resultVar = program.statements[1] as AST.VariableDeclaration
    const resultType = inferResult.nodeTypeMap.get(resultVar)
    expect(resultType).toBeDefined()
    expect(resultType!.kind).toBe("GenericType")
    const genericType = resultType as AST.GenericType
    expect(genericType.name).toBe("Maybe")
    expect(genericType.typeArguments[0].kind).toBe("PrimitiveType")
    expect((genericType.typeArguments[0] as AST.PrimitiveType).name).toBe("Int")
  })

  test("ApplicativeApply with FunctorMap result", () => {
    const code = `
      let f = \\x -> \\y -> x + y
      let func = f <$> Just 10
      let result = func <*> Just 20
    `
    const lexer = new Lexer(code)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const parseResult = parser.parse()
    expect(parseResult.errors).toHaveLength(0)

    const program = {
      statements: parseResult.statements,
      kind: "Program" as const,
      line: 0,
      column: 0,
    }
    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    expect(result.errors).toHaveLength(0)

    // func should be Maybe<(Int -> Int)>
    const funcVar = program.statements[1] as AST.VariableDeclaration
    const funcType = result.nodeTypeMap.get(funcVar)
    expect(funcType).toBeDefined()
    expect(funcType!.kind).toBe("GenericType")
    const funcGenericType = funcType as AST.GenericType
    expect(funcGenericType.name).toBe("Maybe")
    expect(funcGenericType.typeArguments[0].kind).toBe("FunctionType")

    // result should be Maybe<Int>
    const resultVar = program.statements[2] as AST.VariableDeclaration
    const resultType = result.nodeTypeMap.get(resultVar)
    expect(resultType).toBeDefined()
    expect(resultType!.kind).toBe("GenericType")
    const resultGenericType = resultType as AST.GenericType
    expect(resultGenericType.name).toBe("Maybe")
    expect(resultGenericType.typeArguments[0].kind).toBe("PrimitiveType")
    expect((resultGenericType.typeArguments[0] as AST.PrimitiveType).name).toBe(
      "Int"
    )
  })

  test("ApplicativeApply with array access result", () => {
    const code = `
      let f = \\x -> \\y -> x + y
      let arr = [1, 2, 3]
      let func = f <$> arr[0]
      let result = func <*> arr[1]
    `
    const lexer = new Lexer(code)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const parseResult = parser.parse()
    expect(parseResult.errors).toHaveLength(0)

    const program = {
      statements: parseResult.statements,
      kind: "Program" as const,
      line: 0,
      column: 0,
    }
    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    expect(result.errors).toHaveLength(0)

    // func should be Maybe<(Int -> Int)>
    const funcVar = program.statements[2] as AST.VariableDeclaration
    const funcType = result.nodeTypeMap.get(funcVar)
    expect(funcType).toBeDefined()
    expect(funcType!.kind).toBe("GenericType")
    const funcGenericType = funcType as AST.GenericType
    expect(funcGenericType.name).toBe("Maybe")
    expect(funcGenericType.typeArguments[0].kind).toBe("FunctionType")

    // result should be Maybe<Int>
    const resultVar = program.statements[3] as AST.VariableDeclaration
    const resultType = result.nodeTypeMap.get(resultVar)
    expect(resultType).toBeDefined()
    expect(resultType!.kind).toBe("GenericType")
    const resultGenericType = resultType as AST.GenericType
    expect(resultGenericType.name).toBe("Maybe")
    expect(resultGenericType.typeArguments[0].kind).toBe("PrimitiveType")
    expect((resultGenericType.typeArguments[0] as AST.PrimitiveType).name).toBe(
      "Int"
    )
  })

  test("ApplicativeApply with Either type", () => {
    const code = `
      let f = \\x -> \\y -> x * y
      let func = f <$> Right 5
      let result = func <*> Right 6
    `
    const lexer = new Lexer(code)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const parseResult = parser.parse()
    expect(parseResult.errors).toHaveLength(0)

    const program = {
      statements: parseResult.statements,
      kind: "Program" as const,
      line: 0,
      column: 0,
    }
    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    expect(result.errors).toHaveLength(0)

    // result should be Either<'a, Int>
    const resultVar = program.statements[2] as AST.VariableDeclaration
    const resultType = result.nodeTypeMap.get(resultVar)
    expect(resultType).toBeDefined()
    expect(resultType!.kind).toBe("GenericType")
    const resultGenericType = resultType as AST.GenericType
    expect(resultGenericType.name).toBe("Either")
    expect(resultGenericType.typeArguments[1].kind).toBe("PrimitiveType")
    expect((resultGenericType.typeArguments[1] as AST.PrimitiveType).name).toBe(
      "Int"
    )
  })

  test("ApplicativeApply with List type", () => {
    const code = `
      let add = \\x -> \\y -> x + y
      let mul = \\x -> \\y -> x * y
      let funcs = Cons add (Cons mul Empty)
      let values = Cons 3 (Cons 4 Empty)
      let results = funcs <*> values
    `
    const lexer = new Lexer(code)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const parseResult = parser.parse()
    expect(parseResult.errors).toHaveLength(0)

    const program = {
      statements: parseResult.statements,
      kind: "Program" as const,
      line: 0,
      column: 0,
    }
    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    expect(result.errors).toHaveLength(0)

    // results should be List<(Int -> Int)>
    const resultsVar = program.statements[4] as AST.VariableDeclaration
    const resultsType = result.nodeTypeMap.get(resultsVar)
    expect(resultsType).toBeDefined()
    expect(resultsType!.kind).toBe("GenericType")
    const resultsGenericType = resultsType as AST.GenericType
    expect(resultsGenericType.name).toBe("List")
    expect(resultsGenericType.typeArguments[0].kind).toBe("FunctionType")
  })
})
