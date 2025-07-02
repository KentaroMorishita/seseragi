import { describe, test, expect } from "bun:test"
import { Parser } from "../src/parser"
import { TypeChecker } from "../src/typechecker"

function getTypeErrors(code: string) {
  const parser = new Parser(code)
  const ast = parser.parse()
  const typeChecker = new TypeChecker()
  return typeChecker.check(ast)
}

describe("TypeChecker - Enhanced Error Messages", () => {
  test("should provide helpful error for variable type mismatch", () => {
    const errors = getTypeErrors(`let x: String = 42`)

    expect(errors).toHaveLength(1)
    expect(errors[0].message).toBe("Variable 'x' type mismatch")
    expect(errors[0].code).toContain(
      "Declared as 'String' but initialized with 'Int'"
    )
    expect(errors[0].suggestion).toContain(
      "toString() to convert Int to String"
    )
  })

  test("should provide helpful error for undefined variable", () => {
    const errors = getTypeErrors(`let x = unknownVar`)

    expect(errors).toHaveLength(1)
    expect(errors[0].message).toBe("Undefined variable 'unknownVar'")
    expect(errors[0].code).toContain(
      "Variable 'unknownVar' is not declared in this scope"
    )
    expect(errors[0].suggestion).toContain("Did you mean to declare it")
  })

  test("should provide helpful error for binary operation type mismatch", () => {
    const errors = getTypeErrors(`let x = "hello" + 42`)

    expect(errors).toHaveLength(1)
    expect(errors[0].message).toBe("Invalid operands for '+' operator")
    expect(errors[0].code).toContain(
      "Cannot apply '+' to types 'String' and 'Int'"
    )
    expect(errors[0].suggestion).toContain("Use toString() to convert")
  })

  test("should provide helpful error for function return type mismatch", () => {
    const errors = getTypeErrors(`fn bad x: Int -> String = x + 1`)

    expect(errors).toHaveLength(1)
    expect(errors[0].message).toBe("Function 'bad' return type mismatch")
    expect(errors[0].code).toContain(
      "Expected: String, but function body returns: Int"
    )
    expect(errors[0].suggestion).toContain(
      "Check that all return paths return the declared type"
    )
  })

  test("should provide helpful error for arithmetic with wrong types", () => {
    const errors = getTypeErrors(`let x = "hello" - "world"`)

    expect(errors).toHaveLength(1)
    expect(errors[0].message).toBe("Invalid operands for '-' operator")
    expect(errors[0].code).toContain(
      "Arithmetic operations require numeric types"
    )
    expect(errors[0].suggestion).toContain("Use numeric types (Int or Float)")
  })

  test("should provide helpful error for function argument type mismatch", () => {
    const errors = getTypeErrors(`
      fn addOne x: Int -> Int = x + 1
      let result = addOne "hello"
    `)

    expect(errors).toHaveLength(1)
    expect(errors[0].message).toBe("Function argument type mismatch")
    expect(errors[0].code).toContain("Expected 'Int' but got 'String'")
    expect(errors[0].suggestion).toContain("Remove quotes to make it a number")
  })

  test("should format error messages properly", () => {
    const errors = getTypeErrors(`let x: String = 42`)

    const formatted = errors[0].toString()
    expect(formatted).toContain("Error at line")
    expect(formatted).toContain("Code:")
    expect(formatted).toContain("Suggestion:")
  })

  test("should provide suggestions for Float/Int conversion", () => {
    const errors = getTypeErrors(`let x: Float = 42`)

    expect(errors).toHaveLength(1)
    expect(errors[0].suggestion).toContain("Add a decimal point (e.g., 42.0)")
  })
})
