/**
 * 新codegenモジュールのユニットテスト
 */

import { describe, expect, it } from "bun:test"
import {
  ArrayLiteral,
  BinaryOperation,
  BuiltinFunctionCall,
  ConsExpression,
  ExpressionStatement,
  FoldMonoid,
  FunctionApplication,
  FunctionApplicationOperator,
  FunctionCall,
  Identifier,
  IdentifierPattern,
  ListSugar,
  Literal,
  LiteralPattern,
  MethodCall,
  NullishCoalescingExpression,
  Pipeline,
  RangeLiteral,
  ReversePipe,
  TupleExpression,
  UnaryOperation,
} from "../src/ast"
import {
  createContext,
  generateArrayLiteral,
  generateBinaryOperation,
  generateBuiltinFunctionCall,
  generateConsExpression,
  generateExpression,
  generateExpressionStatement,
  generateFoldMonoid,
  generateFunctionApplication,
  generateFunctionApplicationOperator,
  generateFunctionCall,
  generateIdentifier,
  generateListSugar,
  generateLiteral,
  generateMethodCall,
  generateNullishCoalescing,
  generatePatternBindings,
  generatePatternCondition,
  generatePipeline,
  generateRangeLiteral,
  generateReversePipe,
  generateTupleExpression,
  generateUnaryOperation,
} from "../src/codegen/index"

describe("Expression Generators", () => {
  const ctx = createContext()

  describe("generateLiteral", () => {
    it("整数リテラル", () => {
      const lit = new Literal(42, "integer", 0, 0)
      expect(generateLiteral(ctx, lit)).toBe("42")
    })

    it("浮動小数点リテラル", () => {
      const lit = new Literal(3.14, "float", 0, 0)
      expect(generateLiteral(ctx, lit)).toBe("3.14")
    })

    it("文字列リテラル", () => {
      const lit = new Literal("hello", "string", 0, 0)
      expect(generateLiteral(ctx, lit)).toBe('"hello"')
    })

    it("真偽値リテラル", () => {
      const trueLit = new Literal(true, "boolean", 0, 0)
      const falseLit = new Literal(false, "boolean", 0, 0)
      expect(generateLiteral(ctx, trueLit)).toBe("true")
      expect(generateLiteral(ctx, falseLit)).toBe("false")
    })

    it("Unitリテラル", () => {
      const lit = new Literal(null, "unit", 0, 0)
      expect(generateLiteral(ctx, lit)).toBe("Unit")
    })
  })

  describe("generateIdentifier", () => {
    it("通常の識別子", () => {
      const ident = new Identifier("foo", 0, 0)
      expect(generateIdentifier(ctx, ident)).toBe("foo")
    })

    it("プライム付き識別子", () => {
      const ident = new Identifier("x'", 0, 0)
      expect(generateIdentifier(ctx, ident)).toBe("x_prime")
    })

    it("予約語の識別子（現状は変換なし）", () => {
      // NOTE: 現在のsanitizeIdentifierは予約語チェックをしていない
      // 将来的に追加する場合はここを修正
      const ident = new Identifier("class", 0, 0)
      expect(generateIdentifier(ctx, ident)).toBe("class")
    })
  })

  describe("generateUnaryOperation", () => {
    it("否定演算子", () => {
      const expr = new UnaryOperation(
        "-",
        new Literal(5, "integer", 0, 0),
        0,
        0
      )
      expect(generateUnaryOperation(ctx, expr)).toBe("(-5)")
    })

    it("論理否定", () => {
      const expr = new UnaryOperation(
        "!",
        new Literal(true, "boolean", 0, 0),
        0,
        0
      )
      expect(generateUnaryOperation(ctx, expr)).toBe("(!true)")
    })

    it("Signal getValue (*)", () => {
      const expr = new UnaryOperation("*", new Identifier("sig", 0, 0), 0, 0)
      expect(generateUnaryOperation(ctx, expr)).toBe("(sig.getValue())")
    })
  })

  describe("generateArrayLiteral", () => {
    it("空配列", () => {
      const arr = new ArrayLiteral([], 0, 0)
      expect(generateArrayLiteral(ctx, arr)).toBe("[]")
    })

    it("要素あり配列", () => {
      const arr = new ArrayLiteral(
        [
          new Literal(1, "integer", 0, 0),
          new Literal(2, "integer", 0, 0),
          new Literal(3, "integer", 0, 0),
        ],
        0,
        0
      )
      expect(generateArrayLiteral(ctx, arr)).toBe("[1, 2, 3]")
    })
  })

  describe("generateTupleExpression", () => {
    it("空タプル", () => {
      const tuple = new TupleExpression([], 0, 0)
      expect(generateTupleExpression(ctx, tuple)).toBe(
        "{ tag: 'Tuple', elements: [] }"
      )
    })

    it("要素ありタプル", () => {
      const tuple = new TupleExpression(
        [new Literal(1, "integer", 0, 0), new Literal("a", "string", 0, 0)],
        0,
        0
      )
      expect(generateTupleExpression(ctx, tuple)).toBe(
        "{ tag: 'Tuple', elements: [1, \"a\"] }"
      )
    })
  })

  describe("generateListSugar", () => {
    it("空リスト", () => {
      const list = new ListSugar([], 0, 0)
      expect(generateListSugar(ctx, list)).toBe("Empty")
    })

    it("要素ありリスト", () => {
      const list = new ListSugar(
        [new Literal(1, "integer", 0, 0), new Literal(2, "integer", 0, 0)],
        0,
        0
      )
      expect(generateListSugar(ctx, list)).toBe("Cons(1, Cons(2, Empty))")
    })
  })

  describe("generateConsExpression", () => {
    it("Cons式", () => {
      const cons = new ConsExpression(
        new Literal(1, "integer", 0, 0),
        new Identifier("xs", 0, 0),
        0,
        0
      )
      expect(generateConsExpression(ctx, cons)).toBe("Cons(1, xs)")
    })
  })

  describe("generateRangeLiteral", () => {
    it("排他的範囲 (1..5)", () => {
      const range = new RangeLiteral(
        new Literal(1, "integer", 0, 0),
        new Literal(5, "integer", 0, 0),
        false,
        0,
        0
      )
      expect(generateRangeLiteral(ctx, range)).toBe(
        "Array.from({length: 5 - 1}, (_, i) => i + 1)"
      )
    })

    it("包括的範囲 (1..=5)", () => {
      const range = new RangeLiteral(
        new Literal(1, "integer", 0, 0),
        new Literal(5, "integer", 0, 0),
        true,
        0,
        0
      )
      expect(generateRangeLiteral(ctx, range)).toBe(
        "Array.from({length: 5 - 1 + 1}, (_, i) => i + 1)"
      )
    })
  })
})

