import type { Unit } from "./effect"
import { Just, Left, Nothing, Right, type Either, type Maybe } from "./sum"

declare const byteBrand: unique symbol
declare const bytesBrand: unique symbol

/** Runtime representation of Seseragi's opaque unsigned Byte value. */
export type Byte = number & { readonly [byteBrand]: true }

/** Runtime representation of immutable Seseragi Bytes. */
export type Bytes = Uint8Array & { readonly [bytesBrand]: true }

export type ByteError = {
  readonly tag: "ByteOutOfRange"
  readonly value: number
}

export type BytesSliceError = {
  readonly tag: "InvalidByteRange"
  readonly value: {
    readonly start: number
    readonly end: number
    readonly length: number
  }
}

const asByte = (value: number): Byte => value as Byte
const asBytes = (value: Uint8Array): Bytes => value as Bytes

export const ByteOutOfRange = (value: number): ByteError => ({
  tag: "ByteOutOfRange",
  value,
})

export const InvalidByteRange = (value: {
  readonly start: number
  readonly end: number
  readonly length: number
}): BytesSliceError => ({ tag: "InvalidByteRange", value })

export function byte(value: number): Either<ByteError, Byte> {
  return !Number.isInteger(value) || value < 0 || value > 255
    ? Left(ByteOutOfRange(value))
    : Right(asByte(value))
}

export function toInt(value: Byte): number {
  return value
}

export function empty(_unit: Unit): Bytes {
  return asBytes(new Uint8Array())
}

export function singleton(value: Byte): Bytes {
  return asBytes(new Uint8Array([value]))
}

export function fromArray(values: ReadonlyArray<Byte>): Bytes {
  return asBytes(Uint8Array.from(values))
}

export function fromInts(
  values: ReadonlyArray<number>
): Either<ByteError, Bytes> {
  const result = new Uint8Array(values.length)
  for (let index = 0; index < values.length; index += 1) {
    const value = values[index] as number
    if (!Number.isInteger(value) || value < 0 || value > 255) {
      return Left(ByteOutOfRange(value))
    }
    result[index] = value
  }
  return Right(asBytes(result))
}

export function toArray(values: Bytes): ReadonlyArray<Byte> {
  return Array.from(values, asByte)
}

export function toInts(values: Bytes): ReadonlyArray<number> {
  return Array.from(values)
}

export function length(values: Bytes): number {
  return values.length
}

export function isEmpty(values: Bytes): boolean {
  return values.length === 0
}

export function get(index: number, values: Bytes): Maybe<Byte> {
  return index < 0 || index >= values.length
    ? Nothing
    : Just(asByte(values[index] as number))
}

export function slice(
  start: number,
  end: number,
  values: Bytes
): Either<BytesSliceError, Bytes> {
  if (start < 0 || start > end || end > values.length) {
    return Left(InvalidByteRange({ start, end, length: values.length }))
  }
  return Right(asBytes(values.subarray(start, end)))
}

export function copy(values: Bytes): Bytes {
  return asBytes(new Uint8Array(values))
}

export function append(suffix: Bytes, values: Bytes): Bytes {
  const result = new Uint8Array(values.length + suffix.length)
  result.set(values, 0)
  result.set(suffix, values.length)
  return asBytes(result)
}

export function concat(values: ReadonlyArray<Bytes>): Bytes {
  let resultLength = 0
  for (const value of values) resultLength += value.length
  const result = new Uint8Array(resultLength)
  let offset = 0
  for (const value of values) {
    result.set(value, offset)
    offset += value.length
  }
  return asBytes(result)
}

/** Copy a mutable TypeScript view into immutable Seseragi Bytes. */
export function fromUint8Array(values: Uint8Array): Bytes {
  return asBytes(new Uint8Array(values))
}

/** Copy immutable Seseragi Bytes into a mutable TypeScript view. */
export function toUint8Array(values: Bytes): Uint8Array {
  return new Uint8Array(values)
}
