import type { Effect } from "./effect"
import { serviceEffect, type ServiceOperation } from "./service"
import type { Maybe } from "./sum"

export type StdinUnavailable = {
  readonly tag: "StdinUnavailable"
}

export type StdinReadFailure = {
  readonly tag: "StdinReadFailure"
}

export type ConcurrentStdinRead = {
  readonly tag: "ConcurrentStdinRead"
}

export type InvalidStdinUtf8 = {
  readonly tag: "InvalidStdinUtf8"
  readonly value: { readonly offset: bigint }
}

export type StdinLineTooLong = {
  readonly tag: "StdinLineTooLong"
  readonly value: { readonly limitBytes: bigint }
}

export type StdinPositionOverflow = {
  readonly tag: "StdinPositionOverflow"
}

export type StdinError =
  | StdinUnavailable
  | StdinReadFailure
  | ConcurrentStdinRead
  | InvalidStdinUtf8
  | StdinLineTooLong
  | StdinPositionOverflow

export type Stdin = {
  readonly readLine: () => ServiceOperation<StdinError, Maybe<string>>
}

export type StdinEnvironment = {
  readonly stdin: Stdin
}

/** Reads one line from the Stdin service supplied at the runner boundary. */
export function readLine(): Effect<
  StdinEnvironment,
  StdinError,
  Maybe<string>
> {
  return serviceEffect((environment: StdinEnvironment) =>
    environment.stdin.readLine()
  )
}