describe("Pattern Generators", () => {
  const ctx = createContext()

  describe("generatePatternCondition", () => {
    it("リテラルパターン (数値)", () => {
      const pattern = new LiteralPattern(42, 0, 0)
      expect(generatePatternCondition(ctx, pattern, "x")).toBe("x === 42")
    })

    it("リテラルパターン (文字列)", () => {
      const pattern = new LiteralPattern("hello", 0, 0)
      expect(generatePatternCondition(ctx, pattern, "x")).toBe('x === "hello"')
    })

    it("識別子パターン", () => {
      const pattern = new IdentifierPattern("n", 0, 0)
      expect(generatePatternCondition(ctx, pattern, "x")).toBe("true")
    })

    it("ワイルドカードパターン (_)", () => {
      const pattern = new IdentifierPattern("_", 0, 0)
      expect(generatePatternCondition(ctx, pattern, "x")).toBe("true")
    })

    it("nullパターン", () => {
      expect(generatePatternCondition(ctx, null, "x")).toBe("true")
    })
  })

  describe("generatePatternBindings", () => {
    it("リテラルパターン (バインディングなし)", () => {
      const pattern = new LiteralPattern(42, 0, 0)
      expect(generatePatternBindings(ctx, pattern, "x")).toBe("")
    })

    it("識別子パターン", () => {
      const pattern = new IdentifierPattern("n", 0, 0)
      expect(generatePatternBindings(ctx, pattern, "x")).toBe("const n = x;\n")
    })

    it("ワイルドカードパターン (_)", () => {
      const pattern = new IdentifierPattern("_", 0, 0)
      expect(generatePatternBindings(ctx, pattern, "x")).toBe("")
    })

    it("nullパターン", () => {
      expect(generatePatternBindings(ctx, null, "x")).toBe("")
    })
  })
})

describe("Statement Generators", () => {
  const ctx = createContext()

  describe("generateExpressionStatement", () => {
    it("式文", () => {
      const stmt = new ExpressionStatement(
        new Literal(42, "integer", 0, 0),
        0,
        0
      )
      expect(generateExpressionStatement(ctx, stmt)).toBe("42;")
    })
  })
})

