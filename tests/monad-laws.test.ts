import { describe, expect, test } from "bun:test"
import {
  applyArray,
  applyEither,
  applyList,
  applyMaybe,
  applySignal,
  applyTask,
  bindArray,
  bindEither,
  bindList,
  bindMaybe,
  bindSignal,
  bindTask,
  Cons,
  // Signal型
  createSignal,
  type Either,
  // List型
  Empty,
  // Maybe型
  Just,
  // Either型
  Left,
  type List,
  type Maybe,
  // Array型
  mapArray,
  mapEither,
  mapList,
  mapMaybe,
  mapSignal,
  mapTask,
  Nothing,
  Right,
  type Signal,
  // Task型
  Task,
} from "@seseragi/runtime"

// =============================================================================
// ファンクター法則のテスト (Functor Laws)
// =============================================================================

describe("ファンクター法則", () => {
  describe("Maybe - Functor Laws", () => {
    test("恒等法則: fmap(id) = id", () => {
      const identity = <T>(x: T): T => x
      const value = Just(42)

      const result = mapMaybe(value, identity)
      expect(result).toEqual(value)

      // Nothing でも成り立つ
      const nothingResult = mapMaybe(Nothing, identity)
      expect(nothingResult).toEqual(Nothing)
    })

    test("合成法則: fmap(f ∘ g) = fmap(f) ∘ fmap(g)", () => {
      const f = (x: number) => x * 2
      const g = (x: number) => x + 1
      const value = Just(5)

      // f ∘ g を直接適用
      const composed = mapMaybe(value, (x) => f(g(x)))

      // fmap(g) してから fmap(f) を適用
      const sequential = mapMaybe(mapMaybe(value, g), f)

      expect(composed).toEqual(sequential)
      expect(composed).toEqual(Just(12)) // (5 + 1) * 2 = 12
    })
  })

  describe("Either - Functor Laws", () => {
    test("恒等法則: fmap(id) = id", () => {
      const identity = <T>(x: T): T => x
      const rightValue = Right(42)
      const leftValue = Left("error")

      expect(mapEither(rightValue, identity)).toEqual(rightValue)
      expect(mapEither(leftValue, identity)).toEqual(leftValue)
    })

    test("合成法則: fmap(f ∘ g) = fmap(f) ∘ fmap(g)", () => {
      const f = (x: number) => x * 2
      const g = (x: number) => x + 1
      const value = Right(5)

      const composed = mapEither(value, (x) => f(g(x)))
      const sequential = mapEither(mapEither(value, g), f)

      expect(composed).toEqual(sequential)
      expect(composed).toEqual(Right(12))
    })
  })

  describe("List - Functor Laws", () => {
    test("恒等法則: fmap(id) = id", () => {
      const identity = <T>(x: T): T => x
      const list = Cons(1, Cons(2, Cons(3, Empty)))

      const result = mapList(list, identity)
      expect(result).toEqual(list)

      // 空リストでも成り立つ
      expect(mapList(Empty, identity)).toEqual(Empty)
    })

    test("合成法則: fmap(f ∘ g) = fmap(f) ∘ fmap(g)", () => {
      const f = (x: number) => x * 2
      const g = (x: number) => x + 1
      const list = Cons(1, Cons(2, Empty))

      const composed = mapList(list, (x) => f(g(x)))
      const sequential = mapList(mapList(list, g), f)

      expect(composed).toEqual(sequential)
      expect(composed).toEqual(Cons(4, Cons(6, Empty))) // [(1+1)*2, (2+1)*2] = [4, 6]
    })
  })
})

// =============================================================================
// アプリカティブ法則のテスト (Applicative Laws)
// =============================================================================

