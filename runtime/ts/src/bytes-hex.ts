import type { Bytes } from "./bytes"
import { fromUint8Array } from "./bytes"
import type { Eq } from "./equality"
import { type Either, Left, Right } from "./sum"
import { utf8Width } from "./text-core"

export type HexDecodeError =
  | {
      readonly tag: "OddHexLength"
      readonly value: number
    }
  | {
      readonly tag: "InvalidHexDigit"
      readonly value: { readonly offset: number }
    }

export const OddHexLength = (value: number): HexDecodeError => ({
  tag: "OddHexLength",
  value,
})

export const InvalidHexDigit = (value: {
  readonly offset: number
}): HexDecodeError => ({ tag: "InvalidHexDigit", value })

const HEX_DIGITS = "0123456789abcdef"

export function encode(bytes: Bytes): string {
  const output = new Array<string>(bytes.length * 2)
  for (let index = 0; index < bytes.length; index += 1) {
    const value = bytes[index] as number
    output[index * 2] = HEX_DIGITS[value >>> 4] as string
    output[index * 2 + 1] = HEX_DIGITS[value & 0x0f] as string
  }
  return output.join("")
}

export function decode(text: string): Either<HexDecodeError, Bytes> {
  const length = utf8Length(text)
  if (length % 2 !== 0) return Left(OddHexLength(length))

  const output = new Uint8Array(length / 2)
  for (let index = 0; index < text.length; index += 2) {
    const high = hexValue(text.charCodeAt(index))
    if (high < 0) return Left(InvalidHexDigit({ offset: index }))
    const low = hexValue(text.charCodeAt(index + 1))
    if (low < 0) return Left(InvalidHexDigit({ offset: index + 1 }))
    output[index / 2] = (high << 4) | low
  }
  return Right(fromUint8Array(output))
}

function hexValue(code: number): number {
  if (code >= 0x30 && code <= 0x39) return code - 0x30
  if (code >= 0x41 && code <= 0x46) return code - 0x41 + 10
  if (code >= 0x61 && code <= 0x66) return code - 0x61 + 10
  return -1
}

function utf8Length(text: string): number {
  let length = 0
  for (const scalar of text) length += utf8Width(scalar.codePointAt(0)!)
  return length
}

export const hexDecodeErrorEq: Eq<HexDecodeError> = Object.freeze({
  eq:
    (left) =>
    (right): boolean =>
      left.tag === right.tag &&
      (left.tag === "OddHexLength"
        ? right.tag === "OddHexLength" && left.value === right.value
        : right.tag === "InvalidHexDigit" &&
          left.value.offset === right.value.offset),
})
