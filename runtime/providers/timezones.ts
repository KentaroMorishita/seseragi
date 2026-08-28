import {
  bundledTimeZoneDatabaseVersion,
  canonicalTimeZoneId,
} from "@seseragi/runtime/timezone-rules"
import {
  providerRuntimeAbi,
  type ProviderResult,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "@seseragi/runtime/provider-package"

const REQUIRED_VERSION = "2025b"

export type TimeZoneRuleDatabase = Readonly<{
  version: () => string
  canonicalize: (id: string) => string | undefined
}>

const bundledDatabase: TimeZoneRuleDatabase = Object.freeze({
  version: bundledTimeZoneDatabaseVersion,
  canonicalize: canonicalTimeZoneId,
})

export function createTimeZonesProvider(
  database: TimeZoneRuleDatabase = bundledDatabase
): ProviderPackageEntry {
  let version: string | undefined
  let unavailable: string | undefined
  try {
    version = database.version()
  } catch (cause) {
    unavailable = cause instanceof Error ? cause.message : String(cause)
  }
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime#timezones",
    service: "std/time::TimeZones",
    targets: ["bun-process", "browser"],
    operations: {
      async databaseVersion() {
        return success(version ?? "unavailable")
      },
      async loadTimeZone(value) {
        if (typeof value !== "string") {
          throw new TypeError("TimeZone ID must be a string")
        }
        if (version === undefined) {
          return failure({
            tag: "TimeZoneDatabaseUnavailable",
            value: unavailable ?? "bundled timezone database is unavailable",
          })
        }
        if (version !== REQUIRED_VERSION) {
          return failure({
            tag: "TimeZoneDatabaseVersionMismatch",
            value: { required: REQUIRED_VERSION, actual: version },
          })
        }
        const id = database.canonicalize(value)
        return id === undefined
          ? failure({ tag: "UnknownTimeZone", value })
          : success({ id, version })
      },
    },
  })
}

function success(value: unknown): ProviderResult {
  return { kind: "success", value }
}

function failure(value: unknown): ProviderResult {
  return { kind: "failure", failure: value }
}
