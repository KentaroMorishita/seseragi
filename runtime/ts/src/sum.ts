import type { Unit } from "./effect"
import type { RuntimeDictionary } from "./traversable"

export type Nothing = {
  readonly tag: "Nothing"
}

export type Just<Value> = {
  readonly tag: "Just"
  readonly value: Value
}

export type Maybe<Value> = Nothing | Just<Value>

export type Left<Error> = {
  readonly tag: "Left"
  readonly value: Error
}

export type Right<Value> = {
  readonly tag: "Right"
  readonly value: Value
}

export type Either<Error, Value> = Left<Error> | Right<Value>

export type Ordering =
  | { readonly tag: "Less" }
  | { readonly tag: "Equal" }
  | { readonly tag: "Greater" }

export const Nothing: Nothing = Object.freeze({ tag: "Nothing" })

export function Just<Value>(value: Value): Just<Value> {
  return { tag: "Just", value }
}

export function Left<Error>(error: Error): Left<Error> {
  return { tag: "Left", value: error }
}

export function Right<Value>(value: Value): Right<Value> {
  return { tag: "Right", value }
}

export const Less: Ordering = Object.freeze({ tag: "Less" })
export const Equal: Ordering = Object.freeze({ tag: "Equal" })
export const Greater: Ordering = Object.freeze({ tag: "Greater" })

export const maybeFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    (value: Maybe<Value>): Maybe<Result> =>
      value.tag === "Nothing" ? Nothing : Just(f(value.value)),
})

export const maybeApplicative = Object.freeze({
  ...maybeFunctor,
  pure: <Value>(value: Value): Maybe<Value> => Just(value),
  apply:
    <Value, Result>(wrappedFunction: Maybe<(value: Value) => Result>) =>
    (wrappedValue: Maybe<Value>): Maybe<Result> => {
      if (wrappedFunction.tag === "Nothing") return Nothing
      if (wrappedValue.tag === "Nothing") return Nothing
      return Just(wrappedFunction.value(wrappedValue.value))
    },
})

export const maybeMonad = Object.freeze({
  ...maybeApplicative,
  flatMap:
    <Value, Result>(f: (value: Value) => Maybe<Result>) =>
    (value: Maybe<Value>): Maybe<Result> =>
      value.tag === "Nothing" ? Nothing : f(value.value),
})

export const eitherFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    <Error>(value: Either<Error, Value>): Either<Error, Result> =>
      value.tag === "Left" ? value : Right(f(value.value)),
})

export const eitherApplicative = Object.freeze({
  ...eitherFunctor,
  pure: <Error, Value>(value: Value): Either<Error, Value> => Right(value),
  apply:
    <Error, Value, Result>(
      wrappedFunction: Either<Error, (value: Value) => Result>
    ) =>
    (wrappedValue: Either<Error, Value>): Either<Error, Result> => {
      if (wrappedFunction.tag === "Left") return wrappedFunction
      if (wrappedValue.tag === "Left") return wrappedValue
      return Right(wrappedFunction.value(wrappedValue.value))
    },
})

export const eitherMonad = Object.freeze({
  ...eitherApplicative,
  flatMap:
    <Error, Value, Result>(f: (value: Value) => Either<Error, Result>) =>
    (value: Either<Error, Value>): Either<Error, Result> =>
      value.tag === "Left" ? value : f(value.value),
})

export function withDefault<A>(fallback: A, value: Maybe<A>): A {
  return value.tag === "Nothing" ? fallback : value.value
}

export function orElse<A>(fallback: Maybe<A>, value: Maybe<A>): Maybe<A> {
  return value.tag === "Nothing" ? fallback : value
}

export function mapLeft<E, F>(f: (error: E) => F, value: Left<E>): Left<F>
export function mapLeft<E, F, A>(f: (error: E) => F, value: Right<A>): Right<A>
export function mapLeft<E, F, A>(
  f: (error: E) => F,
  value: Either<E, A>
): Either<F, A>
export function mapLeft<E, F, A>(
  f: (error: E) => F,
  value: Either<E, A>
): Either<F, A> {
  return value.tag === "Left" ? Left(f(value.value)) : value
}

export function mapRight<E, A, B>(f: (value: A) => B, value: Left<E>): Left<E>
export function mapRight<A, B>(f: (value: A) => B, value: Right<A>): Right<B>
export function mapRight<E, A, B>(
  f: (value: A) => B,
  value: Either<E, A>
): Either<E, B>
export function mapRight<E, A, B>(
  f: (value: A) => B,
  value: Either<E, A>
): Either<E, B> {
  return eitherFunctor.map(f)(value)
}

