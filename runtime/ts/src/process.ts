import nodeProcess from "node:process"
import {
  type Effect,
  type EffectContext,
  type EffectExecution,
  fail,
  type Unit,
} from "./effect"
import { type List, toArray } from "./list"
import { type Path, pathFromProvider } from "./path"
import { serviceEffect, serviceFailure, serviceSuccess } from "./service"
import { fromPull, type Stream } from "./stream"
import { Just, type Maybe, Nothing } from "./sum"

const processBrand: unique symbol = Symbol("seseragi.process")

export type Process = Readonly<{ readonly [processBrand]: true }>
export type ProcessEnvironment = Readonly<{ readonly process: Process }>

const liveProcessValue = Object.freeze({ [processBrand]: true }) as Process
export const liveProcess: Process = liveProcessValue

export type Interrupt = Readonly<{ readonly tag: "Interrupt" }>
export type Terminate = Readonly<{ readonly tag: "Terminate" }>
export type Hangup = Readonly<{ readonly tag: "Hangup" }>
export type Quit = Readonly<{ readonly tag: "Quit" }>
export type User1 = Readonly<{ readonly tag: "User1" }>
export type User2 = Readonly<{ readonly tag: "User2" }>

export type ProcessSignal =
  | Interrupt
  | Terminate
  | Hangup
  | Quit
  | User1
  | User2

export const Interrupt: Interrupt = Object.freeze({ tag: "Interrupt" })
export const Terminate: Terminate = Object.freeze({ tag: "Terminate" })
export const Hangup: Hangup = Object.freeze({ tag: "Hangup" })
export const Quit: Quit = Object.freeze({ tag: "Quit" })
export const User1: User1 = Object.freeze({ tag: "User1" })
export const User2: User2 = Object.freeze({ tag: "User2" })

export type UnsupportedProcessSignal = Readonly<{
  readonly tag: "UnsupportedProcessSignal"
  readonly value: ProcessSignal
}>
export type ReservedProcessSignal = Readonly<{
  readonly tag: "ReservedProcessSignal"
  readonly value: ProcessSignal
}>
export type InvalidArgumentEncoding = Readonly<{
  readonly tag: "InvalidArgumentEncoding"
  readonly value: number
}>
export type InvalidEnvironmentName = Readonly<{
  readonly tag: "InvalidEnvironmentName"
  readonly value: string
}>
export type InvalidEnvironmentEncoding = Readonly<{
  readonly tag: "InvalidEnvironmentEncoding"
  readonly value: string
}>
export type CurrentDirectoryUnavailable = Readonly<{
  readonly tag: "CurrentDirectoryUnavailable"
}>

export type ProcessError =
  | UnsupportedProcessSignal
  | ReservedProcessSignal
  | InvalidArgumentEncoding
  | InvalidEnvironmentName
  | InvalidEnvironmentEncoding
  | CurrentDirectoryUnavailable

export function UnsupportedProcessSignal(
  value: ProcessSignal
): UnsupportedProcessSignal {
  return Object.freeze({ tag: "UnsupportedProcessSignal", value })
}

export function ReservedProcessSignal(
  value: ProcessSignal
): ReservedProcessSignal {
  return Object.freeze({ tag: "ReservedProcessSignal", value })
}

export function InvalidArgumentEncoding(
  value: number
): InvalidArgumentEncoding {
  return Object.freeze({ tag: "InvalidArgumentEncoding", value })
}

export function InvalidEnvironmentName(value: string): InvalidEnvironmentName {
  return Object.freeze({ tag: "InvalidEnvironmentName", value })
}

export function InvalidEnvironmentEncoding(
  value: string
): InvalidEnvironmentEncoding {
  return Object.freeze({ tag: "InvalidEnvironmentEncoding", value })
}

export const CurrentDirectoryUnavailable: CurrentDirectoryUnavailable =
  Object.freeze({ tag: "CurrentDirectoryUnavailable" })

export function processArguments(
  _unit?: Unit
): Effect<ProcessEnvironment, ProcessError, ReadonlyArray<string>> {
  return serviceEffect(async () => {
    const values = nodeProcess.argv.slice(2)
    if (values.some((value) => typeof value !== "string")) {
      return serviceFailure(InvalidArgumentEncoding(0))
    }
    return serviceSuccess(values)
  })
}

export function processEnvironment(
  name: string
): Effect<ProcessEnvironment, ProcessError, Maybe<string>> {
  return serviceEffect(async () => {
    if (name.length === 0 || name.includes("\0")) {
      return serviceFailure(InvalidEnvironmentName(name))
    }
    const value = nodeProcess.env[name]
    return serviceSuccess(value === undefined ? Nothing : Just(value))
  })
}

export function currentDirectory(
  _unit?: Unit
): Effect<ProcessEnvironment, ProcessError, Path> {
  return serviceEffect(async () => {
    try {
      const value = nodeProcess.cwd().replaceAll("\\", "/")
      return serviceSuccess(pathFromProvider(value))
    } catch {
      return serviceFailure(CurrentDirectoryUnavailable)
    }
  })
}

type NodeSignal =
  | "SIGINT"
  | "SIGTERM"
  | "SIGHUP"
  | "SIGQUIT"
  | "SIGUSR1"
  | "SIGUSR2"

let cancelShutdownInstallations = 0

const signalNames: Readonly<Record<ProcessSignal["tag"], NodeSignal>> = {
  Interrupt: "SIGINT",
  Terminate: "SIGTERM",
  Hangup: "SIGHUP",
  Quit: "SIGQUIT",
  User1: "SIGUSR1",
  User2: "SIGUSR2",
}

