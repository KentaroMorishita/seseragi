import { describe, it, expect } from "bun:test"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import { TypeInferenceSystem } from "../src/type-inference"
import * as AST from "../src/ast"

function transpileCode(code: string): string {
  const parser = new Parser(code)
  const parseResult = parser.parse()

  if (parseResult.errors.length > 0) {
    throw new Error(parseResult.errors.join("\n"))
  }

  if (!parseResult.statements) {
    throw new Error("No statements parsed")
  }

  const program = new AST.Program(parseResult.statements)
  const typeInference = new TypeInferenceSystem()
  typeInference.infer(program)

  return generateTypeScript(parseResult.statements, {
    generateComments: false,
    useArrowFunctions: true,
    runtimeMode: "minimal",
  })
}

describe("Identifier with apostrophes (Haskell-style)", () => {
  it("should handle variable with apostrophe", () => {
    const code = `
let x' = 10
let result = x' + 5
`
    const result = transpileCode(code)
    expect(result).toContain("const x_prime = 10;")
    expect(result).toContain("const result = (x_prime + 5);")
  })

  it("should handle function with apostrophes", () => {
    const code = `
fn f' x = x * 2
let result = f' 5
`
    const result = transpileCode(code)
    expect(result).toContain("const f_prime = (x: any): any =>")
    expect(result).toContain("const result = f_prime(5);")
  })

  it("should handle multiple quotes", () => {
    const code = `
let x = 1
let x' = 2
let x'' = 3
let sum = x + x' + x''
`
    const result = transpileCode(code)
    expect(result).toContain("const x = 1;")
    expect(result).toContain("const x_prime = 2;")
    expect(result).toContain("const x_prime_prime = 3;")
    expect(result).toContain("const sum = ((x + x_prime) + x_prime_prime);")
  })

  it("should handle recursive function with quotes", () => {
    const code = `
fn fact n = if n == 0 then 1 else n * fact (n - 1)
fn fact' n = fact n
let result = fact' 5
`
    const result = transpileCode(code)
    expect(result).toContain("const fact_prime = (n: any): any => fact(n);")
    expect(result).toContain("const result = fact_prime(5);")
  })

  it("should handle pattern matching with apostrophe identifiers", () => {
    const code = `
fn f' x = if x == 0 then "zero" else "non-zero"
let x' = 5
let result = f' x'
`
    const result = transpileCode(code)
    expect(result).toContain("const f_prime = (x: any): any =>")
    expect(result).toContain("const x_prime = 5;")
    expect(result).toContain("const result = f_prime(x_prime);")
  })

  it("should handle higher-order functions with apostrophes", () => {
    const code = `
fn add' x y = x + y
let f' = add' 1
let result = f' 2
`
    const result = transpileCode(code)
    expect(result).toContain("const add_prime = curry(")
    expect(result).toContain("const f_prime = add_prime(1);")
    expect(result).toContain("const result = f_prime(2);")
  })
})

