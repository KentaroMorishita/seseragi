import type { Effect, EffectContext, Unit } from "./effect"
import type { ServiceResult } from "./service"
import { serviceEffect, serviceFailure, serviceSuccess } from "./service"
import { Just, type Maybe, Nothing } from "./sum"

export type StorageArea =
  | Readonly<{ readonly tag: "Local" }>
  | Readonly<{ readonly tag: "Session" }>

export const Local: StorageArea = Object.freeze({ tag: "Local" })
export const Session: StorageArea = Object.freeze({ tag: "Session" })

export type StorageError =
  | Readonly<{
      readonly tag: "StorageQuotaExceeded"
      readonly value: Readonly<{
        readonly area: StorageArea
        readonly key: string
        readonly message: string
      }>
    }>
  | Readonly<{
      readonly tag: "StorageSecurityFailure"
      readonly value: Readonly<{
        readonly area: StorageArea
        readonly message: string
      }>
    }>
  | Readonly<{
      readonly tag: "StorageUnavailable"
      readonly value: Readonly<{
        readonly area: StorageArea
        readonly message: string
      }>
    }>

export const StorageQuotaExceeded = (value: {
  area: StorageArea
  key: string
  message: string
}): StorageError =>
  Object.freeze({
    tag: "StorageQuotaExceeded",
    value: Object.freeze({ ...value }),
  })

export const StorageSecurityFailure = (value: {
  area: StorageArea
  message: string
}): StorageError =>
  Object.freeze({
    tag: "StorageSecurityFailure",
    value: Object.freeze({ ...value }),
  })

export const StorageUnavailable = (value: {
  area: StorageArea
  message: string
}): StorageError =>
  Object.freeze({
    tag: "StorageUnavailable",
    value: Object.freeze({ ...value }),
  })

export type Storage = Readonly<{
  get: (
    area: StorageArea,
    key: string,
    context: EffectContext
  ) => Promise<ServiceResult<StorageError, Maybe<string>>>
  set: (
    area: StorageArea,
    key: string,
    value: string,
    context: EffectContext
  ) => Promise<ServiceResult<StorageError, Unit>>
  remove: (
    area: StorageArea,
    key: string,
    context: EffectContext
  ) => Promise<ServiceResult<StorageError, Unit>>
  clear: (
    area: StorageArea,
    context: EffectContext
  ) => Promise<ServiceResult<StorageError, Unit>>
  keys: (
    area: StorageArea,
    context: EffectContext
  ) => Promise<ServiceResult<StorageError, ReadonlyArray<string>>>
}>

export type StorageEnvironment = Readonly<{ storage: Storage }>

export function get(
  area: StorageArea,
  key: string
): Effect<StorageEnvironment, StorageError, Maybe<string>> {
  return serviceEffect((environment, context) =>
    environment.storage.get(area, key, context)
  )
}

export function set(
  area: StorageArea,
  key: string,
  value: string
): Effect<StorageEnvironment, StorageError, Unit> {
  return serviceEffect((environment, context) =>
    environment.storage.set(area, key, value, context)
  )
}

export function remove(
  area: StorageArea,
  key: string
): Effect<StorageEnvironment, StorageError, Unit> {
  return serviceEffect((environment, context) =>
    environment.storage.remove(area, key, context)
  )
}

export function clear(
  area: StorageArea
): Effect<StorageEnvironment, StorageError, Unit> {
  return serviceEffect((environment, context) =>
    environment.storage.clear(area, context)
  )
}

export function keys(
  area: StorageArea
): Effect<StorageEnvironment, StorageError, ReadonlyArray<string>> {
  return serviceEffect((environment, context) =>
    environment.storage.keys(area, context)
  )
}

export function errorMessage(error: StorageError): string {
  return error.value.message
}

export const storageSuccess = serviceSuccess
export const storageFailure = serviceFailure
export const storageNothing = Nothing
export const storageJust = Just
