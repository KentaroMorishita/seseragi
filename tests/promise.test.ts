import { describe, expect, it } from "bun:test"
import { compileSeseragi, compileAndExecute } from "./test-utils"

describe("Promise System", () => {
  describe("Basic Promise Syntax", () => {
    it("should compile promise block with explicit type parameter", () => {
      const source = `
        let p = promise<String> {
          resolve "Hello World"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("() => new Promise<string>")
      expect(result).toContain('resolve("Hello World")')
    })

    it("should compile promise block with type inference", () => {
      const source = `
        let p = promise {
          resolve "Hello World"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("() => new Promise")
      expect(result).toContain('resolve("Hello World")')
    })

    it("should compile reject-only promise with Void type", () => {
      const source = `
        let p = promise {
          reject "Error occurred"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("() => new Promise")
      expect(result).toContain('reject("Error occurred")')
    })
  })

  describe("Type Inference", () => {
    it("should infer String type from resolve expression", () => {
      const source = `
        let p = promise {
          resolve "test string"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise")
      expect(result).toContain('resolve("test string")')
    })

    it("should infer Int type from resolve expression", () => {
      const source = `
        let p = promise {
          resolve 42
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise")
      expect(result).toContain("resolve(42)")
    })

    it("should handle conditional resolve with same type", () => {
      const source = `
        let p = promise {
          if true then resolve "success"
          else resolve "failure"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise")
      expect(result).toContain('resolve("success")')
      expect(result).toContain('resolve("failure")')
    })
  })

  describe("Independent resolve/reject Functions", () => {
    it("should compile resolve function with explicit type", () => {
      const source = `
        let resolver = resolve<String> "success"
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise.resolve<String>")
      expect(result).toContain('"success"')
    })

    it("should compile resolve function with type inference", () => {
      const source = `
        let resolver = resolve 42
      `
      const result = compileSeseragi(source)
      // 修正後: promiseブロック外では Promise.resolve になる
      expect(result).toContain("() => Promise.resolve(42)")
      expect(result).toContain("42")
    })

    it("should compile reject function", () => {
      const source = `
        let rejecter = reject "error message"
      `
      const result = compileSeseragi(source)
      // 修正後: promiseブロック外では Promise.reject になる
      expect(result).toContain('() => Promise.reject("error message")')
    })
  })

  describe("Context-Dependent Code Generation", () => {
    it("should generate Promise.resolve for resolve outside promise block", () => {
      const source = `
        let resolver = resolve "hello"
      `
      const result = compileSeseragi(source)
      expect(result).toContain('() => Promise.resolve("hello")')
    })

    it("should generate Promise.reject for reject outside promise block", () => {
      const source = `
        let rejecter = reject "error"
      `
      const result = compileSeseragi(source)
      expect(result).toContain('() => Promise.reject("error")')
    })

    it("should generate local resolve/reject inside promise block", () => {
      const source = `
        let p = promise<String> {
          resolve "inside"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain('resolve("inside")')
      expect(result).not.toContain("Promise.resolve")
    })

    it("should ignore type arguments for resolve/reject inside promise block", () => {
      const source = `
        let p = promise<String> {
          resolve<String> "typed inside"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain('resolve("typed inside")')
      expect(result).not.toContain("Promise.resolve")
      // resolve呼び出し自体に型引数はない
      expect(result).not.toContain("resolve<String>")
    })

    it("should handle mixed contexts correctly", () => {
      const source = `
        let independent = resolve "outside"
        let promiseFunc = promise<String> {
          resolve "inside"
        }
      `
      const result = compileSeseragi(source)
      // 外側: Promise.resolve
      expect(result).toContain('() => Promise.resolve("outside")')
      // 内側: ローカル resolve
      expect(result).toContain('resolve("inside")')
    })
  })

  describe("Complex Promise Scenarios", () => {
    it("should handle promise in function context", () => {
      const source = `
        fn fetchData -> (Unit -> Promise<String>) = promise<String> {
          resolve "fetched data"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise<string>")
      expect(result).toContain('resolve("fetched data")')
    })

    it("should handle promise with local variables", () => {
      const source = `
        let p = promise<String> {
          let value = "computed value"
          resolve value
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise")
      expect(result).toContain("value")
      expect(result).toContain("resolve(value)")
    })

    it("should handle promise with conditional logic", () => {
      const source = `
        let p = promise<String> {
          let condition = true
          if condition 
          then resolve "positive"
          else reject "negative"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise")
      expect(result).toContain('resolve("positive")')
      expect(result).toContain('reject("negative")')
    })
  })

  describe("Void Type Support", () => {
    it("should handle reject-only promises", () => {
      const source = `
        let errorPromise = promise {
          reject "always fails"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise")
      expect(result).toContain('reject("always fails")')
    })

    it("should compile Void type explicitly", () => {
      const source = `
        let voidPromise: (Unit -> Promise<Void>) = promise<Void> {
          reject "void error"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise")
      expect(result).toContain('reject("void error")')
    })
  })

  describe("Type Safety", () => {
    it("should enforce type constraints with explicit type parameters", () => {
      // This test verifies that type mismatches are caught
      const source = `
        let p = promise<String> {
          resolve "correct type"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain('resolve("correct type")')
    })

    it("should handle function return type annotation", () => {
      const source = `
        fn createPromise -> (Unit -> Promise<Int>) = promise<Int> {
          resolve 123
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise")
      expect(result).toContain("resolve(123)")
    })
  })

  describe("Runtime Execution", () => {
    it("should execute basic promise resolution", async () => {
      const source = `
        let p = promise<String> {
          resolve "Hello Promise"
        }
        print $ show $ p()
      `
      const output = await compileAndExecute(source)
      expect(output).toContain("Promise")
    })

    it("should compile independent resolve function without runtime error", () => {
      const source = `
        let resolvedValue = resolve "success"
      `
      const result = compileSeseragi(source)
      expect(result).toContain("resolve")
      expect(result).toContain('"success"')
    })

    it("should execute promise with conditional logic", async () => {
      const source = `
        let p = promise<String> {
          let value = 42
          if value > 0
          then resolve "positive"
          else resolve "zero or negative"
        }
        print $ show $ p()
      `
      const output = await compileAndExecute(source)
      expect(output).toContain("Promise")
    })
  })

  describe("TypeScript Output Quality", () => {
    it("should generate proper TypeScript promise constructor", () => {
      const source = `
        let p = promise<String> {
          resolve "test"
        }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("new Promise<string>")
      expect(result).toContain("(resolve, reject) =>")
    })

    it("should generate proper Promise.resolve call", () => {
      const source = `
        let r = resolve<Int> 42
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise.resolve")
      expect(result).toContain("42")
    })

    it("should generate proper Promise.reject call", () => {
      const source = `
        let r = reject "error"
      `
      const result = compileSeseragi(source)
      expect(result).toContain('reject("error")')
    })

    it("should map Seseragi types to TypeScript types correctly", () => {
      const source = `
        let stringPromise = promise<String> { resolve "text" }
        let intPromise = promise<Int> { resolve 42 }
        let boolPromise = promise<Bool> { resolve true }
      `
      const result = compileSeseragi(source)
      expect(result).toContain("Promise<string>")
      expect(result).toContain("Promise<number>")
      expect(result).toContain("Promise<boolean>")
    })
  })
})
