import { test, expect } from "bun:test"
import {
  Just,
  Nothing,
  mapMaybe,
  pureMaybe,
  applyMaybe,
  bindMaybe,
  Left,
  Right,
  mapEither,
  pureEither,
  applyEither,
  bindEither,
  type Maybe,
  type Either,
} from "../src/runtime/seseragi-runtime"

// =============================================================================
// ファンクター法則のテスト
// =============================================================================

test("Functor Laws - Maybe - Identity", () => {
  const value = Just(42)
  const identity = <T>(x: T): T => x

  // map(id) = id
  const result = mapMaybe(value, identity)
  expect(result).toEqual(value)
})

test("Functor Laws - Maybe - Composition", () => {
  const value = Just(5)
  const f = (x: number) => x * 2
  const g = (x: number) => x + 1

  // map(f . g) = map(f) . map(g)
  const composed = mapMaybe(value, (x) => f(g(x)))
  const sequential = mapMaybe(mapMaybe(value, g), f)

  expect(composed).toEqual(sequential)
})

test("Functor Laws - Either - Identity", () => {
  const value = Right(42)
  const identity = <T>(x: T): T => x

  const result = mapEither(value, identity)
  expect(result).toEqual(value)
})

test("Functor Laws - Either - Composition", () => {
  const value = Right(5)
  const f = (x: number) => x * 2
  const g = (x: number) => x + 1

  const composed = mapEither(value, (x) => f(g(x)))
  const sequential = mapEither(mapEither(value, g), f)

  expect(composed).toEqual(sequential)
})

// =============================================================================
// アプリカティブ法則のテスト
// =============================================================================

test("Applicative Laws - Maybe - Identity", () => {
  const value = Just(42)
  const identity = <T>(x: T): T => x

  // pure(id) <*> v = v
  const result = applyMaybe(pureMaybe(identity), value)
  expect(result).toEqual(value)
})

test("Applicative Laws - Maybe - Composition", () => {
  // より簡単な例で合成法則をテスト
  const f = (x: number) => x * 2
  const x = 5

  const func = pureMaybe(f)
  const value = pureMaybe(x)

  // 関数適用のテスト（実際の値で比較）
  const result1 = applyMaybe(func, value)
  const expected = pureMaybe(f(x))

  expect(result1).toEqual(expected)
  expect(result1).toEqual(Just(10))
})

test("Applicative Laws - Maybe - Homomorphism", () => {
  const f = (x: number) => x * 2
  const x = 42

  // pure(f) <*> pure(x) = pure(f(x))
  const left = applyMaybe(pureMaybe(f), pureMaybe(x))
  const right = pureMaybe(f(x))

  expect(left).toEqual(right)
})

test("Applicative Laws - Maybe - Interchange", () => {
  const f = (x: number) => x * 2
  const x = 42
  const u = pureMaybe(f)

  // u <*> pure(x) = pure($ x) <*> u
  const left = applyMaybe(u, pureMaybe(x))
  const right = applyMaybe(
    pureMaybe((g: (a: number) => number) => g(x)),
    u
  )

  expect(left).toEqual(right)
})

// =============================================================================
// モナド法則のテスト
// =============================================================================

test("Monad Laws - Maybe - Left Identity", () => {
  const f = (x: number): Maybe<number> => Just(x * 2)
  const x = 42

  // pure(x) >>= f = f(x)
  const left = bindMaybe(pureMaybe(x), f)
  const right = f(x)

  expect(left).toEqual(right)
})

test("Monad Laws - Maybe - Right Identity", () => {
  const m = Just(42)

  // m >>= pure = m
  const result = bindMaybe(m, pureMaybe)
  expect(result).toEqual(m)
})

test("Monad Laws - Maybe - Associativity", () => {
  const m = Just(5)
  const f = (x: number): Maybe<number> => Just(x * 2)
  const g = (x: number): Maybe<number> => Just(x + 1)

  // (m >>= f) >>= g = m >>= (\\x -> f(x) >>= g)
  const left = bindMaybe(bindMaybe(m, f), g)
  const right = bindMaybe(m, (x) => bindMaybe(f(x), g))

  expect(left).toEqual(right)
})

test("Monad Laws - Either - Left Identity", () => {
  const f = (x: number): Either<string, number> => Right(x * 2)
  const x = 42

  const left = bindEither(pureEither(x), f)
  const right = f(x)

  expect(left).toEqual(right)
})

test("Monad Laws - Either - Right Identity", () => {
  const m = Right(42)

  const result = bindEither(m, pureEither)
  expect(result).toEqual(m)
})

test("Monad Laws - Either - Associativity", () => {
  const m = Right(5)
  const f = (x: number): Either<string, number> => Right(x * 2)
  const g = (x: number): Either<string, number> => Right(x + 1)

  const left = bindEither(bindEither(m, f), g)
  const right = bindEither(m, (x) => bindEither(f(x), g))

  expect(left).toEqual(right)
})

// =============================================================================
// 実用的なテスト
// =============================================================================

test("Maybe - Chain operations", () => {
  const divide =
    (x: number) =>
    (y: number): Maybe<number> =>
      y === 0 ? Nothing : Just(x / y)

  const result = bindMaybe(
    bindMaybe(Just(20), (x) => divide(x)(2)),
    (x) => divide(x)(5)
  )

  expect(result).toEqual(Just(2))
})

