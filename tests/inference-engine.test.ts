/**
 * 新しいInference Engineのテスト
 *
 * TypeInferenceSystemからの移行をテストする
 */

import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { TypeConstraint } from "../src/inference/constraints"
import {
  addConstraint,
  addError,
  bindType,
  createEmptyContext,
  freshTypeVariable,
  generateConstraintsForExpression,
  generateConstraintsForStatement,
  getTypeOfNode,
  getTypeOfVariable,
  infer,
  type InferenceContext,
  isSubtype,
  lookupType,
  popScope,
  pushScope,
  resolveTypeAlias,
  solveConstraints,
  unify,
  unifyOrThrow,
} from "../src/inference/engine"
import { typeToString } from "../src/inference/type-formatter"
import { Parser } from "../src/parser"

describe("InferenceContext", () => {
  test("createEmptyContext creates valid context", () => {
    const ctx = createEmptyContext()

    expect(ctx.nextVarId).toBe(1000)
    expect(ctx.constraints).toEqual([])
    expect(ctx.errors).toEqual([])
    expect(ctx.environment.size).toBe(0)
  })

  test("freshTypeVariable generates unique IDs", () => {
    const ctx = createEmptyContext()

    const tv1 = freshTypeVariable(ctx, 1, 1)
    const tv2 = freshTypeVariable(ctx, 1, 1)
    const tv3 = freshTypeVariable(ctx, 1, 1)

    expect(tv1.id).toBe(1000)
    expect(tv2.id).toBe(1001)
    expect(tv3.id).toBe(1002)
    expect(ctx.nextVarId).toBe(1003)
  })

  test("addConstraint adds constraint to context", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)
    const tv = freshTypeVariable(ctx, 1, 1)
    const constraint = new TypeConstraint(tv, intType, 1, 1)

    addConstraint(ctx, constraint)

    expect(ctx.constraints.length).toBe(1)
    expect(ctx.constraints[0]).toBe(constraint)
  })

  test("addError adds error to context", () => {
    const ctx = createEmptyContext()

    addError(ctx, "Type mismatch", 10, 5, "in function call")

    expect(ctx.errors.length).toBe(1)
    expect(ctx.errors[0].message).toBe("Type mismatch")
    expect(ctx.errors[0].line).toBe(10)
    expect(ctx.errors[0].column).toBe(5)
  })

  test("bindType and lookupType work correctly", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)

    bindType(ctx, "x", intType)
    const result = lookupType(ctx, "x")

    expect(result).toBe(intType)
    expect(lookupType(ctx, "y")).toBeUndefined()
  })

  test("pushScope and popScope create nested scopes", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)
    const stringType = new AST.PrimitiveType("String", 1, 1)

    bindType(ctx, "x", intType)

    const oldEnv = pushScope(ctx)
    bindType(ctx, "x", stringType) // Shadow x
    bindType(ctx, "y", intType)

    expect(lookupType(ctx, "x")).toBe(stringType) // New binding
    expect(lookupType(ctx, "y")).toBe(intType)

    popScope(ctx, oldEnv)

    expect(lookupType(ctx, "x")).toBe(intType) // Original binding
    expect(lookupType(ctx, "y")).toBeUndefined()
  })
})

