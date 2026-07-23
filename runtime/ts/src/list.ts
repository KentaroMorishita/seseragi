import type { Iterator as SeseragiIterator } from "./iterator"
import type { Unit } from "./effect"
import { Just, Nothing, type Maybe } from "./sum"

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

export const Empty: Empty = Object.freeze({ tag: "Empty" })

export function Cons<A>(head: A, tail: List<A>): List<A> {
  return Object.freeze({ tag: "Cons", head, tail })
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

/** Runtime dictionary for the standard `Semigroup<List<A>>` instance. */
export const listSemigroup = Object.freeze({
  append:
    <A>(left: List<A>) =>
    (right: List<A>): List<A> =>
      appendValues(left, right),
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

export function take<A>(count: bigint, values: List<A>): List<A> {
  if (count <= 0n) return Empty
  const result: A[] = []
  let remaining = count
  let cursor = values
  while (remaining > 0n && cursor.tag === "Cons") {
    result.push(cursor.head)
    remaining -= 1n
    cursor = cursor.tail
  }
  return fromArray(result)
}

export function drop<A>(count: bigint, values: List<A>): List<A> {
  if (count <= 0n) return values
  let remaining = count
  let cursor = values
  while (remaining > 0n && cursor.tag === "Cons") {
    remaining -= 1n
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

export function length<A>(values: List<A>): bigint {
  let result = 0n
  let cursor = values
  while (cursor.tag === "Cons") {
    result += 1n
    cursor = cursor.tail
  }
  return result
}

export function isEmpty<A>(values: List<A>): boolean {
  return values.tag === "Empty"
}

export function get<A>(index: bigint, values: List<A>) {
  if (index < 0n) return Nothing
  let remaining = index
  let cursor = values
  while (cursor.tag === "Cons") {
    if (remaining === 0n) return Just(cursor.head)
    remaining -= 1n
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
