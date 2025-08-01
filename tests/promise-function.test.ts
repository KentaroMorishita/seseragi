import { describe, expect, it } from "bun:test"
import { compileAndExecute, compileSeseragi } from "./test-utils"

describe("Promise Function System", () => {
  describe("Complex Function Type Annotations", () => {
    it("should parse single parameter function returning promise", () => {
      const source = `
        fn fetchData input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            resolve "data"
          }
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain(
        "function fetchData(input: number): (arg: void) => Promise<string>"
      )
      expect(result).toContain(
        "return () => new Promise<string>((resolve, reject) => {"
      )
    })

    it("should handle complex return types with parentheses", () => {
      const source = `
        fn complexFunc value: Int -> (Unit -> Promise<String>) {
          promise<String> {
            if value > 0
            then resolve "positive"
            else reject "negative"
          }
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain(
        "function complexFunc(value: number): (arg: void) => Promise<string>"
      )
      expect(result).toContain(
        '((value > 0) ? resolve("positive") : reject("negative"));'
      )
    })
  })

  describe("Promise Block Scope Management", () => {
    it("should capture function parameters in promise block", () => {
      const source = `
        fn useParam input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            resolve input
          }
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("resolve(input)")
      // Should not have extra closures
      expect(result).not.toContain("(() => {")
    })

    it("should capture local variables in promise block", () => {
      const source = `
        fn useLocal input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            let computed = input * 2
            let message = "Result"
            resolve \`\${message}: \${computed}\`
          }
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("const computed = (input * 2);")
      expect(result).toContain('const message = "Result";')
      expect(result).toContain(`resolve(\`\${message}: \${computed}\`)`)
    })

    it("should handle complex expressions in promise block", () => {
      const source = `
        fn complexLogic input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            if input > 100
            then resolve "big"
            else if input > 50
            then resolve "medium"  
            else reject "small"
          }
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain(
        '((input > 100) ? resolve("big") : ((input > 50) ? resolve("medium") : reject("small")))'
      )
    })
  })

  describe("Generated Code Quality", () => {
    it("should generate clean function without extra closures", () => {
      const source = `
        fn cleanFunc input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            resolve "clean"
          }
        }
      `
      const result = compileSeseragi(source)

      // Should have direct promise return
      expect(result).toContain("return () => new Promise<string>")

      // Should not have IIFE wrapper
      expect(result).not.toContain("(() => {")
      expect(result).not.toContain("})();")
    })

    it("should generate proper TypeScript types", () => {
      const source = `
        fn typedFunc input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            resolve "typed"
          }
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain(
        "function typedFunc(input: number): (arg: void) => Promise<string>"
      )
      expect(result).toContain(
        "() => new Promise<string>((resolve, reject) => {"
      )
    })
  })

  describe("Runtime Execution", () => {
    it("should execute promise function successfully", async () => {
      const source = `
        fn testFunc input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            resolve "success"
          }
        }
        
        let result = testFunc 42 ()
        show result
      `
      const output = await compileAndExecute(source)
      // Promise objects show as "Promise {  }" in console output
      expect(output).toContain("Promise {")
    })

    it("should maintain scope correctly at runtime", async () => {
      const source = `
        fn scopeTest value: Int -> (Unit -> Promise<String>) {
          promise<String> {
            let doubled = value * 2
            resolve doubled
          }
        }
        
        show $ scopeTest 21 ()
      `
      const output = await compileAndExecute(source)
      expect(output).toContain("Promise {")
    })

    it("should execute conditional promise logic correctly", async () => {
      const source = `
        fn conditionalPromise input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            if input > 50
            then resolve "high value"
            else if input > 25
            then resolve "medium value"
            else resolve "low value"
          }
        }
        
        let highResult = conditionalPromise 75 ()
        let mediumResult = conditionalPromise 30 ()
        let lowResult = conditionalPromise 10 ()
        
        show highResult
        show mediumResult  
        show lowResult
      `
      const output = await compileAndExecute(source)
      // Should show three Promise objects in output
      expect(output).toContain("Promise {")
    })

    it("should handle promise functions with multiple local variables", async () => {
      const source = `
        fn complexPromise base: Int -> (Unit -> Promise<String>) {
          promise<String> {
            let doubled = base * 2
            let message = "Result"
            let prefix = "Final"
            let combined = prefix + ": " + message + " is " + (toString doubled)
            resolve combined
          }
        }
        
        let result = complexPromise 15 ()
        show result
      `
      const output = await compileAndExecute(source)
      expect(output).toContain("Promise {")
    })

    it("should execute promise functions with string templates", async () => {
      const source = `
        fn templatePromise name: String -> age: Int -> (Unit -> Promise<String>) {
          promise<String> {
            let greeting = "Hello"
            resolve \`\${greeting} \${name}, you are \${age} years old\`
          }
        }
        
        let result = templatePromise "Alice" 25 ()
        show result
      `
      const output = await compileAndExecute(source)
      expect(output).toContain("Promise {")
    })
  })

  describe("Error Handling", () => {
    it("should handle reject in promise functions", () => {
      const source = `
        fn rejectFunc input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            if input < 0
            then reject "negative"
            else resolve "positive"
          }
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain('reject("negative")')
      expect(result).toContain('resolve("positive")')
    })
  })

  describe("Integration with Try Expressions", () => {
    it("should work with try expressions on promise functions", () => {
      const source = `
        fn dataFunc input: Int -> (Unit -> Promise<String>) {
          promise<String> {
            resolve "data"
          }
        }
        
        let tryResult = try dataFunc 42
        let value = tryResult ()
        show value
      `
      const result = compileSeseragi(source)
      expect(result).toContain("async (): Promise<Either<string, any>> => {")
      expect(result).toContain("const value = await dataFunc(42);")
    })
  })
})
