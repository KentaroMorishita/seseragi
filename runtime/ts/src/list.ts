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
