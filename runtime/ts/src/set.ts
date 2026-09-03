import type { Iterable as CollectionIterable } from "./collection"
import type { Unit } from "./effect"
import { type Eq, unitEq } from "./equality"
import type { Hash } from "./hash"
import type { Iterator } from "./iterator"
import { fromArray, type List } from "./list"
import * as maps from "./map"
import { Just, Nothing } from "./sum"

const valuesKey = Symbol("Seseragi.Set")
export type Set<A> = Readonly<{ [valuesKey]: maps.Map<A, Unit> }>
const wrap = <A>(values: maps.Map<A, Unit>): Set<A> =>
  Object.freeze({ [valuesKey]: values })

export const empty = <A>(_unit?: Unit): Set<A> => wrap(maps.empty())
export const singleton = <A>(eq: Eq<A>, hash: Hash<A>, value: A): Set<A> =>
  wrap(maps.singleton(eq, hash, value, undefined))

export function fromIterable<C, A>(
  iterable: CollectionIterable<C, A>,
  eq: Eq<A>,
  hash: Hash<A>,
  values: C
): Set<A> {
  let result = empty<A>()
  let iterator = iterable.iterate(values)
  while (true) {
    const step = iterator.next()
    if (step.tag === "Nothing") return result
    result = insert(eq, hash, step.value[0], result)
    iterator = step.value[1]
  }
}

export const contains = <A>(
  eq: Eq<A>,
  hash: Hash<A>,
  value: A,
  values: Set<A>
): boolean => maps.containsKey(eq, hash, value, values[valuesKey])

export const insert = <A>(
  eq: Eq<A>,
  hash: Hash<A>,
  value: A,
  values: Set<A>
): Set<A> => wrap(maps.insert(eq, hash, value, undefined, values[valuesKey]))

export const remove = <A>(
  eq: Eq<A>,
  hash: Hash<A>,
  value: A,
  values: Set<A>
): Set<A> => wrap(maps.remove(eq, hash, value, values[valuesKey]))

export const filter = <A>(
  predicate: (value: A) => boolean,
  values: Set<A>
): Set<A> => wrap(maps.filter((key) => () => predicate(key), values[valuesKey]))

// There is deliberately no Functor<Set> dictionary: mapping requires Eq/Hash
// of the output, and can collapse distinct inputs into one output element.
export const map = <A, B>(
  eq: Eq<B>,
  hash: Hash<B>,
  f: (value: A) => B,
  values: Set<A>
): Set<B> =>
  wrap(maps.mapKeysWith(eq, hash, () => () => undefined, f, values[valuesKey]))

export const union = <A>(
  eq: Eq<A>,
  hash: Hash<A>,
  right: Set<A>,
  left: Set<A>
): Set<A> =>
  wrap(
    maps.mergeWith(
      eq,
      hash,
      () => () => undefined,
      right[valuesKey],
      left[valuesKey]
    )
  )

export const intersection = <A>(
  eq: Eq<A>,
  hash: Hash<A>,
  right: Set<A>,
  left: Set<A>
): Set<A> => filter((value) => contains(eq, hash, value, right), left)

export const difference = <A>(
  eq: Eq<A>,
  hash: Hash<A>,
  removed: Set<A>,
  values: Set<A>
): Set<A> => filter((value) => !contains(eq, hash, value, removed), values)

export function isSubsetOf<A>(
  eq: Eq<A>,
  hash: Hash<A>,
  superset: Set<A>,
  values: Set<A>
): boolean {
  let iterator = iterate(values)
  while (true) {
    const step = iterator.next()
    if (step.tag === "Nothing") return true
    if (!contains(eq, hash, step.value[0], superset)) return false
    iterator = step.value[1]
  }
}

export const toArray = <A>(values: Set<A>): ReadonlyArray<A> =>
  maps.keys(values[valuesKey])
export const toList = <A>(values: Set<A>): List<A> => fromArray(toArray(values))
export const size = <A>(values: Set<A>): number => maps.size(values[valuesKey])
export const isEmpty = <A>(values: Set<A>): boolean =>
  maps.isEmpty(values[valuesKey])

export function iterate<A>(values: Set<A>): Iterator<A> {
  const stepper = (iterator: Iterator<readonly [A, Unit]>): Iterator<A> =>
    Object.freeze({
      next: () => {
        const step = iterator.next()
        return step.tag === "Nothing"
          ? Nothing
          : Just([step.value[0][0], stepper(step.value[1])] as const)
      },
    })
  return stepper(maps.iterate(values[valuesKey]))
}

export const setIterable = Object.freeze({ iterate })
export const setReducible = Object.freeze({
  reduce:
    <B>(initial: B) =>
    <A>(step: (accumulator: B) => (value: A) => B) =>
    (values: Set<A>): B =>
      maps.reduce(
        initial,
        (accumulator) =>
          ([key]) =>
            step(accumulator)(key),
        values[valuesKey]
      ),
})
export const setEq = <A>(eq: Eq<A>, hash: Hash<A>): Eq<Set<A>> =>
  Object.freeze({
    eq:
      (left: Set<A>) =>
      (right: Set<A>): boolean =>
        maps.mapEq(eq, hash, unitEq).eq(left[valuesKey])(right[valuesKey]),
  })