describe("アプリカティブ法則", () => {
  describe("Maybe - Applicative Laws", () => {
    test("恒等法則: pure(id) <*> v = v", () => {
      const identity = (x: number): number => x
      const value = Just(42)

      const result = applyMaybe(Just(identity), value)
      expect(result).toEqual(value)
    })

    test("合成法則", () => {
      const f = (x: number) => x * 2
      const g = (x: number) => x + 1
      const value = Just(5)

      // 実際の値で検証
      const result = applyMaybe(Just(f), mapMaybe(value, g))
      expect(result).toEqual(Just(12)) // (5 + 1) * 2 = 12
    })

    test("同型法則: pure(f) <*> pure(x) = pure(f(x))", () => {
      const f = (x: number) => x * 2
      const x = 42

      const left = applyMaybe(Just(f), Just(x))
      const right = Just(f(x))

      expect(left).toEqual(right)
      expect(left).toEqual(Just(84))
    })
  })

  describe("Either - Applicative Laws", () => {
    test("恒等法則: pure(id) <*> v = v", () => {
      const identity = (x: number): number => x
      const value = Right(42)

      const result = applyEither(Right(identity), value)
      expect(result).toEqual(value)
    })

    test("同型法則: pure(f) <*> pure(x) = pure(f(x))", () => {
      const f = (x: number) => x * 2
      const x = 42

      const left = applyEither(Right(f), Right(x))
      const right = Right(f(x))

      expect(left).toEqual(right)
      expect(left).toEqual(Right(84))
    })
  })

  describe("List - Applicative Laws", () => {
    test("恒等法則: pure(id) <*> v = v", () => {
      const identity = (x: number): number => x
      const value = Cons(1, Cons(2, Empty))

      const result = applyList(Cons(identity, Empty), value)
      expect(result).toEqual(value)
    })

    test("同型法則: pure(f) <*> pure(x) = pure(f(x))", () => {
      const f = (x: number) => x * 2
      const x = 42

      const left = applyList(Cons(f, Empty), Cons(x, Empty))
      const right = Cons(f(x), Empty)

      expect(left).toEqual(right)
      expect(left).toEqual(Cons(84, Empty))
    })

    test("実際の使用例: 複数の関数と複数の値", () => {
      const add1 = (x: number) => x + 1
      const mul2 = (x: number) => x * 2
      const functions = Cons(add1, Cons(mul2, Empty))
      const values = Cons(3, Cons(4, Empty))

      const result = applyList(functions, values)
      // [add1, mul2] <*> [3, 4] = [add1(3), add1(4), mul2(3), mul2(4)] = [4, 5, 6, 8]
      expect(result).toEqual(Cons(4, Cons(5, Cons(6, Cons(8, Empty)))))
    })
  })
})

// =============================================================================
// モナド法則のテスト (Monad Laws)
// =============================================================================

