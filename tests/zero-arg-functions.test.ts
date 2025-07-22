import { test, expect } from "bun:test"
import { Parser } from "../src/parser.js"
import { generateTypeScript } from "../src/codegen.js"
import { TokenType, Lexer } from "../src/lexer.js"
import * as AST from "../src/ast.js"

test("Zero-argument functions - should parse correctly", () => {
  const source = `
    fn getMessage -> String = "Hello!"
    fn getNumber -> Int = 42
    fn getBool -> Bool = True
  `

  const parser = new Parser(source)
  const parseResult = parser.parse()
  const program = new AST.Program(parseResult.statements || [])

  expect(program.statements).toHaveLength(3)

  const funcDecl = program.statements[0] as AST.FunctionDeclaration
  expect(funcDecl.kind).toBe("FunctionDeclaration")
  expect(funcDecl.name).toBe("getMessage")
  expect(funcDecl.parameters).toHaveLength(0)
  expect(funcDecl.returnType.name).toBe("String")
})

test("Zero-argument functions - should generate correct TypeScript", () => {
  const source = `
    fn getMessage -> String = "Hello!"
    fn getNumber -> Int = 42
    
    let message = getMessage
    let number = getNumber
  `

  const parser = new Parser(source)
  const parseResult = parser.parse()
  const program = new AST.Program(parseResult.statements || [])
  const generated = generateTypeScript(program.statements)

  expect(generated).toContain("function getMessage(): string {")
  expect(generated).toContain("function getNumber(): number {")
  expect(generated).toContain("const message = getMessage;")
  expect(generated).toContain("const number = getNumber;")
})

test("Zero-argument functions - should work with complex expressions", () => {
  const source = `
    fn getBase -> Int = 10
    fn getMultiplier -> Int = 5
    
    let result = getBase | add (getMultiplier)
    print result
  `

  const parser = new Parser(source)
  const parseResult = parser.parse()
  const program = new AST.Program(parseResult.statements || [])
  const generated = generateTypeScript(program.statements)

  expect(generated).toContain("getBase")
  expect(generated).toContain("getMultiplier")
  expect(generated).toContain("ssrgPrint")
})

test("Zero-argument functions - lexer should handle arrow correctly", () => {
  const source = "fn test -> Int = 42"
  const lexer = new Lexer(source)
  const tokens = lexer.tokenize()

  expect(tokens[0].type).toBe(TokenType.FN)
  expect(tokens[1].type).toBe(TokenType.IDENTIFIER)
  expect(tokens[1].value).toBe("test")
  expect(tokens[2].type).toBe(TokenType.ARROW)
  expect(tokens[3].type).toBe(TokenType.IDENTIFIER)
  expect(tokens[3].value).toBe("Int")
})
