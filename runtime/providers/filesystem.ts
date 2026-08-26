import { randomUUID } from "node:crypto"
import {
  access,
  lstat,
  mkdir,
  mkdtemp,
  open,
  opendir,
  realpath,
  rename,
  rm,
  rmdir,
  stat,
  unlink,
  writeFile,
} from "node:fs/promises"
import { tmpdir } from "node:os"
import { dirname, join } from "node:path"
import {
  type ProviderResult,
  providerRuntimeAbi,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
  type ProviderRuntimeTarget,
} from "@seseragi/runtime/provider-package"

export type HostFileHandle = Readonly<{
  read: (
    buffer: Uint8Array,
    offset: number,
    length: number,
    position: null
  ) => Promise<Readonly<{ bytesRead: number }>>
  write?: (
    buffer: Uint8Array,
    offset?: number,
    length?: number,
    position?: null
  ) => Promise<Readonly<{ bytesWritten: number }>>
  sync?: () => Promise<void>
  close: () => Promise<void>
}>

export type HostDirectoryHandle = Readonly<{
  read: () => Promise<HostDirectoryEntry | null>
  close: () => Promise<void>
}>

export type HostDirectoryEntry = Readonly<{
  name: string
  isFile: () => boolean
  isDirectory: () => boolean
  isSymbolicLink: () => boolean
}>

export type FileSystemHost = Readonly<{
  openRead: (path: string) => Promise<HostFileHandle>
  openWrite?: (path: string, mode: WriteMode) => Promise<HostFileHandle>
  openDirectory?: (path: string) => Promise<HostDirectoryHandle>
  access?: (path: string) => Promise<void>
  metadata?: (path: string, symlink: boolean) => Promise<Metadata>
  canonicalize?: (path: string) => Promise<string>
  createDirectory?: (path: string, recursive: boolean) => Promise<void>
  removeFile?: (path: string) => Promise<void>
  removeDirectory?: (path: string) => Promise<void>
  move?: (destination: string, source: string) => Promise<void>
  writeAtomic?: (content: Uint8Array, path: string) => Promise<void>
  createTemporary?: (
    prefix: string,
    kind: TemporaryKind
  ) => Promise<TemporaryResource>
  cleanupTemporary?: (resource: TemporaryResource) => Promise<void>
}>

type WriteMode = "replace" | "create-new" | "append"
type TemporaryKind = "directory" | "file"
type FileOperation =
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

type Metadata = Readonly<{
  fileType: "regular-file" | "directory" | "symbolic-link" | "other"
  sizeBytes: number
  modifiedNanoseconds: string | null
  createdNanoseconds: string | null
}>

type TemporaryResource = Readonly<{
  path: string
  cleanupRoot: string
  identity: Readonly<{ device: number; inode: number }>
}>

type FileToken = {
  readonly kind: "file"
  readonly handle: HostFileHandle
  closeCompletion?: Promise<void>
}

type DirectoryToken = {
  readonly kind: "directory"
  readonly path: string
  readonly handle: HostDirectoryHandle
  closeCompletion?: Promise<void>
}

type TemporaryToken = {
  readonly kind: "temporary"
  readonly resource: TemporaryResource
  cleanupCompletion?: Promise<void>
}

const liveHost: Required<FileSystemHost> = Object.freeze({
  openRead: async (path) => open(path, "r"),
  openWrite: async (path, mode) =>
    open(path, mode === "replace" ? "w" : mode === "create-new" ? "wx" : "a"),
  openDirectory: async (path) => opendir(path),
  access,
  metadata: async (path, symlink) =>
    metadataOf(await (symlink ? lstat(path) : stat(path))),
  canonicalize: realpath,
  createDirectory: async (path, recursive) => {
    await mkdir(path, { recursive })
  },
  removeFile: unlink,
  removeDirectory: rmdir,
  move: async (destination, source) => {
    await rejectExisting(destination)
    await rename(source, destination)
  },
  writeAtomic: atomicWrite,
  createTemporary,
  cleanupTemporary,
})

