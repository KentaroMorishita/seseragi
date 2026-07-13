import { stdin as processStdin } from "node:process"
import { createInterface } from "node:readline"
import { serviceFailure, serviceSuccess } from "./service"
import { Just, Nothing } from "./sum"
import type {
  ConcurrentStdinRead,
  Stdin,
  StdinReadFailure,
} from "./stdin-service"

export type {
  ConcurrentStdinRead,
  InvalidStdinUtf8,
  Stdin,
  StdinEnvironment,
  StdinError,
  StdinLineTooLong,
  StdinPositionOverflow,
  StdinReadFailure,
  StdinUnavailable,
} from "./stdin-service"
export { readLine } from "./stdin-service"

export type ProcessStdin = Stdin & {
  readonly close: () => void
}

const STDIN_READ_FAILURE: StdinReadFailure = Object.freeze({
  tag: "StdinReadFailure",
})

const CONCURRENT_STDIN_READ: ConcurrentStdinRead = Object.freeze({
  tag: "ConcurrentStdinRead",
})

/**
 * Creates one root-run-local adapter over process standard input.
 *
 * The readline interface is allocated lazily. EOF is sticky but does not close
 * the host adapter: only `close` does that. A read after host close is therefore
 * a runtime defect rather than a fabricated EOF result.
 */
export function createProcessStdin(
  input: NodeJS.ReadableStream = processStdin
): ProcessStdin {
  let interface_: ReturnType<typeof createInterface> | undefined
  let lines: AsyncIterator<string> | undefined
  let hostClosed = false
  let eof = false
  // A Node readable error is terminal; cache its typed result instead of
  // creating a fresh readline iterator that can wait forever on a dead input.
  let terminalReadFailure = false
  let readActive = false

  const releaseInterface = () => {
    interface_?.close()
    interface_ = undefined
    lines = undefined
  }

  const close = () => {
    if (hostClosed) return
    hostClosed = true
    releaseInterface()
  }

  return {
    async readLine() {
      if (hostClosed) {
        throw new Error("Stdin adapter was read after host close")
      }
      if (eof) return serviceSuccess(Nothing)
      if (terminalReadFailure) return serviceFailure(STDIN_READ_FAILURE)
      if (readActive) return serviceFailure(CONCURRENT_STDIN_READ)

      readActive = true
      try {
        let next: IteratorResult<string>
        try {
          if (lines === undefined) {
            interface_ = createInterface({
              input,
              crlfDelay: Infinity,
            })
            lines = interface_[Symbol.asyncIterator]()
          }
          next = await lines.next()
        } catch {
          terminalReadFailure = true
          releaseInterface()
          return serviceFailure(STDIN_READ_FAILURE)
        }

        if (hostClosed) {
          throw new Error("Stdin adapter was closed during an active read")
        }
        if (next.done) {
          eof = true
          releaseInterface()
          return serviceSuccess(Nothing)
        }
        return serviceSuccess(Just(next.value))
      } finally {
        readActive = false
      }
    },
    close,
  }
}
