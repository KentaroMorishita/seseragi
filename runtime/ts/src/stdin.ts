import { stdin as processStdin } from "node:process"
import type { Readable } from "node:stream"
import { EffectCancellation } from "./effect"
import { type ServiceResult, serviceFailure, serviceSuccess } from "./service"
import {
  createByteStdin,
  type Stdin,
  type StdinByteSource,
  type StdinError,
  StdinReadFailure,
} from "./stdin-service"
import { Just, type Maybe, Nothing } from "./sum"

export type {
  ByteStdinOptions,
  LineLimit,
  ReadSize,
  Stdin,
  StdinConfigError,
  StdinEnvironment,
  StdinError,
} from "./stdin-service"
export {
  ConcurrentStdinRead,
  createByteStdin,
  defaultLineLimit,
  defaultReadSize,
  InvalidStdinUtf8,
  LineLimitTooLarge,
  lineLimit,
  lines,
  MAX_LINE_LIMIT,
  MAX_READ_SIZE,
  NonPositiveLineLimit,
  NonPositiveReadSize,
  ReadSizeTooLarge,
  readChunk,
  readLine,
  readLineWith,
  readSize,
  StdinLineTooLong,
  StdinPositionOverflow,
  StdinReadFailure,
  StdinUnavailable,
} from "./stdin-service"

export type ProcessStdin = Stdin & Readonly<{ close: () => void }>

/** Creates one root-run-local byte cursor over process standard input. */
export function createProcessStdin(
  input: NodeJS.ReadableStream = processStdin
): ProcessStdin {
  const readable = input as Readable
  let hostClosed = false
  let terminalReadFailure = false
  let started = false
  let ended = false
  const queued: Uint8Array[] = []
  let pending:
    | Readonly<{
        resolve: (result: ServiceResult<StdinError, Maybe<Uint8Array>>) => void
        reject: (error: unknown) => void
        unregisterCancel: () => void
      }>
    | undefined

  const settle = (
    complete: (request: NonNullable<typeof pending>) => void
  ): boolean => {
    const request = pending
    if (request === undefined) return false
    pending = undefined
    request.unregisterCancel()
    complete(request)
    return true
  }

  const data = (value: Uint8Array | string) => {
    const bytes = asBytes(value)
    if (
      !settle((request) =>
        request.resolve(serviceSuccess(Just(new Uint8Array(bytes))))
      )
    ) {
      queued.push(bytes)
    }
  }
  const end = () => {
    ended = true
    settle((request) => request.resolve(serviceSuccess(Nothing)))
  }
  const error = () => {
    terminalReadFailure = true
    settle((request) => request.resolve(serviceFailure(StdinReadFailure)))
  }
  const start = () => {
    if (started) return
    started = true
    readable.on("data", data)
    readable.once("end", end)
    readable.once("error", error)
    readable.resume()
  }

  const source: StdinByteSource = {
    read(_size, context) {
      if (hostClosed) {
        throw new Error("Stdin adapter was read after host close")
      }
      start()
      const chunk = queued.shift()
      if (chunk !== undefined) return serviceSuccess(Just(chunk))
      if (terminalReadFailure) return serviceFailure(StdinReadFailure)
      if (ended) return serviceSuccess(Nothing)
      return new Promise((resolve, reject) => {
        let unregisterCancel: () => void = () => undefined
        pending = Object.freeze({
          resolve,
          reject,
          get unregisterCancel() {
            return unregisterCancel
          },
        })
        unregisterCancel = context.onCancel(() => {
          settle((request) => request.reject(new EffectCancellation()))
        })
      })
    },
  }
  const cursor = createByteStdin(source)
  return Object.freeze({
    ...cursor,
    close() {
      if (hostClosed) return
      hostClosed = true
      readable.off("data", data)
      readable.off("end", end)
      readable.off("error", error)
      readable.pause()
      settle((request) =>
        request.reject(new Error("Stdin adapter closed during active read"))
      )
    },
  })
}

function asBytes(value: Uint8Array | string): Uint8Array {
  return typeof value === "string"
    ? new TextEncoder().encode(value)
    : new Uint8Array(value)
}
