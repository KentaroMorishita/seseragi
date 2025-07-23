import { expect, test } from "bun:test"
import * as AST from "../src/ast"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

test("Function calls with no arguments", () => {
  const content = `
fn getMessage -> String = "Hello from function!"
fn getNumber -> Int = 42

let message = getMessage()
let number = getNumber()
`

  const parser = new Parser(content)
  const parseResult = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const program = new AST.Program(parseResult.statements || [])
  const result = typeInference.infer(program)

  const messageDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "message"
  )
  const numberDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "number"
  )

  const messageType = result.nodeTypeMap.get(messageDecl)
  const numberType = result.nodeTypeMap.get(numberDecl)

  expect(messageType?.kind).toBe("PrimitiveType")
  expect((messageType as AST.PrimitiveType)?.name).toBe("String")

  expect(numberType?.kind).toBe("PrimitiveType")
  expect((numberType as AST.PrimitiveType)?.name).toBe("Int")
})

test("Maybe types with proper polymorphism", () => {
  const content = `
let someValue = Just 42
let nothingValue = Nothing
`

  const parser = new Parser(content)
  const parseResult = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const program = new AST.Program(parseResult.statements || [])
  const result = typeInference.infer(program)

  const someValueDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "someValue"
  )
  const nothingValueDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "nothingValue"
  )

  const someValueType = result.nodeTypeMap.get(someValueDecl)
  const nothingValueType = result.nodeTypeMap.get(nothingValueDecl)

  // Just 42 should be Maybe<Int>
  expect(someValueType?.kind).toBe("GenericType")
  expect((someValueType as AST.GenericType)?.name).toBe("Maybe")
  expect((someValueType as AST.GenericType)?.typeArguments[0]?.name).toBe("Int")

  // Nothing should be Maybe<'a> (polymorphic)
  expect(nothingValueType?.kind).toBe("GenericType")
  expect((nothingValueType as AST.GenericType)?.name).toBe("Maybe")
  expect((nothingValueType as AST.GenericType)?.typeArguments[0]?.kind).toBe(
    "PolymorphicTypeVariable"
  )
  expect((nothingValueType as AST.GenericType)?.typeArguments[0]?.name).toBe(
    "a"
  )
})

test("Either types with proper polymorphism", () => {
  const content = `
let successValue = Right 42
let errorValue = Left "Error occurred"
`

  const parser = new Parser(content)
  const parseResult = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const program = new AST.Program(parseResult.statements || [])
  const result = typeInference.infer(program)

  const successValueDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "successValue"
  )
  const errorValueDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "errorValue"
  )

  const successValueType = result.nodeTypeMap.get(successValueDecl)
  const errorValueType = result.nodeTypeMap.get(errorValueDecl)

  // Right 42 should be Either<'a, Int>
  expect(successValueType?.kind).toBe("GenericType")
  expect((successValueType as AST.GenericType)?.name).toBe("Either")
  expect((successValueType as AST.GenericType)?.typeArguments[0]?.kind).toBe(
    "PolymorphicTypeVariable"
  )
  expect((successValueType as AST.GenericType)?.typeArguments[0]?.name).toBe(
    "a"
  )
  expect((successValueType as AST.GenericType)?.typeArguments[1]?.name).toBe(
    "Int"
  )

  // Left "Error" should be Either<String, 'b>
  expect(errorValueType?.kind).toBe("GenericType")
  expect((errorValueType as AST.GenericType)?.name).toBe("Either")
  expect((errorValueType as AST.GenericType)?.typeArguments[0]?.name).toBe(
    "String"
  )
  expect((errorValueType as AST.GenericType)?.typeArguments[1]?.kind).toBe(
    "PolymorphicTypeVariable"
  )
  expect((errorValueType as AST.GenericType)?.typeArguments[1]?.name).toBe("b")
})

test("Curried functions still work correctly", () => {
  const content = `
fn complexCalculation a: Int -> b: Int -> Int {
  let sum = a + b
  let product = a * b
  if sum > product then sum else product
}

let calculatedA = complexCalculation 3
let calculatedB = complexCalculation 5 6
`

  const parser = new Parser(content)
  const parseResult = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const program = new AST.Program(parseResult.statements || [])
  const result = typeInference.infer(program)

  const calculatedADecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "calculatedA"
  )
  const calculatedBDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "calculatedB"
  )

  const calculatedAType = result.nodeTypeMap.get(calculatedADecl)
  const calculatedBType = result.nodeTypeMap.get(calculatedBDecl)

  // calculatedA should be Int -> Int
  expect(calculatedAType?.kind).toBe("FunctionType")
  expect((calculatedAType as AST.FunctionType)?.paramType?.name).toBe("Int")
  expect((calculatedAType as AST.FunctionType)?.returnType?.name).toBe("Int")

  // calculatedB should be Int
  expect(calculatedBType?.kind).toBe("PrimitiveType")
  expect((calculatedBType as AST.PrimitiveType)?.name).toBe("Int")
})

test("Type string formatting", () => {
  const content = `
let nothingValue = Nothing
let successValue = Right 42
`

  const parser = new Parser(content)
  const parseResult = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const program = new AST.Program(parseResult.statements || [])
  const result = typeInference.infer(program)

  const nothingValueDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "nothingValue"
  )
  const successValueDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "successValue"
  )

  const nothingValueType = result.nodeTypeMap.get(nothingValueDecl)
  const successValueType = result.nodeTypeMap.get(successValueDecl)

  // Test string formatting
  expect(typeInference.typeToString(nothingValueType)).toBe("Maybe<'a>")
  expect(typeInference.typeToString(successValueType)).toBe("Either<'a, Int>")
})
