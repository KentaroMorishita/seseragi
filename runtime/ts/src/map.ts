import type {
  Iterable as CollectionIterable,
  RuntimeDictionary,
} from "./collection"
import type { Unit } from "./effect"
import type { Eq } from "./equality"
import { type Hash, mixHash, processHashSeed } from "./hash"
import type { Iterator } from "./iterator"
import {
  type Index,
  indexGet,
  indexRemove,
  indexSet,
  indexVacant,
} from "./persistent-index"
import { Just, type Maybe, Nothing } from "./sum"

type Entry<K, V> = Readonly<{
  key: K
  value: V
  hash: number
  previous: number | undefined
  next: number | undefined
}>

type State<K, V> = Readonly<{
  seed: number
  buckets: Index<ReadonlyArray<number>>
  order: Index<Entry<K, V>>
  first: number | undefined
  last: number | undefined
}>

const stateKey = Symbol("Seseragi.Map")

/** Opaque persistent insertion-order map, never a mutable host Map. */
export type Map<K, V> = Readonly<{ [stateKey]: State<K, V> }>

const wrap = <K, V>(state: State<K, V>): Map<K, V> =>
  Object.freeze({ [stateKey]: Object.freeze(state) })

const emptyWithSeed = <K, V>(seed: number): Map<K, V> =>
  wrap({
    seed,
    buckets: undefined,
    order: undefined,
    first: undefined,
    last: undefined,
  })

export const empty = <K, V>(_unit?: Unit): Map<K, V> =>
  emptyWithSeed(processHashSeed())

function entryAt<K, V>(state: State<K, V>, id: number): Entry<K, V> {
  const entry = indexGet(state.order, id)
  if (entry === undefined) throw new Error("invalid persistent Map order link")
  return entry
}

function find<K, V>(
  eq: Eq<K> | RuntimeDictionary,
  key: K,
  hash: number,
  state: State<K, V>
): number | undefined {
  for (const id of indexGet(state.buckets, hash) ?? []) {
    if ((eq as Eq<K>).eq(entryAt(state, id).key)(key)) return id
  }
  return undefined
}

/** Append a known-unique key using its cached, already mixed hash. */
function append<K, V>(
  key: K,
  value: V,
  hash: number,
  values: Map<K, V>
): Map<K, V> {
  const state = values[stateKey]
  const id = indexVacant(state.order)
  let order = state.order
  if (state.last !== undefined) {
    order = indexSet(
      order,
      state.last,
      Object.freeze({ ...entryAt(state, state.last), next: id })
    )
  }
  order = indexSet(
    order,
    id,
    Object.freeze({ key, value, hash, previous: state.last, next: undefined })
  )
  const bucket = Object.freeze([...(indexGet(state.buckets, hash) ?? []), id])
  return wrap({
    seed: state.seed,
    buckets: indexSet(state.buckets, hash, bucket),
    order,
    first: state.first ?? id,
    last: id,
  })
}

function replace<K, V>(id: number, value: V, values: Map<K, V>): Map<K, V> {
  const state = values[stateKey]
  return wrap({
    ...state,
    order: indexSet(
      state.order,
      id,
      Object.freeze({ ...entryAt(state, id), value })
    ),
  })
}

export function singleton<K, V>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  key: K,
  value: V
): Map<K, V> {
  return insert(eq, hash, key, value, empty())
}

export function get<K, V>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  key: K,
  values: Map<K, V>
): Maybe<V> {
  const state = values[stateKey]
  const id = find(
    eq,
    key,
    mixHash((hash as Hash<K>).hash(key), state.seed),
    state
  )
  return id === undefined ? Nothing : Just(entryAt(state, id).value)
}

export function containsKey<K, V>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  key: K,
  values: Map<K, V>
): boolean {
  return get(eq, hash, key, values).tag === "Just"
}

export function insert<K, V>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  key: K,
  value: V,
  values: Map<K, V>
): Map<K, V> {
  const state = values[stateKey]
  const mixed = mixHash((hash as Hash<K>).hash(key), state.seed)
  const id = find(eq, key, mixed, state)
  return id === undefined
    ? append(key, value, mixed, values)
    : replace(id, value, values)
}

