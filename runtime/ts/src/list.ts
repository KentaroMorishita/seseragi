import { type Iterable, NonPositiveSize, type SizeError } from "./collection"
import type { Unit } from "./effect"
import type { Eq } from "./equality"
import type { Hash } from "./hash"
import type { Iterator as SeseragiIterator } from "./iterator"
import * as maps from "./map"
import { type Ord, stableSort, stableSortBy } from "./sequence"
import {
  type Either,
  Equal,
  Greater,
  Just,
  Left,
  Less,
  type Maybe,
  Nothing,
  type Ordering,
  Right,
} from "./sum"
import { type RuntimeDictionary, traverseValues } from "./traversable"

/** Immutable persistent linked list used by the Seseragi `List<A>` ABI. */
export type List<A> = Empty | Cons<A>

export type Empty = Readonly<{
  tag: "Empty"
}>

export type Cons<A> = Readonly<{
  tag: "Cons"
  head: A
  tail: List<A>
}>

/** Immutable, non-empty view over a persistent List. */
export type NonEmptyList<A> = Readonly<{
  readonly tag: "NonEmpty"
  readonly head: A
  readonly tail: List<A>
}>

export const Empty: Empty = Object.freeze({ tag: "Empty" })

export const empty = <A>(_unit?: Unit): List<A> => Empty

/** List singleton; the existing singleton runtime export belongs to NonEmptyList. */
export const singletonList = <A>(value: A): List<A> => Cons(value, Empty)

export function fromIterable<C, A>(
  dictionary: Iterable<C, A> | RuntimeDictionary,
  values: C
): List<A> {
  let iterator = (dictionary as Iterable<C, A>).iterate(values)
  let reversed: List<A> = Empty
  while (true) {
    const step = iterator.next()
    if (step.tag === "Nothing") return reverse(reversed)
    reversed = Cons(step.value[0], reversed)
    iterator = step.value[1]
  }
}

export function reduceRight<A, B>(
  initial: B,
  step: (value: A) => (accumulator: B) => B,
  values: List<A>
): B {
  let result = initial
  let cursor = reverse(values)
  while (cursor.tag === "Cons") {
    result = step(cursor.head)(result)
    cursor = cursor.tail
  }
  return result
}

export function findIndex<A>(
  predicate: (value: A) => boolean,
  values: List<A>
): Maybe<number> {
  let cursor = values
  let index = 0
  while (cursor.tag === "Cons") {
    if (predicate(cursor.head)) return Just(index)
    cursor = cursor.tail
    index += 1
  }
  return Nothing
}

export function takeWhile<A>(
  predicate: (value: A) => boolean,
  values: List<A>
): List<A> {
  let reversed: List<A> = Empty
  let cursor = values
  while (cursor.tag === "Cons" && predicate(cursor.head)) {
    reversed = Cons(cursor.head, reversed)
    cursor = cursor.tail
  }
  return reverse(reversed)
}

export function dropWhile<A>(
  predicate: (value: A) => boolean,
  values: List<A>
): List<A> {
  let cursor = values
  while (cursor.tag === "Cons" && predicate(cursor.head)) cursor = cursor.tail
  return cursor
}

export function zipWith<A, B, C>(
  f: (left: A) => (right: B) => C,
  right: List<B>,
  left: List<A>
): List<C> {
  let a = left
  let b = right
  let reversed: List<C> = Empty
  while (a.tag === "Cons" && b.tag === "Cons") {
    reversed = Cons(f(a.head)(b.head), reversed)
    a = a.tail
    b = b.tail
  }
  return reverse(reversed)
}

export function zip<A, B>(
  right: List<B>,
  left: List<A>
): List<readonly [A, B]> {
  return zipWith<A, B, readonly [A, B]>((a) => (b) => [a, b], right, left)
}

export function unzip<A, B>(
  values: List<readonly [A, B]>
): readonly [List<A>, List<B>] {
  let left: List<A> = Empty
  let right: List<B> = Empty
  let cursor = values
  while (cursor.tag === "Cons") {
    left = Cons(cursor.head[0], left)
    right = Cons(cursor.head[1], right)
    cursor = cursor.tail
  }
  return [reverse(left), reverse(right)]
}

export function sort<A>(
  ord: Ord<A> | RuntimeDictionary,
  values: List<A>
): List<A> {
  return fromArray(stableSort(ord, toArray(values)))
}

