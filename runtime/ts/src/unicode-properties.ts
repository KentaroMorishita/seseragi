import {
  ALPHABETIC,
  CASE_IGNORABLE,
  CASED,
  COMBINING_CLASS,
  EXTENDED_PICTOGRAPHIC,
  GENERAL_CATEGORY,
  GRAPHEME_BREAK,
  INDIC_CONJUNCT_BREAK,
  WHITE_SPACE,
} from "./unicode-data"

/** Binary search over inclusive [start, end, value] triples from the pinned UCD. */
export function rangeValue(
  ranges: readonly number[],
  point: number,
  fallback = 0
): number {
  let low = 0
  let high = ranges.length / 3 - 1
  while (low <= high) {
    const middle = Math.floor((low + high) / 2)
    const offset = middle * 3
    if (point < ranges[offset]!) high = middle - 1
    else if (point > ranges[offset + 1]!) low = middle + 1
    else return ranges[offset + 2]!
  }
  return fallback
}

export const categoryIndex = (point: number): number =>
  rangeValue(GENERAL_CATEGORY, point, 28)
export const combiningClass = (point: number): number =>
  rangeValue(COMBINING_CLASS, point)
export const graphemeBreak = (point: number): number =>
  rangeValue(GRAPHEME_BREAK, point)
export const indicConjunctBreak = (point: number): number =>
  rangeValue(INDIC_CONJUNCT_BREAK, point)
export const alphabetic = (point: number): boolean =>
  rangeValue(ALPHABETIC, point) !== 0
export const whitespace = (point: number): boolean =>
  rangeValue(WHITE_SPACE, point) !== 0
export const cased = (point: number): boolean => rangeValue(CASED, point) !== 0
export const caseIgnorable = (point: number): boolean =>
  rangeValue(CASE_IGNORABLE, point) !== 0
export const extendedPictographic = (point: number): boolean =>
  rangeValue(EXTENDED_PICTOGRAPHIC, point) !== 0