describe("モナド法則", () => {
  describe("Maybe - Monad Laws", () => {
    test("左恒等法則: pure(a) >>= f = f(a)", () => {
      const f = (x: number): Maybe<number> => Just(x * 2)
      const a = 42

      const left = bindMaybe(Just(a), f)
      const right = f(a)

      expect(left).toEqual(right)
      expect(left).toEqual(Just(84))
    })

    test("右恒等法則: m >>= pure = m", () => {
      const m = Just(42)

      const result = bindMaybe(m, (x) => Just(x))
      expect(result).toEqual(m)

      // Nothing でも成り立つ
      const nothingResult = bindMaybe(Nothing, (x) => Just(x))
      expect(nothingResult).toEqual(Nothing)
    })

    test("結合法則: (m >>= f) >>= g = m >>= (x => f(x) >>= g)", () => {
      const m = Just(5)
      const f = (x: number): Maybe<number> => Just(x * 2)
      const g = (x: number): Maybe<number> => Just(x + 1)

      // (m >>= f) >>= g
      const left = bindMaybe(bindMaybe(m, f), g)

      // m >>= (x => f(x) >>= g)
      const right = bindMaybe(m, (x) => bindMaybe(f(x), g))

      expect(left).toEqual(right)
      expect(left).toEqual(Just(11)) // ((5 * 2) + 1) = 11
    })

    test("Nothing の伝播", () => {
      const f = (x: number): Maybe<number> => Just(x * 2)

      const result = bindMaybe(Nothing, f)
      expect(result).toEqual(Nothing)
    })
  })

  describe("Either - Monad Laws", () => {
    test("左恒等法則: pure(a) >>= f = f(a)", () => {
      const f = (x: number): Either<string, number> => Right(x * 2)
      const a = 42

      const left = bindEither(Right(a), f)
      const right = f(a)

      expect(left).toEqual(right)
      expect(left).toEqual(Right(84))
    })

    test("右恒等法則: m >>= pure = m", () => {
      const m = Right(42)

      const result = bindEither(m, (x) => Right(x))
      expect(result).toEqual(m)

      // Left でも成り立つ
      const leftResult = bindEither(Left("error"), (x) => Right(x))
      expect(leftResult).toEqual(Left("error"))
    })

    test("結合法則: (m >>= f) >>= g = m >>= (x => f(x) >>= g)", () => {
      const m = Right(5)
      const f = (x: number): Either<string, number> => Right(x * 2)
      const g = (x: number): Either<string, number> => Right(x + 1)

      const left = bindEither(bindEither(m, f), g)
      const right = bindEither(m, (x) => bindEither(f(x), g))

      expect(left).toEqual(right)
      expect(left).toEqual(Right(11))
    })

    test("Left の伝播", () => {
      const f = (x: number): Either<string, number> => Right(x * 2)

      const result = bindEither(Left("error"), f)
      expect(result).toEqual(Left("error"))
    })
  })

  describe("List - Monad Laws", () => {
    test("左恒等法則: pure(a) >>= f = f(a)", () => {
      const f = (x: number): List<number> => Cons(x, Cons(x * 2, Empty))
      const a = 42

      const left = bindList(Cons(a, Empty), f)
      const right = f(a)

      expect(left).toEqual(right)
      expect(left).toEqual(Cons(42, Cons(84, Empty)))
    })

    test("右恒等法則: m >>= pure = m", () => {
      const m = Cons(1, Cons(2, Cons(3, Empty)))

      const result = bindList(m, (x) => Cons(x, Empty))
      expect(result).toEqual(m)

      // 空リストでも成り立つ
      const emptyResult = bindList(Empty, (x) => Cons(x, Empty))
      expect(emptyResult).toEqual(Empty)
    })

    test("結合法則: (m >>= f) >>= g = m >>= (x => f(x) >>= g)", () => {
      const m = Cons(1, Cons(2, Empty))
      const f = (x: number): List<number> => Cons(x, Cons(x + 10, Empty))
      const g = (x: number): List<number> => Cons(x * 2, Empty)

      const left = bindList(bindList(m, f), g)
      const right = bindList(m, (x) => bindList(f(x), g))

      expect(left).toEqual(right)
      // m = [1, 2]
      // f(1) = [1, 11], f(2) = [2, 12]
      // g を適用: [2, 22, 4, 24]
      expect(left).toEqual(Cons(2, Cons(22, Cons(4, Cons(24, Empty)))))
    })

    test("実用例: リストの展開", () => {
      const list = Cons(1, Cons(2, Empty))
      const result = bindList(list, (x) => Cons(x, Cons(x * 2, Empty)))

      // [1, 2] >>= (\x -> [x, x*2]) = [1, 2, 2, 4]
      expect(result).toEqual(Cons(1, Cons(2, Cons(2, Cons(4, Empty)))))
    })
  })
})

// =============================================================================
// Task型のファンクター・アプリカティブ・モナド法則テスト
// =============================================================================

