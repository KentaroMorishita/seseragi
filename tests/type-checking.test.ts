import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

describe("Type Checking Features", () => {
  describe("ワイルドカード型", () => {
    test("Maybe<_>でのワイルドカード型マッチング", () => {
      const source = `
let maybeStr: Maybe<String> = Just("hello")
let maybeInt: Maybe<Int> = Just(42)
let nothing: Maybe<String> = Nothing

let check1 = maybeStr is Maybe<_>
let check2 = maybeInt is Maybe<_>
let check3 = nothing is Maybe<_>
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)

      // 型推論をテスト
      const typeInference = new TypeInferenceSystem()
      const program = new AST.Program(parseResult.statements || [])
      const result = typeInference.infer(program)

      expect(result.errors).toHaveLength(0)

      // check1, check2, check3 すべて Bool 型になることを確認
      expect(result.environment.get("check1")?.kind).toBe("PrimitiveType")
      expect((result.environment.get("check1") as AST.PrimitiveType).name).toBe(
        "Bool"
      )
      expect(result.environment.get("check2")?.kind).toBe("PrimitiveType")
      expect((result.environment.get("check2") as AST.PrimitiveType).name).toBe(
        "Bool"
      )
      expect(result.environment.get("check3")?.kind).toBe("PrimitiveType")
      expect((result.environment.get("check3") as AST.PrimitiveType).name).toBe(
        "Bool"
      )
    })

    test("Either<_, Int>でのワイルドカード型マッチング", () => {
      const source = `
let eitherStr: Either<String, Int> = Left("error")
let eitherInt: Either<Bool, Int> = Right(42)

let check1 = eitherStr is Either<_, Int>
let check2 = eitherInt is Either<_, Int>
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)

      // 型推論をテスト
      const typeInference = new TypeInferenceSystem()
      const program = new AST.Program(parseResult.statements || [])
      const result = typeInference.infer(program)

      expect(result.errors).toHaveLength(0)

      // check1, check2 すべて Bool 型になることを確認
      expect(result.environment.get("check1")?.kind).toBe("PrimitiveType")
      expect((result.environment.get("check1") as AST.PrimitiveType).name).toBe(
        "Bool"
      )
      expect(result.environment.get("check2")?.kind).toBe("PrimitiveType")
      expect((result.environment.get("check2") as AST.PrimitiveType).name).toBe(
        "Bool"
      )
    })

    test("レコード型でのワイルドカード型マッチング", () => {
      const source = `
let user = { name: "Alice", age: 30, active: True }
let check = user is { name: String, age: _, active: _ }
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)

      // 型推論をテスト
      const typeInference = new TypeInferenceSystem()
      const program = new AST.Program(parseResult.statements || [])
      const result = typeInference.infer(program)

      expect(result.errors).toHaveLength(0)

      // check は Bool 型になることを確認
      expect(result.environment.get("check")?.kind).toBe("PrimitiveType")
      expect((result.environment.get("check") as AST.PrimitiveType).name).toBe(
        "Bool"
      )
    })
  })

  describe("is キーワード", () => {
    test("型エイリアスとの型判定", () => {
      const source = `
type User = { name: String, age: Int }
let user = { name: "Alice", age: 30 }
let check = user is User
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)
      expect(parseResult.statements).toHaveLength(3)

      // is式がパースされていることを確認
      const varDecl = parseResult.statements?.[2] as AST.VariableDeclaration
      expect(varDecl.kind).toBe("VariableDeclaration")
      expect(varDecl.name).toBe("check")
    })

    test("レコード型との型判定", () => {
      const source = `
let user = { name: "Alice", age: 30 }
let check = user is { name: String, age: Int }
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)
      expect(parseResult.statements).toHaveLength(2)
    })

    test("配列型との型判定", () => {
      const source = `
let numbers = [1, 2, 3]
let check = numbers is Array<Int>
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)
      expect(parseResult.statements).toHaveLength(2)
    })

    test("is式の型推論 - Bool型を返す", () => {
      const source = `
type User = { name: String }
let user = { name: "Alice" }
let check = user is User
`

      const parser = new Parser(source)
      const parseResult = parser.parse()
      const program = new AST.Program(parseResult.statements || [])

      const typeInference = new TypeInferenceSystem()
      const typeResult = typeInference.infer(program)

      expect(typeResult.errors).toHaveLength(0)

      // check変数がBool型に推論されることを確認
      const checkType = typeResult.environment.get("check")
      expect(checkType?.name).toBe("Bool")
    })

    test("is式のTypeScript生成", () => {
      const source = `
type User = { name: String, age: Int }
let user = { name: "Alice", age: 30 }
let check = user is User
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      const generated = generateTypeScript(parseResult.statements || [])
      expect(generated).toContain('ssrgIsType(user, "User", "user")')
    })
  })

  describe("typeof 関数", () => {
    test("型エイリアスの型名取得", () => {
      const source = `
type User = { name: String, age: Int }
let user = { name: "Alice", age: 30 }
let typeName = typeof(user)
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)
      expect(parseResult.statements).toHaveLength(3)

      // typeof関数呼び出しがパースされていることを確認
      const varDecl = parseResult.statements?.[2] as AST.VariableDeclaration
      expect(varDecl.kind).toBe("VariableDeclaration")
      expect(varDecl.name).toBe("typeName")
    })

    test("無名レコード型の構造表示", () => {
      const source = `
let obj = { x: 10, y: 20 }
let typeStr = typeof(obj)
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)
      expect(parseResult.statements).toHaveLength(2)
    })

    test("配列型の型名取得", () => {
      const source = `
let numbers = [1, 2, 3]
let arrayType = typeof(numbers)
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)
      expect(parseResult.statements).toHaveLength(2)
    })

    test("typeof関数の型推論 - String型を返す", () => {
      const source = `
let obj = { x: 10 }
let typeStr = typeof(obj)
`

      const parser = new Parser(source)
      const parseResult = parser.parse()
      const program = new AST.Program(parseResult.statements || [])

      const typeInference = new TypeInferenceSystem()
      const typeResult = typeInference.infer(program)

      expect(typeResult.errors).toHaveLength(0)

      // typeStr変数がString型に推論されることを確認
      const typeStrType = typeResult.environment.get("typeStr")
      expect(typeStrType?.name).toBe("String")
    })

    test("typeof関数のTypeScript生成", () => {
      const source = `
let obj = { x: 10, y: 20 }
let typeStr = typeof(obj)
`

      const parser = new Parser(source)
      const parseResult = parser.parse()

      const generated = generateTypeScript(parseResult.statements || [])
      expect(generated).toContain('ssrgTypeOf(obj, "obj")')
    })
  })

  describe("型等価性チェック", () => {
    test("構造的に同じレコード型は等価", () => {
      const source = `
type User = { name: String, age: Int }
let obj = { name: "Alice", age: 30 }
let check = obj is User
`

      const parser = new Parser(source)
      const parseResult = parser.parse()
      const program = new AST.Program(parseResult.statements || [])

      const typeInference = new TypeInferenceSystem()
      const typeResult = typeInference.infer(program)

      expect(typeResult.errors).toHaveLength(0)
    })

    test("構造的に異なるレコード型は非等価", () => {
      const source = `
type User = { name: String, age: Int }
let obj = { name: "Alice" }
let check = obj is User
`

      const parser = new Parser(source)
      const parseResult = parser.parse()
      const program = new AST.Program(parseResult.statements || [])

      const typeInference = new TypeInferenceSystem()
      const typeResult = typeInference.infer(program)

      // 型推論エラーは発生しない（isは実行時チェック）
      expect(typeResult.errors).toHaveLength(0)
    })
  })
})