describe("Expression Dispatcher", () => {
  const ctx = createContext()

  describe("generateExpression", () => {
    it("Literalをディスパッチ", () => {
      const expr = new Literal(123, "integer", 0, 0)
      expect(generateExpression(ctx, expr)).toBe("123")
    })

    it("Identifierをディスパッチ", () => {
      const expr = new Identifier("myVar", 0, 0)
      expect(generateExpression(ctx, expr)).toBe("myVar")
    })

    it("ArrayLiteralをディスパッチ", () => {
      const expr = new ArrayLiteral([new Literal(1, "integer", 0, 0)], 0, 0)
      expect(generateExpression(ctx, expr)).toBe("[1]")
    })

    it("TupleExpressionをディスパッチ", () => {
      const expr = new TupleExpression([new Literal(1, "integer", 0, 0)], 0, 0)
      expect(generateExpression(ctx, expr)).toBe(
        "{ tag: 'Tuple', elements: [1] }"
      )
    })

    it("ListSugarをディスパッチ", () => {
      const expr = new ListSugar([new Literal(1, "integer", 0, 0)], 0, 0)
      expect(generateExpression(ctx, expr)).toBe("Cons(1, Empty)")
    })

    it("BinaryOperationをディスパッチ", () => {
      const expr = new BinaryOperation(
        new Literal(1, "integer", 0, 0),
        "+",
        new Literal(2, "integer", 0, 0),
        0,
        0
      )
      // プリミティブ型情報がないため演算子ディスパッチに回る
      expect(generateExpression(ctx, expr)).toBe(
        '__dispatchOperator(1, "+", 2)'
      )
    })
  })
})

describe("Binary Operation Generators", () => {
  const ctx = createContext()

  describe("generateBinaryOperation", () => {
    it("CONS演算子 (:)", () => {
      const expr = new BinaryOperation(
        new Literal(1, "integer", 0, 0),
        ":",
        new Identifier("xs", 0, 0),
        0,
        0
      )
      expect(generateBinaryOperation(ctx, expr)).toBe("Cons(1, xs)")
    })

    it("Signal代入演算子 (:=)", () => {
      const expr = new BinaryOperation(
        new Identifier("sig", 0, 0),
        ":=",
        new Literal(42, "integer", 0, 0),
        0,
        0
      )
      expect(generateBinaryOperation(ctx, expr)).toBe("sig.setValue(42)")
    })

    it("算術演算子（型情報なし）", () => {
      const expr = new BinaryOperation(
        new Literal(10, "integer", 0, 0),
        "+",
        new Literal(20, "integer", 0, 0),
        0,
        0
      )
      // 型推論結果がないため演算子ディスパッチ
      expect(generateBinaryOperation(ctx, expr)).toBe(
        '__dispatchOperator(10, "+", 20)'
      )
    })

    it("比較演算子（型情報なし）", () => {
      const expr = new BinaryOperation(
        new Identifier("x", 0, 0),
        "==",
        new Literal(5, "integer", 0, 0),
        0,
        0
      )
      expect(generateBinaryOperation(ctx, expr)).toBe(
        '__dispatchOperator(x, "==", 5)'
      )
    })
  })
})

describe("Nullish Coalescing Generators", () => {
  const ctx = createContext()

  describe("generateNullishCoalescing", () => {
    it("通常のnull合体演算子", () => {
      const expr = new NullishCoalescingExpression(
        new Identifier("maybeValue", 0, 0),
        new Literal(0, "integer", 0, 0),
        0,
        0
      )
      // 型情報がないためTypeScriptのnull合体演算子を使用
      expect(generateNullishCoalescing(ctx, expr)).toBe("(maybeValue ?? 0)")
    })
  })
})

