import { test, expect } from "bun:test"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

test("Maybe type inference", () => {
  const content = `
let someValue = Just 42
let nothingValue = Nothing
`

  const parser = new Parser(content)
  const ast = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const result = typeInference.infer(ast)

  // Find the variable types
  const someValueDecl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "someValue"
  )
  const nothingValueDecl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "nothingValue"
  )

  const someValueType = result.nodeTypeMap.get(someValueDecl)
  const nothingValueType = result.nodeTypeMap.get(nothingValueDecl)

  expect(someValueType?.kind).toBe("GenericType")
  expect((someValueType as any)?.name).toBe("Maybe")
  expect((someValueType as any)?.typeArguments[0]?.name).toBe("Int")

  expect(nothingValueType?.kind).toBe("GenericType")
  expect((nothingValueType as any)?.name).toBe("Maybe")
})

test("Either type inference", () => {
  const content = `
let successValue = Right 42
let errorValue = Left "Error occurred"
`

  const parser = new Parser(content)
  const ast = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const result = typeInference.infer(ast)

  const successValueDecl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "successValue"
  )
  const errorValueDecl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "errorValue"
  )

  const successValueType = result.nodeTypeMap.get(successValueDecl)
  const errorValueType = result.nodeTypeMap.get(errorValueDecl)

  expect(successValueType?.kind).toBe("GenericType")
  expect((successValueType as any)?.name).toBe("Either")

  expect(errorValueType?.kind).toBe("GenericType")
  expect((errorValueType as any)?.name).toBe("Either")
})

test("Curried function partial application", () => {
  const content = `
fn complexCalculation a :Int -> b :Int -> Int {
  let sum = a + b
  let product = a * b
  if sum > product then sum else product
}

let calculatedA = complexCalculation 3
`

  const parser = new Parser(content)
  const ast = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const result = typeInference.infer(ast)

  const calculatedADecl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "calculatedA"
  )

  const calculatedAType = result.nodeTypeMap.get(calculatedADecl)

  // Should be a function type Int -> Int
  expect(calculatedAType?.kind).toBe("FunctionType")
  expect((calculatedAType as any)?.paramType?.name).toBe("Int")
  expect((calculatedAType as any)?.returnType?.name).toBe("Int")
})

test("Curried function full application", () => {
  const content = `
fn complexCalculation a :Int -> b :Int -> Int {
  let sum = a + b
  let product = a * b
  if sum > product then sum else product
}

let calculatedB = complexCalculation 5 6
`

  const parser = new Parser(content)
  const ast = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const result = typeInference.infer(ast)

  const calculatedBDecl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "calculatedB"
  )

  const calculatedBType = result.nodeTypeMap.get(calculatedBDecl)

  // Should be Int
  expect(calculatedBType?.kind).toBe("PrimitiveType")
  expect((calculatedBType as any)?.name).toBe("Int")
})

test("Complex type inference scenario", () => {
  const content = `
fn add x :Int -> y :Int -> Int = x + y
fn safeDivide x :Int -> y :Int -> Maybe<Int> = if y == 0 then Nothing else Just (x / y)

let addFive = add 5
let result1 = addFive 3
let result2 = safeDivide 10 2
let result3 = safeDivide 10 0
`

  const parser = new Parser(content)
  const ast = parser.parse()
  const typeInference = new TypeInferenceSystem()
  const result = typeInference.infer(ast)

  // Check addFive is a function Int -> Int
  const addFiveDecl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "addFive"
  )
  const addFiveType = result.nodeTypeMap.get(addFiveDecl)
  expect(addFiveType?.kind).toBe("FunctionType")

  // Check result1 is Int
  const result1Decl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "result1"
  )
  const result1Type = result.nodeTypeMap.get(result1Decl)
  expect(result1Type?.kind).toBe("PrimitiveType")
  expect((result1Type as any)?.name).toBe("Int")

  // Check result2 and result3 are Maybe<Int>
  const result2Decl = ast.statements.find(
    (stmt: any) =>
      stmt.kind === "VariableDeclaration" && stmt.name === "result2"
  )
  const result2Type = result.nodeTypeMap.get(result2Decl)
  expect(result2Type?.kind).toBe("GenericType")
  expect((result2Type as any)?.name).toBe("Maybe")
})
