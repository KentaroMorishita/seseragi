import type { Iterator as SeseragiIterator } from "./iterator"
import type { Unit } from "./effect"
import { Just, Nothing, type Maybe } from "./sum"

/** Runtime dictionary for the standard `Semigroup<Array<A>>` instance. */
export const arraySemigroup = Object.freeze({
  append:
    <A>(left: ReadonlyArray<A>) =>
    (right: ReadonlyArray<A>): ReadonlyArray<A> => [...left, ...right],
})

/** Runtime dictionary for the standard `Monoid<Array<A>>` instance. */
export const arrayMonoid = Object.freeze({
  ...arraySemigroup,
  empty: <A>(_unit: Unit): ReadonlyArray<A> => [],
})

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

export function filter<A>(
  predicate: (value: A) => boolean,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  return values.filter(predicate)
}

export function filterMap<A, B>(
  f: (value: A) => Maybe<B>,
  values: ReadonlyArray<A>
): ReadonlyArray<B> {
  const result: B[] = []
  for (const value of values) {
    const mapped = f(value)
    if (mapped.tag === "Just") result.push(mapped.value)
  }
  return result
}

export function flatMap<A, B>(
  f: (value: A) => ReadonlyArray<B>,
  values: ReadonlyArray<A>
): ReadonlyArray<B> {
  const result: B[] = []
  for (const value of values) result.push(...f(value))
  return result
}

export function find<A>(
  predicate: (value: A) => boolean,
  values: ReadonlyArray<A>
) {
  for (const value of values) {
    if (predicate(value)) return Just(value)
  }
  return Nothing
}

export function take<A>(
  count: bigint,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  if (count <= 0n) return []
  if (count >= BigInt(values.length)) return values.slice()
  return values.slice(0, Number(count))
}

export function drop<A>(
  count: bigint,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  if (count <= 0n) return values.slice()
  if (count >= BigInt(values.length)) return []
  return values.slice(Number(count))
}

export function length<A>(values: ReadonlyArray<A>): bigint {
  return BigInt(values.length)
}

export function isEmpty<A>(values: ReadonlyArray<A>): boolean {
  return values.length === 0
}

export function get<A>(index: bigint, values: ReadonlyArray<A>) {
  if (index < 0n || index >= BigInt(values.length)) return Nothing
  return Just(values[Number(index)] as A)
}

export function head<A>(values: ReadonlyArray<A>) {
  return values.length === 0 ? Nothing : Just(values[0] as A)
}

export function tail<A>(values: ReadonlyArray<A>) {
  return values.length === 0 ? Nothing : Just(values.slice(1))
}

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
