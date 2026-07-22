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
