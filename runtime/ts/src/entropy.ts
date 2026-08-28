import { fromUint8Array, type Bytes } from "./bytes"
import type { Effect, EffectContext } from "./effect"
import {
  serviceEffect,
  type serviceFailure,
  type serviceSuccess,
} from "./service"
import { type Either, Left, Right } from "./sum"

declare const entropySizeBrand: unique symbol

export type EntropySize = number & { readonly [entropySizeBrand]: true }

export type EntropyConfigError =
  | Readonly<{ readonly tag: "NonPositiveEntropySize"; readonly value: number }>
  | Readonly<{ readonly tag: "EntropySizeTooLarge"; readonly value: number }>

export type EntropyError =
  | Readonly<{ readonly tag: "EntropyUnavailable" }>
  | Readonly<{ readonly tag: "EntropyReadFailure" }>

export const NonPositiveEntropySize = (value: number): EntropyConfigError => ({
  tag: "NonPositiveEntropySize",
  value,
})
export const EntropySizeTooLarge = (value: number): EntropyConfigError => ({
  tag: "EntropySizeTooLarge",
  value,
})
export const EntropyUnavailable: EntropyError = Object.freeze({
  tag: "EntropyUnavailable",
})
export const EntropyReadFailure: EntropyError = Object.freeze({
  tag: "EntropyReadFailure",
})

export type Entropy = Readonly<{
  secureBytes: (
    size: number,
    context: EffectContext
  ) => Promise<
    | ReturnType<typeof serviceSuccess<Bytes>>
    | ReturnType<typeof serviceFailure<EntropyError>>
  >
}>

export type EntropyEnvironment = Readonly<{ entropy: Entropy }>

const maximumSize = 1024 * 1024

export function entropySize(
  value: number
): Either<EntropyConfigError, EntropySize> {
  if (!Number.isSafeInteger(value) || value <= 0) {
    return Left(NonPositiveEntropySize(value))
  }
  return value > maximumSize
    ? Left(EntropySizeTooLarge(value))
    : Right(value as EntropySize)
}

export function secureBytes(
  size: EntropySize
): Effect<EntropyEnvironment, EntropyError, Bytes> {
  return serviceEffect((environment, context) =>
    environment.entropy.secureBytes(size, context)
  )
}

export function entropyConfigErrorShow() {
  return Object.freeze({
    show: (value: EntropyConfigError): string => `${value.tag} ${value.value}`,
  })
}

export function entropyErrorShow() {
  return Object.freeze({ show: (value: EntropyError): string => value.tag })
}

export function bytesFromProvider(value: unknown): Bytes {
  if (!(value instanceof Uint8Array)) {
    throw new TypeError("Entropy provider bytes must be a Uint8Array")
  }
  return fromUint8Array(value)
}