describe("Unifier", () => {
  test("unify identical primitive types", () => {
    const ctx = createEmptyContext()
    const int1 = new AST.PrimitiveType("Int", 1, 1)
    const int2 = new AST.PrimitiveType("Int", 1, 1)

    const result = unify(ctx, int1, int2)

    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.substitution.isEmpty()).toBe(true)
    }
  })

  test("unify fails for different primitive types", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)
    const stringType = new AST.PrimitiveType("String", 1, 1)

    const result = unify(ctx, intType, stringType)

    expect(result.success).toBe(false)
  })

  test("unify type variable with concrete type", () => {
    const ctx = createEmptyContext()
    const tv = freshTypeVariable(ctx, 1, 1)
    const intType = new AST.PrimitiveType("Int", 1, 1)

    const result = unify(ctx, tv, intType)

    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.substitution.get(tv.id)).toEqual(intType)
    }
  })

  test("unify two type variables", () => {
    const ctx = createEmptyContext()
    const tv1 = freshTypeVariable(ctx, 1, 1)
    const tv2 = freshTypeVariable(ctx, 1, 1)

    const result = unify(ctx, tv1, tv2)

    expect(result.success).toBe(true)
    if (result.success) {
      // tv1 should be mapped to tv2
      expect(result.substitution.get(tv1.id)).toEqual(tv2)
    }
  })

  test("unifyOrThrow throws on failure", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)
    const stringType = new AST.PrimitiveType("String", 1, 1)

    expect(() => unifyOrThrow(ctx, intType, stringType)).toThrow()
  })

  test("unify function types", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)
    const stringType = new AST.PrimitiveType("String", 1, 1)
    const tv = freshTypeVariable(ctx, 1, 1)

    const ft1 = new AST.FunctionType(intType, stringType, 1, 1)
    const ft2 = new AST.FunctionType(intType, tv, 1, 1)

    const result = unify(ctx, ft1, ft2)

    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.substitution.get(tv.id)).toEqual(stringType)
    }
  })

  test("unify generic types", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)
    const tv = freshTypeVariable(ctx, 1, 1)

    const gt1 = new AST.GenericType("Maybe", [intType], 1, 1)
    const gt2 = new AST.GenericType("Maybe", [tv], 1, 1)

    const result = unify(ctx, gt1, gt2)

    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.substitution.get(tv.id)).toEqual(intType)
    }
  })

  test("unify tuple types", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)
    const stringType = new AST.PrimitiveType("String", 1, 1)
    const tv = freshTypeVariable(ctx, 1, 1)

    const tt1 = new AST.TupleType([intType, stringType], 1, 1)
    const tt2 = new AST.TupleType([intType, tv], 1, 1)

    const result = unify(ctx, tt1, tt2)

    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.substitution.get(tv.id)).toEqual(stringType)
    }
  })

  test("unify fails for tuples of different lengths", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)

    const tt1 = new AST.TupleType([intType, intType], 1, 1)
    const tt2 = new AST.TupleType([intType, intType, intType], 1, 1)

    const result = unify(ctx, tt1, tt2)

    expect(result.success).toBe(false)
  })
})

describe("Subtype", () => {
  test("isSubtype for identical types", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)

    expect(isSubtype(ctx, intType, intType)).toBe(true)
  })

  test("isSubtype for record types with extra fields", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)

    // { x: Int, y: Int } <: { x: Int }
    const subRecord = new AST.RecordType(
      [
        new AST.RecordField("x", intType, 1, 1),
        new AST.RecordField("y", intType, 1, 1),
      ],
      1,
      1
    )
    const superRecord = new AST.RecordType(
      [new AST.RecordField("x", intType, 1, 1)],
      1,
      1
    )

    expect(isSubtype(ctx, subRecord, superRecord)).toBe(true)
    expect(isSubtype(ctx, superRecord, subRecord)).toBe(false)
  })
})

describe("Type Alias Resolution", () => {
  test("resolveTypeAlias for primitive types", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)

    // No alias defined, should return same type
    const result = resolveTypeAlias(ctx, intType)
    expect(result.kind).toBe("PrimitiveType")
    expect((result as AST.PrimitiveType).name).toBe("Int")
  })

  test("resolveTypeAlias resolves defined alias", () => {
    const ctx = createEmptyContext()
    const intType = new AST.PrimitiveType("Int", 1, 1)

    // Define alias: UserId = Int
    ctx.environment.set("UserId", intType)

    const aliasType = new AST.PrimitiveType("UserId", 1, 1)
    const result = resolveTypeAlias(ctx, aliasType)

    expect(result.kind).toBe("PrimitiveType")
    expect((result as AST.PrimitiveType).name).toBe("Int")
  })
})

// ============================================================
// Integration Tests - Full Pipeline
// ============================================================

// デフォルト環境を作成（true/false を登録）
function createDefaultEnv(): Map<string, AST.Type> {
  const env = new Map<string, AST.Type>()
  // 組み込みブール値
  env.set("true", new AST.PrimitiveType("Bool", 0, 0))
  env.set("false", new AST.PrimitiveType("Bool", 0, 0))
  return env
}

// ヘルパー: 式をパースして制約生成・解決
function inferExpression(source: string) {
  const parser = new Parser(source)
  const program = parser.parse()
  const ctx = createEmptyContext()
  const env = createDefaultEnv()

  if (
    program.statements.length > 0 &&
    program.statements[0]?.kind === "ExpressionStatement"
  ) {
    const exprStmt = program.statements[0] as AST.ExpressionStatement
    const type = generateConstraintsForExpression(ctx, exprStmt.expression, env)
    const substitution = solveConstraints(ctx)
    return {
      type: substitution.apply(type),
      ctx,
      env,
      errors: ctx.errors,
    }
  }

  throw new Error("Expected expression statement")
}

