import { type EffectContext, throwIfCancelled, type Unit } from "./effect"
import {
  type FileHandle,
  type FilePath,
  type FileSystem,
  type FileSystemError,
  type FileSystemOperation,
  filePath,
  fileSystemFailure,
  fileSystemSuccess,
  renderFilePath,
} from "./filesystem"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import type { ServiceResult } from "./service"

const pathType = Object.freeze({
  kind: "named",
  identity: "std/path::Path",
} as const)
const handleType = Object.freeze({
  kind: "named",
  identity: "std/fs::FileHandle",
} as const)
const errorType = Object.freeze({
  kind: "named",
  identity: "std/fs::FileError",
} as const)
const unit = Object.freeze({ kind: "unit" } as const)
const bytes = Object.freeze({ kind: "primitive", name: "bytes" } as const)
const int = Object.freeze({ kind: "primitive", name: "int" } as const)
const openContract: ProviderOperationContract = Object.freeze({
  identity: "std/fs::FileSystem#openRead",
  kind: "resource",
  input: Object.freeze({
    kind: "record",
    fields: Object.freeze([{ name: "path", type: pathType }]),
  }),
  success: handleType,
  failure: errorType,
})
const readContract: ProviderOperationContract = Object.freeze({
  identity: "std/fs::FileSystem#read",
  kind: "one-shot",
  input: Object.freeze({
    kind: "record",
    fields: Object.freeze([
      { name: "handle", type: handleType },
      { name: "limit", type: int },
    ]),
  }),
  success: bytes,
  failure: errorType,
})
const closeContract: ProviderOperationContract = Object.freeze({
  identity: "std/fs::FileSystem#close",
  kind: "one-shot",
  input: handleType,
  success: unit,
  failure: errorType,
})
const codecs = new ProviderCodecRegistry([
  {
    identity: pathType.identity,
    encode: (value) => renderFilePath(value as FilePath),
    decode: (value) => filePath(stringValue(value, "filesystem path")),
  },
  {
    identity: errorType.identity,
    encode: (value) => value,
    decode: decodeFileSystemError,
  },
])

type HandleState = {
  readonly handle: FileHandle
  readonly loaded: LoadedProviderEntry
  unregisterCleanup: () => void
  closeCompletion?: Promise<ServiceResult<FileSystemError, Unit>>
}

export function createProviderFileSystem(
  loaded: LoadedProviderEntry
): FileSystem {
  if (loaded.service !== "std/fs::FileSystem") {
    throw new TypeError(
      "resolved provider does not implement std/fs::FileSystem"
    )
  }
  const handles = new WeakMap<object, HandleState>()
  return Object.freeze({
    async openRead(path: FilePath, context: EffectContext) {
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: openContract,
        entry: loaded.entry,
        input: { path },
        codecs,
        context,
      })
      if (outcome.kind === "defect") throw outcome.defect
      if (outcome.kind === "failure") {
        return fileSystemFailure(outcome.failure as FileSystemError)
      }
      const handle = outcome.value as FileHandle
      const state: HandleState = {
        handle,
        loaded,
        unregisterCleanup: () => undefined,
      }
      handles.set(handle, state)
      state.unregisterCleanup = context.onCancel(async () => {
        const result = await closeHandle(state)
        if (result.kind === "failure") {
          throw new Error(`filesystem cleanup failed: ${result.error.message}`)
        }
      })
      throwIfCancelled(context)
      return fileSystemSuccess(handle)
    },
    async read(handle: FileHandle, limit: number, context: EffectContext) {
      const state = handles.get(handle)
      if (state?.closeCompletion !== undefined) {
        throw new TypeError("filesystem resource is closed")
      }
      const outcome = await invokeProviderOperation({
        provider: loaded.provider,
        service: loaded.service,
        operation: readContract,
        entry: loaded.entry,
        input: { handle, limit },
        codecs,
        context,
      })
      throwIfCancelled(context)
      return operationResult<Uint8Array>(outcome)
    },
    async close(handle: FileHandle, _context: EffectContext) {
      const state = handles.get(handle)
      if (state !== undefined) return closeHandle(state)
      return operationResult<Unit>(
        await invokeProviderOperation({
          provider: loaded.provider,
          service: loaded.service,
          operation: closeContract,
          entry: loaded.entry,
          input: handle,
          codecs,
        })
      )
    },
  })
}

function closeHandle(
  state: HandleState
): Promise<ServiceResult<FileSystemError, Unit>> {
  state.unregisterCleanup()
  state.closeCompletion ??= (async () =>
    operationResult<Unit>(
      await invokeProviderOperation({
        provider: state.loaded.provider,
        service: state.loaded.service,
        operation: closeContract,
        entry: state.loaded.entry,
        input: state.handle,
        codecs,
      })
    ))()
  return state.closeCompletion
}

function operationResult<Success>(
  outcome: ProviderBridgeOutcome
): ServiceResult<FileSystemError, Success> {
  if (outcome.kind === "defect") throw outcome.defect
  return outcome.kind === "failure"
    ? fileSystemFailure(outcome.failure as FileSystemError)
    : fileSystemSuccess(outcome.value as Success)
}

function decodeFileSystemError(value: unknown): FileSystemError {
  const error = dataRecord(value, ["code", "message", "operation", "tag"])
  if (
    error.tag !== "FileAccessFailed" ||
    !isOperation(error.operation) ||
    typeof error.code !== "string" ||
    typeof error.message !== "string"
  ) {
    throw new TypeError("filesystem failure is invalid")
  }
  return Object.freeze({
    tag: "FileAccessFailed",
    operation: error.operation,
    code: error.code,
    message: error.message,
  })
}

function isOperation(value: unknown): value is FileSystemOperation {
  return value === "openRead" || value === "read" || value === "close"
}

function stringValue(value: unknown, name: string): string {
  if (typeof value !== "string") throw new TypeError(`${name} must be a string`)
  return value
}

function dataRecord(
  value: unknown,
  keys: ReadonlyArray<string>
): Record<string, unknown> {
  if (
    typeof value !== "object" ||
    value === null ||
    ![Object.prototype, null].includes(Object.getPrototypeOf(value))
  ) {
    throw new TypeError("filesystem boundary value must be a plain record")
  }
  const actual = Reflect.ownKeys(value)
  if (
    actual.length !== keys.length ||
    actual.some((key) => typeof key !== "string" || !keys.includes(key))
  ) {
    throw new TypeError("filesystem boundary record shape is invalid")
  }
  const record: Record<string, unknown> = {}
  for (const key of keys) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (
      descriptor === undefined ||
      !("value" in descriptor) ||
      !descriptor.enumerable
    ) {
      throw new TypeError(
        "filesystem boundary fields must be enumerable data values"
      )
    }
    record[key] = descriptor.value
  }
  return record
}
