import { describe, it, expect } from "bun:test"
import { Parser } from "../src/parser"
import type * as AST from "../src/ast"

describe("Parser", () => {
  it("should parse variable declaration", () => {
    const source = "let x = 10"
    const parser = new Parser(source)
    const program = parser.parse()

    expect(program.statements).toHaveLength(1)
    const stmt = program.statements[0] as AST.VariableDeclaration
    expect(stmt.kind).toBe("VariableDeclaration")
    expect(stmt.name).toBe("x")

    const literal = stmt.initializer as AST.Literal
    expect(literal.kind).toBe("Literal")
    expect(literal.value).toBe(10)
    expect(literal.literalType).toBe("integer")
  })

  it("should parse function declaration", () => {
    const source = "fn add a :Int -> b :Int -> Int = a + b"
    const parser = new Parser(source)
    const program = parser.parse()

    expect(program.statements).toHaveLength(1)
    const func = program.statements[0] as AST.FunctionDeclaration
    expect(func.kind).toBe("FunctionDeclaration")
    expect(func.name).toBe("add")
    expect(func.parameters).toHaveLength(2)
    expect(func.parameters[0].name).toBe("a")
    expect(func.parameters[1].name).toBe("b")
    expect(func.isEffectful).toBe(false)
  })

  it("should parse effectful function declaration", () => {
    const source = "effectful fn printMessage msg :String -> IO = msg"
    const parser = new Parser(source)
    const program = parser.parse()

    const func = program.statements[0] as AST.FunctionDeclaration
    expect(func.isEffectful).toBe(true)
    expect(func.name).toBe("printMessage")
  })

  it("should parse type declaration", () => {
    const source = `type Person {
      name :String
      age :Int
    }`
    const parser = new Parser(source)
    const program = parser.parse()

    const typeDecl = program.statements[0] as AST.TypeDeclaration
    expect(typeDecl.kind).toBe("TypeDeclaration")
    expect(typeDecl.name).toBe("Person")
    expect(typeDecl.fields).toHaveLength(2)
    expect(typeDecl.fields[0].name).toBe("name")
    expect(typeDecl.fields[1].name).toBe("age")
  })

  it("should parse binary operations", () => {
    const source = "let result = a + b * c"
    const parser = new Parser(source)
    const program = parser.parse()

    const varDecl = program.statements[0] as AST.VariableDeclaration
    const binOp = varDecl.initializer as AST.BinaryOperation
    expect(binOp.kind).toBe("BinaryOperation")
    expect(binOp.operator).toBe("+")

    // Check operator precedence: a + (b * c)
    const rightSide = binOp.right as AST.BinaryOperation
    expect(rightSide.operator).toBe("*")
  })

  it("should parse pipeline operations", () => {
    const source = "let result = x | add1 | square"
    const parser = new Parser(source)
    const program = parser.parse()

    const varDecl = program.statements[0] as AST.VariableDeclaration
    const pipeline = varDecl.initializer as AST.Pipeline
    expect(pipeline.kind).toBe("Pipeline")

    // Check left-associative: (x | add1) | square
    const leftPipeline = pipeline.left as AST.Pipeline
    expect(leftPipeline.kind).toBe("Pipeline")
  })

  it("should parse conditional expressions", () => {
    const source = "let result = if x > 0 then 1 else 0"
    const parser = new Parser(source)
    const program = parser.parse()

    const varDecl = program.statements[0] as AST.VariableDeclaration
    const conditional = varDecl.initializer as AST.ConditionalExpression
    expect(conditional.kind).toBe("ConditionalExpression")

    const condition = conditional.condition as AST.BinaryOperation
    expect(condition.operator).toBe(">")
  })

  it("should parse match expressions", () => {
    const source = `let result = match value {
      Just x -> x
      Nothing -> 0
    }`
    const parser = new Parser(source)
    const program = parser.parse()

    const varDecl = program.statements[0] as AST.VariableDeclaration
    const matchExpr = varDecl.initializer as AST.MatchExpression
    expect(matchExpr.kind).toBe("MatchExpression")
    expect(matchExpr.cases).toHaveLength(2)

    const justCase = matchExpr.cases[0]
    const justPattern = justCase.pattern as AST.ConstructorPattern
    expect(justPattern.constructorName).toBe("Just")
  })

  it("should parse function calls", () => {
    const source = "let result = add(1, 2)"
    const parser = new Parser(source)
    const program = parser.parse()

    const varDecl = program.statements[0] as AST.VariableDeclaration
    const call = varDecl.initializer as AST.FunctionCall
    expect(call.kind).toBe("FunctionCall")
    expect(call.arguments).toHaveLength(2)

    const func = call.function as AST.Identifier
    expect(func.name).toBe("add")
  })

  it("should parse string literals", () => {
    const source = 'let message = "Hello, World!"'
    const parser = new Parser(source)
    const program = parser.parse()

    const varDecl = program.statements[0] as AST.VariableDeclaration
    const literal = varDecl.initializer as AST.Literal
    expect(literal.value).toBe("Hello, World!")
    expect(literal.literalType).toBe("string")
  })

  it("should parse boolean literals", () => {
    const source = "let flag = True"
    const parser = new Parser(source)
    const program = parser.parse()

    const varDecl = program.statements[0] as AST.VariableDeclaration
    const literal = varDecl.initializer as AST.Literal
    expect(literal.value).toBe(true)
    expect(literal.literalType).toBe("boolean")
  })

  it("should parse generic types", () => {
    const source = "let items :List<Int> = items"
    const parser = new Parser(source)
    const program = parser.parse()

    const varDecl = program.statements[0] as AST.VariableDeclaration
    const genericType = varDecl.type as AST.GenericType
    expect(genericType.kind).toBe("GenericType")
    expect(genericType.name).toBe("List")
    expect(genericType.typeArguments).toHaveLength(1)

    const argType = genericType.typeArguments[0] as AST.PrimitiveType
    expect(argType.name).toBe("Int")
  })
})
