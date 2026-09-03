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
