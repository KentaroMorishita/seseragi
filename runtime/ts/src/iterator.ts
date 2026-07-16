import { Just, Nothing, type Maybe } from "./sum"

/** Persistent pure pull iterator used by the Seseragi `Iterator<A>` ABI. */
export type Iterator<A> = Readonly<{
  next: () => Maybe<readonly [A, Iterator<A>]>
}>

/** Build an iterator without evaluating the first step eagerly. */
export function unfold<S, A>(
  step: (state: S) => Maybe<readonly [A, S]>,
  initial: S,
): Iterator<A> {
  return {
    next: () => {
      const result = step(initial)
      if (result.tag === "Nothing") {
        return Nothing
      }
      const [value, state] = result.value
      return Just([value, unfold(step, state)] as const)
    },
  }
}

/** Observe one step without consuming or mutating the original iterator. */
export function next<A>(
  iterator: Iterator<A>,
): Maybe<readonly [A, Iterator<A>]> {
  return iterator.next()
}