export function sortBy<A, K>(
  ord: Ord<K> | RuntimeDictionary,
  key: (value: A) => K,
  values: List<A>
): List<A> {
  return fromArray(stableSortBy(ord, key, toArray(values)))
}

export function groupBy<A, K>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  key: (value: A) => K,
  values: List<A>
): maps.Map<K, List<A>> {
  let groups = maps.empty<K, List<A>>()
  let cursor = values
  while (cursor.tag === "Cons") {
    const value = cursor.head
    groups = maps.upsert(
      eq,
      hash,
      key(value),
      (group) => Cons(value, group.tag === "Just" ? group.value : Empty),
      groups
    )
    cursor = cursor.tail
  }
  return maps.mapValues(reverse<A>, groups)
}

export function last<A>(values: List<A>): Maybe<A> {
  if (values.tag === "Empty") return Nothing
  let cursor = values
  while (cursor.tail.tag === "Cons") cursor = cursor.tail
  return Just(cursor.head)
}

export function init<A>(values: List<A>): Maybe<List<A>> {
  if (values.tag === "Empty") return Nothing
  let reversed: List<A> = Empty
  let cursor = values
  while (cursor.tail.tag === "Cons") {
    reversed = Cons(cursor.head, reversed)
    cursor = cursor.tail
  }
  return Just(reverse(reversed))
}

export function chunksOf<A>(
  size: number,
  values: List<A>
): Either<SizeError, List<List<A>>> {
  if (size <= 0) return Left(NonPositiveSize(size))
  let chunks: List<List<A>> = Empty
  let cursor = values
  while (cursor.tag === "Cons") {
    let chunk: List<A> = Empty
    for (let count = 0; count < size && cursor.tag === "Cons"; count += 1) {
      chunk = Cons(cursor.head, chunk)
      cursor = cursor.tail
    }
    chunks = Cons(reverse(chunk), chunks)
  }
  return Right(reverse(chunks))
}

export function windows<A>(
  size: number,
  values: List<A>
): Either<SizeError, List<List<A>>> {
  if (size <= 0) return Left(NonPositiveSize(size))
  // Advance the lookahead once, then each start/end cursor once per output.
  // Each window owns only its elements, never an excluded suffix.
  let end = values
  for (let count = 0; count < size; count += 1) {
    if (end.tag === "Empty") return Right(Empty)
    end = end.tail
  }
  let start = values
  let result: List<List<A>> = Empty
  while (start.tag === "Cons") {
    result = Cons(take(size, start), result)
    if (end.tag === "Empty") break
    start = start.tail
    end = end.tail
  }
  return Right(reverse(result))
}

export function Cons<A>(head: A, tail: List<A>): List<A> {
  return Object.freeze({ tag: "Cons", head, tail })
}

export function NonEmptyList<A>(head: A, tail: List<A>): NonEmptyList<A> {
  return Object.freeze({ tag: "NonEmpty", head, tail })
}

export function singleton<A>(value: A): NonEmptyList<A> {
  return NonEmptyList(value, Empty)
}

export function consNonEmpty<A>(head: A, tail: List<A>): NonEmptyList<A> {
  return NonEmptyList(head, tail)
}

export function fromListNonEmpty<A>(values: List<A>): Maybe<NonEmptyList<A>> {
  return values.tag === "Empty"
    ? Nothing
    : Just(NonEmptyList(values.head, values.tail))
}

export function toListNonEmpty<A>(values: NonEmptyList<A>): List<A> {
  return Cons(values.head, values.tail)
}

export function headNonEmpty<A>(values: NonEmptyList<A>): A {
  return values.head
}

export function tailNonEmpty<A>(values: NonEmptyList<A>): List<A> {
  return values.tail
}

export function reduce1NonEmpty<A>(
  step: (accumulator: A) => (value: A) => A,
  values: NonEmptyList<A>
): A {
  let accumulator = values.head
  let cursor = values.tail
  while (cursor.tag === "Cons") {
    accumulator = step(accumulator)(cursor.head)
    cursor = cursor.tail
  }
  return accumulator
}

/** Build a persistent list without exposing its runtime representation to codegen. */
export function fromArray<A>(values: ReadonlyArray<A>): List<A> {
  let result: List<A> = Empty
  for (let index = values.length - 1; index >= 0; index -= 1) {
    result = Cons(values[index] as A, result)
  }
  return result
}