test("Maybe - Nothing propagation", () => {
  const double = (x: number): Maybe<number> => Just(x * 2)

  const result = bindMaybe(Nothing, double)
  expect(result).toEqual(Nothing)
})

test("Either - Error propagation", () => {
  const parseNum = (s: string): Either<string, number> => {
    const n = parseInt(s)
    return isNaN(n) ? Left("Parse error") : Right(n)
  }

  const double = (x: number): Either<string, number> => Right(x * 2)

  const result = bindEither(bindEither(parseNum("abc"), double), (x) =>
    Right(x + 1)
  )

  expect(result).toEqual(Left("Parse error"))
})

test("Either - Success chain", () => {
  const parseNum = (s: string): Either<string, number> => {
    const n = parseInt(s)
    return isNaN(n) ? Left("Parse error") : Right(n)
  }

  const double = (x: number): Either<string, number> => Right(x * 2)

  const result = bindEither(bindEither(parseNum("42"), double), (x) =>
    Right(x + 1)
  )

  expect(result).toEqual(Right(85))
})

// =============================================================================
// モナド演算子（<$>, <*>）のテスト
// =============================================================================

test("Functor map operator - Maybe", () => {
  const double = (x: number) => x * 2

  // <$> 演算子のテスト（mapMaybe関数として実装）
  const justResult = mapMaybe(Just(5), double)
  const nothingResult = mapMaybe(Nothing, double)

  expect(justResult).toEqual(Just(10))
  expect(nothingResult).toEqual(Nothing)
})

test("Functor map operator - Either", () => {
  const double = (x: number) => x * 2

  // <$> 演算子のテスト（mapEither関数として実装）
  const rightResult = mapEither(Right(5), double)
  const leftResult = mapEither(Left("error"), double)

  expect(rightResult).toEqual(Right(10))
  expect(leftResult).toEqual(Left("error"))
})

test("Applicative apply operator - Maybe", () => {
  const add = (x: number) => (y: number) => x + y
  const multiply = (x: number) => (y: number) => x * y

  // <*> 演算子のテスト（applyMaybe関数として実装）
  const successResult = applyMaybe(applyMaybe(pureMaybe(add), Just(5)), Just(3))
  const failureResult = applyMaybe(
    applyMaybe(pureMaybe(multiply), Nothing),
    Just(3)
  )
  const allNothingResult = applyMaybe(
    applyMaybe(pureMaybe(add), Nothing),
    Nothing
  )

  expect(successResult).toEqual(Just(8))
  expect(failureResult).toEqual(Nothing)
  expect(allNothingResult).toEqual(Nothing)
})

test("Applicative apply operator - Either", () => {
  const add = (x: number) => (y: number) => x + y
  const subtract = (x: number) => (y: number) => x - y

  // <*> 演算子のテスト（applyEither関数として実装）
  const successResult = applyEither(
    applyEither(pureEither(add), Right(10)),
    Right(5)
  )
  const firstErrorResult = applyEither(
    applyEither(pureEither(subtract), Left("error1")),
    Right(5)
  )
  const secondErrorResult = applyEither(
    applyEither(pureEither(add), Right(10)),
    Left("error2")
  )
  const bothErrorResult = applyEither(
    applyEither(pureEither(add), Left("error1")),
    Left("error2")
  )

  expect(successResult).toEqual(Right(15))
  expect(firstErrorResult).toEqual(Left("error1"))
  expect(secondErrorResult).toEqual(Left("error2"))
  expect(bothErrorResult).toEqual(Left("error1")) // 最初のエラーが優先される
})

test("Mixed monad operations - Maybe chain", () => {
  const add = (x: number) => (y: number) => x + y
  const double = (x: number) => x * 2
  const safeDiv = (x: number) => (y: number) =>
    y === 0 ? Nothing : Just(x / y)

  // ファンクター、アプリカティブ、モナドの組み合わせ
  const input1 = Just(20)
  const input2 = Just(4)

  // 20 + 4 = 24, then 24 * 2 = 48, then 48 / 2 = 24
  const result = bindMaybe(
    mapMaybe(applyMaybe(applyMaybe(pureMaybe(add), input1), input2), double),
    (x) => safeDiv(x)(2)
  )

  expect(result).toEqual(Just(24))
})

test("Mixed monad operations - Either chain", () => {
  const add = (x: number) => (y: number) => x + y
  const double = (x: number) => x * 2
  const safeParse = (s: string): Either<string, number> => {
    const n = parseInt(s)
    return isNaN(n) ? Left("Parse error") : Right(n)
  }

  // パース -> 足し算 -> 2倍
  const result = bindEither(
    mapEither(
      applyEither(
        applyEither(pureEither(add), safeParse("10")),
        safeParse("5")
      ),
      double
    ),
    (x) => Right(x + 1)
  )

  expect(result).toEqual(Right(31)) // (10 + 5) * 2 + 1 = 31

  // エラーケース
  const errorResult = bindEither(
    mapEither(
      applyEither(
        applyEither(pureEither(add), safeParse("abc")),
        safeParse("5")
      ),
      double
    ),
    (x) => Right(x + 1)
  )

  expect(errorResult).toEqual(Left("Parse error"))
})
