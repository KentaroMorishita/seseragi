import { add } from "./int64"

export type IntRange = Readonly<{
  start: bigint
  end: bigint
  inclusive: boolean
}>

export function exclusive(start: bigint, end: bigint): IntRange {
  return { start, end, inclusive: false }
}

export function inclusive(start: bigint, end: bigint): IntRange {
  return { start, end, inclusive: true }
}

/** Runtime implementation of the standard `Reducible<Range<Int>, Int>` instance. */
export function reduce<B>(
  initial: B,
  step: (accumulator: B) => (value: bigint) => B,
  range: IntRange,
): B {
  let accumulator = initial
  if (range.start > range.end) {
    return accumulator
  }

  let current = range.start
  while (range.inclusive ? current <= range.end : current < range.end) {
    accumulator = step(accumulator)(current)
    // Avoid incrementing past Int64::MAX after consuming an inclusive end.
    if (current === range.end) {
      break
    }
    current = add(current, 1n)
  }
  return accumulator
}

/** Pure comprehension lowering for the standard Range Iterable instance. */
export function collectMap<B>(
  range: IntRange,
  predicate: (value: bigint) => boolean,
  transform: (value: bigint) => B,
): ReadonlyArray<B> {
  return collect(range, predicate, (result, value) => {
    result.push(transform(value))
  })
}

/** Nested pure comprehension lowering for the standard Range Iterable instance. */
export function collectFlatMap<B>(
  range: IntRange,
  predicate: (value: bigint) => boolean,
  transform: (value: bigint) => ReadonlyArray<B>,
): ReadonlyArray<B> {
  return collect(range, predicate, (result, value) => {
    result.push(...transform(value))
  })
}

function collect<B>(
  range: IntRange,
  predicate: (value: bigint) => boolean,
  append: (result: B[], value: bigint) => void,
): ReadonlyArray<B> {
  const result: B[] = []
  if (range.start > range.end) {
    return result
  }
  let current = range.start
  while (range.inclusive ? current <= range.end : current < range.end) {
    if (predicate(current)) {
      append(result, current)
    }
    if (current === range.end) {
      break
    }
    current = add(current, 1n)
  }
  return result
}