export function upsert<K, V>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  key: K,
  update: (value: Maybe<V>) => V,
  values: Map<K, V>
): Map<K, V> {
  const state = values[stateKey]
  const mixed = mixHash((hash as Hash<K>).hash(key), state.seed)
  const id = find(eq, key, mixed, state)
  const value = update(
    id === undefined ? Nothing : Just(entryAt(state, id).value)
  )
  return id === undefined
    ? append(key, value, mixed, values)
    : replace(id, value, values)
}

export function remove<K, V>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  key: K,
  values: Map<K, V>
): Map<K, V> {
  const state = values[stateKey]
  const mixed = mixHash((hash as Hash<K>).hash(key), state.seed)
  const id = find(eq, key, mixed, state)
  if (id === undefined) return values
  const entry = entryAt(state, id)
  let order = indexRemove(state.order, id)
  if (entry.previous !== undefined) {
    order = indexSet(
      order,
      entry.previous,
      Object.freeze({ ...entryAt(state, entry.previous), next: entry.next })
    )
  }
  if (entry.next !== undefined) {
    order = indexSet(
      order,
      entry.next,
      Object.freeze({ ...entryAt(state, entry.next), previous: entry.previous })
    )
  }
  const bucket = (
    indexGet(state.buckets, mixed) as ReadonlyArray<number>
  ).filter((entryId) => entryId !== id)
  return wrap({
    seed: state.seed,
    buckets:
      bucket.length === 0
        ? indexRemove(state.buckets, mixed)
        : indexSet(state.buckets, mixed, Object.freeze(bucket)),
    order,
    first: state.first === id ? entry.next : state.first,
    last: state.last === id ? entry.previous : state.last,
  })
}

function* ordered<K, V>(values: Map<K, V>): Generator<Entry<K, V>> {
  const state = values[stateKey]
  let id = state.first
  while (id !== undefined) {
    const entry = entryAt(state, id)
    yield entry
    id = entry.next
  }
}

export function fromEntries<C, K, V>(
  iterable: CollectionIterable<C, readonly [K, V]> | RuntimeDictionary,
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  source: C
): Map<K, V> {
  let result = empty<K, V>()
  let iterator = (iterable as CollectionIterable<C, readonly [K, V]>).iterate(
    source
  )
  while (true) {
    const step = iterator.next()
    if (step.tag === "Nothing") return result
    const [[key, value], rest] = step.value
    result = insert(eq, hash, key, value, result)
    iterator = rest
  }
}

export function filter<K, V>(
  predicate: (key: K) => (value: V) => boolean,
  values: Map<K, V>
): Map<K, V> {
  const selected: Entry<K, V>[] = []
  for (const entry of ordered(values)) {
    if (predicate(entry.key)(entry.value)) selected.push(entry)
  }
  // Build collision buckets once, rather than copying an ever-growing bucket
  // per selected entry. filter is linear even when every user hash collides.
  const bucketEntries = new globalThis.Map<number, number[]>()
  let order: Index<Entry<K, V>>
  for (let id = 0; id < selected.length; id += 1) {
    const entry = selected[id] as Entry<K, V>
    const bucket = bucketEntries.get(entry.hash)
    if (bucket === undefined) bucketEntries.set(entry.hash, [id])
    else bucket.push(id)
    order = indexSet(
      order,
      id,
      Object.freeze({
        ...entry,
        previous: id === 0 ? undefined : id - 1,
        next: id + 1 === selected.length ? undefined : id + 1,
      })
    )
  }
  let buckets: Index<ReadonlyArray<number>>
  for (const [hash, ids] of bucketEntries)
    buckets = indexSet(buckets, hash, Object.freeze(ids))
  return wrap({
    seed: values[stateKey].seed,
    order,
    buckets,
    first: selected.length === 0 ? undefined : 0,
    last: selected.length === 0 ? undefined : selected.length - 1,
  })
}

export function mapValues<K, A, B>(
  f: (value: A) => B,
  values: Map<K, A>
): Map<K, B> {
  const state = values[stateKey]
  let order: Index<Entry<K, B>>
  let id = state.first
  while (id !== undefined) {
    const entry = entryAt(state, id)
    order = indexSet(
      order,
      id,
      Object.freeze({ ...entry, value: f(entry.value) })
    )
    id = entry.next
  }
  // Buckets contain only integer addresses, so sharing cannot retain old values.
  return wrap({ ...state, order })
}

