/**
 * 型推論システムのテストケース
 */

import { describe, it, expect, beforeEach } from "bun:test"
import * as AST from "../src/ast"
import {
  TypeInferenceSystem,
  TypeVariable,
  TypeConstraint,
  TypeSubstitution,
} from "../src/type-inference"

describe("TypeInferenceSystem", () => {
  let inference: TypeInferenceSystem

  beforeEach(() => {
    inference = new TypeInferenceSystem()
  })

  describe("基本的なリテラル推論", () => {
    it("整数リテラルはInt型と推論される", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(new AST.Literal(42, "integer", 1, 1), 1, 1),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })

    it("文字列リテラルはString型と推論される", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.Literal("hello", "string", 1, 1),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })

    it("真偽値リテラルはBool型と推論される", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.Literal(true, "boolean", 1, 1),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })
  })

  describe("変数宣言の型推論", () => {
    it("型注釈なしの変数宣言で型が推論される", () => {
      const program = new AST.Program([
        new AST.VariableDeclaration(
          "x",
          new AST.Literal(42, "integer", 1, 9),
          undefined, // 型注釈なし
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })

    it("型注釈ありの変数宣言で型が一致することを検証", () => {
      const program = new AST.Program([
        new AST.VariableDeclaration(
          "x",
          new AST.Literal(42, "integer", 1, 12),
          new AST.PrimitiveType("Int", 1, 7),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })

    it("型注釈と初期化式の型が異なる場合エラーになる", () => {
      const program = new AST.Program([
        new AST.VariableDeclaration(
          "x",
          new AST.Literal("hello", "string", 1, 15),
          new AST.PrimitiveType("Int", 1, 7),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors.length).toBeGreaterThan(0)
    })
  })

  describe("二項演算の型推論", () => {
    it("整数の加算はInt型と推論される", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.BinaryOperation(
            new AST.Literal(1, "integer", 1, 1),
            "+",
            new AST.Literal(2, "integer", 1, 5),
            1,
            3
          ),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })

    it("異なる型の比較でエラーになる", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.BinaryOperation(
            new AST.Literal(1, "integer", 1, 1),
            "==",
            new AST.Literal("hello", "string", 1, 6),
            1,
            3
          ),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors.length).toBeGreaterThan(0)
    })

    it("論理演算のオペランドは両方Bool型である必要がある", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.BinaryOperation(
            new AST.Literal(1, "integer", 1, 1),
            "&&",
            new AST.Literal(true, "boolean", 1, 6),
            1,
            3
          ),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors.length).toBeGreaterThan(0)
    })
  })

  describe("関数の型推論", () => {
    it("単純な関数の型が正しく推論される", () => {
      const program = new AST.Program([
        new AST.FunctionDeclaration(
          "increment",
          [new AST.Parameter("x", new AST.PrimitiveType("Int", 1, 12), 1, 10)],
          new AST.PrimitiveType("Int", 1, 18),
          new AST.BinaryOperation(
            new AST.Identifier("x", 1, 25),
            "+",
            new AST.Literal(1, "integer", 1, 29),
            1,
            27
          ),
          false,
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })

    it("戻り値型と関数本体の型が一致しない場合エラーになる", () => {
      const program = new AST.Program([
        new AST.FunctionDeclaration(
          "badFunction",
          [new AST.Parameter("x", new AST.PrimitiveType("Int", 1, 16), 1, 14)],
          new AST.PrimitiveType("String", 1, 22), // String型を宣言
          new AST.BinaryOperation(
            // しかし本体はInt型を返す
            new AST.Identifier("x", 1, 35),
            "+",
            new AST.Literal(1, "integer", 1, 39),
            1,
            37
          ),
          false,
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors.length).toBeGreaterThan(0)
    })
  })

  describe("関数呼び出しの型推論", () => {
    it("関数呼び出しの型が正しく推論される", () => {
      const program = new AST.Program([
        // 関数定義: increment(x: Int) -> Int = x + 1
        new AST.FunctionDeclaration(
          "increment",
          [new AST.Parameter("x", new AST.PrimitiveType("Int", 1, 12), 1, 10)],
          new AST.PrimitiveType("Int", 1, 18),
          new AST.BinaryOperation(
            new AST.Identifier("x", 1, 25),
            "+",
            new AST.Literal(1, "integer", 1, 29),
            1,
            27
          ),
          false,
          1,
          1
        ),
        // 関数呼び出し: increment(5)
        new AST.ExpressionStatement(
          new AST.FunctionCall(
            new AST.Identifier("increment", 2, 1),
            [new AST.Literal(5, "integer", 2, 11)],
            2,
            1
          ),
          2,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })

    it("引数の型が一致しない場合エラーになる", () => {
      const program = new AST.Program([
        // 関数定義: increment(x: Int) -> Int = x + 1
        new AST.FunctionDeclaration(
          "increment",
          [new AST.Parameter("x", new AST.PrimitiveType("Int", 1, 12), 1, 10)],
          new AST.PrimitiveType("Int", 1, 18),
          new AST.BinaryOperation(
            new AST.Identifier("x", 1, 25),
            "+",
            new AST.Literal(1, "integer", 1, 29),
            1,
            27
          ),
          false,
          1,
          1
        ),
        // 関数呼び出し: increment("hello") - 型エラー
        new AST.ExpressionStatement(
          new AST.FunctionCall(
            new AST.Identifier("increment", 2, 1),
            [new AST.Literal("hello", "string", 2, 11)],
            2,
            1
          ),
          2,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors.length).toBeGreaterThan(0)
    })
  })

  describe("条件式の型推論", () => {
    it("条件式の両分岐が同じ型の場合正常に推論される", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.ConditionalExpression(
            new AST.Literal(true, "boolean", 1, 1),
            new AST.Literal(1, "integer", 1, 11),
            new AST.Literal(2, "integer", 1, 18),
            1,
            1
          ),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })

    it("条件式の分岐の型が異なる場合エラーになる", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.ConditionalExpression(
            new AST.Literal(true, "boolean", 1, 1),
            new AST.Literal(1, "integer", 1, 11),
            new AST.Literal("hello", "string", 1, 18),
            1,
            1
          ),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors.length).toBeGreaterThan(0)
    })

    it("条件式の条件部がBool型でない場合エラーになる", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.ConditionalExpression(
            new AST.Literal(1, "integer", 1, 1), // Bool型ではない
            new AST.Literal(2, "integer", 1, 8),
            new AST.Literal(3, "integer", 1, 15),
            1,
            1
          ),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors.length).toBeGreaterThan(0)
    })
  })

  describe("パイプライン演算子の型推論", () => {
    it("パイプライン演算子が正しく型推論される", () => {
      const program = new AST.Program([
        // 関数定義: double(x: Int) -> Int = x * 2
        new AST.FunctionDeclaration(
          "double",
          [new AST.Parameter("x", new AST.PrimitiveType("Int", 1, 9), 1, 7)],
          new AST.PrimitiveType("Int", 1, 15),
          new AST.BinaryOperation(
            new AST.Identifier("x", 1, 22),
            "*",
            new AST.Literal(2, "integer", 1, 26),
            1,
            24
          ),
          false,
          1,
          1
        ),
        // パイプライン: 5 | double
        new AST.ExpressionStatement(
          new AST.Pipeline(
            new AST.Literal(5, "integer", 2, 1),
            new AST.Identifier("double", 2, 5),
            2,
            3
          ),
          2,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors).toHaveLength(0)
    })
  })

  describe("未定義変数のエラー", () => {
    it("未定義変数を参照するとエラーになる", () => {
      const program = new AST.Program([
        new AST.ExpressionStatement(
          new AST.Identifier("undefinedVar", 1, 1),
          1,
          1
        ),
      ])

      const result = inference.infer(program)
      expect(result.errors.length).toBeGreaterThan(0)
      expect(result.errors[0].message).toContain("Undefined variable")
    })
  })
})

describe("TypeSubstitution", () => {
  it("型置換が正しく動作する", () => {
    const substitution = new TypeSubstitution()
    const typeVar = new TypeVariable(0, 1, 1)
    const intType = new AST.PrimitiveType("Int", 1, 1)

    substitution.set(0, intType)

    const result = substitution.apply(typeVar)
    expect(result.kind).toBe("PrimitiveType")
    expect((result as AST.PrimitiveType).name).toBe("Int")
  })

  it("関数型への置換が正しく動作する", () => {
    const substitution = new TypeSubstitution()
    const typeVar = new TypeVariable(0, 1, 1)
    const intType = new AST.PrimitiveType("Int", 1, 1)
    const funcType = new AST.FunctionType(typeVar, intType, 1, 1)

    substitution.set(0, new AST.PrimitiveType("String", 1, 1))

    const result = substitution.apply(funcType) as AST.FunctionType
    expect(result.kind).toBe("FunctionType")
    expect((result.paramType as AST.PrimitiveType).name).toBe("String")
    expect((result.returnType as AST.PrimitiveType).name).toBe("Int")
  })

  it("置換の合成が正しく動作する", () => {
    const sub1 = new TypeSubstitution()
    const sub2 = new TypeSubstitution()

    sub1.set(0, new AST.PrimitiveType("Int", 1, 1))
    sub2.set(1, new TypeVariable(0, 1, 1))

    const composed = sub1.compose(sub2)

    const typeVar1 = new TypeVariable(1, 1, 1)
    const result = composed.apply(typeVar1)
    expect((result as AST.PrimitiveType).name).toBe("Int")
  })
})

describe("TypeConstraint", () => {
  it("制約が正しく文字列化される", () => {
    const constraint = new TypeConstraint(
      new AST.PrimitiveType("Int", 1, 1),
      new TypeVariable(0, 1, 1),
      1,
      1,
      "test constraint"
    )

    const str = constraint.toString()
    expect(str).toContain("Int")
    expect(str).toContain("t0")
    expect(str).toContain("~")
  })
})