// ヘルパー: プログラムをパースして制約生成・解決
function inferProgram(source: string) {
  const parser = new Parser(source)
  const program = parser.parse()
  const ctx = createEmptyContext()
  const env = createDefaultEnv()

  for (const stmt of program.statements) {
    generateConstraintsForStatement(ctx, stmt, env)
  }

  const substitution = solveConstraints(ctx)

  const resolvedEnv = new Map<string, AST.Type>()
  for (const [name, type] of env) {
    resolvedEnv.set(name, substitution.apply(type))
  }

  return {
    ctx,
    env: resolvedEnv,
    substitution,
    errors: ctx.errors,
  }
}

describe("Engine Integration - Literals", () => {
  test("Int literal", () => {
    const result = inferExpression("42")
    expect(typeToString(result.type)).toBe("Int")
  })

  test("Float literal", () => {
    const result = inferExpression("3.14")
    expect(typeToString(result.type)).toBe("Float")
  })

  test("String literal", () => {
    const result = inferExpression('"hello"')
    expect(typeToString(result.type)).toBe("String")
  })

  test("Bool literal", () => {
    const result = inferExpression("true")
    expect(typeToString(result.type)).toBe("Bool")
  })
})

describe("Engine Integration - Binary Operations", () => {
  test("Int addition", () => {
    const result = inferExpression("1 + 2")
    expect(typeToString(result.type)).toBe("Int")
  })

  test("Float addition", () => {
    // Seseragiでは + はFloat同士でも動作
    const result = inferExpression("1.0 + 2.0")
    expect(typeToString(result.type)).toBe("Float")
  })

  test("Comparison", () => {
    const result = inferExpression("1 < 2")
    expect(typeToString(result.type)).toBe("Bool")
  })

  test("String literal type", () => {
    // 文字列リテラルの型確認
    const result = inferExpression('"hello"')
    expect(typeToString(result.type)).toBe("String")
  })
})

describe("Engine Integration - Variable Declaration", () => {
  test("Variable with annotation", () => {
    const result = inferProgram("let x: Int = 42")
    expect(result.errors).toHaveLength(0)
    expect(typeToString(result.env.get("x")!)).toBe("Int")
  })

  test("Variable without annotation", () => {
    const result = inferProgram("let x = 42")
    expect(result.errors).toHaveLength(0)
    expect(typeToString(result.env.get("x")!)).toBe("Int")
  })

  test("Multiple variables", () => {
    const result = inferProgram(`
      let x = 1
      let y = 2
      let z = x + y
    `)
    expect(result.errors).toHaveLength(0)
    expect(typeToString(result.env.get("z")!)).toBe("Int")
  })
})

describe("Engine Integration - Function Declaration", () => {
  test("Simple function", () => {
    // Seseragiの関数構文: fn name param: Type -> param2: Type2 -> ReturnType = body
    const result = inferProgram("fn add x: Int -> y: Int -> Int = x + y")
    expect(result.errors).toHaveLength(0)
    const addType = result.env.get("add")
    expect(addType).toBeDefined()
    // 関数型はカリー化形式で表示される
    expect(typeToString(addType!)).toBe("(Int -> (Int -> Int))")
  })

  test("Unit function", () => {
    // 引数なしの場合: fn name -> ReturnType = body
    const result = inferProgram("fn getZero -> Int = 0")
    expect(result.errors).toHaveLength(0)
    expect(typeToString(result.env.get("getZero")!)).toBe("(Unit -> Int)")
  })
})

describe("Engine Integration - Lambda", () => {
  test("Lambda expression", () => {
    // Seseragiのラムダ構文: \param: Type -> body
    const result = inferExpression("\\x: Int -> x + 1")
    expect(typeToString(result.type)).toBe("(Int -> Int)")
  })

  test("Lambda in variable", () => {
    const result = inferProgram("let f = \\x: Int -> x * 2")
    expect(result.errors).toHaveLength(0)
    expect(typeToString(result.env.get("f")!)).toBe("(Int -> Int)")
  })
})

describe("Engine Integration - Tuple", () => {
  test("Tuple literal", () => {
    const result = inferExpression('(1, true, "hi")')
    expect(typeToString(result.type)).toBe("(Int, Bool, String)")
  })
})

describe("Engine Integration - Array", () => {
  test("Array literal", () => {
    const result = inferExpression("[1, 2, 3]")
    expect(typeToString(result.type)).toBe("Array<Int>")
  })
})

describe("Engine Integration - Conditional", () => {
  test("If-then-else", () => {
    const result = inferExpression("if true then 1 else 2")
    expect(typeToString(result.type)).toBe("Int")
  })
})

describe("Engine Integration - Record", () => {
  test("Record literal", () => {
    const result = inferExpression('{ name: "Alice", age: 30 }')
    expect(result.type.kind).toBe("RecordType")
  })
})

