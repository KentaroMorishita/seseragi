import { describe, expect, test } from "bun:test"
import { compileAndExecute } from "./test-utils"

describe("Runtime Type System", () => {
  describe("typeof function", () => {
    test("struct型のtypeof - 構造体名と構造を返す", async () => {
      const source = `
struct Person { name: String, age: Int }
let person = Person { name: "太郎", age: 25 }
let result = typeof person
print result
`
      const output = await compileAndExecute(source)
      expect(output.trim()).toBe("Person { age: Int, name: String }")
    })

    test("type alias (レコード型) のtypeof - 構造のみ返す", async () => {
      const source = `
type Point = { x: Int, y: Int }
let point = { x: 10, y: 20 }
let result = typeof point
print result
`
      const output = await compileAndExecute(source)
      expect(output.trim()).toBe("{ x: Int, y: Int }")
    })

    test("プリミティブ型のtypeof", async () => {
      const source = `
let number = 42
let text = "hello"
print $ typeof number
print $ typeof text
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("Int")
      expect(lines[1]).toBe("String")
    })

    test("Maybe型のtypeof", async () => {
      const source = `
let maybeStr: Maybe<String> = Just("hello")
let nothing: Maybe<String> = Nothing
print $ typeof maybeStr
print $ typeof nothing
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("Maybe<String>")
      expect(lines[1]).toBe("Maybe<String>")
    })

    test("Either型のtypeof", async () => {
      const source = `
let eitherStr: Either<String, Int> = Left("error")
let eitherInt: Either<String, Int> = Right(42)
print $ typeof eitherStr
print $ typeof eitherInt
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("Either<String, Int>")
      expect(lines[1]).toBe("Either<String, Int>")
    })

    test("フィールド順序正規化", async () => {
      const source = `
let obj1 = { x: 10, y: 20 }
let obj2 = { y: 30, x: 40 }
print $ typeof obj1
print $ typeof obj2
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("{ x: Int, y: Int }")
      expect(lines[1]).toBe("{ x: Int, y: Int }")
    })
  })

  describe("typeof' function", () => {
    test("type alias情報付きのtypeof'", async () => {
      const source = `
type Point = { x: Int, y: Int }
type Vector = { x: Int, y: Int }
let point = { x: 10, y: 20 }
let result = typeof' point
print result
`
      const output = await compileAndExecute(source)
      expect(output.trim()).toBe("{ x: Int, y: Int } (Point, Vector)")
    })

    test("単一type aliasの情報表示", async () => {
      const source = `
type User = { name: String, age: Int }
let user: User = { name: "Alice", age: 30 }
let result = typeof' user
print result
`
      const output = await compileAndExecute(source)
      expect(output.trim()).toBe("User (User)")
    })
  })

  describe("is operator", () => {
    test("struct型のis判定 - 構造体名で判定", async () => {
      const source = `
struct Person { name: String, age: Int }
struct Animal { name: String, species: String }
let person = Person { name: "太郎", age: 25 }
print $ person is Person
print $ person is Animal
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("false")
    })

    test("type aliasのis判定", async () => {
      const source = `
type Point = { x: Int, y: Int }
type Vector = { x: Int, y: Int }
let point = { x: 10, y: 20 }
print $ point is Point
print $ point is Vector
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
    })

    test("構造的型判定 - フィールド順序無関係", async () => {
      const source = `
let obj1 = { x: 10, y: 20 }
let obj2 = { y: 30, x: 40 }
print $ obj1 is { x: Int, y: Int }
print $ obj2 is { x: Int, y: Int }
print $ obj1 is { y: Int, x: Int }
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("true")
    })

    test("Maybe型のis判定", async () => {
      const source = `
let maybeStr: Maybe<String> = Just("hello")
let maybeInt: Maybe<Int> = Just(42)
let nothing: Maybe<String> = Nothing
print $ maybeStr is Maybe<String>
print $ maybeInt is Maybe<Int>
print $ nothing is Maybe<String>
print $ maybeStr is Maybe<Int>
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("true")
      expect(lines[3]).toBe("false")
    })

    test("Either型のis判定", async () => {
      const source = `
let eitherStr: Either<String, Int> = Left("error")
let eitherInt: Either<Bool, Int> = Right(42)
print $ eitherStr is Either<String, Int>
print $ eitherInt is Either<Bool, Int>
print $ eitherStr is Either<Bool, Int>
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("false")
    })
  })

  describe("wildcard type matching", () => {
    test("Maybe<_>のワイルドカードマッチング", async () => {
      const source = `
let maybeStr: Maybe<String> = Just("hello")
let maybeInt: Maybe<Int> = Just(42)
let nothing: Maybe<String> = Nothing
print $ maybeStr is Maybe<_>
print $ maybeInt is Maybe<_>
print $ nothing is Maybe<_>
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("true")
    })

    test("Either<_, _>のワイルドカードマッチング", async () => {
      const source = `
let eitherStr: Either<String, Int> = Left("error")
let eitherInt: Either<Bool, Int> = Right(42)
print $ eitherStr is Either<_, _>
print $ eitherInt is Either<_, _>
print $ eitherStr is Either<String, _>
print $ eitherStr is Either<_, Int>
print $ eitherInt is Either<Bool, _>
print $ eitherInt is Either<_, Int>
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("true")
      expect(lines[3]).toBe("true")
      expect(lines[4]).toBe("true")
      expect(lines[5]).toBe("true")
    })

    test("Array<_>のワイルドカードマッチング", async () => {
      const source = `
let numbers: Array<Int> = [1, 2, 3]
print $ numbers is Array<_>
`
      const output = await compileAndExecute(source)
      expect(output.trim()).toBe("true")
    })

    test("レコード型ワイルドカードマッチング", async () => {
      const source = `
let user = { name: "Alice", age: 30 }
print $ user is { name: String, age: _ }
print $ user is { name: _, age: Int }
print $ user is { name: _, age: _ }
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("true")
    })

    test("Tupleワイルドカードマッチング", async () => {
      const source = `
let tuple: (Int, String) = (42, "hello")
print $ tuple is (_, _)
print $ tuple is (Int, _)
print $ tuple is (_, String)
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("true")
    })
  })

  describe("edge cases", () => {
    test("struct vs type alias判定", async () => {
      const source = `
struct Person { name: String, age: Int }
type User = { name: String, age: Int }
let person = Person { name: "太郎", age: 25 }
let user: User = { name: "Alice", age: 30 }
print $ person is Person
print $ user is User
print $ person is User  // false - struct vs type alias
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("true")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("false")
    })

    test("変数型テーブル情報の保持確認", async () => {
      const source = `
let eitherTest: Either<String, Int> = Left("test")
print $ typeof eitherTest
print $ eitherTest is Either<String, Int>
print $ eitherTest is Either<String, _>
`
      const output = await compileAndExecute(source)
      const lines = output.trim().split("\n")
      expect(lines[0]).toBe("Either<String, Int>")
      expect(lines[1]).toBe("true")
      expect(lines[2]).toBe("true")
    })
  })
})
