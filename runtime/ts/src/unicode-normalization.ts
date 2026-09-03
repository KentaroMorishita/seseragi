import { stringFromPoints } from "./text-core"
import {
  CANONICAL_DECOMPOSITION,
  COMPATIBILITY_DECOMPOSITION,
  COMPOSITIONS,
} from "./unicode-data"
import { combiningClass } from "./unicode-properties"

function decompose(point: number, compatibility: boolean, output: number[]) {
  const syllable = point - 0xac00
  if (syllable >= 0 && syllable < 11172) {
    output.push(0x1100 + Math.floor(syllable / 588))
    output.push(0x1161 + Math.floor((syllable % 588) / 28))
    if (syllable % 28 !== 0) output.push(0x11a7 + (syllable % 28))
    return
  }
  const mapping =
    (compatibility ? COMPATIBILITY_DECOMPOSITION[point] : undefined) ??
    CANONICAL_DECOMPOSITION[point]
  if (mapping) {
    for (const child of mapping) decompose(child, compatibility, output)
  } else output.push(point)
}

/** Stable CCC ordering with at most 255 buckets, not quadratic insertion sort. */
function order(points: readonly number[]): number[] {
  const output: number[] = []
  const marks = new Map<number, number[]>()
  const flush = () => {
    for (const ccc of [...marks.keys()].sort((left, right) => left - right)) {
      for (const point of marks.get(ccc)!) output.push(point)
    }
    marks.clear()
  }
  for (const point of points) {
    const ccc = combiningClass(point)
    if (ccc === 0) {
      flush()
      output.push(point)
    } else {
      const bucket = marks.get(ccc)
      if (bucket) bucket.push(point)
      else marks.set(ccc, [point])
    }
  }
  flush()
  return output
}

function composite(first: number, second: number): number | undefined {
  if (
    first >= 0x1100 &&
    first < 0x1113 &&
    second >= 0x1161 &&
    second < 0x1176
  ) {
    return 0xac00 + (first - 0x1100) * 588 + (second - 0x1161) * 28
  }
  if (
    first >= 0xac00 &&
    first < 0xd7a4 &&
    (first - 0xac00) % 28 === 0 &&
    second > 0x11a7 &&
    second < 0x11c3
  )
    return first + second - 0x11a7
  return COMPOSITIONS[`${first},${second}`]
}

function compose(points: readonly number[]): number[] {
  const output: number[] = []
  let starter = -1
  let lastClass = 0
  for (const point of points) {
    const ccc = combiningClass(point)
    const combined =
      starter >= 0 && (lastClass === 0 || lastClass < ccc)
        ? composite(output[starter]!, point)
        : undefined
    if (combined !== undefined) {
      output[starter] = combined
    } else {
      if (ccc === 0) starter = output.length
      output.push(point)
      lastClass = ccc
    }
  }
  return output
}

export function normalizeText(
  form: "NFC" | "NFD" | "NFKC" | "NFKD",
  text: string
): string {
  const decomposed: number[] = []
  const compatibility = form === "NFKC" || form === "NFKD"
  for (const scalar of text)
    decompose(scalar.codePointAt(0)!, compatibility, decomposed)
  const ordered = order(decomposed)
  return stringFromPoints(
    form === "NFC" || form === "NFKC" ? compose(ordered) : ordered
  )
}
