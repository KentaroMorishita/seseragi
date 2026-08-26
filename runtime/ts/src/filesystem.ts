import { type Bytes, fromUint8Array, toUint8Array } from "./bytes"
import { createInstant, type Instant } from "./clock-value"
import {
  attempt,
  createEffectExecution,
  type Effect,
  type EffectContext,
  fail,
  mapError as mapEffectError,
  type Unit,
  unit,
} from "./effect"
import { current, normalize, type Path, parse, render } from "./path"
import type { ProviderHandle } from "./provider"
import {
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"
import {
  type BufferCapacity,
  fromPull,
  mapError as mapStreamError,
  runForEach,
  type Stream,
} from "./stream"
import { type Either, Just, Left, type Maybe, Nothing, Right } from "./sum"
import { decodeUtf8, encodeUtf8, type Utf8DecodeError } from "./text"

export type FileHandle = ProviderHandle
export type DirectoryHandle = ProviderHandle
export type TemporaryHandle = ProviderHandle

export type ProviderFileOperation =
  | "openRead"
  | "read"
  | "openWrite"
  | "write"
  | "flush"
  | "close"
  | "openDirectory"
  | "readDirectory"
  | "closeDirectory"
  | "exists"
  | "metadata"
  | "symlinkMetadata"
  | "canonicalize"
  | "createDirectory"
  | "createDirectories"
  | "removeFile"
  | "removeDirectory"
  | "move"
  | "writeAtomic"
  | "createTemporary"
  | "temporaryPath"
  | "cleanupTemporary"

/** Provider-owned error without application path context. */
export type FileError = Readonly<{
  tag: "FileAccessFailed"
  operation: ProviderFileOperation
  code: string
  message: string
}>

export type ProviderFileType =
  | "regular-file"
  | "directory"
  | "symbolic-link"
  | "other"

export type ProviderFileMetadata = Readonly<{
  fileType: ProviderFileType
  sizeBytes: number
  modifiedNanoseconds: string | null
  createdNanoseconds: string | null
}>

export type ProviderDirectoryEntry = Readonly<{
  name: string
  path: Path
  fileType: ProviderFileType | null
}>

export type ProviderDirectoryRead =
  | Readonly<{ tag: "done" }>
  | Readonly<{ tag: "entry"; value: ProviderDirectoryEntry }>

export type ProviderWriteMode = "replace" | "create-new" | "append"
export type ProviderTemporaryKind = "directory" | "file"

export type FileSystem = Readonly<{
  openRead: FilePathOperation<FileHandle>
  read: (
    handle: FileHandle,
    limit: number,
    context: EffectContext
  ) => Promise<ServiceResult<FileError, Uint8Array>>
  openWrite: (
    path: Path,
    mode: ProviderWriteMode,
    context: EffectContext
  ) => Promise<ServiceResult<FileError, FileHandle>>
  write: (
    handle: FileHandle,
    content: Uint8Array,
    context: EffectContext
  ) => Promise<ServiceResult<FileError, Unit>>
  flush: HandleOperation<FileHandle, Unit>
  closeFile: HandleOperation<FileHandle, Unit>
  openDirectory: FilePathOperation<DirectoryHandle>
  readDirectory: HandleOperation<DirectoryHandle, ProviderDirectoryRead>
  closeDirectory: HandleOperation<DirectoryHandle, Unit>
  exists: FilePathOperation<boolean>
  metadata: FilePathOperation<ProviderFileMetadata>
  symlinkMetadata: FilePathOperation<ProviderFileMetadata>
  canonicalize: FilePathOperation<Path>
  createDirectory: FilePathOperation<Unit>
  createDirectories: FilePathOperation<Unit>
  removeFile: FilePathOperation<Unit>
  removeDirectory: FilePathOperation<Unit>
  move: (
    destination: Path,
    source: Path,
    context: EffectContext
  ) => Promise<ServiceResult<FileError, Unit>>
  writeAtomic: (
    content: Uint8Array,
    path: Path,
    context: EffectContext
  ) => Promise<ServiceResult<FileError, Unit>>
  createTemporary: (
    prefix: string,
    kind: ProviderTemporaryKind,
    context: EffectContext
  ) => Promise<ServiceResult<FileError, TemporaryHandle>>
  temporaryPath: HandleOperation<TemporaryHandle, Path>
  cleanupTemporary: HandleOperation<TemporaryHandle, Unit>
}>

type FilePathOperation<Success> = (
  path: Path,
  context: EffectContext
) => Promise<ServiceResult<FileError, Success>>

type HandleOperation<Handle, Success> = (
  handle: Handle,
  context: EffectContext
) => Promise<ServiceResult<FileError, Success>>

export type FileSystemEnvironment = Readonly<{ fileSystem: FileSystem }>

/** Compatibility alias for the original Provider vertical-slice probe. */
export type FilePath = Path

export type FileType =
  | Readonly<{ tag: "RegularFile" }>
  | Readonly<{ tag: "Directory" }>
  | Readonly<{ tag: "SymbolicLink" }>
  | Readonly<{ tag: "OtherFileType" }>

export type FileSystemOperation =
  | Readonly<{ tag: "ReadFile" }>
  | Readonly<{ tag: "WriteFile" }>
  | Readonly<{ tag: "OpenDirectory" }>
  | Readonly<{ tag: "ReadMetadata" }>
  | Readonly<{ tag: "CreateDirectory" }>
  | Readonly<{ tag: "RemovePath" }>
  | Readonly<{ tag: "MovePath" }>
  | Readonly<{ tag: "CanonicalizePath" }>
  | Readonly<{ tag: "CreateTemporary" }>

export type FileSystemErrorKind =
  | Readonly<{ tag: "FileNotFound" }>
  | Readonly<{ tag: "FileAlreadyExists" }>
  | Readonly<{ tag: "PermissionDenied" }>
  | Readonly<{ tag: "NotADirectory" }>
  | Readonly<{ tag: "IsADirectory" }>
  | Readonly<{ tag: "DirectoryNotEmpty" }>
  | Readonly<{ tag: "SymbolicLinkLoop" }>
  | Readonly<{ tag: "CrossDeviceMove" }>
  | Readonly<{ tag: "PathNotSupported" }>
  | Readonly<{ tag: "FileSystemUnavailable" }>
  | Readonly<{ tag: "OtherFileSystemError"; value: string }>

export type FileSystemError = Readonly<{
  operation: FileSystemOperation
  path: Path
  otherPath: Maybe<Path>
  kind: FileSystemErrorKind
}>

export type FileMetadata = Readonly<{
  fileType: FileType
  sizeBytes: number
  modified: Maybe<Instant>
  created: Maybe<Instant>
}>

export type DirectoryEntry = Readonly<{
  name: string
  path: Path
  fileType: Maybe<FileType>
}>

export type WriteMode =
  | Readonly<{ tag: "Replace" }>
  | Readonly<{ tag: "CreateNew" }>
  | Readonly<{ tag: "Append" }>

export type FileTextError =
  | Readonly<{ tag: "FileAccessFailure"; value: FileSystemError }>
  | Readonly<{ tag: "FileUtf8Failure"; value: Utf8DecodeError }>

export const RegularFile: FileType = variant("RegularFile")
export const Directory: FileType = variant("Directory")
export const SymbolicLink: FileType = variant("SymbolicLink")
export const OtherFileType: FileType = variant("OtherFileType")
export const ReadFile: FileSystemOperation = variant("ReadFile")
export const WriteFile: FileSystemOperation = variant("WriteFile")
export const OpenDirectory: FileSystemOperation = variant("OpenDirectory")
export const ReadMetadata: FileSystemOperation = variant("ReadMetadata")
export const CreateDirectory: FileSystemOperation = variant("CreateDirectory")
export const RemovePath: FileSystemOperation = variant("RemovePath")
export const MovePath: FileSystemOperation = variant("MovePath")
export const CanonicalizePath: FileSystemOperation = variant("CanonicalizePath")
export const CreateTemporary: FileSystemOperation = variant("CreateTemporary")
export const FileNotFound: FileSystemErrorKind = variant("FileNotFound")
export const FileAlreadyExists: FileSystemErrorKind =
  variant("FileAlreadyExists")
export const PermissionDenied: FileSystemErrorKind = variant("PermissionDenied")
export const NotADirectory: FileSystemErrorKind = variant("NotADirectory")
export const IsADirectory: FileSystemErrorKind = variant("IsADirectory")
export const DirectoryNotEmpty: FileSystemErrorKind =
  variant("DirectoryNotEmpty")
export const SymbolicLinkLoop: FileSystemErrorKind = variant("SymbolicLinkLoop")
export const CrossDeviceMove: FileSystemErrorKind = variant("CrossDeviceMove")
export const PathNotSupported: FileSystemErrorKind = variant("PathNotSupported")
export const FileSystemUnavailable: FileSystemErrorKind = variant(
  "FileSystemUnavailable"
)
export const Replace: WriteMode = variant("Replace")
export const CreateNew: WriteMode = variant("CreateNew")
export const Append: WriteMode = variant("Append")

export function OtherFileSystemError(value: string): FileSystemErrorKind {
  return Object.freeze({ tag: "OtherFileSystemError", value })
}

export function FileAccessFailure(value: FileSystemError): FileTextError {
  return Object.freeze({ tag: "FileAccessFailure", value })
}

export function FileUtf8Failure(value: Utf8DecodeError): FileTextError {
  return Object.freeze({ tag: "FileUtf8Failure", value })
}

export function exists(
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, boolean> {
  return pathEffect(path, ReadMetadata, (fs, context) =>
    fs.exists(path, context)
  )
}

/** Compatibility constructor retained for Provider ABI probes. */
export function filePath(text: string): FilePath {
  const value = parse(text)
  if (value.tag === "Left") {
    throw new TypeError(`filesystem path is invalid: ${value.value.tag}`)
  }
  return value.value
}

export const renderFilePath = render

/** Low-level compatibility surface; applications use readBytes/readChunks. */
export function openRead(
  path: FilePath
): Effect<FileSystemEnvironment, FileSystemError, FileHandle> {
  return rawOpenRead(path)
}

/** Low-level compatibility surface; applications use readBytes/readChunks. */
export function read(
  handle: FileHandle,
  limit: number
): Effect<FileSystemEnvironment, FileSystemError, Uint8Array> {
  return rawRead(handle, limit, current())
}

/** Low-level compatibility surface for the resource ownership probe. */
export function close(
  handle: FileHandle
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return rawCloseFile(handle, current())
}

export function metadata(
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, FileMetadata> {
  return mapMetadata(path, false)
}

export function symlinkMetadata(
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, FileMetadata> {
  return mapMetadata(path, true)
}

export function canonicalize(
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Path> {
  return pathEffect(path, CanonicalizePath, (fs, context) =>
    fs.canonicalize(path, context)
  )
}

export function readBytes(
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Bytes> {
  return async (environment, context) => {
    const active = activeContext(context)
    const handle = await rawOpenRead(path)(environment, active)
    const chunks: Uint8Array[] = []
    try {
      while (true) {
        const chunk = await rawRead(
          handle,
          64 * 1024,
          path
        )(environment, active)
        if (chunk.length === 0) break
        chunks.push(chunk)
      }
    } catch (error) {
      await bestEffort(rawCloseFile(handle, path), environment)
      throw error
    }
    await rawCloseFile(handle, path)(environment, active)
    const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0)
    const result = new Uint8Array(length)
    let offset = 0
    for (const chunk of chunks) {
      result.set(chunk, offset)
      offset += chunk.length
    }
    return fromUint8Array(result)
  }
}

export function readTextUtf8(
  path: Path
): Effect<FileSystemEnvironment, FileTextError, string> {
  return async (environment, context) => {
    const bytes = await mapEffectError(FileAccessFailure, readBytes(path))(
      environment,
      context
    )
    const decoded = decodeUtf8(bytes)
    return decoded.tag === "Right"
      ? decoded.value
      : fail(FileUtf8Failure(decoded.value))(environment, context)
  }
}

export function readChunks(
  size: BufferCapacity,
  path: Path
): Stream<FileSystemEnvironment, FileSystemError, Bytes> {
  return fromPull<FileSystemEnvironment, FileSystemError, Bytes>(
    async (environment, context) => {
      const handle = await rawOpenRead(path)(environment, context)
      let closed = false
      return Object.freeze({
        async pull(active: EffectContext) {
          const chunk = await rawRead(
            handle,
            size.value,
            path
          )(environment, active)
          if (chunk.length === 0) {
            await rawCloseFile(handle, path)(environment, active)
            closed = true
            return Object.freeze({
              done: true,
              value: undefined,
            }) as IteratorResult<Bytes>
          }
          return Object.freeze({
            done: false,
            value: fromUint8Array(chunk),
          }) as IteratorResult<Bytes>
        },
        async close() {
          if (closed) return
          closed = true
          await rawCloseFile(handle, path)(environment, activeContext())
        },
      })
    }
  )
}

export function writeBytes(
  mode: WriteMode,
  content: Bytes,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return writeWith(mode, path, async (handle, environment, context) => {
    await rawWrite(handle, toUint8Array(content), path)(environment, context)
  })
}

export function writeTextUtf8(
  mode: WriteMode,
  content: string,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return writeBytes(mode, encodeUtf8(content), path)
}

export function writeChunks<Environment, Failure>(
  mode: WriteMode,
  source: Stream<Environment, Failure, Bytes>,
  path: Path
): Effect<
  Environment & FileSystemEnvironment,
  Either<Failure, FileSystemError>,
  Unit
> {
  const tagged = mapStreamError(
    (error: Failure) => Left(error),
    source
  ) as Stream<
    Environment & FileSystemEnvironment,
    Either<Failure, FileSystemError>,
    Bytes
  >
  return mapEffectError(
    (error: FileSystemError | Either<Failure, FileSystemError>) =>
      isEither(error) ? error : Right(error),
    writeWith<
      Environment & FileSystemEnvironment,
      Either<Failure, FileSystemError>
    >(mode, path, async (handle, environment, context) => {
      await runForEach(
        (chunk) =>
          ((activeEnvironment, activeContext) =>
            mapEffectError(
              (error: FileSystemError) => Right(error),
              rawWrite(handle, toUint8Array(chunk), path)
            )(activeEnvironment, activeContext)) as Effect<
            Environment & FileSystemEnvironment,
            Either<Failure, FileSystemError>,
            Unit
          >,
        tagged
      )(environment, context)
    })
  )
}

export function writeAtomic(
  content: Bytes,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return pathEffect(path, WriteFile, (fs, context) =>
    fs.writeAtomic(toUint8Array(content), normalize(path), context)
  )
}

export function list(
  path: Path
): Stream<FileSystemEnvironment, FileSystemError, DirectoryEntry> {
  return fromPull<FileSystemEnvironment, FileSystemError, DirectoryEntry>(
    async (environment, context) => {
      const handle = await rawOpenDirectory(path)(environment, context)
      let closed = false
      return Object.freeze({
        async pull(active: EffectContext) {
          const result = await rawReadDirectory(handle, path)(
            environment,
            active
          )
          if (result.tag === "done") {
            await rawCloseDirectory(handle, path)(environment, active)
            closed = true
            return Object.freeze({
              done: true,
              value: undefined,
            }) as IteratorResult<DirectoryEntry>
          }
          return Object.freeze({
            done: false,
            value: Object.freeze({
              name: result.value.name,
              path: result.value.path,
              fileType:
                result.value.fileType === null
                  ? Nothing
                  : Just(publicFileType(result.value.fileType)),
            }),
          }) as IteratorResult<DirectoryEntry>
        },
        async close() {
          if (closed) return
          closed = true
          await rawCloseDirectory(handle, path)(environment, activeContext())
        },
      })
    }
  )
}

export const createDirectory = unaryUnit("createDirectory", CreateDirectory)
export const createDirectories = unaryUnit("createDirectories", CreateDirectory)
export const removeFile = unaryUnit("removeFile", RemovePath)
export const removeDirectory = unaryUnit("removeDirectory", RemovePath)

export function move(
  destination: Path,
  source: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return serviceEffect(async (environment, context) =>
    mapServiceError(
      environment.fileSystem.move(
        normalize(destination),
        normalize(source),
        context
      ),
      source,
      MovePath,
      destination
    )
  )
}

export function withTemporaryDirectory<Environment, Failure, Success>(
  prefix: string,
  use: (path: Path) => Effect<Environment, Failure, Success>
): Effect<
  Environment & FileSystemEnvironment,
  Either<FileSystemError, Failure>,
  Success
> {
  return withTemporary("directory", prefix, use)
}

export function withTemporaryFile<Environment, Failure, Success>(
  prefix: string,
  use: (path: Path) => Effect<Environment, Failure, Success>
): Effect<
  Environment & FileSystemEnvironment,
  Either<FileSystemError, Failure>,
  Success
> {
  return withTemporary("file", prefix, use)
}

export function fileSystemSuccess<Success>(
  value: Success
): ServiceResult<never, Success> {
  return serviceSuccess(value)
}

export function fileSystemFailure(
  error: FileError
): ServiceResult<FileError, never> {
  return serviceFailure(error)
}

function mapMetadata(
  path: Path,
  symlink: boolean
): Effect<FileSystemEnvironment, FileSystemError, FileMetadata> {
  return async (environment, context) => {
    const active = activeContext(context)
    const raw = await pathEffect(path, ReadMetadata, (fs, operationContext) =>
      symlink
        ? fs.symlinkMetadata(normalize(path), operationContext)
        : fs.metadata(normalize(path), operationContext)
    )(environment, active)
    return Object.freeze({
      fileType: publicFileType(raw.fileType),
      sizeBytes: raw.sizeBytes,
      modified: instant(raw.modifiedNanoseconds),
      created: instant(raw.createdNanoseconds),
    })
  }
}

function unaryUnit(
  operation:
    | "createDirectory"
    | "createDirectories"
    | "removeFile"
    | "removeDirectory",
  publicOperation: FileSystemOperation
): (path: Path) => Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return (path) =>
    pathEffect(path, publicOperation, (fs, context) =>
      fs[operation](normalize(path), context)
    )
}

function writeWith<
  Environment extends FileSystemEnvironment,
  Failure = FileSystemError,
>(
  mode: WriteMode,
  path: Path,
  write: (
    handle: FileHandle,
    environment: Environment,
    context: EffectContext
  ) => Promise<void>
): Effect<Environment, Failure | FileSystemError, Unit> {
  return async (environment, context) => {
    const active = activeContext(context)
    const handle = await rawOpenWrite(path, providerWriteMode(mode))(
      environment,
      active
    )
    try {
      await write(handle, environment, active)
      await rawFlush(handle, path)(environment, active)
    } catch (error) {
      await bestEffort(rawCloseFile(handle, path), environment)
      throw error
    }
    await rawCloseFile(handle, path)(environment, active)
    return unit
  }
}

function withTemporary<Environment, Failure, Success>(
  kind: ProviderTemporaryKind,
  prefix: string,
  use: (path: Path) => Effect<Environment, Failure, Success>
): Effect<
  Environment & FileSystemEnvironment,
  Either<FileSystemError, Failure>,
  Success
> {
  return async (environment, context) => {
    const active = activeContext(context)
    if (!validTemporaryPrefix(prefix)) {
      return fail(Left(filesystemError(CreateTemporary, current(), "EINVAL")))(
        environment,
        active
      )
    }
    const acquired = await mapEffectError(
      (error: FileSystemError) => Left(error),
      rawCreateTemporary(prefix, kind)
    )(environment, active)
    const path = await mapEffectError(
      (error: FileSystemError) => Left(error),
      rawTemporaryPath(acquired, current())
    )(environment, active)
    const used = await attempt(use(path))(environment, active)
    const cleaned = await attempt(rawCleanupTemporary(acquired, path))(
      environment,
      activeContext()
    )
    if (used.tag === "Left") {
      return fail(Right(used.value))(environment, active)
    }
    if (cleaned.tag === "Left") {
      return fail(Left(cleaned.value))(environment, active)
    }
    return used.value
  }
}

function pathEffect<Success>(
  path: Path,
  operation: FileSystemOperation,
  invoke: (
    fileSystem: FileSystem,
    context: EffectContext
  ) => Promise<ServiceResult<FileError, Success>>
): Effect<FileSystemEnvironment, FileSystemError, Success> {
  return serviceEffect(async (environment, context) =>
    mapServiceError(invoke(environment.fileSystem, context), path, operation)
  )
}

async function mapServiceError<Success>(
  result: Promise<ServiceResult<FileError, Success>>,
  path: Path,
  operation: FileSystemOperation,
  otherPath?: Path
): Promise<ServiceResult<FileSystemError, Success>> {
  const outcome = await result
  return outcome.kind === "success"
    ? outcome
    : serviceFailure(
        filesystemError(operation, path, outcome.error.code, otherPath)
      )
}

function rawOpenRead(
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, FileHandle> {
  return pathEffect(path, ReadFile, (fs, context) =>
    fs.openRead(normalize(path), context)
  )
}

function rawRead(
  handle: FileHandle,
  limit: number,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Uint8Array> {
  return pathEffect(path, ReadFile, (fs, context) =>
    fs.read(handle, limit, context)
  )
}

function rawOpenWrite(
  path: Path,
  mode: ProviderWriteMode
): Effect<FileSystemEnvironment, FileSystemError, FileHandle> {
  return pathEffect(path, WriteFile, (fs, context) =>
    fs.openWrite(normalize(path), mode, context)
  )
}

function rawWrite(
  handle: FileHandle,
  content: Uint8Array,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return pathEffect(path, WriteFile, (fs, context) =>
    fs.write(handle, content, context)
  )
}

function rawFlush(
  handle: FileHandle,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return pathEffect(path, WriteFile, (fs, context) => fs.flush(handle, context))
}

function rawCloseFile(
  handle: FileHandle,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return pathEffect(path, ReadFile, (fs, context) =>
    fs.closeFile(handle, context)
  )
}

function rawOpenDirectory(
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, DirectoryHandle> {
  return pathEffect(path, OpenDirectory, (fs, context) =>
    fs.openDirectory(normalize(path), context)
  )
}

function rawReadDirectory(
  handle: DirectoryHandle,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, ProviderDirectoryRead> {
  return pathEffect(path, OpenDirectory, (fs, context) =>
    fs.readDirectory(handle, context)
  )
}

function rawCloseDirectory(
  handle: DirectoryHandle,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return pathEffect(path, OpenDirectory, (fs, context) =>
    fs.closeDirectory(handle, context)
  )
}

function rawCreateTemporary(
  prefix: string,
  kind: ProviderTemporaryKind
): Effect<FileSystemEnvironment, FileSystemError, TemporaryHandle> {
  const path = current()
  return pathEffect(path, CreateTemporary, (fs, context) =>
    fs.createTemporary(prefix, kind, context)
  )
}

function rawTemporaryPath(
  handle: TemporaryHandle,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Path> {
  return pathEffect(path, CreateTemporary, (fs, context) =>
    fs.temporaryPath(handle, context)
  )
}

function rawCleanupTemporary(
  handle: TemporaryHandle,
  path: Path
): Effect<FileSystemEnvironment, FileSystemError, Unit> {
  return pathEffect(path, RemovePath, (fs, context) =>
    fs.cleanupTemporary(handle, context)
  )
}

async function bestEffort<Environment>(
  effect: Effect<Environment, unknown, Unit>,
  environment: Environment
): Promise<void> {
  try {
    await attempt(effect)(environment, activeContext())
  } catch {
    // Preserve the primary failure/cancellation. The host diagnostic channel
    // receives resource-finalizer defects from the Provider bridge.
  }
}

function activeContext(context?: EffectContext): EffectContext {
  return context ?? createEffectExecution().context
}

function filesystemError(
  operation: FileSystemOperation,
  path: Path,
  code: string,
  otherPath?: Path
): FileSystemError {
  return Object.freeze({
    operation,
    path,
    otherPath: otherPath === undefined ? Nothing : Just(otherPath),
    kind: errorKind(code),
  })
}

function errorKind(code: string): FileSystemErrorKind {
  switch (code) {
    case "ENOENT":
      return FileNotFound
    case "EEXIST":
      return FileAlreadyExists
    case "EACCES":
    case "EPERM":
      return PermissionDenied
    case "ENOTDIR":
      return NotADirectory
    case "EISDIR":
      return IsADirectory
    case "ENOTEMPTY":
      return DirectoryNotEmpty
    case "ELOOP":
      return SymbolicLinkLoop
    case "EXDEV":
      return CrossDeviceMove
    case "EINVAL":
    case "ENAMETOOLONG":
      return PathNotSupported
    case "ENOSYS":
    case "ENODEV":
      return FileSystemUnavailable
    default:
      return OtherFileSystemError(code)
  }
}

function publicFileType(value: ProviderFileType): FileType {
  switch (value) {
    case "regular-file":
      return RegularFile
    case "directory":
      return Directory
    case "symbolic-link":
      return SymbolicLink
    default:
      return OtherFileType
  }
}

function instant(value: string | null): Maybe<Instant> {
  if (value === null) return Nothing
  try {
    return Just(createInstant(BigInt(value)))
  } catch {
    return Nothing
  }
}

function providerWriteMode(mode: WriteMode): ProviderWriteMode {
  switch (mode.tag) {
    case "CreateNew":
      return "create-new"
    case "Append":
      return "append"
    default:
      return "replace"
  }
}

function validTemporaryPrefix(prefix: string): boolean {
  return (
    prefix.length > 0 &&
    prefix !== "." &&
    prefix !== ".." &&
    !prefix.includes("/") &&
    !prefix.includes("\\") &&
    !prefix.includes("\0")
  )
}

function isEither(value: unknown): value is Either<unknown, unknown> {
  return (
    typeof value === "object" &&
    value !== null &&
    ((value as { tag?: unknown }).tag === "Left" ||
      (value as { tag?: unknown }).tag === "Right")
  )
}

function variant<Tag extends string>(tag: Tag): Readonly<{ tag: Tag }> {
  return Object.freeze({ tag })
}