export function createFileSystemProvider(
  provider: string,
  target: ProviderRuntimeTarget,
  host: FileSystemHost = liveHost
): ProviderPackageEntry {
  const selected = Object.freeze({ ...liveHost, ...host })
  const files = new Set<FileToken>()
  const directories = new Set<DirectoryToken>()
  const temporaries = new Set<TemporaryToken>()

  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/fs::FileSystem",
    targets: [target],
    operations: {
      async openRead(value) {
        try {
          const path = pathRequest(value)
          const token: FileToken = {
            kind: "file",
            handle: await selected.openRead(path),
          }
          files.add(token)
          return success(token)
        } catch (cause) {
          return failure("openRead", cause)
        }
      },
      async read(value) {
        try {
          const request = dataRecord(value, ["handle", "limit"])
          const token = owned(request.handle, files, "file")
          const limit = positiveInt(request.limit, "filesystem read limit")
          ensureOpen(token.closeCompletion)
          const buffer = new Uint8Array(limit)
          const { bytesRead } = await token.handle.read(buffer, 0, limit, null)
          return success(new Uint8Array(buffer.subarray(0, bytesRead)))
        } catch (cause) {
          return failure("read", cause)
        }
      },
      async openWrite(value) {
        try {
          const request = dataRecord(value, ["mode", "path"])
          const path = stringField(request.path, "filesystem path")
          const mode = writeMode(request.mode)
          const token: FileToken = {
            kind: "file",
            handle: await selected.openWrite(path, mode),
          }
          files.add(token)
          return success(token)
        } catch (cause) {
          return failure("openWrite", cause)
        }
      },
      async write(value) {
        try {
          const request = dataRecord(value, ["content", "handle"])
          const token = owned(request.handle, files, "file")
          ensureOpen(token.closeCompletion)
          if (!(request.content instanceof Uint8Array)) {
            throw new TypeError("filesystem content must be Bytes")
          }
          if (token.handle.write === undefined) throw unsupported("write")
          let offset = 0
          while (offset < request.content.length) {
            const { bytesWritten } = await token.handle.write(
              request.content,
              offset,
              request.content.length - offset,
              null
            )
            if (bytesWritten <= 0)
              throw new Error("filesystem write made no progress")
            offset += bytesWritten
          }
          return success(undefined)
        } catch (cause) {
          return failure("write", cause)
        }
      },
      async flush(value) {
        try {
          const token = owned(value, files, "file")
          ensureOpen(token.closeCompletion)
          if (token.handle.sync === undefined) throw unsupported("flush")
          await token.handle.sync()
          return success(undefined)
        } catch (cause) {
          return failure("flush", cause)
        }
      },
      async close(value) {
        try {
          await closeFile(owned(value, files, "file"))
          return success(undefined)
        } catch (cause) {
          return failure("close", cause)
        }
      },
      async openDirectory(value) {
        try {
          const path = pathRequest(value)
          const token: DirectoryToken = {
            kind: "directory",
            path,
            handle: await selected.openDirectory(path),
          }
          directories.add(token)
          return success(token)
        } catch (cause) {
          return failure("openDirectory", cause)
        }
      },
      async readDirectory(value) {
        try {
          const token = owned(value, directories, "directory")
          ensureOpen(token.closeCompletion)
          const entry = await token.handle.read()
          return success(
            entry === null
              ? { tag: "done", value: null }
              : {
                  tag: "entry",
                  value: {
                    name: entry.name,
                    path: portable(join(token.path, entry.name)),
                    fileType: directoryEntryType(entry),
                  },
                }
          )
        } catch (cause) {
          return failure("readDirectory", cause)
        }
      },
      async closeDirectory(value) {
        try {
          await closeDirectory(owned(value, directories, "directory"))
          return success(undefined)
        } catch (cause) {
          return failure("closeDirectory", cause)
        }
      },
      async exists(value) {
        try {
          await selected.access(stringField(value, "filesystem path"))
          return success(true)
        } catch (cause) {
          return errorCode(cause) === "ENOENT"
            ? success(false)
            : failure("exists", cause)
        }
      },
      async metadata(value) {
        return metadataOperation("metadata", value, false, selected)
      },
      async symlinkMetadata(value) {
        return metadataOperation("symlinkMetadata", value, true, selected)
      },
      async canonicalize(value) {
        try {
          return success(
            portable(
              await selected.canonicalize(stringField(value, "filesystem path"))
            )
          )
        } catch (cause) {
          return failure("canonicalize", cause)
        }
      },
      async createDirectory(value) {
        return directoryMutation("createDirectory", value, false, selected)
      },
      async createDirectories(value) {
        return directoryMutation("createDirectories", value, true, selected)
      },
      async removeFile(value) {
        try {
          await selected.removeFile(stringField(value, "filesystem path"))
          return success(undefined)
        } catch (cause) {
          return failure("removeFile", cause)
        }
      },
      async removeDirectory(value) {
        try {
          await selected.removeDirectory(stringField(value, "filesystem path"))
          return success(undefined)
        } catch (cause) {
          return failure("removeDirectory", cause)
        }
      },
      async move(value) {
        try {
          const request = dataRecord(value, ["destination", "source"])
          const destination = stringField(
            request.destination,
            "move destination"
          )
          const source = stringField(request.source, "move source")
          if (
            [...temporaries].some(
              (token) =>
                token.resource.path === source ||
                token.resource.path === destination
            )
          ) {
            throw permissionDenied("active temporary root cannot be moved")
          }
          await selected.move(destination, source)
          return success(undefined)
        } catch (cause) {
          return failure("move", cause)
        }
      },
      async writeAtomic(value) {
        try {
          const request = dataRecord(value, ["content", "path"])
          if (!(request.content instanceof Uint8Array)) {
            throw new TypeError("atomic content must be Bytes")
          }
          await selected.writeAtomic(
            request.content,
            stringField(request.path, "filesystem path")
          )
          return success(undefined)
        } catch (cause) {
          return failure("writeAtomic", cause)
        }
      },
      async createTemporary(value) {
        try {
          const request = dataRecord(value, ["kind", "prefix"])
          const prefix = stringField(request.prefix, "temporary prefix")
          const kind = temporaryKind(request.kind)
          const token: TemporaryToken = {
            kind: "temporary",
            resource: await selected.createTemporary(prefix, kind),
          }
          temporaries.add(token)
          return success(token)
        } catch (cause) {
          return failure("createTemporary", cause)
        }
      },
      async temporaryPath(value) {
        try {
          const token = owned(value, temporaries, "temporary")
          ensureOpen(token.cleanupCompletion)
          return success(portable(token.resource.path))
        } catch (cause) {
          return failure("temporaryPath", cause)
        }
      },
      async cleanupTemporary(value) {
        try {
          await cleanupToken(
            owned(value, temporaries, "temporary"),
            selected.cleanupTemporary
          )
          return success(undefined)
        } catch (cause) {
          return failure("cleanupTemporary", cause)
        }
      },
    },
    shutdown: async () => {
      for (const token of [...directories].reverse())
        await closeDirectory(token)
      for (const token of [...files].reverse()) await closeFile(token)
      for (const token of [...temporaries].reverse()) {
        await cleanupToken(token, selected.cleanupTemporary)
      }
      directories.clear()
      files.clear()
      temporaries.clear()
    },
  })
}

