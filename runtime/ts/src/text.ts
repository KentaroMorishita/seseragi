import type { Bytes } from "./bytes"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"
import { copySubstring, utf8Width } from "./text-core"
import { literalMatches } from "./text-search"
import { fullFold, lowerCase, upperCase } from "./unicode-case"
import { whitespace } from "./unicode-properties"

export type TextSliceError = {
  readonly tag: "InvalidScalarRange"
  readonly value: {
    readonly start: number
    readonly end: number
    readonly length: number
  }
}

export const InvalidScalarRange = (
  value: TextSliceError["value"]
): TextSliceError => ({ tag: "InvalidScalarRange", value })

export const isEmpty = (text: string): boolean => text.length === 0

export const textSliceErrorEq = Object.freeze({
  eq: (left: TextSliceError) => (right: TextSliceError) =>
    left.value.start === right.value.start &&
    left.value.end === right.value.end &&
    left.value.length === right.value.length,
})

export function lengthScalars(text: string): number {
  let count = 0
  for (const _scalar of text) count++
  return count
}

export function lengthBytes(text: string): number {
  let count = 0
  for (const scalar of text) count += utf8Width(scalar.codePointAt(0)!)
  return count
}

export const concat = (texts: ReadonlyArray<string>): string => texts.join("")
export const join = (separator: string, texts: ReadonlyArray<string>): string =>
  texts.join(separator)

export function split(separator: string, text: string): ReadonlyArray<string> {
  if (separator === "")
    return Array.from(text, (scalar) =>
      String.fromCodePoint(scalar.codePointAt(0)!)
    )
  const output: string[] = []
  let previous = 0
  for (const index of literalMatches(separator, text)) {
    output.push(copySubstring(text, previous, index))
    previous = index + separator.length
  }
  output.push(copySubstring(text, previous, text.length))
  return output
}

export function lines(text: string): ReadonlyArray<string> {
  const output: string[] = []
  let previous = 0
  for (let index = 0; index < text.length; ) {
    const point = text.codePointAt(index)!
    let width = point > 0xffff ? 2 : 1
    if (
      point === 0x0d ||
      point === 0x0a ||
      point === 0x85 ||
      point === 0x2028 ||
      point === 0x2029
    ) {
      output.push(copySubstring(text, previous, index))
      if (point === 0x0d && text.charCodeAt(index + 1) === 0x0a) width++
      previous = index + width
    }
    index += width
  }
  if (previous < text.length)
    output.push(copySubstring(text, previous, text.length))
  return output
}

export function words(text: string): ReadonlyArray<string> {
  const output: string[] = []
  let start = -1,
    index = 0
  for (const scalar of text) {
    if (whitespace(scalar.codePointAt(0)!)) {
      if (start >= 0) output.push(copySubstring(text, start, index))
      start = -1
    } else if (start < 0) start = index
    index += scalar.length
  }
  if (start >= 0) output.push(copySubstring(text, start, index))
  return output
}

export function trimStart(text: string): string {
  let start = 0
  for (const scalar of text) {
    if (!whitespace(scalar.codePointAt(0)!)) break
    start += scalar.length
  }
  return copySubstring(text, start, text.length)
}

export function trimEnd(text: string): string {
  let end = 0,
    index = 0
  for (const scalar of text) {
    index += scalar.length
    if (!whitespace(scalar.codePointAt(0)!)) end = index
  }
  return copySubstring(text, 0, end)
}

export function trim(text: string): string {
  let start = -1,
    end = 0,
    index = 0
  for (const scalar of text) {
    if (!whitespace(scalar.codePointAt(0)!)) {
      if (start < 0) start = index
      end = index + scalar.length
    }
    index += scalar.length
  }
  return start < 0 ? "" : copySubstring(text, start, end)
}

export const startsWith = (prefix: string, text: string): boolean =>
  text.startsWith(prefix)
export const endsWith = (suffix: string, text: string): boolean =>
  text.endsWith(suffix)
export const contains = (needle: string, text: string): boolean =>
  !literalMatches(needle, text).next().done