describe("Task - モナド法則", () => {
  describe("Task - Functor Laws", () => {
    test("恒等法則: fmap(id) = id", async () => {
      const identity = <T>(x: T): T => x
      const task = Task(() => Promise.resolve(42))

      const result = mapTask(identity, task)

      expect(await result.computation()).toBe(42)
    })

    test("合成法則: fmap(f ∘ g) = fmap(f) ∘ fmap(g)", async () => {
      const f = (x: number) => x * 2
      const g = (x: number) => x + 1
      const task = Task(() => Promise.resolve(5))

      // f ∘ g を直接適用
      const composed = mapTask((x) => f(g(x)), task)

      // fmap(g) してから fmap(f) を適用
      const sequential = mapTask(f, mapTask(g, task))

      const composedResult = await composed.computation()
      const sequentialResult = await sequential.computation()

      expect(composedResult).toBe(sequentialResult)
      expect(composedResult).toBe(12) // (5 + 1) * 2 = 12
    })
  })

  describe("Task - Applicative Laws", () => {
    test("恒等法則: pure(id) <*> v = v", async () => {
      const identity = <T>(x: T): T => x
      const taskFunc = Task(() => Promise.resolve(identity))
      const taskValue = Task(() => Promise.resolve(42))

      const result = applyTask(taskFunc, taskValue)

      expect(await result.computation()).toBe(42)
    })

    test("同型法則: pure(f) <*> pure(x) = pure(f(x))", async () => {
      const f = (x: number) => x * 2
      const x = 42

      const left = applyTask(
        Task(() => Promise.resolve(f)),
        Task(() => Promise.resolve(x))
      )
      const right = Task(() => Promise.resolve(f(x)))

      const leftResult = await left.computation()
      const rightResult = await right.computation()

      expect(leftResult).toBe(rightResult)
      expect(leftResult).toBe(84)
    })
  })

  describe("Task - Monad Laws", () => {
    test("左恒等法則: pure(a) >>= f = f(a)", async () => {
      const f = (x: number): Task<number> => Task(() => Promise.resolve(x * 2))
      const a = 42

      const left = bindTask(
        Task(() => Promise.resolve(a)),
        f
      )
      const right = f(a)

      const leftResult = await left.computation()
      const rightResult = await right.computation()

      expect(leftResult).toBe(rightResult)
      expect(leftResult).toBe(84)
    })

    test("右恒等法則: m >>= pure = m", async () => {
      const m = Task(() => Promise.resolve(42))
      const pure = <T>(x: T): Task<T> => Task(() => Promise.resolve(x))

      const result = bindTask(m, pure)

      expect(await result.computation()).toBe(42)
    })

    test("結合法則: (m >>= f) >>= g = m >>= (x => f(x) >>= g)", async () => {
      const m = Task(() => Promise.resolve(5))
      const f = (x: number): Task<number> => Task(() => Promise.resolve(x * 2))
      const g = (x: number): Task<number> => Task(() => Promise.resolve(x + 1))

      // (m >>= f) >>= g
      const left = bindTask(bindTask(m, f), g)

      // m >>= (x => f(x) >>= g)
      const right = bindTask(m, (x) => bindTask(f(x), g))

      const leftResult = await left.computation()
      const rightResult = await right.computation()

      expect(leftResult).toBe(rightResult)
      expect(leftResult).toBe(11) // ((5 * 2) + 1) = 11
    })
  })
})

// =============================================================================
// Array型のファンクター・アプリカティブ・モナド法則テスト
// =============================================================================

describe("Array - モナド法則", () => {
  describe("Array - Functor Laws", () => {
    test("恒等法則: fmap(id) = id", () => {
      const identity = <T>(x: T): T => x
      const array = [1, 2, 3]

      const result = mapArray(array, identity)
      expect(result).toEqual(array)

      // 空配列でも成り立つ
      expect(mapArray([], identity)).toEqual([])
    })

    test("合成法則: fmap(f ∘ g) = fmap(f) ∘ fmap(g)", () => {
      const f = (x: number) => x * 2
      const g = (x: number) => x + 1
      const array = [1, 2]

      // f ∘ g を直接適用
      const composed = mapArray(array, (x) => f(g(x)))

      // fmap(g) してから fmap(f) を適用
      const sequential = mapArray(mapArray(array, g), f)

      expect(composed).toEqual(sequential)
      expect(composed).toEqual([4, 6]) // [(1+1)*2, (2+1)*2] = [4, 6]
    })
  })

  describe("Array - Applicative Laws", () => {
    test("恒等法則: pure(id) <*> v = v", () => {
      const identity = <T>(x: T): T => x
      const array = [1, 2]

      const result = applyArray([identity], array)
      expect(result).toEqual(array)
    })

    test("同型法則: pure(f) <*> pure(x) = pure(f(x))", () => {
      const f = (x: number) => x * 2
      const x = 42

      const left = applyArray([f], [x])
      const right = [f(x)]

      expect(left).toEqual(right)
      expect(left).toEqual([84])
    })

    test("実際の使用例: 複数の関数と複数の値", () => {
      const add1 = (x: number) => x + 1
      const mul2 = (x: number) => x * 2
      const functions = [add1, mul2]
      const values = [3, 4]

      const result = applyArray(functions, values)
      // [add1, mul2] <*> [3, 4] = [add1(3), add1(4), mul2(3), mul2(4)] = [4, 5, 6, 8]
      expect(result).toEqual([4, 5, 6, 8])
    })
  })

  describe("Array - Monad Laws", () => {
    test("左恒等法則: pure(a) >>= f = f(a)", () => {
      const f = (x: number): number[] => [x, x * 2]
      const a = 42

      const left = bindArray([a], f)
      const right = f(a)

      expect(left).toEqual(right)
      expect(left).toEqual([42, 84])
    })

    test("右恒等法則: m >>= pure = m", () => {
      const m = [1, 2, 3]
      const pure = <T>(x: T): T[] => [x]

      const result = bindArray(m, pure)
      expect(result).toEqual(m)

      // 空配列でも成り立つ
      const emptyResult = bindArray([], pure)
      expect(emptyResult).toEqual([])
    })

    test("結合法則: (m >>= f) >>= g = m >>= (x => f(x) >>= g)", () => {
      const m = [1, 2]
      const f = (x: number): number[] => [x, x + 10]
      const g = (x: number): number[] => [x * 2]

      // (m >>= f) >>= g
      const left = bindArray(bindArray(m, f), g)

      // m >>= (x => f(x) >>= g)
      const right = bindArray(m, (x) => bindArray(f(x), g))

      expect(left).toEqual(right)
      // m = [1, 2]
      // f(1) = [1, 11], f(2) = [2, 12] → [1, 11, 2, 12]
      // g を適用: [2, 22, 4, 24]
      expect(left).toEqual([2, 22, 4, 24])
    })

    test("実用例: 配列の展開", () => {
      const array = [1, 2]
      const result = bindArray(array, (x) => [x, x * 2])

      // [1, 2] >>= (\x -> [x, x*2]) = [1, 2, 2, 4]
      expect(result).toEqual([1, 2, 2, 4])
    })
  })
})