async function metadataOperation(
  operation: "metadata" | "symlinkMetadata",
  value: unknown,
  symlink: boolean,
  host: Required<FileSystemHost>
): Promise<ProviderResult> {
  try {
    return success(
      await host.metadata(stringField(value, "filesystem path"), symlink)
    )
  } catch (cause) {
    return failure(operation, cause)
  }
}

async function directoryMutation(
  operation: "createDirectory" | "createDirectories",
  value: unknown,
  recursive: boolean,
  host: Required<FileSystemHost>
): Promise<ProviderResult> {
  try {
    await host.createDirectory(stringField(value, "filesystem path"), recursive)
    return success(undefined)
  } catch (cause) {
    return failure(operation, cause)
  }
}

async function atomicWrite(content: Uint8Array, path: string): Promise<void> {
  const temporary = join(dirname(path), `.seseragi-atomic-${randomUUID()}`)
  let created = false
  try {
    await writeFile(temporary, content, { flag: "wx" })
    created = true
    const handle = await open(temporary, "r+")
    try {
      await handle.sync()
    } finally {
      await handle.close()
    }
    await rename(temporary, path)
    created = false
  } finally {
    if (created) await rm(temporary, { force: true }).catch(() => undefined)
  }
}

async function createTemporary(
  prefix: string,
  kind: TemporaryKind
): Promise<TemporaryResource> {
  const cleanupRoot = await mkdtemp(join(tmpdir(), prefix))
  const path = kind === "directory" ? cleanupRoot : join(cleanupRoot, prefix)
  if (kind === "file") await writeFile(path, new Uint8Array(), { flag: "wx" })
  const details = await lstat(path)
  return Object.freeze({
    path,
    cleanupRoot,
    identity: Object.freeze({ device: details.dev, inode: details.ino }),
  })
}

async function cleanupTemporary(resource: TemporaryResource): Promise<void> {
  try {
    const details = await lstat(resource.path)
    if (
      details.dev !== resource.identity.device ||
      details.ino !== resource.identity.inode
    ) {
      throw permissionDenied("temporary resource identity changed")
    }
  } catch (cause) {
    if (errorCode(cause) !== "ENOENT") throw cause
  }
  await rm(resource.cleanupRoot, { recursive: true, force: true })
}

function closeFile(token: FileToken): Promise<void> {
  token.closeCompletion ??= token.handle.close()
  return token.closeCompletion
}

function closeDirectory(token: DirectoryToken): Promise<void> {
  token.closeCompletion ??= token.handle.close()
  return token.closeCompletion
}

