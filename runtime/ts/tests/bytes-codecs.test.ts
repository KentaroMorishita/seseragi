import { describe, expect, test } from "bun:test"
import {
  decode as decodeBase64,
  decodeUrl,
  encode as encodeBase64,
  encodeUrl,
  InvalidBase64Digit,
  InvalidBase64Length,
  InvalidBase64Padding,
  NonCanonicalBase64Bits,
  base64DecodeErrorEq,
} from "../src/bytes-base64"
import {
  decode as decodeHex,
  encode as encodeHex,
  hexDecodeErrorEq,
  InvalidHexDigit,
  OddHexLength,
} from "../src/bytes-hex"
import { fromUint8Array, type Bytes } from "../src/bytes"
import { Left } from "../src/sum"

const bytes = (...values: number[]): Bytes =>
  fromUint8Array(Uint8Array.from(values))

const decoded = <Error>(value: {
  readonly tag: "Left" | "Right"
  readonly value: Error | Bytes
}): number[] => {
  expect(value.tag).toBe("Right")
  return Array.from(value.value as Bytes)
}

describe("hex codec", () => {
  test("emits one lowercase canonical spelling and accepts uppercase input", () => {
    expect(encodeHex(bytes())).toBe("")
    expect(encodeHex(bytes(0, 1, 15, 16, 171, 255))).toBe("00010f10abff")
    expect(decoded(decodeHex("00010F10aBfF"))).toEqual([0, 1, 15, 16, 171, 255])
  })

  test("round-trips every byte value", () => {
    const value = bytes(...Array.from({ length: 256 }, (_, index) => index))
    expect(decoded(decodeHex(encodeHex(value)))).toEqual(Array.from(value))
  })

  test("reports length and the first invalid UTF-8 byte offset", () => {
    expect(decodeHex("0")).toEqual(Left(OddHexLength(1)))
    expect(decodeHex("é0")).toEqual(Left(OddHexLength(3)))
    expect(decodeHex("é")).toEqual(Left(InvalidHexDigit({ offset: 0 })))
    expect(decodeHex("00é")).toEqual(Left(InvalidHexDigit({ offset: 2 })))
    expect(decodeHex("00xz")).toEqual(Left(InvalidHexDigit({ offset: 2 })))
  })

  test("provides structural Eq for typed failures", () => {
    expect(hexDecodeErrorEq.eq(OddHexLength(3))(OddHexLength(3))).toBe(true)
    expect(
      hexDecodeErrorEq.eq(InvalidHexDigit({ offset: 2 }))(
        InvalidHexDigit({ offset: 2 })
      )
    ).toBe(true)
    expect(
      hexDecodeErrorEq.eq(InvalidHexDigit({ offset: 2 }))(
        InvalidHexDigit({ offset: 3 })
      )
    ).toBe(false)
  })
})

describe("base64 codec", () => {
  test("matches the RFC 4648 canonical vectors", () => {
    for (const [plain, encoded] of [
      ["", ""],
      ["f", "Zg=="],
      ["fo", "Zm8="],
      ["foo", "Zm9v"],
      ["foob", "Zm9vYg=="],
      ["fooba", "Zm9vYmE="],
      ["foobar", "Zm9vYmFy"],
    ]) {
      const value = bytes(...new TextEncoder().encode(plain))
      expect(encodeBase64(value)).toBe(encoded)
      expect(decoded(decodeBase64(encoded))).toEqual(Array.from(value))
    }
  })

  test("uses the unpadded URL-safe alphabet", () => {
    expect(encodeBase64(bytes(0xfb, 0xef, 0xff))).toBe("++//")
    expect(encodeUrl(bytes(0xfb, 0xef, 0xff))).toBe("--__")
    expect(encodeUrl(bytes(0xfb, 0xff))).toBe("-_8")
    expect(decoded(decodeUrl("-_8"))).toEqual([0xfb, 0xff])
  })

  test("round-trips deterministic inputs of every tail length", () => {
    for (let length = 0; length <= 64; length += 1) {
      const value = bytes(
        ...Array.from({ length }, (_, index) => (index * 67 + length) & 0xff)
      )
      expect(decoded(decodeBase64(encodeBase64(value)))).toEqual(
        Array.from(value)
      )
      expect(decoded(decodeUrl(encodeUrl(value)))).toEqual(Array.from(value))
    }
  })

  test("rejects noncanonical standard length, alphabet, and padding", () => {
    expect(decodeBase64("Zg")).toEqual(Left(InvalidBase64Length(2)))
    expect(decodeBase64("AA A")).toEqual(
      Left(InvalidBase64Digit({ offset: 2 }))
    )
    expect(decodeBase64("AA-A")).toEqual(
      Left(InvalidBase64Digit({ offset: 2 }))
    )
    expect(decodeBase64("AA=A")).toEqual(
      Left(InvalidBase64Padding({ offset: 2 }))
    )
    expect(decodeBase64("A===")).toEqual(
      Left(InvalidBase64Padding({ offset: 1 }))
    )
    expect(decodeBase64("AAAA====")).toEqual(
      Left(InvalidBase64Padding({ offset: 4 }))
    )
  })

  test("rejects URL padding, the standard alphabet, and invalid length", () => {
    expect(decodeUrl("A")).toEqual(Left(InvalidBase64Length(1)))
    expect(decodeUrl("AA=")).toEqual(Left(InvalidBase64Padding({ offset: 2 })))
    expect(decodeUrl("+A")).toEqual(Left(InvalidBase64Digit({ offset: 0 })))
    expect(decodeUrl("/w")).toEqual(Left(InvalidBase64Digit({ offset: 0 })))
  })

  test("rejects nonzero unused trailing bits at the responsible sextet", () => {
    expect(decodeBase64("AB==")).toEqual(
      Left(NonCanonicalBase64Bits({ offset: 1 }))
    )
    expect(decodeBase64("AAB=")).toEqual(
      Left(NonCanonicalBase64Bits({ offset: 2 }))
    )
    expect(decodeUrl("AB")).toEqual(Left(NonCanonicalBase64Bits({ offset: 1 })))
    expect(decodeUrl("AAB")).toEqual(
      Left(NonCanonicalBase64Bits({ offset: 2 }))
    )
  })

  test("reports the first invalid digit as a UTF-8 byte offset", () => {
    expect(decodeBase64("AAé")).toEqual(Left(InvalidBase64Digit({ offset: 2 })))
    expect(decodeUrl("éAA")).toEqual(Left(InvalidBase64Digit({ offset: 0 })))
  })

  test("provides structural Eq for every typed failure", () => {
    for (const error of [
      InvalidBase64Length(5),
      InvalidBase64Digit({ offset: 2 }),
      InvalidBase64Padding({ offset: 3 }),
      NonCanonicalBase64Bits({ offset: 1 }),
    ]) {
      expect(base64DecodeErrorEq.eq(error)(error)).toBe(true)
    }
    expect(
      base64DecodeErrorEq.eq(InvalidBase64Digit({ offset: 2 }))(
        InvalidBase64Padding({ offset: 2 })
      )
    ).toBe(false)
  })
})
