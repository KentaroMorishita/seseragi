/**
 * Seseragi Runtime Library
 * 関数型プログラミング機能のランタイムサポート
 */

// =============================================================================
// 型定義
// =============================================================================

export type Maybe<T> = { tag: "Just"; value: T } | { tag: "Nothing" }
export type Either<L, R> =
  | { tag: "Left"; value: L }
  | { tag: "Right"; value: R }
export type List<T> = { tag: "Empty" } | { tag: "Cons"; head: T; tail: List<T> }

// =============================================================================
// カリー化関数
// =============================================================================

export const curry = (fn: Function) => {
  return function curried(...args: any[]) {
    if (args.length >= fn.length) {
      return fn.apply(null, args)
    } else {
      return function (...args2: any[]) {
        return curried.apply(null, args.concat(args2))
      }
    }
  }
}

// =============================================================================
// パイプライン演算子
// =============================================================================

export const pipe = <T, U>(value: T, fn: (arg: T) => U): U => fn(value)

export const reversePipe = <T, U>(fn: (arg: T) => U, value: T): U => fn(value)

// =============================================================================
// 関数適用演算子
// =============================================================================

export const apply = <T, U>(fn: (arg: T) => U, value: T): U => fn(value)

// =============================================================================
// Maybe型 - Functor → Applicative → Monad
// =============================================================================

export const Just = <T>(value: T): Maybe<T> => ({ tag: "Just", value })

export const Nothing: Maybe<never> = { tag: "Nothing" }

// Functor: map (<$>)
export const mapMaybe = <T, U>(fa: Maybe<T>, f: (a: T) => U): Maybe<U> =>
  fa.tag === "Just" ? Just(f(fa.value)) : Nothing

// Applicative: pure + apply (<*>)
export const pureMaybe = <T>(value: T): Maybe<T> => Just(value)

export const applyMaybe = <T, U>(
  ff: Maybe<(a: T) => U>,
  fa: Maybe<T>
): Maybe<U> =>
  ff.tag === "Just" && fa.tag === "Just" ? Just(ff.value(fa.value)) : Nothing

// Monad: flatMap (>>=)
export const bindMaybe = <T, U>(
  ma: Maybe<T>,
  f: (value: T) => Maybe<U>
): Maybe<U> => {
  return ma.tag === "Just" ? f(ma.value) : Nothing
}

// Utility functions
export const isJust = <T>(
  maybe: Maybe<T>
): maybe is { tag: "Just"; value: T } => maybe.tag === "Just"

export const isNothing = <T>(maybe: Maybe<T>): maybe is { tag: "Nothing" } =>
  maybe.tag === "Nothing"

export const fromMaybe = <T>(defaultValue: T, maybe: Maybe<T>): T =>
  maybe.tag === "Just" ? maybe.value : defaultValue

// =============================================================================
// Either型 - Functor → Applicative → Monad
// =============================================================================

export const Left = <L>(value: L): Either<L, never> => ({ tag: "Left", value })

export const Right = <R>(value: R): Either<never, R> => ({
  tag: "Right",
  value,
})

// Functor: map (<$>) - Right側のみにmapを適用
export const mapEither = <L, R, U>(
  fa: Either<L, R>,
  f: (a: R) => U
): Either<L, U> =>
  fa.tag === "Right" ? Right(f(fa.value)) : (fa as Either<L, U>)

// Applicative: pure + apply (<*>)
export const pureEither = <R>(value: R): Either<never, R> => Right(value)

export const applyEither = <L, R, U>(
  ff: Either<L, (a: R) => U>,
  fa: Either<L, R>
): Either<L, U> =>
  ff.tag === "Right" && fa.tag === "Right"
    ? Right(ff.value(fa.value))
    : ff.tag === "Left"
      ? (ff as Either<L, U>)
      : (fa as Either<L, U>)

// Monad: flatMap (>>=)
export const bindEither = <L, R, U>(
  ea: Either<L, R>,
  f: (value: R) => Either<L, U>
): Either<L, U> => {
  return ea.tag === "Right" ? f(ea.value) : (ea as Either<L, U>)
}

// Utility functions
export const isLeft = <L, R>(
  either: Either<L, R>
): either is { tag: "Left"; value: L } => either.tag === "Left"

export const isRight = <L, R>(
  either: Either<L, R>
): either is { tag: "Right"; value: R } => either.tag === "Right"

export const fromLeft = <L, R>(defaultValue: L, either: Either<L, R>): L =>
  either.tag === "Left" ? either.value : defaultValue

export const fromRight = <L, R>(defaultValue: R, either: Either<L, R>): R =>
  either.tag === "Right" ? either.value : defaultValue

// =============================================================================
// List型 - Functor → Applicative → Monad
// =============================================================================

export const Empty: List<never> = { tag: "Empty" }

export const Cons = <T>(head: T, tail: List<T>): List<T> => ({
  tag: "Cons",
  head,
  tail,
})

// Functor: map (<$>)
export const mapList = <T, U>(fa: List<T>, f: (a: T) => U): List<U> => {
  const mapHelper = (list: List<T>): List<U> => {
    if (list.tag === "Empty") {
      return Empty
    } else {
      return Cons(f(list.head), mapHelper(list.tail))
    }
  }
  return mapHelper(fa)
}

// Applicative: pure + apply (<*>)
export const pureList = <T>(value: T): List<T> => Cons(value, Empty)