describe("Engine Integration - Struct", () => {
  test("Struct declaration", () => {
    const result = inferProgram(`
      struct Point {
        x: Float,
        y: Float
      }
    `)
    expect(result.errors).toHaveLength(0)
    expect(result.env.has("Point")).toBe(true)
  })
})

describe("Engine Integration - ADT", () => {
  test("Simple ADT", () => {
    // Seseragi ADT構文: type Name = | Variant1 | Variant2
    const result = inferProgram("type Color = | Red | Green | Blue")
    expect(result.errors).toHaveLength(0)
    expect(result.env.has("Red")).toBe(true)
    expect(result.env.has("Green")).toBe(true)
    expect(result.env.has("Blue")).toBe(true)
  })
})

describe("Engine Integration - Solver", () => {
  test("Basic constraint solving", () => {
    const ctx = createEmptyContext()
    const t1 = freshTypeVariable(ctx, 1, 1)
    const intType = new AST.PrimitiveType("Int", 1, 1)

    addConstraint(ctx, new TypeConstraint(t1, intType, 1, 1))
    const substitution = solveConstraints(ctx)

    expect(typeToString(substitution.apply(t1))).toBe("Int")
  })

  test("Function type constraint", () => {
    const ctx = createEmptyContext()
    const t1 = freshTypeVariable(ctx, 1, 1)
    const t2 = freshTypeVariable(ctx, 1, 1)
    const funcType1 = new AST.FunctionType(t1, t2, 1, 1)

    const intType = new AST.PrimitiveType("Int", 1, 1)
    const boolType = new AST.PrimitiveType("Bool", 1, 1)
    const funcType2 = new AST.FunctionType(intType, boolType, 1, 1)

    addConstraint(ctx, new TypeConstraint(funcType1, funcType2, 1, 1))
    const substitution = solveConstraints(ctx)

    expect(typeToString(substitution.apply(t1))).toBe("Int")
    expect(typeToString(substitution.apply(t2))).toBe("Bool")
  })
})

// ============================================================
// Main API Tests - infer()
// ============================================================

describe("infer() API", () => {
  test("infer simple program", () => {
    const parser = new Parser("let x = 42")
    const program = parser.parse()
    const result = infer(program)

    expect(result.success).toBe(true)
    expect(result.errors).toHaveLength(0)
    expect(getTypeOfVariable(result, "x")).toBeDefined()
    expect(typeToString(getTypeOfVariable(result, "x")!)).toBe("Int")
  })

  test("infer function declaration", () => {
    const parser = new Parser("fn double x: Int -> Int = x * 2")
    const program = parser.parse()
    const result = infer(program)

    expect(result.success).toBe(true)
    expect(result.errors).toHaveLength(0)
    const doubleType = getTypeOfVariable(result, "double")
    expect(doubleType).toBeDefined()
    expect(typeToString(doubleType!)).toBe("(Int -> Int)")
  })

  test("infer program with multiple declarations", () => {
    const parser = new Parser(`
      let x = 10
      let y = 20
      fn add a: Int -> b: Int -> Int = a + b
    `)
    const program = parser.parse()
    const result = infer(program)

    expect(result.success).toBe(true)
    expect(result.errors).toHaveLength(0)
    expect(typeToString(getTypeOfVariable(result, "x")!)).toBe("Int")
    expect(typeToString(getTypeOfVariable(result, "y")!)).toBe("Int")
    expect(typeToString(getTypeOfVariable(result, "add")!)).toBe("(Int -> (Int -> Int))")
  })

  test("infer struct declaration", () => {
    const parser = new Parser(`
      struct Point {
        x: Float,
        y: Float
      }
    `)
    const program = parser.parse()
    const result = infer(program)

    expect(result.success).toBe(true)
    expect(result.errors).toHaveLength(0)
    expect(getTypeOfVariable(result, "Point")).toBeDefined()
  })

  test("infer ADT declaration", () => {
    const parser = new Parser("type Direction = | North | South | East | West")
    const program = parser.parse()
    const result = infer(program)

    expect(result.success).toBe(true)
    expect(getTypeOfVariable(result, "North")).toBeDefined()
    expect(getTypeOfVariable(result, "South")).toBeDefined()
    expect(getTypeOfVariable(result, "East")).toBeDefined()
    expect(getTypeOfVariable(result, "West")).toBeDefined()
  })

  test("getTypeOfNode returns type for AST node", () => {
    const parser = new Parser("let x = 42")
    const program = parser.parse()
    const result = infer(program)

    // 変数宣言のイニシャライザのタイプを確認
    const varDecl = program.statements[0] as AST.VariableDeclaration
    const initType = getTypeOfNode(result, varDecl.initializer)
    expect(initType).toBeDefined()
    expect(typeToString(initType!)).toBe("Int")
  })
})
