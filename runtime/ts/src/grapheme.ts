import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"
import { copySubstring, utf8Width } from "./text-core"
import {
  extendedPictographic,
  graphemeBreak,
  indicConjunctBreak,
} from "./unicode-properties"

export type GraphemeSliceError = {
  readonly tag: "InvalidGraphemeRange"
  readonly value: {
    readonly start: number
    readonly end: number
    readonly length: number
  }
}
export const InvalidGraphemeRange = (
  value: GraphemeSliceError["value"]
): GraphemeSliceError => ({ tag: "InvalidGraphemeRange", value })

export const graphemeSliceErrorEq = Object.freeze({
  eq: (left: GraphemeSliceError) => (right: GraphemeSliceError) =>
    left.value.start === right.value.start &&
    left.value.end === right.value.end &&
    left.value.length === right.value.length,
})

// Indices match the pinned Grapheme_Cluster_Break projection. Never use ICU.
const CR = 1,
  LF = 2,
  CONTROL = 3,
  EXTEND = 4,
  ZWJ = 5,
  RI = 6,
  PREPEND = 7,
  SPACING_MARK = 8,
  L = 9,
  V = 10,
  T = 11,
  LV = 12,
  LVT = 13
const control = (value: number) =>
  value === CR || value === LF || value === CONTROL

/** Streaming UAX #29 extended boundaries, including GB9c, GB11 and RI parity. */
export function* boundaries(
  text: string
): Generator<{ index: number; byte: number }> {
  let index = 0,
    byte = 0,
    previous = -1,
    regionalRun = 0,
    emoji = 0,
    indic = 0
  yield { index, byte }
  for (const scalar of text) {
    const point = scalar.codePointAt(0)!
    const current = graphemeBreak(point)
    const conjunct = indicConjunctBreak(point)
    const pictographic = extendedPictographic(point)
    let joined = false
    if (previous === CR && current === LF)
      joined = true // GB3
    else if (control(previous) || control(current))
      joined = false // GB4/5
    else if (previous === L && [L, V, LV, LVT].includes(current))
      joined = true // GB6
    else if ([LV, V].includes(previous) && [V, T].includes(current))
      joined = true // GB7
    else if ([LVT, T].includes(previous) && current === T)
      joined = true // GB8
    else if (current === EXTEND || current === ZWJ || current === SPACING_MARK)
      joined = true // GB9/9a
    else if (previous === PREPEND)
      joined = true // GB9b
    else if (conjunct === 1 && indic === 2)
      joined = true // GB9c
    else if (pictographic && emoji === 2)
      joined = true // GB11
    else if (previous === RI && current === RI && regionalRun % 2 === 1)
      joined = true // GB12/13
    if (index !== 0 && !joined) yield { index, byte }

    regionalRun = current === RI ? regionalRun + 1 : 0
    if (pictographic) emoji = 1
    else if (current === EXTEND && emoji === 1) {
      /* EP Extend* */
    } else if (current === ZWJ && emoji === 1) emoji = 2
    else emoji = 0
    if (conjunct === 1) indic = 1
    else if (conjunct === 3 && indic !== 0) indic = 2
    else if (conjunct !== 2) indic = 0
    previous = current
    index += scalar.length
    byte += utf8Width(point)
  }
  if (index !== 0) yield { index, byte }
}

export function length(text: string): number {
  let result = -1
  for (const _boundary of boundaries(text)) result++
  return result
}

export function clusters(text: string): ReadonlyArray<string> {
  const output: string[] = []
  let previous = 0
  for (const { index } of boundaries(text)) {
    if (index !== 0) output.push(copySubstring(text, previous, index))
    previous = index
  }
  return output
}

export function byteBoundaries(text: string): ReadonlyArray<number> {
  return Array.from(boundaries(text), (boundary) => boundary.byte)
}

export function at(index: number, text: string): Maybe<string> {
  if (!Number.isInteger(index) || index < 0) return Nothing
  let cluster = -1,
    previous = 0
  for (const boundary of boundaries(text)) {
    if (cluster === index)
      return Just(copySubstring(text, previous, boundary.index))
    previous = boundary.index
    cluster++
  }
  return Nothing
}

export function slice(
  start: number,
  end: number,
  text: string
): Either<GraphemeSliceError, string> {
  let size = -1,
    first = 0,
    last = 0
  for (const boundary of boundaries(text)) {
    size++
    if (size === start) first = boundary.index
    if (size === end) last = boundary.index
  }
  return !Number.isInteger(start) ||
    !Number.isInteger(end) ||
    start < 0 ||
    start > end ||
    end > size
    ? Left(InvalidGraphemeRange({ start, end, length: size }))
    : Right(copySubstring(text, first, last))
}
