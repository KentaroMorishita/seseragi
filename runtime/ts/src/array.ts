/** Runtime implementation of the standard `Reducible<Array<A>, A>` instance. */
export function reduce<A, B>(
  initial: B,
  step: (accumulator: B) => (value: A) => B,
  values: ReadonlyArray<A>,
): B {
  let accumulator = initial
  for (const value of values) {
    accumulator = step(accumulator)(value)
  }
  return accumulator
}

/** Pure comprehension lowering for the standard Array Iterable instance. */
export function collectMap<A, B>(
  values: ReadonlyArray<A>,
  predicate: (value: A) => boolean,
  transform: (value: A) => B,
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
  transform: (value: A) => ReadonlyArray<B>,
): ReadonlyArray<B> {
  const result: B[] = []
  for (const value of values) {
    if (predicate(value)) {
      result.push(...transform(value))
    }
  }
  return result
}
