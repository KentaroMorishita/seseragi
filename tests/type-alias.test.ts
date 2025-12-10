/**
 * Type Alias Tests for Seseragi Language
 */

import { expect, test } from "bun:test"
import * as AST from "../src/ast"
import { CodeGenerator } from "../src/codegen"
import { Parser } from "../src/parser"
import { infer } from "../src/inference/engine/infer"

test("Type alias declaration parsing", () => {
  const source = `
type UserId = Int
type UserName = String
type Age = Int
`

  const parser = new Parser(source)
  const parseResult = parser.parse()

  expect(parseResult.statements).toHaveLength(3)

  // First type alias: UserId = Int
  const userIdDecl = parseResult.statements![0] as AST.TypeAliasDeclaration
  expect(userIdDecl.kind).toBe("TypeAliasDeclaration")
  expect(userIdDecl.name).toBe("UserId")
  expect((userIdDecl.aliasedType as AST.PrimitiveType).name).toBe("Int")

  // Second type alias: UserName = String
  const userNameDecl = parseResult.statements![1] as AST.TypeAliasDeclaration
  expect(userNameDecl.kind).toBe("TypeAliasDeclaration")
  expect(userNameDecl.name).toBe("UserName")
  expect((userNameDecl.aliasedType as AST.PrimitiveType).name).toBe("String")

  // Third type alias: Age = Int
  const ageDecl = parseResult.statements![2] as AST.TypeAliasDeclaration
  expect(ageDecl.kind).toBe("TypeAliasDeclaration")
  expect(ageDecl.name).toBe("Age")
  expect((ageDecl.aliasedType as AST.PrimitiveType).name).toBe("Int")
})

test("Type alias vs union type distinction", () => {
  const source = `
type Status = String
type Color = | Red | Green | Blue
type Point { x: Int, y: Int }
`

  const parser = new Parser(source)
  const parseResult = parser.parse()

  expect(parseResult.statements).toHaveLength(3)

  // Type alias
  const statusDecl = parseResult.statements![0] as AST.TypeAliasDeclaration
  expect(statusDecl.kind).toBe("TypeAliasDeclaration")
  expect(statusDecl.name).toBe("Status")
  expect((statusDecl.aliasedType as AST.PrimitiveType).name).toBe("String")

  // Union type (ADT)
  const colorDecl = parseResult.statements![1] as AST.TypeDeclaration
  expect(colorDecl.kind).toBe("TypeDeclaration")
  expect(colorDecl.name).toBe("Color")
  expect(colorDecl.fields).toHaveLength(3)

  // Struct type
  const pointDecl = parseResult.statements![2] as AST.TypeDeclaration
  expect(pointDecl.kind).toBe("TypeDeclaration")
  expect(pointDecl.name).toBe("Point")
  expect(pointDecl.fields).toHaveLength(2)
})

test("Generic type alias", () => {
  const source = `
type Result = Maybe<String>
type UserList = List<Int>
`

  const parser = new Parser(source)
  const parseResult = parser.parse()

  expect(parseResult.statements).toHaveLength(2)

  // Generic type alias: Result = Maybe<String>
  const resultDecl = parseResult.statements![0] as AST.TypeAliasDeclaration
  expect(resultDecl.kind).toBe("TypeAliasDeclaration")
  expect(resultDecl.name).toBe("Result")
  expect(resultDecl.aliasedType.kind).toBe("GenericType")

  const resultGeneric = resultDecl.aliasedType as AST.GenericType
  expect(resultGeneric.name).toBe("Maybe")
  expect(resultGeneric.typeArguments).toHaveLength(1)
  expect((resultGeneric.typeArguments[0] as AST.PrimitiveType).name).toBe(
    "String"
  )

  // Generic type alias: UserList = List<Int>
  const userListDecl = parseResult.statements![1] as AST.TypeAliasDeclaration
  expect(userListDecl.kind).toBe("TypeAliasDeclaration")
  expect(userListDecl.name).toBe("UserList")
  expect(userListDecl.aliasedType.kind).toBe("GenericType")

  const userListGeneric = userListDecl.aliasedType as AST.GenericType
  expect(userListGeneric.name).toBe("List")
  expect(userListGeneric.typeArguments).toHaveLength(1)
  expect((userListGeneric.typeArguments[0] as AST.PrimitiveType).name).toBe(
    "Int"
  )
})

test("Type alias code generation", () => {
  const source = `
type UserId = Int
type Status = String
type Result = Maybe<String>
`

  const parser = new Parser(source)
  const parseResult = parser.parse()

  const codegen = new CodeGenerator({
    indent: "  ",
    runtimeMode: "embedded",
    generateComments: false,
  })
  const output = codegen.generateProgram(parseResult.statements!)

  expect(output).toContain("type UserId = number;")
  expect(output).toContain("type Status = string;")
  expect(output).toContain("type Result = Maybe<string>;")
})

test("Type alias in type inference", () => {
  const source = `
type UserId = Int
let userId: UserId = 42
`

  const parser = new Parser(source)
  const parseResult = parser.parse()

  const program = new AST.Program(parseResult.statements!)
  const result = infer(program)

  expect(result.errors).toHaveLength(0)
  expect(result.nodeTypeMap.size).toBeGreaterThan(0)
})

test("Function type alias", () => {
  const source = `
type Predicate = (Int -> Bool)
type Transform = (String -> String)
`

  const parser = new Parser(source)
  const parseResult = parser.parse()

  expect(parseResult.statements).toHaveLength(2)

  // Function type alias: Predicate = (Int -> Bool)
  const predicateDecl = parseResult.statements![0] as AST.TypeAliasDeclaration
  expect(predicateDecl.kind).toBe("TypeAliasDeclaration")
  expect(predicateDecl.name).toBe("Predicate")
  expect(predicateDecl.aliasedType.kind).toBe("FunctionType")

  const predicateFunc = predicateDecl.aliasedType as AST.FunctionType
  expect((predicateFunc.paramType as AST.PrimitiveType).name).toBe("Int")
  expect((predicateFunc.returnType as AST.PrimitiveType).name).toBe("Bool")

  // Function type alias: Transform = (String -> String)
  const transformDecl = parseResult.statements![1] as AST.TypeAliasDeclaration
  expect(transformDecl.kind).toBe("TypeAliasDeclaration")
  expect(transformDecl.name).toBe("Transform")
  expect(transformDecl.aliasedType.kind).toBe("FunctionType")

  const transformFunc = transformDecl.aliasedType as AST.FunctionType
  expect((transformFunc.paramType as AST.PrimitiveType).name).toBe("String")
  expect((transformFunc.returnType as AST.PrimitiveType).name).toBe("String")
})
