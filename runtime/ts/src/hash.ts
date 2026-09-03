import type { Unit } from "./effect"
import { assertInt } from "./int"
import type { NonEmptyList } from "./list"

export type Hash<Value> = Readonly<{
  hash: (value: Value) => number
}>

const FNV_OFFSET = 0x811c9dc5
const FNV_PRIME = 0x01000193
const MAX_SEED_HIGH = 0x1fffff
const TWO_TO_32 = 0x1_0000_0000

/** Internal seed, distinct from the safe-integer user-level Hash result. */
export type HashSeed = number | bigint

type HashSeedGlobal = typeof globalThis & {
  __SESERAGI_HASH_SEED__?: HashSeed
  process?: Readonly<{
    env?: Readonly<Record<string, string | undefined>>
  }>
}

let cachedProcessSeed: HashSeed | undefined

const finishHash = (value: number): number => value | 0

const hashScalarSequence = (value: string): number => {
  let state = FNV_OFFSET
  for (const scalar of value) {
    state = Math.imul(state ^ (scalar.codePointAt(0) as number), FNV_PRIME)
  }
  return finishHash(state)
}

const requireScalar = (value: string): string => {
  const scalars = [...value]
  if (scalars.length !== 1) {
    throw new RangeError(
      "Char hash value must contain exactly one Unicode scalar"
    )
  }
  const scalar = scalars[0]
  const codePoint = scalar?.codePointAt(0)
  if (
    scalar === undefined ||
    codePoint === undefined ||
    (codePoint >= 0xd800 && codePoint <= 0xdfff)
  ) {
    throw new RangeError(
      "Char hash value must contain exactly one Unicode scalar"
    )
  }
  return scalar
}

/** Standard `Hash<Int>` dictionary. */
export const intHash: Hash<number> = Object.freeze({
  hash: (value: number): number => assertInt(value),
})

/** Standard `Hash<Bool>` dictionary. */
export const boolHash: Hash<boolean> = Object.freeze({
  hash: (value: boolean): number => (value ? 1 : 0),
})

/** Standard `Hash<Char>` dictionary. */
export const charHash: Hash<string> = Object.freeze({
  hash: (value: string): number => hashScalarSequence(requireScalar(value)),
})

/** Standard `Hash<String>` dictionary. */
export const stringHash: Hash<string> = Object.freeze({
  hash: hashScalarSequence,
})

/** Standard `Hash<Unit>` dictionary. */
export const unitHash: Hash<Unit> = Object.freeze({
  hash: (_value: Unit): number => 0,
})

/** Ordered structural hash for the standard `Hash<NonEmptyList<A>>` instance. */
export const nonEmptyListHash = <Value>(
  element: Hash<Value>
): Hash<NonEmptyList<Value>> =>
  Object.freeze({
    hash: (values: NonEmptyList<Value>): number => {
      let state = Math.imul(
        FNV_OFFSET ^ foldInt(element.hash(values.head)),
        FNV_PRIME
      )
      let cursor = values.tail
      while (cursor.tag === "Cons") {
        state = Math.imul(state ^ foldInt(element.hash(cursor.head)), FNV_PRIME)
        cursor = cursor.tail
      }
      return finishHash(state)
    },
  })

const foldInt = (value: number): number => {
  const bits = BigInt.asUintN(64, BigInt(assertInt(value)))
  return Number(BigInt.asUintN(32, bits ^ (bits >> 32n)))
}

/**
 * Mixes a pure user-level hash with a process-local seed for a hash index.
 * The result is backend-internal and must never be serialized or observed as
 * the value returned by `Hash.hash`.
 */
export function mixHash(hash: number, seed: HashSeed): number {
  const seedBits = BigInt.asUintN(64, BigInt(validateSeed(seed)))
  const foldedSeed = Number(BigInt.asUintN(32, seedBits ^ (seedBits >> 32n)))
  let state = Math.imul(FNV_OFFSET ^ foldedSeed, FNV_PRIME)
  state = Math.imul(state ^ foldInt(hash), FNV_PRIME)
  state ^= state >>> 16
  return state >>> 0
}

function validateSeed(seed: HashSeed): HashSeed {
  if (typeof seed === "number") return assertInt(seed)
  if (seed < -(1n << 63n) || seed >= 1n << 63n) {
    throw new RangeError("hash seed must be a signed 64-bit integer")
  }
  return seed
}

const fixedSeed = (): HashSeed | undefined => {
  const host = globalThis as HashSeedGlobal
  if (host.__SESERAGI_HASH_SEED__ !== undefined) {
    return validateSeed(host.__SESERAGI_HASH_SEED__)
  }
  const configured = host.process?.env?.SESERAGI_HASH_SEED
  if (configured === undefined) return undefined
  if (!/^[+-]?\d+$/.test(configured)) {
    throw new RangeError("SESERAGI_HASH_SEED must be a signed 64-bit integer")
  }
  const seed = validateSeed(BigInt(configured)) as bigint
  return seed >= BigInt(Number.MIN_SAFE_INTEGER) &&
    seed <= BigInt(Number.MAX_SAFE_INTEGER)
    ? Number(seed)
    : seed
}

const entropySeed = (): number => {
  const crypto = globalThis.crypto
  if (crypto === undefined || typeof crypto.getRandomValues !== "function") {
    throw new Error(
      "secure entropy is unavailable for the process-local hash seed"
    )
  }
  const words = new Uint32Array(2)
  crypto.getRandomValues(words)
  return (
    ((words[1] as number) & MAX_SEED_HIGH) * TWO_TO_32 + (words[0] as number)
  )
}

/** Resolves and caches the one process-local seed used by Map and Set indexes. */
export function processHashSeed(): HashSeed {
  if (cachedProcessSeed === undefined) {
    cachedProcessSeed = fixedSeed() ?? entropySeed()
  }
  return cachedProcessSeed
}

/** Canonical lookup adapter reserved for the persistent Map / Set runtime. */
export const hashIndex =
  <Value>(dictionary: Hash<Value>, seed = processHashSeed()) =>
  (value: Value): number =>
    mixHash(dictionary.hash(value), seed)

/** Test-only cache reset; production code must keep one seed per process. */
export function resetProcessHashSeedForTest(): void {
  cachedProcessSeed = undefined
}