export function mapKeysWith<K1, K2, V>(
  eq: Eq<K2> | RuntimeDictionary,
  hash: Hash<K2> | RuntimeDictionary,
  resolve: (current: V) => (incoming: V) => V,
  key: (key: K1) => K2,
  values: Map<K1, V>
): Map<K2, V> {
  let result = emptyWithSeed<K2, V>(values[stateKey].seed)
  for (const entry of ordered(values)) {
    result = upsert(
      eq,
      hash,
      key(entry.key),
      (current) =>
        current.tag === "Nothing"
          ? entry.value
          : resolve(current.value)(entry.value),
      result
    )
  }
  return result
}

export function mergeWith<K, V>(
  eq: Eq<K> | RuntimeDictionary,
  hash: Hash<K> | RuntimeDictionary,
  resolve: (left: V) => (right: V) => V,
  right: Map<K, V>,
  left: Map<K, V>
): Map<K, V> {
  let result = left
  for (const entry of ordered(right)) {
    result = upsert(
      eq,
      hash,
      entry.key,
      (current) =>
        current.tag === "Nothing"
          ? entry.value
          : resolve(current.value)(entry.value),
      result
    )
  }
  return result
}

export const keys = <K, V>(values: Map<K, V>): ReadonlyArray<K> =>
  Object.freeze(Array.from(ordered(values), (entry) => entry.key))

export const values = <K, V>(source: Map<K, V>): ReadonlyArray<V> =>
  Object.freeze(Array.from(ordered(source), (entry) => entry.value))

export const entries = <K, V>(
  values: Map<K, V>
): ReadonlyArray<readonly [K, V]> =>
  Object.freeze(
    Array.from(ordered(values), (entry) =>
      Object.freeze([entry.key, entry.value] as const)
    )
  )

export const size = <K, V>(values: Map<K, V>): number =>
  values[stateKey].order?.size ?? 0
export const isEmpty = <K, V>(values: Map<K, V>): boolean => size(values) === 0

export function iterate<K, V>(values: Map<K, V>): Iterator<readonly [K, V]> {
  const state = values[stateKey]
  const at = (id: number | undefined): Iterator<readonly [K, V]> =>
    Object.freeze({
      next: () => {
        if (id === undefined) return Nothing
        const entry = entryAt(state, id)
        return Just([
          Object.freeze([entry.key, entry.value] as const),
          at(entry.next),
        ] as const)
      },
    })
  return at(state.first)
}

export function reduce<K, V, A>(
  initial: A,
  step: (accumulator: A) => (entry: readonly [K, V]) => A,
  values: Map<K, V>
): A {
  let result = initial
  for (const entry of ordered(values))
    result = step(result)(Object.freeze([entry.key, entry.value] as const))
  return result
}

export const mapIterable = Object.freeze({ iterate })
export const mapReducible = Object.freeze({
  reduce:
    <A>(initial: A) =>
    <K, V>(step: (accumulator: A) => (entry: readonly [K, V]) => A) =>
    (values: Map<K, V>): A =>
      reduce(initial, step, values),
})
export const mapFunctor = Object.freeze({
  map:
    <A, B>(f: (value: A) => B) =>
    <K>(values: Map<K, A>): Map<K, B> =>
      mapValues(f, values),
})

export const mapEq = <K, V>(
  keyEq: Eq<K> | RuntimeDictionary,
  keyHash: Hash<K> | RuntimeDictionary,
  valueEq: Eq<V> | RuntimeDictionary
): Eq<Map<K, V>> =>
  Object.freeze({
    eq:
      (left: Map<K, V>) =>
      (right: Map<K, V>): boolean => {
        if (size(left) !== size(right)) return false
        for (const entry of ordered(left)) {
          const value = get(keyEq, keyHash, entry.key, right)
          if (
            value.tag === "Nothing" ||
            !(valueEq as Eq<V>).eq(entry.value)(value.value)
          )
            return false
        }
        return true
      },
  })
