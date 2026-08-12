import { open } from "node:fs/promises"
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
  close: () => Promise<void>
}>
export type FileSystemHost = Readonly<{
  openRead: (path: string) => Promise<HostFileHandle>
}>

type FileToken = {
  readonly handle: HostFileHandle
  closeCompletion?: Promise<void>
}
type FileOperation = "openRead" | "read" | "close"

const liveHost: FileSystemHost = Object.freeze({
  openRead: async (path) => open(path, "r"),
})

export function createFileSystemProvider(
  provider: string,
  target: ProviderRuntimeTarget,
  host: FileSystemHost = liveHost
): ProviderPackageEntry {
  const handles = new Set<FileToken>()
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/fs::FileSystem",
    targets: [target],
    operations: {
      async openRead(value) {
        try {
          const request = dataRecord(value, ["path"])
          if (typeof request.path !== "string") {
            throw new TypeError("filesystem path must be a string")
          }
          const token: FileToken = { handle: await host.openRead(request.path) }
          handles.add(token)
          return { kind: "success", value: token }
        } catch (cause) {
          return failure("openRead", cause)
        }
      },
      async read(value) {
        let token: FileToken | undefined
        try {
          const request = dataRecord(value, ["handle", "limit"])
          token = ownedToken(request.handle, handles)
          if (
            !Number.isSafeInteger(request.limit) ||
            (request.limit as number) <= 0 ||
            (request.limit as number) > 0x7fff_ffff
          ) {
            throw new RangeError("filesystem read limit is invalid")
          }
          if (token.closeCompletion !== undefined) {
            throw resourceClosed()
          }
          const buffer = new Uint8Array(request.limit as number)
          const { bytesRead } = await token.handle.read(
            buffer,
            0,
            buffer.length,
            null
          )
          return {
            kind: "success",
            value: new Uint8Array(buffer.subarray(0, bytesRead)),
          }
        } catch (cause) {
          return failure("read", cause)
        }
      },
      async close(value) {
        try {
          const token = ownedToken(value, handles)
          await closeToken(token)
          return { kind: "success", value: undefined }
        } catch (cause) {
          return failure("close", cause)
        }
      },
    },
    shutdown: async () => {
      for (const token of [...handles].reverse()) await closeToken(token)
      handles.clear()
    },
  })
}

function closeToken(token: FileToken): Promise<void> {
  token.closeCompletion ??= token.handle.close()
  return token.closeCompletion
}

function ownedToken(value: unknown, handles: Set<FileToken>): FileToken {
  if (
    typeof value !== "object" ||
    value === null ||
    !handles.has(value as FileToken)
  ) {
    throw new TypeError("filesystem handle is not owned by this provider")
  }
  return value as FileToken
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
