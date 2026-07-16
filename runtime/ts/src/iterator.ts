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

/** Strict Array collection for an arbitrary user-defined Iterable dictionary. */
export function collectMap<A, B>(
  iterator: Iterator<A>,
  predicate: (value: A) => boolean,
  transform: (value: A) => B,
): ReadonlyArray<B> {
  return collect(iterator, predicate, (result, value) => {
    result.push(transform(value))
  })
}

/** Nested strict Array collection for an arbitrary user-defined Iterable dictionary. */
export function collectFlatMap<A, B>(
  iterator: Iterator<A>,
  predicate: (value: A) => boolean,
  transform: (value: A) => ReadonlyArray<B>,
): ReadonlyArray<B> {
  return collect(iterator, predicate, (result, value) => {
    result.push(...transform(value))
  })
}

function collect<A, B>(
  initial: Iterator<A>,
  predicate: (value: A) => boolean,
  append: (result: B[], value: A) => void,
): ReadonlyArray<B> {
  const result: B[] = []
  let iterator = initial
  while (true) {
    const step = iterator.next()
    if (step.tag === "Nothing") {
      return result
    }
    const [value, rest] = step.value
    if (predicate(value)) {
      append(result, value)
    }
    iterator = rest
  }
}
