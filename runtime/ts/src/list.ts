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

/** Runtime implementation of the standard `Reducible<List<A>, A>` instance. */
export function reduce<A, B>(
  initial: B,
  step: (accumulator: B) => (value: A) => B,
  values: List<A>,
): B {
  let accumulator = initial
  let cursor = values
  while (cursor.tag === "Cons") {
    accumulator = step(accumulator)(cursor.head)
    cursor = cursor.tail
  }
  return accumulator
}

/** Pure comprehension lowering for the standard List Iterable instance. */
export function collectMap<A, B>(
  values: List<A>,
  predicate: (value: A) => boolean,
  transform: (value: A) => B,
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
  transform: (value: A) => ReadonlyArray<B>,
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
