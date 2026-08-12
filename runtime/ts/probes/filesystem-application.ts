import type { Effect } from "@seseragi/runtime/effect"
import {
  close,
  type FileHandle,
  type FileSystemEnvironment,
  type FileSystemError,
  filePath,
  openRead,
  read,
} from "@seseragi/runtime/filesystem"

export function openFixture(
  path: string
): Effect<FileSystemEnvironment, FileSystemError, FileHandle> {
  return openRead(filePath(path))
}

export function readFixture(
  handle: FileHandle,
  limit: number
): Effect<FileSystemEnvironment, FileSystemError, Uint8Array> {
  return read(handle, limit)
}

export function closeFixture(
  handle: FileHandle
): Effect<FileSystemEnvironment, FileSystemError, undefined> {
  return close(handle)
}
