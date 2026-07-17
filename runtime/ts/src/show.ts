import type { ConsoleError } from "./console-service"
import type { StdinError } from "./stdin-service"

/**
 * The runtime dictionary representation of Seseragi's pure `Show<A>` trait.
 *
 * Compiler-generated dictionaries use this same shape. The dictionary does
 * not perform I/O and must not expose host error objects or stack traces.
 */
export type Show<Value> = {
  readonly show: (value: Value) => string
}

/** String Show is identity: user-facing output does not add quotes. */
export const stringShow: Show<string> = Object.freeze({
  show(value: string): string {
    return value
  },
})

/** Int Show uses the canonical signed base-10 spelling without separators. */
export const intShow: Show<bigint> = Object.freeze({
  show(value: bigint): string {
    return value.toString(10)
  },
})

/** Stable, user-facing rendering for the opaque Console failure boundary. */
export const consoleErrorShow: Show<ConsoleError> = Object.freeze({
  show(error: ConsoleError): string {
    return `ConsoleError: ${error.message}`
  },
})

/** Source-like rendering for the standard Stdin failure ADT. */
export const stdinErrorShow: Show<StdinError> = Object.freeze({
  show(error: StdinError): string {
    switch (error.tag) {
      case "StdinUnavailable":
      case "StdinReadFailure":
      case "ConcurrentStdinRead":
      case "StdinPositionOverflow":
        return error.tag
      case "InvalidStdinUtf8":
        return `InvalidStdinUtf8 { offset: ${error.value.offset} }`
      case "StdinLineTooLong":
        return `StdinLineTooLong { limitBytes: ${error.value.limitBytes} }`
    }
  },
})