/** Append two persistent lists while preserving their source order. */
function appendValues<A>(left: List<A>, right: List<A>): List<A> {
  const values: A[] = []
  let cursor = left
  while (cursor.tag === "Cons") {
    values.push(cursor.head)
    cursor = cursor.tail
  }
  let result = right
  for (let index = values.length - 1; index >= 0; index -= 1) {
    result = Cons(values[index] as A, result)
  }
  return result
}

type OrdEvidence<Value> = Readonly<{
  compare: (left: Value) => (right: Value) => Ordering
}>

/** Runtime dictionary for lexicographic `Ord<NonEmptyList<A>>`. */
export const nonEmptyListOrd = <Value>(
  element: OrdEvidence<Value>
): OrdEvidence<NonEmptyList<Value>> =>
  Object.freeze({
    compare:
      (left: NonEmptyList<Value>) =>
      (right: NonEmptyList<Value>): Ordering => {
        let leftCursor = toListNonEmpty(left)
        let rightCursor = toListNonEmpty(right)
        while (leftCursor.tag === "Cons" && rightCursor.tag === "Cons") {
          const ordering = element.compare(leftCursor.head)(rightCursor.head)
          if (ordering.tag !== "Equal") return ordering
          leftCursor = leftCursor.tail
          rightCursor = rightCursor.tail
        }
        if (leftCursor.tag === "Cons") return Greater
        if (rightCursor.tag === "Cons") return Less
        return Equal
      },
  })

/** Runtime dictionary for the standard `Semigroup<List<A>>` instance. */
export const listSemigroup = Object.freeze({
  append:
    <A>(left: List<A>) =>
    (right: List<A>): List<A> =>
      appendValues(left, right),
})

/** Runtime dictionary for source-order `Semigroup<NonEmptyList<A>>`. */
export const nonEmptyListSemigroup = Object.freeze({
  append:
    <A>(left: NonEmptyList<A>) =>
    (right: NonEmptyList<A>): NonEmptyList<A> =>
      NonEmptyList(left.head, appendValues(left.tail, toListNonEmpty(right))),
})

/** Runtime dictionary for the standard `Monoid<List<A>>` instance. */
export const listMonoid = Object.freeze({
  ...listSemigroup,
  empty: <A>(_unit: Unit): List<A> => Empty,
})

/** Runtime implementation of the standard `Reducible<List<A>, A>` instance. */
export function reduce<A, B>(
  initial: B,
  step: (accumulator: B) => (value: A) => B,
  values: List<A>
): B {
  let accumulator = initial
  let cursor = values
  while (cursor.tag === "Cons") {
    accumulator = step(accumulator)(cursor.head)
    cursor = cursor.tail
  }
  return accumulator
}

export const listReducible = Object.freeze({
  reduce:
    <A, B>(initial: B) =>
    (step: (accumulator: B) => (value: A) => B) =>
    (values: List<A>): B =>
      reduce(initial, step, values),
})

export const nonEmptyListReducible = Object.freeze({
  reduce:
    <B>(initial: B) =>
    <A>(step: (accumulator: B) => (value: A) => B) =>
    (values: NonEmptyList<A>): B =>
      reduce(initial, step, toListNonEmpty(values)),
})

function listIterator<A>(values: List<A>): SeseragiIterator<A> {
  return {
    next: () =>
      values.tag === "Cons"
        ? Just([values.head, listIterator(values.tail)] as const)
        : Nothing,
  }
}

export const listIterable = Object.freeze({
  iterate: <A>(values: List<A>): SeseragiIterator<A> => listIterator(values),
})

export const nonEmptyListIterable = Object.freeze({
  iterate: <A>(values: NonEmptyList<A>): SeseragiIterator<A> =>
    listIterator(toListNonEmpty(values)),
})

/** Convert a persistent List to an Array without exposing its representation. */
export function toArray<A>(values: List<A>): ReadonlyArray<A> {
  const result: A[] = []
  let cursor = values
  while (cursor.tag === "Cons") {
    result.push(cursor.head)
    cursor = cursor.tail
  }
  return result
}

export function filter<A>(
  predicate: (value: A) => boolean,
  values: List<A>
): List<A> {
  const result: A[] = []
  let cursor = values
  while (cursor.tag === "Cons") {
    if (predicate(cursor.head)) result.push(cursor.head)
    cursor = cursor.tail
  }
  return fromArray(result)
}

