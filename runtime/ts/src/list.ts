import type { Iterator as SeseragiIterator } from "./iterator"
import type { Unit } from "./effect"
import { Just, Nothing } from "./sum"

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
export function append<A>(left: List<A>, right: List<A>): List<A> {
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
      append(left, right),
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
