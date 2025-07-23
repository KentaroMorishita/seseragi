import { expect, test } from "bun:test"
import * as AST from "../src/ast"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

test("Maybe type inference", () => {
  const content = `
let someValue = Just 42
let nothingValue = Nothing
`

  const parser = new Parser(content)
  const parseResult = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const program = new AST.Program(parseResult.statements || [])
  const result = typeInference.infer(program)

  // Find the variable types
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

  expect(someValueType?.kind).toBe("GenericType")
  expect((someValueType as AST.GenericType)?.name).toBe("Maybe")
  expect((someValueType as AST.GenericType)?.typeArguments[0]?.name).toBe("Int")

  expect(nothingValueType?.kind).toBe("GenericType")
  expect((nothingValueType as AST.GenericType)?.name).toBe("Maybe")
})

test("Either type inference", () => {
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

  expect(successValueType?.kind).toBe("GenericType")
  expect((successValueType as AST.GenericType)?.name).toBe("Either")

  expect(errorValueType?.kind).toBe("GenericType")
  expect((errorValueType as AST.GenericType)?.name).toBe("Either")
})

test("Curried function partial application", () => {
  const content = `
fn complexCalculation a: Int -> b: Int -> Int {
  let sum = a + b
  let product = a * b
  if sum > product then sum else product
}

let calculatedA = complexCalculation 3
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

  const calculatedAType = result.nodeTypeMap.get(calculatedADecl)

  // Should be a function type Int -> Int
  expect(calculatedAType?.kind).toBe("FunctionType")
  expect((calculatedAType as AST.FunctionType)?.paramType?.name).toBe("Int")
  expect((calculatedAType as AST.FunctionType)?.returnType?.name).toBe("Int")
})

test("Curried function full application", () => {
  const content = `
fn complexCalculation a: Int -> b: Int -> Int {
  let sum = a + b
  let product = a * b
  if sum > product then sum else product
}

let calculatedB = complexCalculation 5 6
`

  const parser = new Parser(content)
  const parseResult = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const program = new AST.Program(parseResult.statements || [])
  const result = typeInference.infer(program)

  const calculatedBDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "calculatedB"
  )

  const calculatedBType = result.nodeTypeMap.get(calculatedBDecl)

  // Should be Int
  expect(calculatedBType?.kind).toBe("PrimitiveType")
  expect((calculatedBType as AST.PrimitiveType)?.name).toBe("Int")
})

test("Complex type inference scenario", () => {
  const content = `
fn add x: Int -> y: Int -> Int = x + y
fn safeDivide x: Int -> y: Int -> Maybe<Int> = if y == 0 then Nothing else Just (x / y)

let addFive = add 5
let result1 = addFive 3
let result2 = safeDivide 10 2
let result3 = safeDivide 10 0
`

  const parser = new Parser(content)
  const parseResult = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const program = new AST.Program(parseResult.statements || [])
  const result = typeInference.infer(program)

  // Check addFive is a function Int -> Int
  const addFiveDecl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "addFive"
  )
  const addFiveType = result.nodeTypeMap.get(addFiveDecl)
  expect(addFiveType?.kind).toBe("FunctionType")

  // Check result1 is Int
  const result1Decl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "result1"
  )
  const result1Type = result.nodeTypeMap.get(result1Decl)
  expect(result1Type?.kind).toBe("PrimitiveType")
  expect((result1Type as AST.PrimitiveType)?.name).toBe("Int")

  // Check result2 and result3 are Maybe<Int>
  const result2Decl = program.statements.find(
    (stmt: AST.Statement) =>
      stmt.kind === "VariableDeclaration" &&
      (stmt as AST.VariableDeclaration).name === "result2"
  )
  const result2Type = result.nodeTypeMap.get(result2Decl)
  expect(result2Type?.kind).toBe("GenericType")
  expect((result2Type as AST.GenericType)?.name).toBe("Maybe")
})
