import type { EffectContext } from "./effect"
import {
  createTimeZone,
  TimeZoneDatabaseUnavailable,
  TimeZoneDatabaseVersionMismatch,
  type TimeZone,
  type TimeZoneError,
  type TimeZones,
  UnknownTimeZone,
} from "./time"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import { serviceFailure, serviceSuccess } from "./service"

const unit = Object.freeze({ kind: "unit" } as const)
const never = Object.freeze({ kind: "never" } as const)
const string = Object.freeze({ kind: "primitive", name: "string" } as const)
const timeZone = Object.freeze({
  kind: "named",
  identity: "std/time::TimeZone",
} as const)
const timeZoneError = Object.freeze({
  kind: "named",
  identity: "std/time::TimeZoneError",
} as const)

const databaseVersionContract: ProviderOperationContract = Object.freeze({
  identity: "std/time::TimeZones#databaseVersion",
  kind: "one-shot",
  input: unit,
  success: string,
  failure: never,
})

const loadTimeZoneContract: ProviderOperationContract = Object.freeze({
  identity: "std/time::TimeZones#loadTimeZone",
  kind: "one-shot",
  input: string,
  success: timeZone,
  failure: timeZoneError,
})

const codecs = new ProviderCodecRegistry([
  {
    identity: timeZone.identity,
    encode: (value) => value,
    decode: (value) => {
      const record = providerRecord(value, "TimeZone")
      if (typeof record.id !== "string" || typeof record.version !== "string") {
        throw new TypeError("TimeZone provider value requires id and version")
      }
      return createTimeZone(record.id, record.version)
    },
  },
  {
    identity: timeZoneError.identity,
    encode: (value) => value,
    decode: decodeTimeZoneError,
  },
])

export function createProviderTimeZones(
  loaded: LoadedProviderEntry
): TimeZones {
  if (loaded.service !== "std/time::TimeZones") {
    throw new TypeError(
      "resolved provider does not implement std/time::TimeZones"
    )
  }
  const invoke = (
    operation: ProviderOperationContract,
    input: unknown,
    context: EffectContext
  ) =>
    invokeProviderOperation({
      provider: loaded.provider,
      service: loaded.service,
      operation,
      entry: loaded.entry,
      input,
      codecs,
      context,
    })
  return Object.freeze({
    async databaseVersion(context) {
      return success(
        await invoke(databaseVersionContract, undefined, context)
      ) as string
    },
    async loadTimeZone(id, context) {
      const outcome = await invoke(loadTimeZoneContract, id, context)
      return outcome.kind === "failure"
        ? serviceFailure(outcome.failure as TimeZoneError)
        : serviceSuccess(success(outcome) as TimeZone)
    },
  })
}

function decodeTimeZoneError(value: unknown): TimeZoneError {
  const record = providerRecord(value, "TimeZoneError")
  switch (record.tag) {
    case "UnknownTimeZone":
      if (typeof record.value !== "string") break
      return UnknownTimeZone(record.value)
    case "TimeZoneDatabaseUnavailable":
      if (typeof record.value !== "string") break
      return TimeZoneDatabaseUnavailable(record.value)
    case "TimeZoneDatabaseVersionMismatch": {
      const detail = providerRecord(record.value, "version mismatch")
      if (
        typeof detail.required !== "string" ||
        typeof detail.actual !== "string"
      ) {
        break
      }
      return TimeZoneDatabaseVersionMismatch({
        required: detail.required,
        actual: detail.actual,
      })
    }
  }
  throw new TypeError("TimeZone provider failure is invalid")
}

function providerRecord(
  value: unknown,
  description: string
): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError(`${description} provider value must be a record`)
  }
  return value as Record<string, unknown>
}

function success(outcome: ProviderBridgeOutcome): unknown {
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    throw new TypeError("TimeZones provider returned an unexpected failure")
  }
  return outcome.value
}
