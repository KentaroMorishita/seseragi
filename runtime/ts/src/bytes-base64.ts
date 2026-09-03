import type { Bytes } from "./bytes"
import { fromUint8Array } from "./bytes"
import type { Eq } from "./equality"
import { type Either, Left, Right } from "./sum"
import { utf8Width } from "./text-core"

export type Base64DecodeError =
  | {
      readonly tag: "InvalidBase64Length"
      readonly value: number
    }
  | {
      readonly tag: "InvalidBase64Digit"
      readonly value: { readonly offset: number }
    }
  | {
      readonly tag: "InvalidBase64Padding"
      readonly value: { readonly offset: number }
    }
  | {
      readonly tag: "NonCanonicalBase64Bits"
      readonly value: { readonly offset: number }
    }

export const InvalidBase64Length = (value: number): Base64DecodeError => ({
  tag: "InvalidBase64Length",
  value,
})

export const InvalidBase64Digit = (value: {
  readonly offset: number
}): Base64DecodeError => ({ tag: "InvalidBase64Digit", value })

export const InvalidBase64Padding = (value: {
  readonly offset: number
}): Base64DecodeError => ({ tag: "InvalidBase64Padding", value })

export const NonCanonicalBase64Bits = (value: {
  readonly offset: number
}): Base64DecodeError => ({ tag: "NonCanonicalBase64Bits", value })

const STANDARD_ALPHABET =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
const URL_ALPHABET =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"

export function encode(bytes: Bytes): string {
  return encodeWithAlphabet(bytes, STANDARD_ALPHABET, true)
}

export function encodeUrl(bytes: Bytes): string {
  return encodeWithAlphabet(bytes, URL_ALPHABET, false)
}

function encodeWithAlphabet(
  bytes: Bytes,
  alphabet: string,
  padded: boolean
): string {
  const output: string[] = []
  for (let index = 0; index < bytes.length; index += 3) {
    const first = bytes[index] as number
    const second = bytes[index + 1]
    const third = bytes[index + 2]
    output.push(
      alphabet[first >>> 2] as string,
      alphabet[((first & 0x03) << 4) | ((second ?? 0) >>> 4)] as string
    )
    if (second === undefined) {
      if (padded) output.push("=", "=")
      continue
    }
    output.push(
      alphabet[((second & 0x0f) << 2) | ((third ?? 0) >>> 6)] as string
    )
    if (third === undefined) {
      if (padded) output.push("=")
      continue
    }
    output.push(alphabet[third & 0x3f] as string)
  }
  return output.join("")
}

export function decode(text: string): Either<Base64DecodeError, Bytes> {
  const length = utf8Length(text)
  if (length % 4 !== 0) return Left(InvalidBase64Length(length))
  return decodeValidated(text, false)
}

export function decodeUrl(text: string): Either<Base64DecodeError, Bytes> {
  const length = utf8Length(text)
  if (length % 4 === 1) return Left(InvalidBase64Length(length))
  return decodeValidated(text, true)
}

function decodeValidated(
  text: string,
  urlSafe: boolean
): Either<Base64DecodeError, Bytes> {
  const sextets: number[] = []
  let firstPadding = -1

  for (let index = 0; index < text.length; index += 1) {
    const code = text.charCodeAt(index)
    if (code === 0x3d) {
      if (urlSafe) return Left(InvalidBase64Padding({ offset: index }))
      if (firstPadding < 0) firstPadding = index
      continue
    }

    const value = base64Value(code, urlSafe)
    if (value < 0) return Left(InvalidBase64Digit({ offset: index }))
    if (firstPadding >= 0) {
      return Left(InvalidBase64Padding({ offset: firstPadding }))
    }
    sextets.push(value)
  }

  if (!urlSafe) {
    const padding = text.length - sextets.length
    const expectedPadding =
      sextets.length % 4 === 2 ? 2 : sextets.length % 4 === 3 ? 1 : 0
    if (
      sextets.length % 4 === 1 ||
      padding > 2 ||
      padding !== expectedPadding
    ) {
      return Left(
        InvalidBase64Padding({
          offset: firstPadding < 0 ? text.length : firstPadding,
        })
      )
    }
  }

  const remainder = sextets.length % 4
  const lastOffset = sextets.length - 1
  if (remainder === 2 && ((sextets.at(-1) as number) & 0x0f) !== 0) {
    return Left(NonCanonicalBase64Bits({ offset: lastOffset }))
  }
  if (remainder === 3 && ((sextets.at(-1) as number) & 0x03) !== 0) {
    return Left(NonCanonicalBase64Bits({ offset: lastOffset }))
  }

  const output = new Uint8Array(Math.floor((sextets.length * 6) / 8))
  let bits = 0
  let buffer = 0
  let outputIndex = 0
  for (const sextet of sextets) {
    buffer = (buffer << 6) | sextet
    bits += 6
    if (bits < 8) continue
    bits -= 8
    output[outputIndex] = (buffer >>> bits) & 0xff
    outputIndex += 1
    buffer &= (1 << bits) - 1
  }
  return Right(fromUint8Array(output))
}

function base64Value(code: number, urlSafe: boolean): number {
  if (code >= 0x41 && code <= 0x5a) return code - 0x41
  if (code >= 0x61 && code <= 0x7a) return code - 0x61 + 26
  if (code >= 0x30 && code <= 0x39) return code - 0x30 + 52
  if (urlSafe && code === 0x2d) return 62
  if (urlSafe && code === 0x5f) return 63
  if (!urlSafe && code === 0x2b) return 62
  if (!urlSafe && code === 0x2f) return 63
  return -1
}

function utf8Length(text: string): number {
  let length = 0
  for (const scalar of text) length += utf8Width(scalar.codePointAt(0)!)
  return length
}

export const base64DecodeErrorEq: Eq<Base64DecodeError> = Object.freeze({
  eq:
    (left) =>
    (right): boolean => {
      if (left.tag !== right.tag) return false
      if (left.tag === "InvalidBase64Length") {
        return right.tag === "InvalidBase64Length" && left.value === right.value
      }
      return (
        right.tag !== "InvalidBase64Length" &&
        left.value.offset === right.value.offset
      )
    },
})