export function bimap<E, F, A, B>(
  left: (error: E) => F,
  right: (value: A) => B,
  value: Either<E, A>
): Either<F, B> {
  return value.tag === "Left"
    ? Left(left(value.value))
    : Right(right(value.value))
}

export function fold<E, A, B>(
  left: (error: E) => B,
  right: (value: A) => B,
  value: Either<E, A>
): B {
  return value.tag === "Left" ? left(value.value) : right(value.value)
}

export function swap<E>(value: Left<E>): Right<E>
export function swap<A>(value: Right<A>): Left<A>
export function swap<E, A>(value: Either<E, A>): Either<A, E>
export function swap<E, A>(value: Either<E, A>): Either<A, E> {
  return value.tag === "Left" ? Right(value.value) : Left(value.value)
}

// Source F<_> is erased at this boundary. The selected Traversable owns source
// shape, evaluation order, and traversal; these wrappers only choose the target.
type TraversableDictionary = Readonly<{
  traverse: (
    f: (value: any) => unknown
  ) => (values: unknown) => (target: RuntimeDictionary) => unknown
}>

export function maybeTraverse<A, B>(
  evidence: RuntimeDictionary,
  f: (value: A) => Maybe<B>,
  values: unknown
): any {
  return (evidence as TraversableDictionary).traverse(f)(values)(
    maybeApplicative
  )
}

export function maybeSequence(
  evidence: RuntimeDictionary,
  values: unknown
): any {
  return maybeTraverse(evidence, (value: Maybe<unknown>) => value, values)
}

export function eitherTraverse<E, A, B>(
  evidence: RuntimeDictionary,
  f: (value: A) => Either<E, B>,
  values: unknown
): any {
  return (evidence as TraversableDictionary).traverse(f)(values)(
    eitherApplicative
  )
}

export function eitherSequence(
  evidence: RuntimeDictionary,
  values: unknown
): any {
  return eitherTraverse(
    evidence,
    (value: Either<unknown, unknown>) => value,
    values
  )
}

type Semigroup<A> = Readonly<{ append: (left: A) => (right: A) => A }>

export function maybeSemigroup<A>(evidence: RuntimeDictionary) {
  const element = evidence as Semigroup<A>
  return Object.freeze({
    append:
      (left: Maybe<A>) =>
      (right: Maybe<A>): Maybe<A> => {
        if (left.tag === "Nothing") return right
        if (right.tag === "Nothing") return left
        return Just(element.append(left.value)(right.value))
      },
  })
}

export function maybeMonoid<A>(element: RuntimeDictionary) {
  return Object.freeze({
    ...maybeSemigroup<A>(element),
    empty: (_unit: Unit): Maybe<A> => Nothing,
  })
}

/** Nominal choice of add composition; source evidence owns numeric behavior. */
export type Sum<A> = Readonly<{ tag: "Sum"; value: A }>
export function Sum<A>(value: A): Sum<A> {
  return Object.freeze({ tag: "Sum", value })
}
export function sumSemigroup<A>(operation: RuntimeDictionary) {
  const selected = operation as Readonly<{ add: (left: A) => (right: A) => A }>
  return Object.freeze({
    append:
      (left: Sum<A>) =>
      (right: Sum<A>): Sum<A> =>
        Sum(selected.add(left.value)(right.value)),
  })
}
export function sumMonoid<A>(
  identity: RuntimeDictionary,
  operation: RuntimeDictionary
) {
  const selected = identity as Readonly<{ zero: (unit: Unit) => A }>
  return Object.freeze({
    ...sumSemigroup<A>(operation),
    empty: (_unit: Unit): Sum<A> => Sum(selected.zero(undefined)),
  })
}

/** Nominal choice of mul composition; source evidence owns numeric behavior. */
export type Product<A> = Readonly<{ tag: "Product"; value: A }>
export function Product<A>(value: A): Product<A> {
  return Object.freeze({ tag: "Product", value })
}
export function productSemigroup<A>(operation: RuntimeDictionary) {
  const selected = operation as Readonly<{ mul: (left: A) => (right: A) => A }>
  return Object.freeze({
    append:
      (left: Product<A>) =>
      (right: Product<A>): Product<A> =>
        Product(selected.mul(left.value)(right.value)),
  })
}
export function productMonoid<A>(
  identity: RuntimeDictionary,
  operation: RuntimeDictionary
) {
  const selected = identity as Readonly<{ one: (unit: Unit) => A }>
  return Object.freeze({
    ...productSemigroup<A>(operation),
    empty: (_unit: Unit): Product<A> => Product(selected.one(undefined)),
  })
}
