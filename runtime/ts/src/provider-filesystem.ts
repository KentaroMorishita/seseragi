import {
  type EffectContext,
  registerResourceFinalizer,
  throwIfCancelled,
  type Unit,
} from "./effect"
import {
  type DirectoryHandle,
  type FileError,
  type FileHandle,
  type FileSystem,
  fileSystemFailure,
  fileSystemSuccess,
  type ProviderDirectoryRead,
  type ProviderFileMetadata,
  type ProviderFileOperation,
  type ProviderTemporaryKind,
  type ProviderWriteMode,
  type TemporaryHandle,
} from "./filesystem"
import { type Path, pathFromProvider, render } from "./path"
import {
  invokeProviderOperation,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderLogicalType,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import type { ServiceResult } from "./service"

const named = (identity: string) =>
  Object.freeze({ kind: "named", identity } as const)
const primitive = (name: "bool" | "bytes" | "int" | "string") =>
  Object.freeze({ kind: "primitive", name } as const)
const record = (
  fields: ReadonlyArray<Readonly<{ name: string; type: ProviderLogicalType }>>
) => Object.freeze({ kind: "record", fields: Object.freeze(fields) } as const)

const pathType = named("std/path::Path")
const fileHandleType = named("std/fs::FileHandle")
const directoryHandleType = named("std/fs::DirectoryHandle")
const temporaryHandleType = named("std/fs::TemporaryHandle")
const errorType = named("std/fs::FileError")
const metadataType = named("std/fs::ProviderFileMetadata")
const directoryReadType = named("std/fs::ProviderDirectoryRead")
const unitType = Object.freeze({ kind: "unit" } as const)
const bytesType = primitive("bytes")
const intType = primitive("int")
const stringType = primitive("string")

function operation(
  name: string,
  kind: "one-shot" | "resource",
  input: ProviderLogicalType,
  success: ProviderLogicalType
): ProviderOperationContract {
  return Object.freeze({
    identity: `std/fs::FileSystem#${name}`,
    kind,
    input,
    success,
    failure: errorType,
  })
}

const contracts = Object.freeze({
  openRead: operation(
    "openRead",
    "resource",
    record([{ name: "path", type: pathType }]),
    fileHandleType
  ),
  read: operation(
    "read",
    "one-shot",
    record([
      { name: "handle", type: fileHandleType },
      { name: "limit", type: intType },
    ]),
    bytesType
  ),
  openWrite: operation(
    "openWrite",
    "resource",
    record([
      { name: "path", type: pathType },
      { name: "mode", type: stringType },
    ]),
    fileHandleType
  ),
  write: operation(
    "write",
    "one-shot",
    record([
      { name: "handle", type: fileHandleType },
      { name: "content", type: bytesType },
    ]),
    unitType
  ),
  flush: operation("flush", "one-shot", fileHandleType, unitType),
  close: operation("close", "one-shot", fileHandleType, unitType),
  openDirectory: operation(
    "openDirectory",
    "resource",
    record([{ name: "path", type: pathType }]),
    directoryHandleType
  ),
  readDirectory: operation(
    "readDirectory",
    "one-shot",
    directoryHandleType,
    directoryReadType
  ),
  closeDirectory: operation(
    "closeDirectory",
    "one-shot",
    directoryHandleType,
    unitType
  ),
  exists: operation("exists", "one-shot", pathType, primitive("bool")),
  metadata: operation("metadata", "one-shot", pathType, metadataType),
  symlinkMetadata: operation(
    "symlinkMetadata",
    "one-shot",
    pathType,
    metadataType
  ),
  canonicalize: operation("canonicalize", "one-shot", pathType, pathType),
  createDirectory: operation("createDirectory", "one-shot", pathType, unitType),
  createDirectories: operation(
    "createDirectories",
    "one-shot",
    pathType,
    unitType
  ),
  removeFile: operation("removeFile", "one-shot", pathType, unitType),
  removeDirectory: operation("removeDirectory", "one-shot", pathType, unitType),
  move: operation(
    "move",
    "one-shot",
    record([
      { name: "destination", type: pathType },
      { name: "source", type: pathType },
    ]),
    unitType
  ),
  writeAtomic: operation(
    "writeAtomic",
    "one-shot",
    record([
      { name: "content", type: bytesType },
      { name: "path", type: pathType },
    ]),
    unitType
  ),
  createTemporary: operation(
    "createTemporary",
    "resource",
    record([
      { name: "prefix", type: stringType },
      { name: "kind", type: stringType },
    ]),
    temporaryHandleType
  ),
  temporaryPath: operation(
    "temporaryPath",
    "one-shot",
    temporaryHandleType,
    pathType
  ),
  cleanupTemporary: operation(
    "cleanupTemporary",
    "one-shot",
    temporaryHandleType,
    unitType
  ),
})

const codecs = new ProviderCodecRegistry([
  {
    identity: pathType.identity,
    encode: (value) => render(value as Path),
    decode: (value) => pathFromProvider(stringValue(value, "filesystem path")),
  },
  {
    identity: errorType.identity,
    encode: (value) => value,
    decode: decodeFileError,
  },
  {
    identity: metadataType.identity,
    encode: (value) => value,
    decode: decodeMetadata,
  },
  {
    identity: directoryReadType.identity,
    encode: (value) => value,
    decode: decodeDirectoryRead,
  },
])

type HandleState = {
  readonly handle: FileHandle | DirectoryHandle | TemporaryHandle
  readonly loaded: LoadedProviderEntry
  readonly closeContract: ProviderOperationContract
  unregisterCleanup: () => void
  closeCompletion?: Promise<ServiceResult<FileError, Unit>>
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

  const acquire = async <Handle extends FileHandle>(
    contract: ProviderOperationContract,
    input: unknown,
    closeContract: ProviderOperationContract,
    context: EffectContext
  ): Promise<ServiceResult<FileError, Handle>> => {
    const outcome = await invoke(loaded, contract, input, context)
    if (outcome.kind !== "success") return operationResult(outcome)
    const handle = outcome.value as Handle
    const state: HandleState = {
      handle,
      loaded,
      closeContract,
      unregisterCleanup: () => undefined,
    }
    handles.set(handle, state)
    const registration = registerResourceFinalizer(context, async () => {
      const result = await closeState(state)
      if (result.kind === "failure") {
        throw new Error(`filesystem cleanup failed: ${result.error.message}`)
      }
    })
    state.unregisterCleanup = registration.unregister
    await registration.ready
    throwIfCancelled(context)
    return fileSystemSuccess(handle)
  }

  const handleCall = async <Success>(
    contract: ProviderOperationContract,
    handle: FileHandle | DirectoryHandle | TemporaryHandle,
    input: unknown,
    context: EffectContext
  ): Promise<ServiceResult<FileError, Success>> => {
    const state = handles.get(handle)
    if (state?.closeCompletion !== undefined) {
      throw new TypeError("filesystem resource is closed")
    }
    return operationResult(await invoke(loaded, contract, input, context))
  }

  const close = async (
    handle: FileHandle | DirectoryHandle | TemporaryHandle,
    contract: ProviderOperationContract
  ): Promise<ServiceResult<FileError, Unit>> => {
    const state = handles.get(handle)
    if (state !== undefined) return closeState(state)
    return operationResult(await invoke(loaded, contract, handle))
  }

  return Object.freeze({
    openRead: (path, context) =>
      acquire(contracts.openRead, { path }, contracts.close, context),
    read: (handle, limit, context) =>
      handleCall(contracts.read, handle, { handle, limit }, context),
    openWrite: (path, mode: ProviderWriteMode, context) =>
      acquire(contracts.openWrite, { path, mode }, contracts.close, context),
    write: (handle, content, context) =>
      handleCall(contracts.write, handle, { handle, content }, context),
    flush: (handle, context) =>
      handleCall(contracts.flush, handle, handle, context),
    closeFile: (handle) => close(handle, contracts.close),
    openDirectory: (path, context) =>
      acquire(
        contracts.openDirectory,
        { path },
        contracts.closeDirectory,
        context
      ),
    readDirectory: (handle, context) =>
      handleCall(contracts.readDirectory, handle, handle, context),
    closeDirectory: (handle) => close(handle, contracts.closeDirectory),
    exists: (path, context) => oneShot(loaded, contracts.exists, path, context),
    metadata: (path, context) =>
      oneShot(loaded, contracts.metadata, path, context),
    symlinkMetadata: (path, context) =>
      oneShot(loaded, contracts.symlinkMetadata, path, context),
    canonicalize: (path, context) =>
      oneShot(loaded, contracts.canonicalize, path, context),
    createDirectory: (path, context) =>
      oneShot(loaded, contracts.createDirectory, path, context),
    createDirectories: (path, context) =>
      oneShot(loaded, contracts.createDirectories, path, context),
    removeFile: (path, context) =>
      oneShot(loaded, contracts.removeFile, path, context),
    removeDirectory: (path, context) =>
      oneShot(loaded, contracts.removeDirectory, path, context),
    move: (destination, source, context) =>
      oneShot(loaded, contracts.move, { destination, source }, context),
    writeAtomic: (content, path, context) =>
      oneShot(loaded, contracts.writeAtomic, { content, path }, context),
    createTemporary: (
      prefix: string,
      kind: ProviderTemporaryKind,
      context: EffectContext
    ) =>
      acquire(
        contracts.createTemporary,
        { prefix, kind },
        contracts.cleanupTemporary,
        context
      ),
    temporaryPath: (handle, context) =>
      handleCall(contracts.temporaryPath, handle, handle, context),
    cleanupTemporary: (handle) => close(handle, contracts.cleanupTemporary),
  })
}

async function oneShot<Success>(
  loaded: LoadedProviderEntry,
  contract: ProviderOperationContract,
  input: unknown,
  context: EffectContext
): Promise<ServiceResult<FileError, Success>> {
  return operationResult(await invoke(loaded, contract, input, context))
}

function invoke(
  loaded: LoadedProviderEntry,
  contract: ProviderOperationContract,
  input: unknown,
  context?: EffectContext
): Promise<ProviderBridgeOutcome> {
  return invokeProviderOperation({
    provider: loaded.provider,
    service: loaded.service,
    operation: contract,
    entry: loaded.entry,
    input,
    codecs,
    ...(context === undefined ? {} : { context }),
  })
}

function closeState(
  state: HandleState
): Promise<ServiceResult<FileError, Unit>> {
  state.unregisterCleanup()
  state.closeCompletion ??= (async () =>
    operationResult(
      await invoke(state.loaded, state.closeContract, state.handle)
    ))()
  return state.closeCompletion
}

function operationResult<Success>(
  outcome: ProviderBridgeOutcome
): ServiceResult<FileError, Success> {
  if (outcome.kind === "defect") throw outcome.defect
  return outcome.kind === "failure"
    ? fileSystemFailure(outcome.failure as FileError)
    : fileSystemSuccess(outcome.value as Success)
}

function decodeFileError(value: unknown): FileError {
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

function decodeMetadata(value: unknown): ProviderFileMetadata {
  const result = dataRecord(value, [
    "createdNanoseconds",
    "fileType",
    "modifiedNanoseconds",
    "sizeBytes",
  ])
  if (
    !isFileType(result.fileType) ||
    !Number.isSafeInteger(result.sizeBytes) ||
    !nullableIntegerString(result.modifiedNanoseconds) ||
    !nullableIntegerString(result.createdNanoseconds)
  ) {
    throw new TypeError("filesystem metadata is invalid")
  }
  return Object.freeze({
    fileType: result.fileType,
    sizeBytes: result.sizeBytes as number,
    modifiedNanoseconds: result.modifiedNanoseconds,
    createdNanoseconds: result.createdNanoseconds,
  })
}

function decodeDirectoryRead(value: unknown): ProviderDirectoryRead {
  const record = dataRecord(value, ["tag", "value"])
  if (record.tag === "done" && record.value === null) {
    return Object.freeze({ tag: "done" })
  }
  if (record.tag !== "entry") {
    throw new TypeError("filesystem directory result is invalid")
  }
  const entry = dataRecord(record.value, ["fileType", "name", "path"])
  if (
    typeof entry.name !== "string" ||
    typeof entry.path !== "string" ||
    !(entry.fileType === null || isFileType(entry.fileType))
  ) {
    throw new TypeError("filesystem directory entry is invalid")
  }
  return Object.freeze({
    tag: "entry",
    value: Object.freeze({
      name: entry.name,
      path: pathFromProvider(entry.path),
      fileType: entry.fileType,
    }),
  })
}

function isOperation(value: unknown): value is ProviderFileOperation {
  return [
    "openRead",
    "read",
    "openWrite",
    "write",
    "flush",
    "close",
    "openDirectory",
    "readDirectory",
    "closeDirectory",
    "exists",
    "metadata",
    "symlinkMetadata",
    "canonicalize",
    "createDirectory",
    "createDirectories",
    "removeFile",
    "removeDirectory",
    "move",
    "writeAtomic",
    "createTemporary",
    "temporaryPath",
    "cleanupTemporary",
  ].includes(value as string)
}

function isFileType(value: unknown): value is ProviderFileMetadata["fileType"] {
  return ["regular-file", "directory", "symbolic-link", "other"].includes(
    value as string
  )
}

function nullableIntegerString(value: unknown): value is string | null {
  return value === null || (typeof value === "string" && /^\d+$/.test(value))
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
  const result: Record<string, unknown> = {}
  for (const key of keys) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (descriptor === undefined || !("value" in descriptor)) {
      throw new TypeError("filesystem boundary fields must be data values")
    }
    result[key] = descriptor.value
  }
  return result
}
