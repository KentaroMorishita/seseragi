import type { Bytes } from "./bytes"
import { Left, Right, type Either } from "./sum"

export type Utf8DecodeError = {
  readonly tag: "InvalidUtf8"
  readonly value: { readonly offset: number }
}

export const InvalidUtf8 = (value: {
  readonly offset: number
}): Utf8DecodeError => ({ tag: "InvalidUtf8", value })

const encoder = new TextEncoder()
const strictDecoder = new TextDecoder("utf-8", { fatal: true })
const lossyDecoder = new TextDecoder("utf-8")

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
