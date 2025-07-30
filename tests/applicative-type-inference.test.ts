import { expect, test, describe } from "bun:test"
import { Lexer } from "../src/lexer"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

function typeInferAndCheck(code: string) {
  const lexer = new Lexer(code)
  const tokens = lexer.tokenize()
  const parser = new Parser(tokens)
  const ast = parser.parse()
  const inference = new TypeInferenceSystem()
  const result = inference.infer(ast)
  
  return {
    errors: result.errors,
    inferredTypes: result.inferredTypes
  }
}

describe("Applicative type inference", () => {
  test("Maybe applicative type inference", () => {
    const code = `
      let maybeFunc: Maybe<(Int -> String)> = Some $ \\x -> "Number: " ++ toString x
      let maybeValue: Maybe<Int> = Some 42
      let result = maybeFunc <*> maybeValue
    `
    const { errors, inferredTypes } = typeInferAndCheck(code)
    expect(errors).toHaveLength(0)
    expect(inferredTypes.get("result")?.toString()).toBe("Maybe<String>")
  })

  test("Either applicative type inference", () => {
    const code = `
      let eitherFunc: Either<String, (Int -> Int)> = Right $ \\x -> x * 2
      let eitherValue: Either<String, Int> = Right 21
      let result = eitherFunc <*> eitherValue
    `
    const { errors, inferredTypes } = typeInferAndCheck(code)
    expect(errors).toHaveLength(0)
    expect(inferredTypes.get("result")?.toString()).toBe("Either<String, Int>")
  })

  test("List applicative type inference", () => {
    const code = `
      let listFunc: List<(Int -> Int)> = [(\\x -> x + 1), (\\x -> x * 2)]
      let listValue: List<Int> = [1, 2, 3]
      let result = listFunc <*> listValue
    `
    const { errors, inferredTypes } = typeInferAndCheck(code)
    expect(errors).toHaveLength(0)
    expect(inferredTypes.get("result")?.toString()).toBe("List<Int>")
  })

  test("Task applicative type inference", () => {
    const code = `
      let taskFunc: Task<(Int -> String)> = Task $ resolve $ \\x -> "Value: " ++ toString x
      let taskValue: Task<Int> = Task $ resolve 100
      let result = taskFunc <*> taskValue
    `
    const { errors, inferredTypes } = typeInferAndCheck(code)
    expect(errors).toHaveLength(0)
    expect(inferredTypes.get("result")?.toString()).toBe("Task<String>")
  })

  test("Array applicative type inference", () => {
    const code = `
      let arrayFunc: Array = [(\\x -> x + 1), (\\x -> x * 2)]
      let arrayValue: Array = [10, 20, 30]
      let result = arrayFunc <*> arrayValue
    `
    const { errors, inferredTypes } = typeInferAndCheck(code)
    expect(errors).toHaveLength(0)
    expect(inferredTypes.get("result")?.toString()).toBe("Array")
  })

  test("Functor map type inference with Maybe", () => {
    const code = `
      let maybeValue: Maybe<Int> = Some 42
      let result = maybeValue <$> (\\x -> x * 2)
    `
    const { errors, inferredTypes } = typeInferAndCheck(code)
    expect(errors).toHaveLength(0)
    expect(inferredTypes.get("result")?.toString()).toBe("Maybe<Int>")
  })

  test("Functor map type inference with Task", () => {
    const code = `
      let taskValue: Task<Int> = Task $ resolve 42
      let result = taskValue <$> (\\x -> toString x)
    `
    const { errors, inferredTypes } = typeInferAndCheck(code)
    expect(errors).toHaveLength(0)
    expect(inferredTypes.get("result")?.toString()).toBe("Task<String>")
  })

  test("Functor map type inference with List", () => {
    const code = `
      let listValue: List<Int> = [1, 2, 3]
      let result = listValue <$> (\\x -> x * x)
    `
    const { errors, inferredTypes } = typeInferAndCheck(code)
    expect(errors).toHaveLength(0)
    expect(inferredTypes.get("result")?.toString()).toBe("List<Int>")
  })
})