export function replace(
  needle: string,
  replacement: string,
  text: string
): string {
  const found = literalMatches(needle, text).next()
  if (found.done) return text
  return [
    copySubstring(text, 0, found.value),
    replacement,
    copySubstring(text, found.value + needle.length, text.length),
  ].join("")
}

export function replaceAll(
  needle: string,
  replacement: string,
  text: string
): string {
  const output: string[] = []
  let previous = 0
  for (const index of literalMatches(needle, text)) {
    output.push(copySubstring(text, previous, index), replacement)
    previous = index + needle.length
  }
  output.push(copySubstring(text, previous, text.length))
  return output.join("")
}

export const toLower = lowerCase
export const toUpper = upperCase
export const caseFold = fullFold

export function scalarAt(index: number, text: string): Maybe<string> {
  if (!Number.isInteger(index) || index < 0) return Nothing
  let count = 0
  for (const scalar of text) {
    if (count === index)
      return Just(String.fromCodePoint(scalar.codePointAt(0)!))
    count++
  }
  return Nothing
}

export function sliceScalars(
  start: number,
  end: number,
  text: string
): Either<TextSliceError, string> {
  let size = 0,
    index = 0,
    first = 0,
    last = 0
  for (const scalar of text) {
    if (size === start) first = index
    if (size === end) last = index
    size++
    index += scalar.length
  }
  if (size === start) first = index
  if (size === end) last = index
  return !Number.isInteger(start) ||
    !Number.isInteger(end) ||
    start < 0 ||
    start > end ||
    end > size
    ? Left(InvalidScalarRange({ start, end, length: size }))
    : Right(copySubstring(text, first, last))
}

export type Utf8DecodeError = {
  readonly tag: "InvalidUtf8"
  readonly value: { readonly offset: number }
}

export const InvalidUtf8 = (value: {
  readonly offset: number
}): Utf8DecodeError => ({ tag: "InvalidUtf8", value })

const encoder = new TextEncoder()
const strictDecoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true })
const lossyDecoder = new TextDecoder("utf-8", { ignoreBOM: true })

export function encodeUtf8(text: string): Bytes {
  return encoder.encode(text) as Bytes
}

export function decodeUtf8(bytes: Bytes): Either<Utf8DecodeError, string> {
  const offset = firstInvalidUtf8Offset(bytes)
  return offset === undefined
    ? Right(strictDecoder.decode(bytes))
    : Left(InvalidUtf8({ offset }))
}

export function decodeUtf8Lossy(bytes: Bytes): string {
  return lossyDecoder.decode(bytes)
}

function firstInvalidUtf8Offset(bytes: Uint8Array): number | undefined {
  let index = 0
  while (index < bytes.length) {
    const first = bytes[index] as number
    if (first <= 0x7f) {
      index += 1
      continue
    }

    if (first >= 0xc2 && first <= 0xdf) {
      if (!continuation(bytes[index + 1])) return index
      index += 2
      continue
    }

    if (first >= 0xe0 && first <= 0xef) {
      const second = bytes[index + 1]
      const secondValid =
        first === 0xe0
          ? second !== undefined && second >= 0xa0 && second <= 0xbf
          : first === 0xed
            ? second !== undefined && second >= 0x80 && second <= 0x9f
            : continuation(second)
      if (!secondValid || !continuation(bytes[index + 2])) return index
      index += 3
      continue
    }

    if (first >= 0xf0 && first <= 0xf4) {
      const second = bytes[index + 1]
      const secondValid =
        first === 0xf0
          ? second !== undefined && second >= 0x90 && second <= 0xbf
          : first === 0xf4
            ? second !== undefined && second >= 0x80 && second <= 0x8f
            : continuation(second)
      if (
        !secondValid ||
        !continuation(bytes[index + 2]) ||
        !continuation(bytes[index + 3])
      ) {
        return index
      }
      index += 4
      continue
    }

    return index
  }
  return undefined
}

function continuation(value: number | undefined): boolean {
  return value !== undefined && value >= 0x80 && value <= 0xbf
}
