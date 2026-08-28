import { fromUint8Array, type Bytes } from "./bytes"
import type { Effect, EffectContext } from "./effect"
import type { NonEmptyList } from "./list"
import { serviceEffect, type serviceFailure, serviceSuccess } from "./service"
import { type Either, Left, Right } from "./sum"

declare const randomSizeBrand: unique symbol

export type RandomSize = number & { readonly [randomSizeBrand]: true }

export type RandomRangeError =
  | Readonly<{
      readonly tag: "EmptyRandomIntRange"
      readonly value: Readonly<{ lower: number; upperExclusive: number }>
    }>
  | Readonly<{ readonly tag: "InvalidProbability"; readonly value: number }>

export type RandomConfigError =
  | Readonly<{ readonly tag: "NonPositiveRandomSize"; readonly value: number }>
  | Readonly<{ readonly tag: "RandomSizeTooLarge"; readonly value: number }>

export const EmptyRandomIntRange = (value: {
  readonly lower: number
  readonly upperExclusive: number
}): RandomRangeError => ({ tag: "EmptyRandomIntRange", value })

export const InvalidProbability = (value: number): RandomRangeError => ({
  tag: "InvalidProbability",
  value,
})

export const NonPositiveRandomSize = (value: number): RandomConfigError => ({
  tag: "NonPositiveRandomSize",
  value,
})

export const RandomSizeTooLarge = (value: number): RandomConfigError => ({
  tag: "RandomSizeTooLarge",
  value,
})

export type Random = Readonly<{
  algorithmId: (context: EffectContext) => Promise<string>
  nextBool: (context: EffectContext) => Promise<boolean>
  nextInt: (context: EffectContext) => Promise<number>
  intBetween: (
    lower: number,
    upperExclusive: number,
    context: EffectContext
  ) => Promise<
    | ReturnType<typeof serviceSuccess<number>>
    | ReturnType<typeof serviceFailure<RandomRangeError>>
  >
  unitFloat: (context: EffectContext) => Promise<number>
  chance: (
    probability: number,
    context: EffectContext
  ) => Promise<
    | ReturnType<typeof serviceSuccess<boolean>>
    | ReturnType<typeof serviceFailure<RandomRangeError>>
  >
  randomBytes: (size: number, context: EffectContext) => Promise<Bytes>
  chooseIndex: (length: number, context: EffectContext) => Promise<number>
  shuffleIndices: (
    length: number,
    context: EffectContext
  ) => Promise<ReadonlyArray<number>>
}>

export type RandomEnvironment = Readonly<{ random: Random }>

const maximumSize = 1024 * 1024

export function randomSize(
  value: number
): Either<RandomConfigError, RandomSize> {
  if (!Number.isSafeInteger(value) || value <= 0) {
    return Left(NonPositiveRandomSize(value))
  }
  return value > maximumSize
    ? Left(RandomSizeTooLarge(value))
    : Right(value as RandomSize)
}

export function algorithmId(): Effect<RandomEnvironment, never, string> {
  return serviceEffect(async (environment, context) =>
    serviceSuccess(await environment.random.algorithmId(context))
  )
}

export function nextBool(): Effect<RandomEnvironment, never, boolean> {
  return serviceEffect(async (environment, context) =>
    serviceSuccess(await environment.random.nextBool(context))
  )
}

export function nextInt(): Effect<RandomEnvironment, never, number> {
  return serviceEffect(async (environment, context) =>
    serviceSuccess(await environment.random.nextInt(context))
  )
}

export function intBetween(
  lower: number,
  upperExclusive: number
): Effect<RandomEnvironment, RandomRangeError, number> {
  return serviceEffect((environment, context) =>
    environment.random.intBetween(lower, upperExclusive, context)
  )
}

export function unitFloat(): Effect<RandomEnvironment, never, number> {
  return serviceEffect(async (environment, context) =>
    serviceSuccess(await environment.random.unitFloat(context))
  )
}

export function chance(
  probability: number
): Effect<RandomEnvironment, RandomRangeError, boolean> {
  return serviceEffect((environment, context) =>
    environment.random.chance(probability, context)
  )
}

export function randomBytes(
  size: RandomSize
): Effect<RandomEnvironment, never, Bytes> {
  return serviceEffect(async (environment, context) =>
    serviceSuccess(await environment.random.randomBytes(size, context))
  )
}

export function choose<Value>(
  values: NonEmptyList<Value>
): Effect<RandomEnvironment, never, Value> {
  return serviceEffect(async (environment, context) => {
    const entries = [values.head]
    let cursor = values.tail
    while (cursor.tag === "Cons") {
      entries.push(cursor.head)
      cursor = cursor.tail
    }
    const index = await environment.random.chooseIndex(entries.length, context)
    return serviceSuccess(entries[index] as Value)
  })
}

export function shuffle<Value>(
  values: ReadonlyArray<Value>
): Effect<RandomEnvironment, never, ReadonlyArray<Value>> {
  return serviceEffect(async (environment, context) => {
    const indices = await environment.random.shuffleIndices(
      values.length,
      context
    )
    return serviceSuccess(indices.map((index) => values[index] as Value))
  })
}

export function randomRangeErrorShow() {
  return Object.freeze({
    show: (value: RandomRangeError): string =>
      value.tag === "InvalidProbability"
        ? `InvalidProbability ${value.value}`
        : `EmptyRandomIntRange { lower: ${value.value.lower}, upperExclusive: ${value.value.upperExclusive} }`,
  })
}

export function randomConfigErrorShow() {
  return Object.freeze({
    show: (value: RandomConfigError): string => `${value.tag} ${value.value}`,
  })
}

export function bytesFromProvider(value: unknown): Bytes {
  if (!(value instanceof Uint8Array)) {
    throw new TypeError("Random provider bytes must be a Uint8Array")
  }
  return fromUint8Array(value)
}