function cleanupToken(
  token: TemporaryToken,
  cleanup: (resource: TemporaryResource) => Promise<void>
): Promise<void> {
  token.cleanupCompletion ??= cleanup(token.resource)
  return token.cleanupCompletion
}

function owned<Token extends object, Kind extends string>(
  value: unknown,
  tokens: Set<Token>,
  kind: Kind
): Token {
  if (
    typeof value !== "object" ||
    value === null ||
    !tokens.has(value as Token)
  ) {
    throw new TypeError(
      `filesystem ${kind} handle is not owned by this provider`
    )
  }
  return value as Token
}

function ensureOpen(completion: Promise<void> | undefined): void {
  if (completion !== undefined) throw resourceClosed()
}

function pathRequest(value: unknown): string {
  return stringField(dataRecord(value, ["path"]).path, "filesystem path")
}

function positiveInt(value: unknown, name: string): number {
  if (
    !Number.isSafeInteger(value) ||
    (value as number) <= 0 ||
    (value as number) > 0x7fff_ffff
  ) {
    throw new RangeError(`${name} is invalid`)
  }
  return value as number
}

function writeMode(value: unknown): WriteMode {
  if (value === "replace" || value === "create-new" || value === "append")
    return value
  throw new TypeError("filesystem write mode is invalid")
}

function temporaryKind(value: unknown): TemporaryKind {
  if (value === "directory" || value === "file") return value
  throw new TypeError("filesystem temporary kind is invalid")
}

function directoryEntryType(
  entry: HostDirectoryEntry
): Metadata["fileType"] | null {
  if (entry.isFile()) return "regular-file"
  if (entry.isDirectory()) return "directory"
  if (entry.isSymbolicLink()) return "symbolic-link"
  return "other"
}

function metadataOf(value: {
  isFile: () => boolean
  isDirectory: () => boolean
  isSymbolicLink: () => boolean
  size: number
  mtimeNs?: bigint
  birthtimeNs?: bigint
  mtimeMs: number
  birthtimeMs: number
}): Metadata {
  return Object.freeze({
    fileType: value.isFile()
      ? "regular-file"
      : value.isDirectory()
        ? "directory"
        : value.isSymbolicLink()
          ? "symbolic-link"
          : "other",
    sizeBytes: value.size,
    modifiedNanoseconds: String(
      value.mtimeNs ?? BigInt(Math.trunc(value.mtimeMs * 1_000_000))
    ),
    createdNanoseconds: String(
      value.birthtimeNs ?? BigInt(Math.trunc(value.birthtimeMs * 1_000_000))
    ),
  })
}

async function rejectExisting(path: string): Promise<void> {
  try {
    await lstat(path)
    throw Object.assign(new Error("destination already exists"), {
      code: "EEXIST",
    })
  } catch (cause) {
    if (errorCode(cause) !== "ENOENT") throw cause
  }
}

function success(value: unknown): ProviderResult {
  return { kind: "success", value }
}

function failure(operation: FileOperation, cause: unknown): ProviderResult {
  const error = cause as { code?: unknown; message?: unknown }
  return {
    kind: "failure",
    failure: Object.freeze({
      tag: "FileAccessFailed",
      operation,
      code: typeof error?.code === "string" ? error.code : "FILESYSTEM_ERROR",
      message:
        typeof error?.message === "string"
          ? error.message
          : "filesystem failed",
    }),
  }
}

function resourceClosed(): Error & { code: string } {
  return Object.assign(new Error("filesystem resource is closed"), {
    code: "RESOURCE_CLOSED",
  })
}

function unsupported(operation: string): Error & { code: string } {
  return Object.assign(
    new Error(`filesystem host does not support ${operation}`),
    {
      code: "ENOSYS",
    }
  )
}

function permissionDenied(message: string): Error & { code: string } {
  return Object.assign(new Error(message), { code: "EPERM" })
}

function errorCode(value: unknown): string | undefined {
  return (value as { code?: unknown })?.code as string | undefined
}

function portable(path: string): string {
  return path.replaceAll("\\", "/")
}

function stringField(value: unknown, name: string): string {
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
    throw new TypeError("filesystem provider input must be a plain record")
  }
  const actual = Reflect.ownKeys(value)
  if (
    actual.length !== keys.length ||
    actual.some((key) => typeof key !== "string" || !keys.includes(key))
  ) {
    throw new TypeError("filesystem provider input shape is invalid")
  }
  const record: Record<string, unknown> = {}
  for (const key of keys) {
    const descriptor = Object.getOwnPropertyDescriptor(value, key)
    if (descriptor === undefined || !("value" in descriptor)) {
      throw new TypeError("filesystem provider input must use data fields")
    }
    record[key] = descriptor.value
  }
  return record
}