export function filterMap<A, B>(
  f: (value: A) => Maybe<B>,
  values: List<A>
): List<B> {
  const result: B[] = []
  let cursor = values
  while (cursor.tag === "Cons") {
    const mapped = f(cursor.head)
    if (mapped.tag === "Just") result.push(mapped.value)
    cursor = cursor.tail
  }
  return fromArray(result)
}

export function flatMap<A, B>(
  f: (value: A) => List<B>,
  values: List<A>
): List<B> {
  const result: B[] = []
  let outer = values
  while (outer.tag === "Cons") {
    let inner = f(outer.head)
    while (inner.tag === "Cons") {
      result.push(inner.head)
      inner = inner.tail
    }
    outer = outer.tail
  }
  return fromArray(result)
}

export function find<A>(predicate: (value: A) => boolean, values: List<A>) {
  let cursor = values
  while (cursor.tag === "Cons") {
    if (predicate(cursor.head)) return Just(cursor.head)
    cursor = cursor.tail
  }
  return Nothing
}

export function take<A>(count: number, values: List<A>): List<A> {
  if (count <= 0) return Empty
  const result: A[] = []
  let remaining = count
  let cursor = values
  while (remaining > 0 && cursor.tag === "Cons") {
    result.push(cursor.head)
    remaining -= 1
    cursor = cursor.tail
  }
  return fromArray(result)
}

export function drop<A>(count: number, values: List<A>): List<A> {
  if (count <= 0) return values
  let remaining = count
  let cursor = values
  while (remaining > 0 && cursor.tag === "Cons") {
    remaining -= 1
    cursor = cursor.tail
  }
  return cursor
}

export function append<A>(suffix: List<A>, values: List<A>): List<A> {
  return appendValues(values, suffix)
}

export function concat<A>(values: List<List<A>>): List<A> {
  const result: A[] = []
  let outer = values
  while (outer.tag === "Cons") {
    let inner = outer.head
    while (inner.tag === "Cons") {
      result.push(inner.head)
      inner = inner.tail
    }
    outer = outer.tail
  }
  return fromArray(result)
}

export function reverse<A>(values: List<A>): List<A> {
  let result: List<A> = Empty
  let cursor = values
  while (cursor.tag === "Cons") {
    result = Cons(cursor.head, result)
    cursor = cursor.tail
  }
  return result
}

export function length<A>(values: List<A>): number {
  let result = 0
  let cursor = values
  while (cursor.tag === "Cons") {
    result += 1
    cursor = cursor.tail
  }
  return result
}

export function isEmpty<A>(values: List<A>): boolean {
  return values.tag === "Empty"
}

export function get<A>(index: number, values: List<A>) {
  if (index < 0) return Nothing
  let remaining = index
  let cursor = values
  while (cursor.tag === "Cons") {
    if (remaining === 0) return Just(cursor.head)
    remaining -= 1
    cursor = cursor.tail
  }
  return Nothing
}

export function head<A>(values: List<A>) {
  return values.tag === "Empty" ? Nothing : Just(values.head)
}

export function tail<A>(values: List<A>) {
  return values.tag === "Empty" ? Nothing : Just(values.tail)
}

/** Pure comprehension lowering for the standard List Iterable instance. */
export function collectMap<A, B>(
  values: List<A>,
  predicate: (value: A) => boolean,
  transform: (value: A) => B
): ReadonlyArray<B> {
  const result: B[] = []
  let cursor = values
  while (cursor.tag === "Cons") {
    if (predicate(cursor.head)) {
      result.push(transform(cursor.head))
    }
    cursor = cursor.tail
  }
  return result
}

/** Nested comprehension lowering for the standard List Iterable instance. */
export function collectFlatMap<A, B>(
  values: List<A>,
  predicate: (value: A) => boolean,
  transform: (value: A) => ReadonlyArray<B>
): ReadonlyArray<B> {
  const result: B[] = []
  let cursor = values
  while (cursor.tag === "Cons") {
    if (predicate(cursor.head)) {
      result.push(...transform(cursor.head))
    }
    cursor = cursor.tail
  }
  return result
}

function appendToArray<A>(values: List<A>, result: A[]): void {
  let cursor = values
  while (cursor.tag === "Cons") {
    result.push(cursor.head)
    cursor = cursor.tail
  }
}

