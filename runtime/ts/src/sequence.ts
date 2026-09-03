import type { RuntimeDictionary } from "./collection"
import type { Ordering } from "./sum"

export type Ord<A> = Readonly<{
  compare: (left: A) => (right: A) => Ordering
}>

/** Bottom-up stable merge sort with O(n log n) comparisons, independent of host sort. */
export function stableSort<A>(
  dictionary: Ord<A> | RuntimeDictionary,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  const ord = dictionary as Ord<A>
  let source = values.slice()
  let target = new Array<A>(source.length)
  for (let width = 1; width < source.length; width *= 2) {
    for (let start = 0; start < source.length; start += 2 * width) {
      const middle = Math.min(start + width, source.length)
      const end = Math.min(start + 2 * width, source.length)
      let left = start
      let right = middle
      for (let output = start; output < end; output += 1) {
        // Taking the left item on Equal preserves the original order.
        target[output] =
          left < middle &&
          (right >= end ||
            ord.compare(source[left] as A)(source[right] as A).tag !==
              "Greater")
            ? (source[left++] as A)
            : (source[right++] as A)
      }
    }
    ;[source, target] = [target, source]
  }
  return source
}

export function stableSortBy<A, K>(
  dictionary: Ord<K> | RuntimeDictionary,
  key: (value: A) => K,
  values: ReadonlyArray<A>
): ReadonlyArray<A> {
  const ord = dictionary as Ord<K>
  const keyed = values.map((value) => ({ value, key: key(value) }))
  const keyedOrd: Ord<{ value: A; key: K }> = {
    compare: (left) => (right) => ord.compare(left.key)(right.key),
  }
  return stableSort<{ value: A; key: K }>(keyedOrd, keyed).map(
    ({ value }) => value
  )
}
