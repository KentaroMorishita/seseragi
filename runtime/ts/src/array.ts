import type { Iterator as SeseragiIterator } from "./iterator"
import { Just, Nothing } from "./sum"

/** Runtime implementation of the standard `Reducible<Array<A>, A>` instance. */
export function reduce<A, B>(
  initial: B,
  step: (accumulator: B) => (value: A) => B,
  values: ReadonlyArray<A>
): B {
  let accumulator = initial
  for (const value of values) {
    accumulator = step(accumulator)(value)
  }
  return accumulator
}

export const arrayReducible = Object.freeze({
  reduce:
    <A, B>(initial: B) =>
    (step: (accumulator: B) => (value: A) => B) =>
    (values: ReadonlyArray<A>): B =>
      reduce(initial, step, values),
})

function arrayIterator<A>(
  values: ReadonlyArray<A>,
  index: number
): SeseragiIterator<A> {
  return {
    next: () =>
      index < values.length
        ? Just([values[index] as A, arrayIterator(values, index + 1)] as const)
        : Nothing,
  }
}

export const arrayIterable = Object.freeze({
  iterate: <A>(values: ReadonlyArray<A>): SeseragiIterator<A> =>
    arrayIterator(values, 0),
})

/** Pure comprehension lowering for the standard Array Iterable instance. */
export function collectMap<A, B>(
  values: ReadonlyArray<A>,
  predicate: (value: A) => boolean,
  transform: (value: A) => B
): ReadonlyArray<B> {
  const result: B[] = []
  for (const value of values) {
    if (predicate(value)) {
      result.push(transform(value))
    }
  }
  return result
}

/** Nested pure comprehension lowering for the standard Array Iterable instance. */
export function collectFlatMap<A, B>(
  values: ReadonlyArray<A>,
  predicate: (value: A) => boolean,
  transform: (value: A) => ReadonlyArray<B>
): ReadonlyArray<B> {
  const result: B[] = []
  for (const value of values) {
    if (predicate(value)) {
      result.push(...transform(value))
    }
  }
  return result
}

export const arrayFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    (values: ReadonlyArray<Value>): ReadonlyArray<Result> =>
      values.map(f),
})

export const arrayApplicative = Object.freeze({
  ...arrayFunctor,
  pure: <Value>(value: Value): ReadonlyArray<Value> => [value],
  apply:
    <Value, Result>(functions: ReadonlyArray<(value: Value) => Result>) =>
    (values: ReadonlyArray<Value>): ReadonlyArray<Result> => {
      const result: Result[] = []
      for (const f of functions) {
        for (const value of values) {
          result.push(f(value))
        }
      }
      return result
    },
})

export const arrayMonad = Object.freeze({
  ...arrayApplicative,
  flatMap:
    <Value, Result>(f: (value: Value) => ReadonlyArray<Result>) =>
    (values: ReadonlyArray<Value>): ReadonlyArray<Result> => {
      const result: Result[] = []
      for (const value of values) {
        result.push(...f(value))
      }
      return result
    },
})