function signalForTag(tag: ProcessSignal["tag"]): ProcessSignal {
  switch (tag) {
    case "Interrupt":
      return Interrupt
    case "Terminate":
      return Terminate
    case "Hangup":
      return Hangup
    case "Quit":
      return Quit
    case "User1":
      return User1
    case "User2":
      return User2
  }
}

function supportsSignal(signal: ProcessSignal): boolean {
  return (
    nodeProcess.platform !== "win32" ||
    signal.tag === "Interrupt" ||
    signal.tag === "Terminate"
  )
}

export function signals(
  watched: Readonly<{
    readonly tag: "NonEmpty"
    readonly head: ProcessSignal
    readonly tail: List<ProcessSignal>
  }>
): Stream<ProcessEnvironment, ProcessError, ProcessSignal> {
  const requested = [watched.head, ...toArray(watched.tail)]
  const unique = requested.filter(
    (signal, index) =>
      requested.findIndex((candidate) => candidate.tag === signal.tag) === index
  )
  return fromPull<ProcessEnvironment, ProcessError, ProcessSignal>(
    async (_environment, context) => {
      const unsupported = unique.find((signal) => !supportsSignal(signal))
      if (unsupported !== undefined) {
        return await fail(UnsupportedProcessSignal(unsupported))(
          _environment,
          context
        )
      }
      if (cancelShutdownInstallations > 0) {
        const reserved = unique.find(
          (signal) => signal.tag === "Interrupt" || signal.tag === "Terminate"
        )
        if (reserved !== undefined) {
          return await fail(ReservedProcessSignal(reserved))(
            _environment,
            context
          )
        }
      }
      const queue: ProcessSignal[] = []
      let pending:
        | Readonly<{
            readonly resolve: (result: IteratorResult<ProcessSignal>) => void
            readonly unregister: () => void
          }>
        | undefined
      let closed = false
      const handlers = new Map<NodeSignal, () => void>()
      const settlePending = (
        result: IteratorResult<ProcessSignal>
      ): boolean => {
        const request = pending
        if (request === undefined) return false
        pending = undefined
        request.unregister()
        request.resolve(result)
        return true
      }
      const enqueue = (signal: ProcessSignal): void => {
        if (closed) return
        if (queue.some((queued) => queued.tag === signal.tag)) return
        if (settlePending({ done: false, value: signal })) return
        queue.push(signal)
      }
      for (const signal of unique) {
        const name = signalNames[signal.tag]
        const handler = () => enqueue(signalForTag(signal.tag))
        handlers.set(name, handler)
        nodeProcess.on(name, handler)
      }
      const close = (): void => {
        if (closed) return
        closed = true
        for (const [name, handler] of handlers) nodeProcess.off(name, handler)
        handlers.clear()
        settlePending({ done: true, value: undefined })
        queue.length = 0
      }
      context.onCancel(close)
      return {
        pull: async (pullContext: EffectContext) => {
          if (closed) return { done: true, value: undefined } as const
          if (queue.length > 0) {
            return {
              done: false,
              value: queue.shift() as ProcessSignal,
            } as const
          }
          return await new Promise<IteratorResult<ProcessSignal>>((resolve) => {
            let unregister = (): void => undefined
            unregister = pullContext.onCancel(() => {
              settlePending({ done: true, value: undefined })
            })
            pending = Object.freeze({ resolve, unregister })
          })
        },
        close,
      }
    }
  )
}

export type ProcessSignalMode = "cancel" | "forward"

export type ProcessShutdown = Readonly<{
  readonly close: () => Promise<void>
  readonly exitCode: () => number | undefined
}>

export function installProcessShutdown(
  execution: EffectExecution,
  options: Readonly<{
    readonly mode: ProcessSignalMode
    readonly graceMs: number
  }>
): ProcessShutdown {
  let closed = false
  const reservesSignals = options.mode === "cancel"
  if (reservesSignals) cancelShutdownInstallations += 1
  let firstSignal: NodeSignal | undefined
  let cancellation: Promise<void> | undefined
  let timer: ReturnType<typeof setTimeout> | undefined
  const listeners: ReadonlyArray<readonly [NodeSignal, () => void]> = (
    ["SIGINT", "SIGTERM"] as const
  ).map((name) => {
    const handler = (): void => {
      if (closed) return
      if (firstSignal !== undefined) {
        nodeProcess.exit(exitCodeForSignal(name))
        return
      }
      firstSignal = name
      if (options.mode !== "cancel") return
      cancellation = execution.cancel()
      timer = setTimeout(
        () => {
          nodeProcess.exit(exitCodeForSignal(name))
        },
        Math.max(0, options.graceMs)
      )
      void cancellation
        .finally(() => {
          if (timer !== undefined) clearTimeout(timer)
          timer = undefined
        })
        .catch(() => undefined)
    }
    nodeProcess.on(name, handler)
    return [name, handler] as const
  })
  return Object.freeze({
    exitCode: () =>
      firstSignal === undefined ? undefined : exitCodeForSignal(firstSignal),
    async close() {
      if (closed) {
        await cancellation
        return
      }
      closed = true
      for (const [name, handler] of listeners) nodeProcess.off(name, handler)
      if (timer !== undefined) clearTimeout(timer)
      timer = undefined
      if (reservesSignals) cancelShutdownInstallations -= 1
      await cancellation
    },
  })
}

function exitCodeForSignal(signal: NodeSignal): number {
  return signal === "SIGTERM" ? 143 : 130
}
