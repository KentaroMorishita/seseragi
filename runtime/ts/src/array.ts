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
