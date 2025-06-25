/**
 * Seseragi Runtime Library
 * 関数型プログラミング機能のランタイムサポート
 */

// =============================================================================
// 型定義
// =============================================================================

export type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };
export type Either<L, R> = { tag: 'Left'; value: L } | { tag: 'Right'; value: R };

// =============================================================================
// カリー化関数
// =============================================================================

export const curry = (fn: Function) => {
  return function curried(...args: any[]) {
    if (args.length >= fn.length) {
      return fn.apply(this, args);
    } else {
      return function(...args2: any[]) {
        return curried.apply(this, args.concat(args2));
      };
    }
  };
};

// =============================================================================
// パイプライン演算子
// =============================================================================

export const pipe = <T, U>(value: T, fn: (arg: T) => U): U => fn(value);

export const reversePipe = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);

// =============================================================================
// 関数適用演算子
// =============================================================================

export const apply = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);

// =============================================================================
// Maybe型 - Functor → Applicative → Monad
// =============================================================================

export const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });

export const Nothing: Maybe<never> = { tag: 'Nothing' };

// Functor: map (<$>)
export const mapMaybe = <T, U>(fa: Maybe<T>, f: (a: T) => U): Maybe<U> => 
  fa.tag === 'Just' ? Just(f(fa.value)) : Nothing;

// Applicative: pure + apply (<*>)
export const pureMaybe = <T>(value: T): Maybe<T> => Just(value);

export const applyMaybe = <T, U>(ff: Maybe<(a: T) => U>, fa: Maybe<T>): Maybe<U> =>
  ff.tag === 'Just' && fa.tag === 'Just' 
    ? Just(ff.value(fa.value)) 
    : Nothing;

// Monad: flatMap (>>=)
export const bindMaybe = <T, U>(
  ma: Maybe<T>, 
  f: (value: T) => Maybe<U>
): Maybe<U> => {
  return ma.tag === 'Just' ? f(ma.value) : Nothing;
};

// Utility functions
export const isJust = <T>(maybe: Maybe<T>): maybe is { tag: 'Just'; value: T } => 
  maybe.tag === 'Just';

export const isNothing = <T>(maybe: Maybe<T>): maybe is { tag: 'Nothing' } => 
  maybe.tag === 'Nothing';

export const fromMaybe = <T>(defaultValue: T, maybe: Maybe<T>): T =>
  maybe.tag === 'Just' ? maybe.value : defaultValue;

// =============================================================================
// Either型 - Functor → Applicative → Monad
// =============================================================================

export const Left = <L>(value: L): Either<L, never> => ({ tag: 'Left', value });

export const Right = <R>(value: R): Either<never, R> => ({ tag: 'Right', value });

// Functor: map (<$>) - Right側のみにmapを適用
export const mapEither = <L, R, U>(fa: Either<L, R>, f: (a: R) => U): Either<L, U> =>
  fa.tag === 'Right' ? Right(f(fa.value)) : fa as Either<L, U>;

// Applicative: pure + apply (<*>)
export const pureEither = <R>(value: R): Either<never, R> => Right(value);

export const applyEither = <L, R, U>(
  ff: Either<L, (a: R) => U>, 
  fa: Either<L, R>
): Either<L, U> =>
  ff.tag === 'Right' && fa.tag === 'Right'
    ? Right(ff.value(fa.value))
    : ff.tag === 'Left' ? ff as Either<L, U> : fa as Either<L, U>;

// Monad: flatMap (>>=)
export const bindEither = <L, R, U>(
  ea: Either<L, R>,
  f: (value: R) => Either<L, U>
): Either<L, U> => {
  return ea.tag === 'Right' ? f(ea.value) : ea as Either<L, U>;
};

// Utility functions
export const isLeft = <L, R>(either: Either<L, R>): either is { tag: 'Left'; value: L } => 
  either.tag === 'Left';

export const isRight = <L, R>(either: Either<L, R>): either is { tag: 'Right'; value: R } => 
  either.tag === 'Right';

export const fromLeft = <L, R>(defaultValue: L, either: Either<L, R>): L =>
  either.tag === 'Left' ? either.value : defaultValue;

export const fromRight = <L, R>(defaultValue: R, either: Either<L, R>): R =>
  either.tag === 'Right' ? either.value : defaultValue;

// =============================================================================
// モナド演算子
// =============================================================================

// 一般的なbind関数（Maybeのみサポート、下位互換性のため）
export const bind = <T, U>(
  maybe: Maybe<T>, 
  fn: (value: T) => Maybe<U>
): Maybe<U> => bindMaybe(maybe, fn);

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
  return arr.reduce(combine, empty);
};

// =============================================================================
// 組み込み関数
// =============================================================================

export const print = (value: any): void => console.log(value);

export const putStrLn = (value: string): void => console.log(value);

export const toString = (value: any): string => String(value);

// =============================================================================
// ユーティリティ
// =============================================================================

export const identity = <T>(value: T): T => value;

export const compose = <A, B, C>(
  f: (b: B) => C, 
  g: (a: A) => B
) => (a: A): C => f(g(a));