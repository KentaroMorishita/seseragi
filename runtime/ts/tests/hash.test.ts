import { afterEach, describe, expect, test } from "bun:test"
import {
  boolHash,
  charHash,
  hashIndex,
  intHash,
  mixHash,
  processHashSeed,
  resetProcessHashSeedForTest,
  stringHash,
  unitHash,
} from "../src/hash"

type HashSeedGlobal = typeof globalThis & {
  __SESERAGI_HASH_SEED__?: number
}

const host = globalThis as HashSeedGlobal

afterEach(() => {
  delete host.__SESERAGI_HASH_SEED__
  resetProcessHashSeedForTest()
})

describe("standard Hash dictionaries", () => {
  test("hash primitive values through pure user-visible dictionaries", () => {
    expect(intHash.hash(42)).toBe(42)
    expect(intHash.hash(-0)).toBe(0)
    expect(boolHash.hash(false)).toBe(0)
    expect(boolHash.hash(true)).toBe(1)
    expect(charHash.hash("瀬")).toBe(stringHash.hash("瀬"))
    expect(() => charHash.hash("ab")).toThrow("exactly one Unicode scalar")
    expect(stringHash.hash("せせらぎ")).toBe(stringHash.hash("せせらぎ"))
    expect(unitHash.hash(undefined)).toBe(0)
  })

  test("keeps Eq-equal primitive values hash-equal", () => {
    for (const [left, right] of [
      [42, 42],
      [0, -0],
    ] as const) {
      expect(intHash.hash(left)).toBe(intHash.hash(right))
    }
    expect(stringHash.hash("A")).toBe(stringHash.hash("A"))
    expect(boolHash.hash(false)).toBe(boolHash.hash(false))
    expect(charHash.hash("瀬")).toBe(charHash.hash("瀬"))
    expect(unitHash.hash(undefined)).toBe(unitHash.hash(undefined))
  })
})

describe("process-local hash indexing", () => {
  test("mixes the same pure hash differently without changing iteration", () => {
    const values = ["first", "second", "third"] as const
    const firstIndex = hashIndex(stringHash, 11)
    const secondIndex = hashIndex(stringHash, 29)

    expect(values.map(firstIndex)).not.toEqual(values.map(secondIndex))
    expect([...values]).toEqual(["first", "second", "third"])
    expect(stringHash.hash("first")).toBe(stringHash.hash("first"))
  })

  test("caches one fixed process seed and validates its Int boundary", () => {
    host.__SESERAGI_HASH_SEED__ = -7
    expect(processHashSeed()).toBe(-7)
    host.__SESERAGI_HASH_SEED__ = 99
    expect(processHashSeed()).toBe(-7)

    resetProcessHashSeedForTest()
    host.__SESERAGI_HASH_SEED__ = Number.MAX_SAFE_INTEGER + 1
    expect(() => processHashSeed()).toThrow("Seseragi Int overflow")
  })

  test("mixes deterministically for an explicit seed", () => {
    expect(mixHash(123, 456)).toBe(mixHash(123, 456))
    expect(mixHash(123, 456)).not.toBe(mixHash(123, 457))
  })
})