describe("Function Call Generators", () => {
  const ctx = createContext()

  describe("generateFunctionCall", () => {
    it("通常の関数呼び出し", () => {
      const call = new FunctionCall(
        new Identifier("foo", 0, 0),
        [new Literal(1, "integer", 0, 0), new Literal(2, "integer", 0, 0)],
        0,
        0
      )
      expect(generateFunctionCall(ctx, call)).toBe("foo(1, 2)")
    })

    it("ビルトイン関数呼び出し (print)", () => {
      const call = new FunctionCall(
        new Identifier("print", 0, 0),
        [new Literal("hello", "string", 0, 0)],
        0,
        0
      )
      expect(generateFunctionCall(ctx, call)).toBe('ssrgPrint("hello")')
    })

    it("引数なし関数呼び出し", () => {
      const call = new FunctionCall(new Identifier("getTime", 0, 0), [], 0, 0)
      expect(generateFunctionCall(ctx, call)).toBe("getTime()")
    })
  })

  describe("generateMethodCall", () => {
    it("メソッド呼び出し", () => {
      const call = new MethodCall(
        new Identifier("obj", 0, 0),
        "method",
        [new Literal(42, "integer", 0, 0)],
        0,
        0
      )
      expect(generateMethodCall(ctx, call)).toBe(
        '__dispatchMethod(obj, "method", 42)'
      )
    })

    it("引数なしメソッド呼び出し", () => {
      const call = new MethodCall(
        new Identifier("obj", 0, 0),
        "getValue",
        [],
        0,
        0
      )
      expect(generateMethodCall(ctx, call)).toBe(
        '__dispatchMethod(obj, "getValue")'
      )
    })
  })

  describe("generateFunctionApplication", () => {
    it("通常の関数適用", () => {
      const app = new FunctionApplication(
        new Identifier("double", 0, 0),
        new Literal(5, "integer", 0, 0),
        0,
        0
      )
      expect(generateFunctionApplication(ctx, app)).toBe("double(5)")
    })

    it("ビルトイン関数適用 (print)", () => {
      const app = new FunctionApplication(
        new Identifier("print", 0, 0),
        new Literal("test", "string", 0, 0),
        0,
        0
      )
      expect(generateFunctionApplication(ctx, app)).toBe('ssrgPrint("test")')
    })

    it("run関数適用", () => {
      const app = new FunctionApplication(
        new Identifier("run", 0, 0),
        new Identifier("task", 0, 0),
        0,
        0
      )
      expect(generateFunctionApplication(ctx, app)).toBe("ssrgRun(task)")
    })
  })

  describe("generateBuiltinFunctionCall", () => {
    it("toString呼び出し", () => {
      const call = new BuiltinFunctionCall(
        "toString",
        [new Literal(42, "integer", 0, 0)],
        0,
        0
      )
      expect(generateBuiltinFunctionCall(ctx, call)).toBe("ssrgToString(42)")
    })

    it("head呼び出し", () => {
      const call = new BuiltinFunctionCall(
        "head",
        [new Identifier("list", 0, 0)],
        0,
        0
      )
      expect(generateBuiltinFunctionCall(ctx, call)).toBe("headList(list)")
    })

    it("subscribe呼び出し", () => {
      const call = new BuiltinFunctionCall(
        "subscribe",
        [new Identifier("signal", 0, 0), new Identifier("callback", 0, 0)],
        0,
        0
      )
      expect(generateBuiltinFunctionCall(ctx, call)).toBe(
        "ssrgSignalSubscribe(signal, callback)"
      )
    })
  })
})

describe("Pipeline Generators", () => {
  const ctx = createContext()

  describe("generatePipeline", () => {
    it("パイプライン演算子", () => {
      const pipe = new Pipeline(
        new Literal(5, "integer", 0, 0),
        new Identifier("double", 0, 0),
        0,
        0
      )
      expect(generatePipeline(ctx, pipe)).toBe("pipe(5, double)")
    })
  })

  describe("generateReversePipe", () => {
    it("逆パイプ演算子", () => {
      const pipe = new ReversePipe(
        new Identifier("double", 0, 0),
        new Literal(5, "integer", 0, 0),
        0,
        0
      )
      expect(generateReversePipe(ctx, pipe)).toBe("reversePipe(double, 5)")
    })

    it("ビルトイン関数への逆パイプ", () => {
      const pipe = new ReversePipe(
        new Identifier("print", 0, 0),
        new Literal("hello", "string", 0, 0),
        0,
        0
      )
      expect(generateReversePipe(ctx, pipe)).toBe('ssrgPrint("hello")')
    })
  })

  describe("generateFoldMonoid", () => {
    it("畳み込みモノイド", () => {
      const fold = new FoldMonoid(
        new Identifier("xs", 0, 0),
        new Identifier("f", 0, 0),
        0,
        0
      )
      expect(generateFoldMonoid(ctx, fold)).toBe(
        "foldMonoid(xs, /* empty */, f)"
      )
    })
  })

  describe("generateFunctionApplicationOperator", () => {
    it("関数適用演算子 ($)", () => {
      const app = new FunctionApplicationOperator(
        new Identifier("f", 0, 0),
        new Literal(42, "integer", 0, 0),
        0,
        0
      )
      expect(generateFunctionApplicationOperator(ctx, app)).toBe("f(42)")
    })

    it("ビルトイン関数への適用", () => {
      const app = new FunctionApplicationOperator(
        new Identifier("print", 0, 0),
        new Literal("test", "string", 0, 0),
        0,
        0
      )
      expect(generateFunctionApplicationOperator(ctx, app)).toBe(
        'ssrgPrint("test")'
      )
    })
  })
})