export const listFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    (values: List<Value>): List<Result> => {
      const result: Result[] = []
      let cursor = values
      while (cursor.tag === "Cons") {
        result.push(f(cursor.head))
        cursor = cursor.tail
      }
      return fromArray(result)
    },
})

export const listApplicative = Object.freeze({
  ...listFunctor,
  pure: <Value>(value: Value): List<Value> => Cons(value, Empty),
  apply:
    <Value, Result>(functions: List<(value: Value) => Result>) =>
    (values: List<Value>): List<Result> => {
      const result: Result[] = []
      let functionCursor = functions
      while (functionCursor.tag === "Cons") {
        let valueCursor = values
        while (valueCursor.tag === "Cons") {
          result.push(functionCursor.head(valueCursor.head))
          valueCursor = valueCursor.tail
        }
        functionCursor = functionCursor.tail
      }
      return fromArray(result)
    },
})

export const listMonad = Object.freeze({
  ...listApplicative,
  flatMap:
    <Value, Result>(f: (value: Value) => List<Result>) =>
    (values: List<Value>): List<Result> => {
      const result: Result[] = []
      let cursor = values
      while (cursor.tag === "Cons") {
        appendToArray(f(cursor.head), result)
        cursor = cursor.tail
      }
      return fromArray(result)
    },
})

export const listTraversable = Object.freeze({
  ...listFunctor,
  traverse:
    <Value, Result>(f: (value: Value) => unknown) =>
    (values: List<Value>) =>
    (applicativeEvidence: RuntimeDictionary) =>
      traverseValues<Value, Result, List<Result>>(
        toArray(values),
        f,
        applicativeEvidence,
        fromArray
      ),
})

function fromNonEmptyArray<A>(values: ReadonlyArray<A>): NonEmptyList<A> {
  const head = values[0]
  if (head === undefined && values.length === 0) {
    throw new Error("NonEmptyList operation produced an empty result")
  }
  return NonEmptyList(head as A, fromArray(values.slice(1)))
}

function appendNonEmptyToArray<A>(values: NonEmptyList<A>, result: A[]): void {
  result.push(values.head)
  appendToArray(values.tail, result)
}

export const nonEmptyListFunctor = Object.freeze({
  map:
    <Value, Result>(f: (value: Value) => Result) =>
    (values: NonEmptyList<Value>): NonEmptyList<Result> =>
      NonEmptyList(f(values.head), listFunctor.map(f)(values.tail)),
})

export const nonEmptyListApplicative = Object.freeze({
  ...nonEmptyListFunctor,
  pure: <Value>(value: Value): NonEmptyList<Value> => singleton(value),
  apply:
    <Value, Result>(functions: NonEmptyList<(value: Value) => Result>) =>
    (values: NonEmptyList<Value>): NonEmptyList<Result> => {
      const result: Result[] = []
      let functionCursor = toListNonEmpty(functions)
      while (functionCursor.tag === "Cons") {
        let valueCursor = toListNonEmpty(values)
        while (valueCursor.tag === "Cons") {
          result.push(functionCursor.head(valueCursor.head))
          valueCursor = valueCursor.tail
        }
        functionCursor = functionCursor.tail
      }
      return fromNonEmptyArray(result)
    },
})

export const nonEmptyListMonad = Object.freeze({
  ...nonEmptyListApplicative,
  flatMap:
    <Value, Result>(f: (value: Value) => NonEmptyList<Result>) =>
    (values: NonEmptyList<Value>): NonEmptyList<Result> => {
      const result: Result[] = []
      let cursor = toListNonEmpty(values)
      while (cursor.tag === "Cons") {
        appendNonEmptyToArray(f(cursor.head), result)
        cursor = cursor.tail
      }
      return fromNonEmptyArray(result)
    },
})

export const nonEmptyListTraversable = Object.freeze({
  ...nonEmptyListFunctor,
  traverse:
    <Value, Result>(f: (value: Value) => unknown) =>
    (values: NonEmptyList<Value>) =>
    (applicativeEvidence: RuntimeDictionary) =>
      traverseValues<Value, Result, NonEmptyList<Result>>(
        toArray(toListNonEmpty(values)),
        f,
        applicativeEvidence,
        fromNonEmptyArray
      ),
})
