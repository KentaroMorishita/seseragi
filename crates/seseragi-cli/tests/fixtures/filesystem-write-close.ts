import assert from "node:assert/strict"
import { fromUint8Array } from "../../../../runtime/ts/src/bytes"
import { type Effect, run } from "../../../../runtime/ts/src/effect"
import {
  Append,
  CreateNew,
  type FileHandle,
  type FileSystem,
  type FileSystemEnvironment,
  type FileSystemError,
  filePath,
  fileSystemFailure,
  fileSystemSuccess,
  type ProviderFileOperation,
  Replace,
  readBytes,
  readTextUtf8,
  writeBytes,
  writeTextUtf8,
} from "../../../../runtime/ts/src/filesystem"
import {
  fileSystemErrorShow,
  fileTextErrorShow,
} from "../../../../runtime/ts/src/show"

const path = filePath("output-雪.txt")
const handle = Object.freeze({}) as FileHandle
for (const api of ["text", "bytes", "read-text", "read-bytes"]) {
  const reading = api.startsWith("read-")
  for (const mode of [Replace, Append, CreateNew]) {
    for (const content of ["", "A\0雪🙂\n", `${"x".repeat(65535)}🙂`]) {
      const bytes = new TextEncoder().encode(content)
      for (const failure of [
        "ok",
        "open",
        "read",
        "write",
        "flush",
        "close",
        "write+close",
        "flush+close",
      ]) {
        let closes = 0
        const written: number[] = []
        const stage = <Value>(
          name: string,
          operation: ProviderFileOperation,
          value: Value
        ) =>
          failure.split("+").includes(name)
            ? fileSystemFailure({
                tag: "FileAccessFailed",
                operation,
                code: name === "close" ? "EACCES" : "EEXIST",
                message: name,
              })
            : fileSystemSuccess(value)
        const methods: Pick<
          FileSystem,
          "openRead" | "openWrite" | "read" | "write" | "flush" | "closeFile"
        > = {
          async openRead() {
            return stage("open", "openRead", handle)
          },
          async openWrite() {
            return stage("open", "openWrite", handle)
          },
          async read() {
            return stage("read", "read", new Uint8Array())
          },
          async write(_handle, data) {
            const result =
              data.length === 0
                ? fileSystemSuccess(undefined)
                : stage("write", "write", undefined)
            if (result.kind === "success") written.push(...data)
            return result
          },
          async flush() {
            return stage("flush", "flush", undefined)
          },
          async closeFile() {
            closes += 1
            return stage("close", "close", undefined)
          },
        }
        const environment = {
          fileSystem: new Proxy(methods as FileSystem, {
            get(target, key, receiver) {
              assert(
                key in target,
                `unexpected filesystem operation ${String(key)}`
              )
              return Reflect.get(target, key, receiver)
            },
          }),
        }
        let result: string
        if (api === "read-text") {
          const outcome = await run(readTextUtf8(path), environment)
          result =
            outcome.kind === "success"
              ? "success"
              : fileTextErrorShow.show(outcome.error)
        } else {
          const action: Effect<
            FileSystemEnvironment,
            FileSystemError,
            unknown
          > =
            api === "text"
              ? writeTextUtf8(mode, content, path)
              : api === "bytes"
                ? writeBytes(mode, fromUint8Array(bytes), path)
                : readBytes(path)
          const outcome = await run(action, environment)
          result =
            outcome.kind === "success"
              ? "success"
              : fileSystemErrorShow.show(outcome.error)
        }
        assert.deepEqual(
          written,
          reading || failure === "open" || failure.includes("write")
            ? []
            : Array.from(bytes)
        )
        assert.equal(closes, failure === "open" ? 0 : 1)
        console.log(`${api}/${mode.tag}/${bytes.length}/${failure}: ${result}`)
      }
    }
  }
}
