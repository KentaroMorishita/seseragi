import type { Effect, EffectContext, Unit } from "./effect"
import type { ProviderHandle } from "./provider"
import {
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"

const filePathBrand: unique symbol = Symbol("seseragi.file-path")
const filePathValues = new WeakMap<object, string>()

export type FilePath = Readonly<{ readonly [filePathBrand]: true }>
export type FileHandle = ProviderHandle
export type FileSystemOperation = "openRead" | "read" | "close"
export type FileSystemError = Readonly<{
  tag: "FileAccessFailed"
  operation: FileSystemOperation
  code: string
  message: string
}>
export type FileSystem = Readonly<{
  openRead: (
    path: FilePath,
    context: EffectContext
  ) => Promise<ServiceResult<FileSystemError, FileHandle>>
  read: (
    handle: FileHandle,
    limit: number,
    context: EffectContext
  ) => Promise<ServiceResult<FileSystemError, Uint8Array>>
  close: (
    handle: FileHandle,
    context: EffectContext
  ) => Promise<ServiceResult<FileSystemError, Unit>>
}>
export type FileSystemEnvironment = Readonly<{ fileSystem: FileSystem }>

export function filePath(text: string): FilePath {
  if (
    text.length === 0 ||
    text.includes("\0") ||
    text.includes("\\") ||
    text.split("/").some((segment) => segment === "." || segment === "..")
  ) {
    throw new TypeError("filesystem path must be a normalized portable path")
  }
  const value = Object.create(null) as FilePath
  Object.defineProperty(value, filePathBrand, {
    enumerable: false,
    value: true,
  })
  filePathValues.set(value, text)
  return Object.freeze(value)
}

export function renderFilePath(path: FilePath): string {
  const value = filePathValues.get(path)
  if (value === undefined) throw new TypeError("filesystem path is invalid")
  return value
}

export function openRead(
  path: FilePath
): Effect<FileSystemEnvironment, FileSystemError, FileHandle> {
  return serviceEffect((environment, context) =>
    environment.fileSystem.openRead(path, context)
  )
}

export function read(
  handle: FileHandle,
  limit: number
): Effect<FileSystemEnvironment, FileSystemError, Uint8Array> {
  return serviceEffect((environment, context) =>
    environment.fileSystem.read(handle, limit, context)
  )
}

export function close(
  handle: FileHandle
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return serviceEffect((environment, context) =>
    environment.fileSystem.close(handle, context)
  )
}

export function fileSystemSuccess<Success>(
  value: Success
): ServiceResult<never, Success> {
  return serviceSuccess(value)
}

export function fileSystemFailure(
  error: FileSystemError
): ServiceResult<FileSystemError, never> {
  return serviceFailure(error)
}