// =============================================================================
// Signal型のファンクター・アプリカティブ・モナド法則テスト
// =============================================================================

describe("Signal - モナド法則", () => {
  describe("Signal - Functor Laws", () => {
    test("恒等法則: fmap(id) = id", () => {
      const identity = <T>(x: T): T => x
      const signal = createSignal(42)

      const result = mapSignal(signal, identity)
      expect(result.getValue()).toBe(42)

      // 値を変更して確認
      signal.setValue(100)
      expect(result.getValue()).toBe(100)

      // クリーンアップ
      result.detach()
      signal.detach()
    })

    test("合成法則: fmap(f ∘ g) = fmap(f) ∘ fmap(g)", () => {
      const f = (x: number) => x * 2
      const g = (x: number) => x + 1
      const signal = createSignal(5)

      // f ∘ g を直接適用
      const composed = mapSignal(signal, (x) => f(g(x)))

      // fmap(g) してから fmap(f) を適用
      const sequential = mapSignal(mapSignal(signal, g), f)

      expect(composed.getValue()).toBe(sequential.getValue())
      expect(composed.getValue()).toBe(12) // (5 + 1) * 2 = 12

      // 値を変更して確認
      signal.setValue(10)
      expect(composed.getValue()).toBe(sequential.getValue())
      expect(composed.getValue()).toBe(22) // (10 + 1) * 2 = 22

      // クリーンアップ
      composed.detach()
      sequential.detach()
      signal.detach()
    })
  })

  describe("Signal - Applicative Laws", () => {
    test("恒等法則: pure(id) <*> v = v", () => {
      const identity = (x: number): number => x
      const signalFunc = createSignal(identity)
      const signalValue = createSignal(42)

      const result = applySignal(signalFunc, signalValue)
      expect(result.getValue()).toBe(42)

      // 値を変更して確認
      signalValue.setValue(100)
      expect(result.getValue()).toBe(100)

      // クリーンアップ
      result.detach()
      signalFunc.detach()
      signalValue.detach()
    })

    test("同型法則: pure(f) <*> pure(x) = pure(f(x))", () => {
      const f = (x: number) => x * 2
      const x = 42

      const left = applySignal(createSignal(f), createSignal(x))
      const right = createSignal(f(x))

      expect(left.getValue()).toBe(right.getValue())
      expect(left.getValue()).toBe(84)

      // クリーンアップ
      left.detach()
      right.detach()
    })
  })

  describe("Signal - Monad Laws", () => {
    test("左恒等法則: pure(a) >>= f = f(a)", () => {
      const f = (x: number): Signal<number> => createSignal(x * 2)
      const a = 42

      const left = bindSignal(createSignal(a), f)
      const right = f(a)

      expect(left.getValue()).toBe(right.getValue())
      expect(left.getValue()).toBe(84)

      // クリーンアップ
      left.detach()
      right.detach()
    })

    test("右恒等法則: m >>= pure = m", () => {
      const m = createSignal(42)
      const pure = <T>(x: T): Signal<T> => createSignal(x)

      const result = bindSignal(m, pure)
      expect(result.getValue()).toBe(42)

      // 値を変更して確認
      m.setValue(100)
      expect(result.getValue()).toBe(100)

      // クリーンアップ
      result.detach()
      m.detach()
    })

    test("結合法則: (m >>= f) >>= g = m >>= (x => f(x) >>= g)", () => {
      const m = createSignal(5)
      const f = (x: number): Signal<number> => createSignal(x * 2)
      const g = (x: number): Signal<number> => createSignal(x + 1)

      // (m >>= f) >>= g
      const left = bindSignal(bindSignal(m, f), g)

      // m >>= (x => f(x) >>= g)
      const right = bindSignal(m, (x) => bindSignal(f(x), g))

      expect(left.getValue()).toBe(right.getValue())
      expect(left.getValue()).toBe(11) // ((5 * 2) + 1) = 11

      // 値を変更して確認
      m.setValue(3)
      expect(left.getValue()).toBe(right.getValue())
      expect(left.getValue()).toBe(7) // ((3 * 2) + 1) = 7

      // クリーンアップ
      left.detach()
      right.detach()
      m.detach()
    })

    test("実用例: Signal の変換", () => {
      const signal = createSignal(10)
      const result = bindSignal(signal, (x) => createSignal(x * 3))

      expect(result.getValue()).toBe(30)

      // 値を変更して確認
      signal.setValue(5)
      expect(result.getValue()).toBe(15)

      // クリーンアップ
      result.detach()
      signal.detach()
    })
  })
})

