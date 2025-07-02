import { describe, test, expect } from "bun:test"
import { Lexer } from "../src/lexer"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import { TypeInferenceSystem } from "../src/type-inference"
import type * as AST from "../src/ast"
import { compileSeseragi } from "../src/main"

describe("Struct Tests", () => {
  test("should parse struct declaration", () => {
    const source = `
      struct Person {
        name: String,
        age: Int
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.kind).toBe("StructDeclaration")
    expect(structDecl.name).toBe("Person")
    expect(structDecl.fields).toHaveLength(2)

    expect(structDecl.fields[0].name).toBe("name")
    expect(structDecl.fields[0].type.kind).toBe("PrimitiveType")
    expect((structDecl.fields[0].type as AST.PrimitiveType).name).toBe("String")

    expect(structDecl.fields[1].name).toBe("age")
    expect(structDecl.fields[1].type.kind).toBe("PrimitiveType")
    expect((structDecl.fields[1].type as AST.PrimitiveType).name).toBe("Int")
  })

  test("should parse struct instantiation", () => {
    const source = `Person { name: "Alice", age: 30 }`
    const parser = new Parser(source)
    const expr = parser["primaryExpression"]()

    expect(expr.kind).toBe("StructExpression")
    const structExpr = expr as AST.StructExpression
    expect(structExpr.structName).toBe("Person")
    expect(structExpr.fields).toHaveLength(2)

    expect(structExpr.fields[0].name).toBe("name")
    expect(structExpr.fields[0].value.kind).toBe("Literal")

    expect(structExpr.fields[1].name).toBe("age")
    expect(structExpr.fields[1].value.kind).toBe("Literal")
  })

  test("should generate TypeScript interface for struct", () => {
    const source = `
      struct Person {
        name: String,
        age: Int
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()
    const typescript = generateTypeScript(result.statements!)

    expect(typescript).toContain("class Person")
    expect(typescript).toContain("public name: string")
    expect(typescript).toContain("public age: number")
  })

  test("should generate TypeScript for struct instantiation", () => {
    const source = `let person = Person { name: "Alice", age: 30 }`
    const parser = new Parser(source)
    const result = parser.parse()
    const typescript = generateTypeScript(result.statements!)

    expect(typescript).toContain('new Person("Alice", 30)')
  })

  test("should parse struct with multiple field types", () => {
    const source = `
      struct User {
        id: Int,
        name: String,
        email: String,
        isActive: Bool,
        score: Float
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.fields).toHaveLength(5)

    const typescript = generateTypeScript(result.statements!)
    expect(typescript).toContain("class User")
    expect(typescript).toContain("public id: number")
    expect(typescript).toContain("public name: string")
    expect(typescript).toContain("public email: string")
    expect(typescript).toContain("public isActive: boolean")
    expect(typescript).toContain("public score: number")
  })

  test("should parse struct with generic types", () => {
    const source = `
      struct Node {
        value: Int,
        next: Maybe<Node>
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.fields).toHaveLength(2)

    expect(structDecl.fields[1].name).toBe("next")
    expect(structDecl.fields[1].type.kind).toBe("GenericType")
    const nextType = structDecl.fields[1].type as AST.GenericType
    expect(nextType.name).toBe("Maybe")
    expect(nextType.typeArguments).toHaveLength(1)
  })

  test("should parse empty struct", () => {
    const source = `struct Empty {}`
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.fields).toHaveLength(0)

    const typescript = generateTypeScript(result.statements!)
    expect(typescript).toContain("class Empty")
    expect(typescript).toContain("{")
    expect(typescript).toContain("}")
  })

  test("should handle struct field access", () => {
    const source = `
      struct Person {
        name: String,
        age: Int
      }
      
      let person = Person { name: "Bob", age: 25 }
      let name = person.name
    `
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(3)

    const typescript = generateTypeScript(result.statements!)
    expect(typescript).toContain("class Person")
    expect(typescript).toContain('new Person("Bob", 25)')
    expect(typescript).toContain("person.name")
  })

  test("should parse struct with record field", () => {
    const source = `
      struct Employee {
        info: { name: String, department: String },
        salary: Int
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.fields).toHaveLength(2)

    expect(structDecl.fields[0].type.kind).toBe("RecordType")
    const infoType = structDecl.fields[0].type as AST.RecordType
    expect(infoType.fields).toHaveLength(2)
  })

  test("should differentiate struct from type alias", () => {
    const source1 = `type PersonType = { name: String, age: Int }`
    const source2 = `struct PersonStruct { name: String, age: Int }`

    const parser1 = new Parser(source1)
    const result1 = parser1.parse()
    const stmt1 = result1.statements![0]
    expect(stmt1.kind).toBe("TypeAliasDeclaration")

    const parser2 = new Parser(source2)
    const result2 = parser2.parse()
    const stmt2 = result2.statements![0]
    expect(stmt2.kind).toBe("StructDeclaration")

    const ts1 = generateTypeScript(result1.statements!)
    const ts2 = generateTypeScript(result2.statements!)

    expect(ts1).toContain("type PersonType =")
    expect(ts2).toContain("class PersonStruct")
  })

  test("should handle float arithmetic in struct fields correctly", () => {
    const source = `
struct Point {
  x: Float,
  y: Float
}

fn distance p1: Point -> p2: Point -> Float {
  let dx = p2.x - p1.x
  let dy = p2.y - p1.y
  dx * dx + dy * dy
}

let p1 = Point { x: 3.0, y: 0.0 }
let p2 = Point { x: 0.0, y: 4.0 }
distance p1 p2
`
    const output = compileSeseragi(source)

    // プリミティブ型の演算は直接演算子を使用
    expect(output).toContain("(p2.x - p1.x)")
    expect(output).toContain("(p2.y - p1.y)")
    expect(output).toContain("((dx * dx) + (dy * dy))")

    // __dispatchOperatorを使わないこと
    expect(output).not.toContain("__dispatchOperator(p2.x")
  })

  test("should handle struct operator overloading correctly", () => {
    const source = `
struct Vec2 {
  x: Float,
  y: Float
}

impl Vec2 {
  operator + self: Vec2 -> other: Vec2 -> Vec2 =
    Vec2 { x: self.x + other.x, y: self.y + other.y }
}

let v1 = Vec2 { x: 1.0, y: 2.0 }
let v2 = Vec2 { x: 3.0, y: 4.0 }
v1 + v2
`
    const output = compileSeseragi(source)

    // 構造体同士の演算は__dispatchOperatorを使用
    expect(output).toContain("__dispatchOperator(v1")

    // 構造体内部のFloat演算は直接演算子を使用
    expect(output).toContain("(self.x + other.x)")
    expect(output).toContain("(self.y + other.y)")
  })

  test("should handle basic arithmetic without dispatch", () => {
    const source = `
let x = 1 + 2
let y = 3.5 - 1.5
let z = 2 * 3
let w = 10 / 2
`
    const output = compileSeseragi(source)

    // 基本的な算術演算は直接演算子を使用
    expect(output).toContain("(1 + 2)")
    expect(output).toContain("(3.5 - 1.5)")
    expect(output).toContain("(2 * 3)")
    expect(output).toContain("(10 / 2)")

    // __dispatchOperatorを使わないこと
    expect(output).not.toContain("__dispatchOperator")
  })

  test("should generate dispatch helper when using structs", () => {
    const source = `
struct Point {
  x: Float,
  y: Float
}

let p = Point { x: 1.0, y: 2.0 }
p.x
`
    const output = compileSeseragi(source)

    // 構造体を使用している場合はディスパッチヘルパーを生成
    expect(output).toContain("__dispatchMethod")
    expect(output).toContain("__dispatchOperator")
  })

  test("should parse impl block with method using block syntax", () => {
    const source = `
struct Point {
  x: Int,
  y: Int
}

impl Point {
  fn add self -> other: Point -> Point {
    let newX = self.x + other.x
    let newY = self.y + other.y
    Point { x: newX, y: newY }
  }
}
`
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(2)

    const implBlock = result.statements![1] as AST.ImplBlock
    expect(implBlock.kind).toBe("ImplBlock")
    expect(implBlock.methods).toHaveLength(1)

    const method = implBlock.methods[0]
    expect(method.kind).toBe("MethodDeclaration")
    expect(method.name).toBe("add")
    expect(method.body.kind).toBe("BlockExpression")
  })

  test("should parse impl block with operator using block syntax", () => {
    const source = `
struct Vec2 {
  x: Float,
  y: Float
}

impl Vec2 {
  operator + self: Vec2 -> other: Vec2 -> Vec2 {
    let sumX = self.x + other.x
    let sumY = self.y + other.y
    Vec2 { x: sumX, y: sumY }
  }
}
`
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(2)

    const implBlock = result.statements![1] as AST.ImplBlock
    expect(implBlock.kind).toBe("ImplBlock")
    expect(implBlock.operators).toHaveLength(1)

    const operator = implBlock.operators[0]
    expect(operator.kind).toBe("OperatorDeclaration")
    expect(operator.operator).toBe("+")
    expect(operator.body.kind).toBe("BlockExpression")
  })

  test("should compile impl block with mixed syntax (both = and {})", () => {
    const source = `
struct Point {
  x: Int,
  y: Int
}

impl Point {
  fn simple self -> Point = self
  
  fn complex self -> other: Point -> Point {
    let newX = self.x + other.x
    let newY = self.y + other.y
    Point { x: newX, y: newY }
  }
  
  operator + self: Point -> other: Point -> Point = self complex other
  
  operator - self: Point -> other: Point -> Point {
    Point { x: self.x - other.x, y: self.y - other.y }
  }
}

let p1 = Point { x: 1, y: 2 }
let p2 = Point { x: 3, y: 4 }
p1 + p2
`
    const output = compileSeseragi(source)
    
    // エラーなくコンパイルできることを確認
    expect(output).toContain("class Point")
    expect(output).toContain("simple")
    expect(output).toContain("complex")
    expect(output).toContain("__dispatchOperator")
  })

  test("should parse method call with parentheses for zero-argument methods", () => {
    const source = `
struct Point {
  x: Int,
  y: Int
}

impl Point {
  fn magnitude self -> Int = self.x * self.x + self.y * self.y
  fn getX self -> Int = self.x
}

let p = Point { x: 3, y: 4 }
p magnitude()
p getX()
`
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(5)

    // 最後の2つの文は p magnitude() と p getX() の呼び出し
    const magnitudeCall = result.statements![3] as AST.ExpressionStatement
    const getXCall = result.statements![4] as AST.ExpressionStatement

    expect(magnitudeCall.expression.kind).toBe("MethodCall")
    expect(getXCall.expression.kind).toBe("MethodCall")

    const magnitudeMethodCall = magnitudeCall.expression as AST.MethodCall
    const getXMethodCall = getXCall.expression as AST.MethodCall

    expect(magnitudeMethodCall.methodName).toBe("magnitude")
    expect(magnitudeMethodCall.arguments).toHaveLength(0)
    expect(getXMethodCall.methodName).toBe("getX")
    expect(getXMethodCall.arguments).toHaveLength(0)
  })

  test("should compile and run method call with parentheses", () => {
    const source = `
struct Point {
  x: Int,
  y: Int
}

impl Point {
  fn magnitude self -> Int = self.x * self.x + self.y * self.y
  fn double self -> Point {
    Point { x: self.x * 2, y: self.y * 2 }
  }
}

let p = Point { x: 3, y: 4 }
show (p magnitude())
show (p double())
`
    const output = compileSeseragi(source)
    
    // エラーなくコンパイルできることを確認
    expect(output).toContain("class Point")
    expect(output).toContain("magnitude")
    expect(output).toContain("double")
    expect(output).toContain("__dispatchMethod")
  })

  test("should support both parentheses and space-separated method calls", () => {
    const source = `
struct Point {
  x: Int,
  y: Int
}

impl Point {
  fn magnitude self -> Int = self.x * self.x + self.y * self.y
  fn add self -> other: Point -> Point = Point { x: self.x + other.x, y: self.y + other.y }
}

let p1 = Point { x: 3, y: 4 }
let p2 = Point { x: 1, y: 2 }

show (p1 magnitude())      // 括弧付きメソッド呼び出し
show (p1 add p2)           // スペース区切りメソッド呼び出し
`
    const output = compileSeseragi(source)
    
    // 両方の構文がサポートされることを確認
    expect(output).toContain("__dispatchMethod")
    expect(output).toContain("magnitude")
    expect(output).toContain("add")
  })
})
