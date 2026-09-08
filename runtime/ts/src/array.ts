import { type Iterable, NonPositiveSize, type SizeError } from "./collection"
import type { Unit } from "./effect"
import type { Eq } from "./equality"
import type { Hash } from "./hash"
import type { Iterator as SeseragiIterator } from "./iterator"
import { fromArray, type List } from "./list"
import * as maps from "./map"
import { stableSort, stableSortBy } from "./sequence"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"
import { type RuntimeDictionary, traverseValues } from "./traversable"

export const empty = <A>(_unit?: Unit): ReadonlyArray<A> => []

export const singleton = <A>(value: A): ReadonlyArray<A> => [value]

export function fromIterable<C, A>(
  dictionary: Iterable<C, A> | RuntimeDictionary,
  values: C
): ReadonlyArray<A> {
  let iterator = (dictionary as Iterable<C, A>).iterate(values)
  const result: A[] = []
  while (true) {
    const step = iterator.next()
    if (step.tag === "Nothing") return result
    result.push(step.value[0])
    iterator = step.value[1]
  }
}

export function reduceRight<A, B>(
  initial: B,
  step: (value: A) => (accumulator: B) => B,
  values: ReadonlyArray<A>
): B {
  let result = initial
  for (let index = values.length - 1; index >= 0; index -= 1) {
    result = step(values[index] as A)(result)
  }
  return result
}

export function findIndex<A>(
  predicate: (value: A) => boolean,
  values: ReadonlyArray<A>
): Maybe<number> {
  for (let index = 0; index < values.length; index += 1) {
    if (predicate(values[index] as A)) return Just(index)
  }
  return Nothing
}

export function takeWhile<A>(
  predicate: (value: A) => boolean,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  let end = 0
  while (end < values.length && predicate(values[end] as A)) end += 1
  return values.slice(0, end)
}

export function dropWhile<A>(
  predicate: (value: A) => boolean,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  let start = 0
  while (start < values.length && predicate(values[start] as A)) start += 1
  return values.slice(start)
}

export function zipWith<A, B, C>(
  f: (left: A) => (right: B) => C,
  right: ReadonlyArray<B>,
  left: ReadonlyArray<A>
): ReadonlyArray<C> {
  const result: C[] = []
  const size = Math.min(left.length, right.length)
  for (let index = 0; index < size; index += 1) {
    result.push(f(left[index] as A)(right[index] as B))
  }
  return result
}

export function zip<A, B>(
  right: ReadonlyArray<B>,
  left: ReadonlyArray<A>
): ReadonlyArray<readonly [A, B]> {
  return zipWith<A, B, readonly [A, B]>((a) => (b) => [a, b], right, left)
}

export function unzip<A, B>(
  values: ReadonlyArray<readonly [A, B]>
): readonly [ReadonlyArray<A>, ReadonlyArray<B>] {
  const left: A[] = []
  const right: B[] = []
  for (const [a, b] of values) {
    left.push(a)
    right.push(b)
  }
  return [left, right]
}

export const sort = stableSort
export const sortBy = stableSortBy

export function groupBy<A, K>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  key: (value: A) => K,
  values: ReadonlyArray<A>
): maps.Map<K, ReadonlyArray<A>> {
  // Builders are private to this call: no repeated immutable append per group.
  let groups = maps.empty<K, A[]>()
  for (const value of values) {
    const k = key(value)
    const group = maps.get(eq, hash, k, groups)
    if (group.tag === "Just") group.value.push(value)
    else groups = maps.insert(eq, hash, k, [value], groups)
  }
  return maps.mapValues((group: A[]) => Object.freeze(group), groups)
}

export function last<A>(values: ReadonlyArray<A>): Maybe<A> {
  return values.length === 0 ? Nothing : Just(values[values.length - 1] as A)
}

export function init<A>(values: ReadonlyArray<A>): Maybe<ReadonlyArray<A>> {
  return values.length === 0 ? Nothing : Just(values.slice(0, -1))
}

export function chunksOf<A>(
  size: number,
  values: ReadonlyArray<A>
): Either<SizeError, ReadonlyArray<ReadonlyArray<A>>> {
  if (size <= 0) return Left(NonPositiveSize(size))
  const result: ReadonlyArray<A>[] = []
  for (let start = 0; start < values.length; start += size) {
    result.push(values.slice(start, start + size))
  }
  return Right(result)
}

export function windows<A>(
  size: number,
  values: ReadonlyArray<A>
): Either<SizeError, ReadonlyArray<ReadonlyArray<A>>> {
  if (size <= 0) return Left(NonPositiveSize(size))
  const result: ReadonlyArray<A>[] = []
  for (let start = 0; start <= values.length - size; start += 1) {
    result.push(values.slice(start, start + size))
  }
  return Right(result)
}

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

/** Convert an Array to the persistent List representation in source order. */
export function toList<A>(values: ReadonlyArray<A>): List<A> {
  return fromArray(values)
}

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
  count: number,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  if (count <= 0) return []
  if (count >= values.length) return values.slice()
  return values.slice(0, count)
}

export function drop<A>(
  count: number,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  if (count <= 0) return values.slice()
  if (count >= values.length) return []
  return values.slice(count)
}

export function append<A>(
  suffix: ReadonlyArray<A>,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  return [...values, ...suffix]
}

export function concat<A>(
  values: ReadonlyArray<ReadonlyArray<A>>
): ReadonlyArray<A> {
  const result: A[] = []
  for (const value of values) result.push(...value)
  return result
}

export function reverse<A>(values: ReadonlyArray<A>): ReadonlyArray<A> {
  return values.slice().reverse()
}

export function length<A>(values: ReadonlyArray<A>): number {
  return values.length
}

export function isEmpty<A>(values: ReadonlyArray<A>): boolean {
  return values.length === 0
}

// Receiver-first syntax ABI; safe bounds semantics remain owned by get.
export function index<A>(values: ReadonlyArray<A>, offset: number) {
  return get(offset, values)
}

export function get<A>(index: number, values: ReadonlyArray<A>) {
  if (index < 0 || index >= values.length) return Nothing
  return Just(values[index] as A)
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

export const arrayTraversable = Object.freeze({
  ...arrayFunctor,
  traverse:
    <Value, Result>(f: (value: Value) => unknown) =>
    (values: ReadonlyArray<Value>) =>
    (applicativeEvidence: RuntimeDictionary) =>
      traverseValues<Value, Result, ReadonlyArray<Result>>(
        values,
        f,
        applicativeEvidence,
        (results) => results
      ),
})