export const applyList = <T, U>(
  ff: List<(a: T) => U>,
  fa: List<T>
): List<U> => {
  const applyHelper = (funcs: List<(a: T) => U>, values: List<T>): List<U> => {
    if (funcs.tag === "Empty") {
      return Empty
    } else {
      const mappedValues = mapList(values, funcs.head)
      const restApplied = applyHelper(funcs.tail, values)
      return concatList(mappedValues, restApplied)
    }
  }
  return applyHelper(ff, fa)
}

// Monad: flatMap (>>=)
export const bindList = <T, U>(
  ma: List<T>,
  f: (value: T) => List<U>
): List<U> => {
  const bindHelper = (list: List<T>): List<U> => {
    if (list.tag === "Empty") {
      return Empty
    } else {
      const headResult = f(list.head)
      const tailResult = bindHelper(list.tail)
      return concatList(headResult, tailResult)
    }
  }
  return bindHelper(ma)
}

// Utility functions
export const isEmpty = <T>(list: List<T>): list is { tag: "Empty" } =>
  list.tag === "Empty"

export const isCons = <T>(
  list: List<T>
): list is { tag: "Cons"; head: T; tail: List<T> } => list.tag === "Cons"

export const headList = <T>(list: List<T>): T | undefined =>
  list.tag === "Cons" ? list.head : undefined

export const tailList = <T>(list: List<T>): List<T> =>
  list.tag === "Cons" ? list.tail : Empty

export const lengthList = <T>(list: List<T>): number => {
  const lengthHelper = (l: List<T>, acc: number): number => {
    if (l.tag === "Empty") {
      return acc
    } else {
      return lengthHelper(l.tail, acc + 1)
    }
  }
  return lengthHelper(list, 0)
}

export const concatList = <T>(list1: List<T>, list2: List<T>): List<T> => {
  const concatHelper = (l1: List<T>): List<T> => {
    if (l1.tag === "Empty") {
      return list2
    } else {
      return Cons(l1.head, concatHelper(l1.tail))
    }
  }
  return concatHelper(list1)
}

export const reverseList = <T>(list: List<T>): List<T> => {
  const reverseHelper = (l: List<T>, acc: List<T>): List<T> => {
    if (l.tag === "Empty") {
      return acc
    } else {
      return reverseHelper(l.tail, Cons(l.head, acc))
    }
  }
  return reverseHelper(list, Empty)
}

export const takeList = <T>(n: number, list: List<T>): List<T> => {
  if (n <= 0 || list.tag === "Empty") {
    return Empty
  } else {
    return Cons(list.head, takeList(n - 1, list.tail))
  }
}

export const dropList = <T>(n: number, list: List<T>): List<T> => {
  if (n <= 0) {
    return list
  } else if (list.tag === "Empty") {
    return Empty
  } else {
    return dropList(n - 1, list.tail)
  }
}

export const filterList = <T>(
  predicate: (value: T) => boolean,
  list: List<T>
): List<T> => {
  const filterHelper = (l: List<T>): List<T> => {
    if (l.tag === "Empty") {
      return Empty
    } else {
      const rest = filterHelper(l.tail)
      return predicate(l.head) ? Cons(l.head, rest) : rest
    }
  }
  return filterHelper(list)
}

export const foldLeftList = <T, U>(
  f: (acc: U, value: T) => U,
  initial: U,
  list: List<T>
): U => {
  const foldHelper = (l: List<T>, acc: U): U => {
    if (l.tag === "Empty") {
      return acc
    } else {
      return foldHelper(l.tail, f(acc, l.head))
    }
  }
  return foldHelper(list, initial)
}

export const foldRightList = <T, U>(
  f: (value: T, acc: U) => U,
  initial: U,
  list: List<T>
): U => {
  const foldHelper = (l: List<T>): U => {
    if (l.tag === "Empty") {
      return initial
    } else {
      return f(l.head, foldHelper(l.tail))
    }
  }
  return foldHelper(list)
}

// Convert from/to Array for interop
export const fromArray = <T>(arr: T[]): List<T> => {
  let result: List<T> = Empty
  for (let i = arr.length - 1; i >= 0; i--) {
    result = Cons(arr[i], result)
  }
  return result
}

export const toArray = <T>(list: List<T>): T[] => {
  const result: T[] = []
  let current = list
  while (current.tag === "Cons") {
    result.push(current.head)
    current = current.tail
  }
  return result
}

// =============================================================================
// モナド演算子
// =============================================================================

// 一般的なbind関数（Maybeのみサポート、下位互換性のため）
export const bind = <T, U>(
  maybe: Maybe<T>,
  fn: (value: T) => Maybe<U>
): Maybe<U> => bindMaybe(maybe, fn)

// 型固有のbind関数は上記で既に定義済み
// export { bindMaybe, bindEither }

// =============================================================================
// モノイド演算子
// =============================================================================

export const foldMonoid = <T>(
  arr: T[],
  empty: T,
  combine: (a: T, b: T) => T
): T => {
  return arr.reduce(combine, empty)
}

// =============================================================================
// 組み込み関数
// =============================================================================

export const print = (value: any): void => console.log(value)

export const putStrLn = (value: string): void => console.log(value)

export const toString = (value: any): string => String(value)

// =============================================================================
// ユーティリティ
// =============================================================================

export const identity = <T>(value: T): T => value

export const compose =
  <A, B, C>(f: (b: B) => C, g: (a: A) => B) =>
  (a: A): C =>
    f(g(a))
