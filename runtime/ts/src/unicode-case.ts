import { stringFromPoints } from "./text-core"
import {
  FINAL_SIGMA,
  FULL_FOLD,
  LOWER,
  SIMPLE_FOLD,
  UPPER,
} from "./unicode-data"
import { cased, caseIgnorable } from "./unicode-properties"

export function simpleFold(scalar: string): string {
  const point = scalar.codePointAt(0)!
  return String.fromCodePoint(SIMPLE_FOLD[point]?.[0] ?? point)
}

const foldEquivalenceCache = new Map<number, readonly number[]>()
let foldAdjacency: ReadonlyMap<number, readonly number[]> | undefined

/** Equivalence class induced by the pinned default simple-case-fold table. */
export function simpleFoldEquivalents(point: number): readonly number[] {
  const cached = foldEquivalenceCache.get(point)
  if (cached !== undefined) return cached
  if (foldAdjacency === undefined) {
    const mutable = new Map<number, Set<number>>()
    const connect = (left: number, right: number): void => {
      const neighbors = mutable.get(left) ?? new Set<number>()
      neighbors.add(right)
      mutable.set(left, neighbors)
    }
    for (const [source, mapping] of Object.entries(SIMPLE_FOLD)) {
      const from = Number(source)
      const to = mapping[0]!
      connect(from, to)
      connect(to, from)
    }
    foldAdjacency = new Map(
      [...mutable].map(([value, neighbors]) => [
        value,
        Object.freeze([...neighbors]),
      ])
    )
  }
  const seen = new Set<number>([point])
  const pending = [point]
  while (pending.length > 0) {
    for (const neighbor of foldAdjacency.get(pending.pop()!) ?? []) {
      if (!seen.has(neighbor)) {
        seen.add(neighbor)
        pending.push(neighbor)
      }
    }
  }
  const result = Object.freeze([...seen])
  for (const member of result) foldEquivalenceCache.set(member, result)
  return result
}

function mappedText(
  text: string,
  mapping: Readonly<Record<number, readonly number[]>>
): string {
  const output: number[] = []
  for (const scalar of text) {
    const point = scalar.codePointAt(0)!
    for (const mapped of mapping[point] ?? [point]) output.push(mapped)
  }
  return stringFromPoints(output)
}

export const fullFold = (text: string): string => mappedText(text, FULL_FOLD)
export const upperCase = (text: string): string => mappedText(text, UPPER)

export function lowerCase(text: string): string {
  const points = Array.from(text, (scalar) => scalar.codePointAt(0)!)
  // Default casing has one locale-independent context rule: Final_Sigma.
  // Precompute the suffix context to avoid repeatedly scanning ignorable runs.
  const followedByCased = new Uint8Array(points.length)
  let following = false
  for (let index = points.length - 1; index >= 0; index--) {
    followedByCased[index] = following ? 1 : 0
    if (!caseIgnorable(points[index]!)) following = cased(points[index]!)
  }
  const output: number[] = []
  let precededByCased = false
  for (let index = 0; index < points.length; index++) {
    const point = points[index]!
    const mapping =
      precededByCased && followedByCased[index] === 0
        ? (FINAL_SIGMA[point] ?? LOWER[point])
        : LOWER[point]
    for (const mapped of mapping ?? [point]) output.push(mapped)
    if (!caseIgnorable(point)) precededByCased = cased(point)
  }
  return stringFromPoints(output)
}
