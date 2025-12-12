import { describe, expect, it } from "bun:test"
import { compileAndExecute, compileSeseragi } from "./test-utils"

describe("Try Expression System", () => {
  describe("Basic Try Expressions", () => {
    it("should compile basic try expression without error type parameter", () => {
      const source = `
        let result = try 42
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      expect(result).toContain('return { tag: "Right" as const, value: 42 };')
      expect(result).toContain(
        'return { tag: "Left" as const, value: String(error) };'
      )
    })

    it("should compile try expression with custom error type parameter", () => {
      const source = `
        let result = try<MyError> 42
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<MyError, any> => {")
      expect(result).toContain('return { tag: "Right" as const, value: 42 };')
      expect(result).toContain(
        'return { tag: "Left" as const, value: error as MyError };'
      )
    })

    it("should compile try expression with string literal", () => {
      const source = `
        let result = try "hello world"
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      expect(result).toContain(
        'return { tag: "Right" as const, value: "hello world" };'
      )
    })

    it("should compile try expression with arithmetic operation", () => {
      const source = `
        let result = try (10 + 20)
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      expect(result).toContain(
        'return { tag: "Right" as const, value: (10 + 20) };'
      )
    })
  })

  describe("Promise Try Expressions", () => {
    it("should compile try expression with promise block", () => {
      const source = `
        let asyncResult = try promise {
          resolve "async test"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      expect(result).toContain(
        "const value = await (() => new Promise((resolve, reject) => {"
      )
      expect(result).toContain('resolve("async test");')
      expect(result).toContain('return { tag: "Right" as const, value };')
      expect(result).toContain(
        'return { tag: "Left" as const, value: String(error) };'
      )
    })

    it("should compile try expression with reject promise", () => {
      const source = `
        let rejectResult = try promise {
          reject "error message"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      expect(result).toContain(
        "const value = await (() => new Promise((resolve, reject) => {"
      )
      expect(result).toContain('reject("error message");')
    })

    it("should compile try expression with typed promise", () => {
      const source = `
        let typedResult = try<CustomError> promise {
          resolve "typed async"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain(
        "async (): Promise<Either<CustomError, any>> => {"
      )
      expect(result).toContain(
        'return { tag: "Left" as const, value: error as CustomError };'
      )
    })

    it("should compile try expression with direct resolve call", () => {
      const source = `
        let directResolve = try resolve "direct"
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      // ResolveExpressionは () => Promise.resolve(...) を生成し、
      // try式内ではそれを呼び出す必要がある: (() => ...)()
      expect(result).toContain(
        'const value = await (() => Promise.resolve("direct"))();'
      )
    })

    it("should compile try expression with direct reject call", () => {
      const source = `
        let directReject = try reject "direct error"
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      // RejectExpressionは () => Promise.reject(...) を生成し、
      // try式内ではそれを呼び出す必要がある: (() => ...)()
      expect(result).toContain(
        'const value = await (() => Promise.reject("direct error"))();'
      )
    })
  })

  describe("Variable Try Expressions", () => {
    it("should compile try expression with variable containing promise function", () => {
      const source = `
        let promiseFunc = promise { resolve "from variable" }
        let result = try promiseFunc
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      expect(result).toContain("const value = await (promiseFunc)();")
    })

    it("should compile try expression with variable containing sync value", () => {
      const source = `
        let value = 42
        let result = try value
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      expect(result).toContain(
        'return { tag: "Right" as const, value: value };'
      )
    })
  })

  describe("Conditional Try Expressions", () => {
    it("should compile try expression with conditional returning promise functions", () => {
      const source = `
        let condition = True
        let result = try (condition ? promise {resolve "true"} : promise {reject "false"})
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      expect(result).toContain(
        "const value = await ((condition ? () => new Promise((resolve, reject) => {"
      )
      expect(result).toContain('resolve("true");')
      expect(result).toContain('reject("false");')
      expect(result).toContain("))();")
    })

    it("should compile try expression with conditional returning sync values", () => {
      const source = `
        let condition = True
        let result = try (condition ? 100 : 200)
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      expect(result).toContain(
        'return { tag: "Right" as const, value: (condition ? 100 : 200) };'
      )
    })

    it("should compile try expression with mixed conditional (promise and sync)", () => {
      const source = `
        let condition = True
        let result = try (condition ? promise {resolve "async"} : 42)
      `
      const result = compileSeseragi(source)
      // Should be treated as sync since not all branches are promises
      expect(result).toContain("(): Either<string, any> => {")
    })
  })

  describe("Nested Try Expressions", () => {
    it("should compile nested try expressions", () => {
      const source = `
        let nested = try (try 42)
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      // Should contain nested try function calls
      expect(result).toContain("(): Either<string, any> => {")
    })

    it("should compile try with function call result", () => {
      const source = `
        fn getValue: Int
        getValue = 42
        
        let result = try getValue
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      expect(result).toContain(
        'return { tag: "Right" as const, value: getValue };'
      )
    })
  })

  describe("Right-Associative Parsing", () => {
    it("should parse try expressions right-associatively", () => {
      const source = `
        let chainedTry = try promise {resolve "first"}
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      expect(result).toContain(
        "const value = await (() => new Promise((resolve, reject) => {"
      )
      expect(result).toContain('resolve("first");')
    })

    it("should handle complex right-associative expressions", () => {
      const source = `
        let a = 10
        let complex = try a == 10 ? promise {resolve "match"} : promise {reject "no match"}
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      expect(result).toContain(
        "const value = await (((a === 10) ? () => new Promise((resolve, reject) => {"
      )
    })
  })

  describe("Error Type Handling", () => {
    it("should handle custom error types correctly", () => {
      const source = `
        type MyError = String
        let result = try<MyError> 42
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<MyError, any> => {")
      expect(result).toContain(
        'return { tag: "Left" as const, value: error as MyError };'
      )
    })

    it("should default to String error type when not specified", () => {
      const source = `
        let result = try 42
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      expect(result).toContain(
        'return { tag: "Left" as const, value: String(error) };'
      )
    })
  })

  describe("Integration with Either Type", () => {
    it("should generate proper Either type structure", () => {
      const source = `
        let result = try 42
        let value = result ()
      `
      const result = compileSeseragi(source)
      expect(result).toContain("const value = result();")
      // The result should be an Either<string, number>
      expect(result).toContain("(): Either<string, any> => {")
    })

    it("should work with Either type structure", () => {
      const source = `
        let result = try 42
        let value = result ()
        show value
      `
      const result = compileSeseragi(source)
      expect(result).toContain("(): Either<string, any> => {")
      // Should contain Either tag structure
      expect(result).toContain("tag")
    })
  })

  describe("Runtime Execution", () => {
    it("should execute successful try expression", async () => {
      const source = `
        let result = try 42
        let value = result ()
        show value
      `
      const output = await compileAndExecute(source)
      expect(output).toContain("Right(42)")
    })

    it("should handle thrown errors in try expression", async () => {
      const source = `
        let result = try (10 / 0)  // Division by zero
        let value = result ()
        show value
      `
      const output = await compileAndExecute(source)
      // Should execute without compilation errors
      expect(output).toBeDefined()
    })

    it("should execute async try expression successfully", async () => {
      const source = `
        let asyncResult = try promise {
          resolve "async success"
        }
        let value = asyncResult ()
        show value
      `
      const output = await compileAndExecute(source)
      expect(output).toContain("Promise")
    })
  })

  describe("Type Inference", () => {
    it("should infer correct return type for sync try expressions", () => {
      const source = `
        let result = try 42
      `
      const result = compileSeseragi(source)
      // Should be typed as () => Either<string, number>
      expect(result).toContain("(): Either<string, any> => {")
    })

    it("should infer correct return type for async try expressions", () => {
      const source = `
        let asyncResult = try promise { resolve "test" }
      `
      const result = compileSeseragi(source)
      // Should be typed as () => Promise<Either<string, string>>
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
    })

    it("should infer correct return type for variable-based try expressions", () => {
      const source = `
        let promiseVar = promise { resolve "from var" }
        let result = try promiseVar
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      expect(result).toContain("const value = await (promiseVar)();")
    })
  })
})
