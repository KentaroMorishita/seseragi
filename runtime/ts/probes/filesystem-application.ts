import { fromUint8Array } from "@seseragi/runtime/bytes"
import type { Effect } from "@seseragi/runtime/effect"
import {
  close,
  type FileHandle,
  type FileSystemEnvironment,
  type FileSystemError,
  filePath,
  openRead,
  Replace,
  read,
  readBytes,
  withTemporaryDirectory,
  writeBytes,
} from "@seseragi/runtime/filesystem"
import { child, type Path } from "@seseragi/runtime/path"

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

export function temporaryRoundTripFixture(prefix: string) {
  return withTemporaryDirectory(
    prefix,
    (directory) => async (environment, context) => {
      const parsed = child("round-trip.bin", directory)
      if (parsed.tag === "Left") {
        throw new Error("static temporary child path must be valid")
      }
      const content = fromUint8Array(
        new TextEncoder().encode("seseragi-filesystem-round-trip")
      )
      await writeBytes(Replace, content, parsed.value)(environment, context)
      return Object.freeze({
        directory: directory as Path,
        content: await readBytes(parsed.value)(environment, context),
      })
    }
  )
}
