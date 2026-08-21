import { providerRuntimeAbi } from "../provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
} from "../provider-package"

export type BrowserStorageArea = "local" | "session"

export type BrowserStorageHost = Readonly<{
  get: (area: BrowserStorageArea, key: string) => string | null
  set: (area: BrowserStorageArea, key: string, value: string) => void
  remove: (area: BrowserStorageArea, key: string) => void
  clear: (area: BrowserStorageArea) => void
  keys: (area: BrowserStorageArea) => ReadonlyArray<string>
}>

export function createWindowStorageHost(
  windowHost?: Window
): BrowserStorageHost {
  const live = (): Window => {
    const selected = windowHost ?? globalThis.window
    if (selected === undefined) {
      throw new Error("browser storage host is unavailable")
    }
    return selected
  }
  const select = (area: BrowserStorageArea): Storage =>
    area === "local" ? live().localStorage : live().sessionStorage
  return Object.freeze({
    get: (area, key) => select(area).getItem(key),
    set: (area, key, value) => select(area).setItem(key, value),
    remove: (area, key) => select(area).removeItem(key),
    clear: (area) => select(area).clear(),
    keys: (area) => {
      const storage = select(area)
      const keys: string[] = []
      for (let index = 0; index < storage.length; index += 1) {
        const key = storage.key(index)
        if (key !== null) keys.push(key)
      }
      return Object.freeze(keys.sort())
    },
  })
}

export function createBrowserStorageProvider(
  host: BrowserStorageHost = createWindowStorageHost()
): ProviderPackageEntry {
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider: "seseragi/runtime-browser#storage",
    service: "std/web/storage::Storage",
    targets: ["browser"],
    operations: {
      async get(input) {
        const parsed = areaAndKey(input)
        if (parsed instanceof Error) return Promise.reject(parsed)
        try {
          return { kind: "success", value: host.get(parsed.area, parsed.key) }
        } catch (cause) {
          return {
            kind: "failure",
            failure: hostFailure(parsed.area, undefined, cause),
          }
        }
      },
      async set(input) {
        const parsed = areaKeyAndValue(input)
        if (parsed instanceof Error) return Promise.reject(parsed)
        try {
          host.set(parsed.area, parsed.key, parsed.value)
          return { kind: "success", value: undefined }
        } catch (cause) {
          return {
            kind: "failure",
            failure: hostFailure(parsed.area, parsed.key, cause),
          }
        }
      },
      async remove(input) {
        const parsed = areaAndKey(input)
        if (parsed instanceof Error) return Promise.reject(parsed)
        try {
          host.remove(parsed.area, parsed.key)
          return { kind: "success", value: undefined }
        } catch (cause) {
          return {
            kind: "failure",
            failure: hostFailure(parsed.area, undefined, cause),
          }
        }
      },
      async clear(input) {
        const area = storageArea(input)
        if (area instanceof Error) return Promise.reject(area)
        try {
          host.clear(area)
          return { kind: "success", value: undefined }
        } catch (cause) {
          return {
            kind: "failure",
            failure: hostFailure(area, undefined, cause),
          }
        }
      },
      async keys(input) {
        const area = storageArea(input)
        if (area instanceof Error) return Promise.reject(area)
        try {
          return { kind: "success", value: host.keys(area) }
        } catch (cause) {
          return {
            kind: "failure",
            failure: hostFailure(area, undefined, cause),
          }
        }
      },
    },
  })
}

function storageArea(value: unknown): BrowserStorageArea | TypeError {
  return value === "local" || value === "session"
    ? value
    : new TypeError("storage area must be local or session")
}

function areaAndKey(
  value: unknown
): Readonly<{ area: BrowserStorageArea; key: string }> | TypeError {
  if (
    typeof value !== "object" ||
    value === null ||
    !("area" in value) ||
    !("key" in value)
  ) {
    return new TypeError("storage input must contain area and key")
  }
  const area = storageArea(value.area)
  if (area instanceof Error) return area
  if (typeof value.key !== "string") {
    return new TypeError("storage key must be a string")
  }
  return Object.freeze({ area, key: value.key })
}

function areaKeyAndValue(
  input: unknown
):
  | Readonly<{ area: BrowserStorageArea; key: string; value: string }>
  | TypeError {
  const parsed = areaAndKey(input)
  if (parsed instanceof Error) return parsed
  if (
    typeof input !== "object" ||
    input === null ||
    !("value" in input) ||
    typeof input.value !== "string"
  ) {
    return new TypeError("storage value must be a string")
  }
  return Object.freeze({ ...parsed, value: input.value })
}

function hostFailure(
  area: BrowserStorageArea,
  key: string | undefined,
  cause: unknown
) {
  const name = cause instanceof Error ? cause.name : ""
  const message = cause instanceof Error ? cause.message : "storage failed"
  if (
    key !== undefined &&
    (name === "QuotaExceededError" || name === "NS_ERROR_DOM_QUOTA_REACHED")
  ) {
    return Object.freeze({
      tag: "StorageQuotaExceeded" as const,
      value: Object.freeze({ area, key, message }),
    })
  }
  return Object.freeze({
    tag:
      name === "SecurityError"
        ? ("StorageSecurityFailure" as const)
        : ("StorageUnavailable" as const),
    value: Object.freeze({ area, message }),
  })
}

export const provider = createBrowserStorageProvider()
