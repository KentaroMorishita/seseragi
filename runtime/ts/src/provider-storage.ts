import type { EffectContext } from "./effect"
import {
  Local,
  Session,
  type Storage,
  type StorageArea,
  type StorageError,
  storageFailure,
  storageJust,
  storageNothing,
  storageSuccess,
} from "./storage"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderLogicalType,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"

const unit = Object.freeze({ kind: "unit" } as const)
const stringType = Object.freeze({ kind: "primitive", name: "string" } as const)
const areaType = Object.freeze({
  kind: "named",
  identity: "std/web/storage::StorageArea",
} as const)
const lookupType = Object.freeze({
  kind: "named",
  identity: "std/web/storage::StorageLookup",
} as const)
const storageErrorType = Object.freeze({
  kind: "named",
  identity: "std/web/storage::StorageError",
} as const)
const stringArrayType = Object.freeze({
  kind: "array",
  items: stringType,
} as const)

const record = (
  fields: ReadonlyArray<Readonly<{ name: string; type: ProviderLogicalType }>>
) => Object.freeze({ kind: "record", fields } as const)

const areaAndKeyType = record([
  Object.freeze({ name: "area", type: areaType }),
  Object.freeze({ name: "key", type: stringType }),
])
const setInputType = record([
  Object.freeze({ name: "area", type: areaType }),
  Object.freeze({ name: "key", type: stringType }),
  Object.freeze({ name: "value", type: stringType }),
])

const contract = (
  name: string,
  input: ProviderOperationContract["input"],
  success: ProviderOperationContract["success"]
): ProviderOperationContract =>
  Object.freeze({
    identity: `std/web/storage::Storage#${name}`,
    kind: "one-shot",
    input,
    success,
    failure: storageErrorType,
  })

const getContract = contract("get", areaAndKeyType, lookupType)
const setContract = contract("set", setInputType, unit)
const removeContract = contract("remove", areaAndKeyType, unit)
const clearContract = contract("clear", areaType, unit)
const keysContract = contract("keys", areaType, stringArrayType)

const codecs = new ProviderCodecRegistry([
  {
    identity: areaType.identity,
    encode: (value) => encodeArea(value as StorageArea),
    decode: (value) => decodeArea(value),
  },
  {
    identity: lookupType.identity,
    encode: (value) => value,
    decode: (value) => {
      if (value === null) return storageNothing
      if (typeof value === "string") return storageJust(value)
      throw new TypeError("storage lookup ABI value must be string or null")
    },
  },
  {
    identity: storageErrorType.identity,
    encode: (value) => value,
    decode: decodeStorageError,
  },
])

export function createProviderStorage(loaded: LoadedProviderEntry): Storage {
  if (loaded.service !== "std/web/storage::Storage") {
    throw new TypeError(
      "resolved provider does not implement std/web/storage::Storage"
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
    async get(area, key, context) {
      return outcome(await invoke(getContract, { area, key }, context))
    },
    async set(area, key, value, context) {
      return outcome(await invoke(setContract, { area, key, value }, context))
    },
    async remove(area, key, context) {
      return outcome(await invoke(removeContract, { area, key }, context))
    },
    async clear(area, context) {
      return outcome(await invoke(clearContract, area, context))
    },
    async keys(area, context) {
      return outcome(await invoke(keysContract, area, context))
    },
  })
}

function outcome<Success>(value: ProviderBridgeOutcome) {
  if (value.kind === "defect") throw value.defect
  return value.kind === "failure"
    ? storageFailure(value.failure as StorageError)
    : storageSuccess(value.value as Success)
}

function encodeArea(area: StorageArea): "local" | "session" {
  return area.tag === "Local" ? "local" : "session"
}

function decodeArea(value: unknown): StorageArea {
  if (value === "local") return Local
  if (value === "session") return Session
  throw new TypeError("storage area ABI value must be local or session")
}

function decodeStorageError(value: unknown): StorageError {
  if (typeof value !== "object" || value === null || !("tag" in value)) {
    throw new TypeError("storage error ABI value is invalid")
  }
  const error = value as { tag?: unknown; value?: unknown }
  if (
    typeof error.value !== "object" ||
    error.value === null ||
    !("area" in error.value) ||
    !("message" in error.value) ||
    typeof error.value.message !== "string"
  ) {
    throw new TypeError("storage error ABI value is invalid")
  }
  const area = decodeArea(error.value.area)
  if (
    error.tag === "StorageQuotaExceeded" &&
    "key" in error.value &&
    typeof error.value.key === "string"
  ) {
    return Object.freeze({
      tag: error.tag,
      value: Object.freeze({
        area,
        key: error.value.key,
        message: error.value.message,
      }),
    })
  }
  if (
    error.tag === "StorageSecurityFailure" ||
    error.tag === "StorageUnavailable"
  ) {
    return Object.freeze({
      tag: error.tag,
      value: Object.freeze({ area, message: error.value.message }),
    })
  }
  throw new TypeError("storage error ABI value is invalid")
}