// =============================================================================
// 実用的な組み合わせテスト
// =============================================================================

describe("実用的な組み合わせテスト", () => {
  test("Maybe - 安全な計算チェーン", () => {
    const safeSqrt = (x: number): Maybe<number> =>
      x >= 0 ? Just(Math.sqrt(x)) : Nothing

    const safeDiv =
      (x: number) =>
      (y: number): Maybe<number> =>
        y !== 0 ? Just(x / y) : Nothing

    // 成功ケース: 16 -> sqrt(16) = 4 -> 4/2 = 2
    const success = bindMaybe(bindMaybe(Just(16), safeSqrt), (x) =>
      safeDiv(x)(2)
    )
    expect(success).toEqual(Just(2))

    // 失敗ケース: 負の数
    const failure = bindMaybe(bindMaybe(Just(-4), safeSqrt), (x) =>
      safeDiv(x)(2)
    )
    expect(failure).toEqual(Nothing)
  })

  test("Either - エラー処理チェーン", () => {
    const parseNum = (s: string): Either<string, number> => {
      const n = Number.parseFloat(s)
      return Number.isNaN(n) ? Left(`Cannot parse: ${s}`) : Right(n)
    }

    const ensurePositive = (n: number): Either<string, number> =>
      n > 0 ? Right(n) : Left(`Not positive: ${n}`)

    // 成功ケース
    const success = bindEither(
      bindEither(parseNum("42.5"), ensurePositive),
      (n) => Right(n * 2)
    )
    expect(success).toEqual(Right(85))

    // パースエラー
    const parseError = bindEither(
      bindEither(parseNum("not-a-number"), ensurePositive),
      (n) => Right(n * 2)
    )
    expect(parseError).toEqual(Left("Cannot parse: not-a-number"))

    // 負の数エラー
    const negativeError = bindEither(
      bindEither(parseNum("-5"), ensurePositive),
      (n) => Right(n * 2)
    )
    expect(negativeError).toEqual(Left("Not positive: -5"))
  })

  test("List - リスト内包表記風の操作", () => {
    const numbers = Cons(1, Cons(2, Cons(3, Empty)))

    // 各数値を2倍して、その数値とその平方を含むリストを作る
    const result = bindList(numbers, (x) =>
      Cons(x * 2, Cons(x * 2 * (x * 2), Empty))
    )

    // [1, 2, 3] >>= (\x -> [x*2, (x*2)^2]) = [2, 4, 4, 16, 6, 36]
    expect(result).toEqual(
      Cons(2, Cons(4, Cons(4, Cons(16, Cons(6, Cons(36, Empty))))))
    )
  })
})
