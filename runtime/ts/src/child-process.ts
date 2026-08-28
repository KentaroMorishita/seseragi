import { type Bytes, fromUint8Array, toUint8Array } from "./bytes"
import { durationNanoseconds, type Duration } from "./clock-value"
import { attempt, type Effect, type EffectContext, type Unit } from "./effect"
import { type Path, render as renderPath } from "./path"
import type { ProcessSignal } from "./process"
import {
  type ServiceResult,
  serviceEffect,
  serviceFailure,
  serviceSuccess,
} from "./service"
import type { BufferCapacity, Stream, StreamCursor } from "./stream"
import { type Either, Left, Right } from "./sum"

const commandBrand: unique symbol = Symbol("seseragi.child-process.command")
const captureLimitBrand: unique symbol = Symbol(
  "seseragi.child-process.capture-limit"
)
const childProcessesBrand: unique symbol = Symbol(
  "seseragi.child-process.service"
)

export type SearchPath = Readonly<{
  readonly tag: "SearchPath"
  readonly value: string
}>
export type ExecutablePath = Readonly<{
  readonly tag: "ExecutablePath"
  readonly value: Path
}>
export type Executable = SearchPath | ExecutablePath

export type ChildProcessConfigError =
  | Readonly<{ readonly tag: "EmptyExecutableName" }>
  | Readonly<{
      readonly tag: "ExecutableNameContainsSeparator"
      readonly value: string
    }>
  | Readonly<{
      readonly tag: "ArgumentContainsNul"
      readonly value: Readonly<{ index: number; offset: number }>
    }>
  | Readonly<{
      readonly tag: "EnvironmentNameContainsNul"
      readonly value: string
    }>
  | Readonly<{
      readonly tag: "EnvironmentValueContainsNul"
      readonly value: string
    }>
  | Readonly<{ readonly tag: "InvalidCaptureLimit"; readonly value: number }>

export type ChildOutputChannel =
  | Readonly<{ readonly tag: "ChildStdout" }>
  | Readonly<{ readonly tag: "ChildStderr" }>

export type ChildProcessError =
  | Readonly<{
      readonly tag: "ChildSpawnFailed"
      readonly value: Readonly<{
        executable: Executable
        detail: string
      }>
    }>
  | Readonly<{ readonly tag: "ChildInputAfterClose" }>
  | Readonly<{
      readonly tag: "ChildOutputReadFailed"
      readonly value: Readonly<{
        channel: ChildOutputChannel
        detail: string
      }>
    }>
  | Readonly<{
      readonly tag: "UnsupportedChildSignal"
      readonly value: ProcessSignal
    }>
  | Readonly<{ readonly tag: "ChildInputFailed"; readonly value: string }>
  | Readonly<{
      readonly tag: "ChildOutputLimitExceeded"
      readonly value: Readonly<{
        channel: ChildOutputChannel
        limitBytes: number
      }>
    }>
  | Readonly<{ readonly tag: "ChildWaitFailed"; readonly value: string }>
  | Readonly<{
      readonly tag: "ChildTerminationFailed"
      readonly value: string
    }>

export type ChildExitStatus =
  | Readonly<{ readonly tag: "ChildExited"; readonly value: number }>
  | Readonly<{
      readonly tag: "ChildSignaled"
      readonly value: ProcessSignal
    }>
  | Readonly<{
      readonly tag: "ChildHostTerminated"
      readonly value: string
    }>

export type ChildInput =
  | Readonly<{ readonly tag: "WriteChildStdin"; readonly value: Bytes }>
  | Readonly<{ readonly tag: "CloseChildStdin" }>
  | Readonly<{ readonly tag: "SignalChild"; readonly value: ProcessSignal }>
  | Readonly<{ readonly tag: "KillChild" }>

export type ChildEvent =
  | Readonly<{ readonly tag: "ChildStdoutChunk"; readonly value: Bytes }>
  | Readonly<{ readonly tag: "ChildStderrChunk"; readonly value: Bytes }>
  | Readonly<{
      readonly tag: "ChildExitedWith"
      readonly value: ChildExitStatus
    }>

export type CapturedProcess = Readonly<{
  status: ChildExitStatus
  stdout: Bytes
  stderr: Bytes
}>

export type ProviderExecutable =
  | Readonly<{ tag: "search-path"; value: string }>
  | Readonly<{ tag: "path"; value: string }>

export type ProviderCommand = Readonly<{
  executable: ProviderExecutable
  arguments: ReadonlyArray<string>
  directory: string
  hasDirectory: boolean
  clearEnvironment: boolean
  environment: ReadonlyArray<
    Readonly<{ name: string; value: string; unset: boolean }>
  >
  terminationGraceMilliseconds: number
  outputBufferChunks: number
}>

export type ProviderCapturedProcess = Readonly<{
  status: ChildExitStatus
  stdout: Uint8Array
  stderr: Uint8Array
}>

export type ProviderStreamingFailure<Failure> =
  | Readonly<{ kind: "input"; error: Failure }>
  | Readonly<{ kind: "child"; error: ChildProcessError }>

export type ChildProcesses = Readonly<{
  readonly [childProcessesBrand]: true
  runCaptured: (
    command: ProviderCommand,
    input: Uint8Array,
    limitBytes: number,
    context: EffectContext
  ) => Promise<ServiceResult<ChildProcessError, ProviderCapturedProcess>>
  runInherited: (
    command: ProviderCommand,
    context: EffectContext
  ) => Promise<ServiceResult<ChildProcessError, ChildExitStatus>>
  openStreaming: <Environment, Failure>(
    input: Stream<Environment, Failure, ChildInput>,
    command: ProviderCommand,
    environment: Environment,
    context: EffectContext
  ) => Promise<StreamCursor<ChildEvent>>
}>

export type ChildProcessesEnvironment = Readonly<{
  childProcesses: ChildProcesses
}>

type CommandState = Readonly<{
  executable: Executable
  arguments: ReadonlyArray<string>
  directory?: Path
  clearEnvironment: boolean
  environment: ReadonlyMap<string, string | undefined>
  terminationGraceMilliseconds: number
  outputBufferChunks: number
}>

export type Command = Readonly<{ readonly [commandBrand]: true }>
export type CaptureLimit = Readonly<{ readonly [captureLimitBrand]: true }>

const commandStates = new WeakMap<object, CommandState>()
const captureLimits = new WeakMap<object, number>()

export function SearchPath(value: string): Executable {
  return Object.freeze({ tag: "SearchPath", value })
}

export function ExecutablePath(value: Path): Executable {
  return Object.freeze({ tag: "ExecutablePath", value })
}

export const EmptyExecutableName: ChildProcessConfigError = variant(
  "EmptyExecutableName"
)
export const ChildStdout: ChildOutputChannel = variant("ChildStdout")
export const ChildStderr: ChildOutputChannel = variant("ChildStderr")
export const ChildInputAfterClose: ChildProcessError = variant(
  "ChildInputAfterClose"
)
export const CloseChildStdin: ChildInput = variant("CloseChildStdin")
export const KillChild: ChildInput = variant("KillChild")

export function ExecutableNameContainsSeparator(
  value: string
): ChildProcessConfigError {
  return tagged("ExecutableNameContainsSeparator", value)
}

export function ArgumentContainsNul(
  value: Readonly<{ index: number; offset: number }>
): ChildProcessConfigError {
  return tagged("ArgumentContainsNul", Object.freeze({ ...value }))
}

export function EnvironmentNameContainsNul(
  value: string
): ChildProcessConfigError {
  return tagged("EnvironmentNameContainsNul", value)
}

export function EnvironmentValueContainsNul(
  value: string
): ChildProcessConfigError {
  return tagged("EnvironmentValueContainsNul", value)
}

export function InvalidCaptureLimit(value: number): ChildProcessConfigError {
  return tagged("InvalidCaptureLimit", value)
}

export function ChildSpawnFailed(
  value: Readonly<{ executable: Executable; detail: string }>
): ChildProcessError {
  return tagged("ChildSpawnFailed", Object.freeze({ ...value }))
}

export function ChildOutputReadFailed(
  value: Readonly<{ channel: ChildOutputChannel; detail: string }>
): ChildProcessError {
  return tagged("ChildOutputReadFailed", Object.freeze({ ...value }))
}

export function UnsupportedChildSignal(
  value: ProcessSignal
): ChildProcessError {
  return tagged("UnsupportedChildSignal", value)
}

export function ChildInputFailed(value: string): ChildProcessError {
  return tagged("ChildInputFailed", value)
}

export function ChildOutputLimitExceeded(
  value: Readonly<{ channel: ChildOutputChannel; limitBytes: number }>
): ChildProcessError {
  return tagged("ChildOutputLimitExceeded", Object.freeze({ ...value }))
}

export function ChildWaitFailed(value: string): ChildProcessError {
  return tagged("ChildWaitFailed", value)
}

export function ChildTerminationFailed(value: string): ChildProcessError {
  return tagged("ChildTerminationFailed", value)
}

export function ChildExited(value: number): ChildExitStatus {
  return tagged("ChildExited", value)
}

export function ChildSignaled(value: ProcessSignal): ChildExitStatus {
  return tagged("ChildSignaled", value)
}

export function ChildHostTerminated(value: string): ChildExitStatus {
  return tagged("ChildHostTerminated", value)
}

export function WriteChildStdin(value: Bytes): ChildInput {
  return tagged("WriteChildStdin", fromUint8Array(value))
}

export function SignalChild(value: ProcessSignal): ChildInput {
  return tagged("SignalChild", value)
}

export function ChildStdoutChunk(value: Uint8Array): ChildEvent {
  return tagged("ChildStdoutChunk", fromUint8Array(value))
}

export function ChildStderrChunk(value: Uint8Array): ChildEvent {
  return tagged("ChildStderrChunk", fromUint8Array(value))
}

export function ChildExitedWith(value: ChildExitStatus): ChildEvent {
  return tagged("ChildExitedWith", value)
}

export function command(
  executable: Executable
): Either<ChildProcessConfigError, Command> {
  if (executable.tag === "SearchPath") {
    if (executable.value.length === 0) return Left(EmptyExecutableName)
    if (executable.value.includes("/") || executable.value.includes("\\")) {
      return Left(ExecutableNameContainsSeparator(executable.value))
    }
  }
  return Right(
    makeCommand({
      executable,
      arguments: Object.freeze([]),
      clearEnvironment: false,
      environment: new Map(),
      terminationGraceMilliseconds: 5_000,
      outputBufferChunks: 16,
    })
  )
}

export function addArgument(
  value: string,
  current: Command
): Either<ChildProcessConfigError, Command> {
  const state = commandState(current)
  const offset = value.indexOf("\0")
  if (offset >= 0) {
    return Left(ArgumentContainsNul({ index: state.arguments.length, offset }))
  }
  return Right(
    makeCommand({
      ...state,
      arguments: Object.freeze([...state.arguments, value]),
    })
  )
}

export function addArguments(
  values: ReadonlyArray<string>,
  current: Command
): Either<ChildProcessConfigError, Command> {
  const state = commandState(current)
  for (let index = 0; index < values.length; index += 1) {
    const offset = (values[index] as string).indexOf("\0")
    if (offset >= 0) {
      return Left(
        ArgumentContainsNul({
          index: state.arguments.length + index,
          offset,
        })
      )
    }
  }
  return Right(
    makeCommand({
      ...state,
      arguments: Object.freeze([...state.arguments, ...values]),
    })
  )
}

export function inDirectory(path: Path, current: Command): Command {
  return makeCommand({ ...commandState(current), directory: path })
}

export function setEnvironment(
  name: string,
  value: string,
  current: Command
): Either<ChildProcessConfigError, Command> {
  if (name.includes("\0")) return Left(EnvironmentNameContainsNul(name))
  if (value.includes("\0")) return Left(EnvironmentValueContainsNul(value))
  return Right(withEnvironment(name, value, current))
}

export function unsetEnvironment(
  name: string,
  current: Command
): Either<ChildProcessConfigError, Command> {
  if (name.includes("\0")) return Left(EnvironmentNameContainsNul(name))
  return Right(withEnvironment(name, undefined, current))
}

export function clearEnvironment(current: Command): Command {
  return makeCommand({ ...commandState(current), clearEnvironment: true })
}

export function terminationGrace(
  duration: Duration,
  current: Command
): Command {
  const milliseconds = durationNanoseconds(duration) / 1_000_000n
  return makeCommand({
    ...commandState(current),
    terminationGraceMilliseconds: Number(
      milliseconds > BigInt(Number.MAX_SAFE_INTEGER)
        ? BigInt(Number.MAX_SAFE_INTEGER)
        : milliseconds
    ),
  })
}

export function outputBuffer(
  capacity: BufferCapacity,
  current: Command
): Command {
  return makeCommand({
    ...commandState(current),
    outputBufferChunks: capacity.value,
  })
}

export function captureLimit(
  bytes: number
): Either<ChildProcessConfigError, CaptureLimit> {
  if (!Number.isSafeInteger(bytes) || bytes <= 0) {
    return Left(InvalidCaptureLimit(bytes))
  }
  return Right(makeCaptureLimit(bytes))
}

export function defaultCaptureLimit(_unit?: Unit): CaptureLimit {
  return makeCaptureLimit(8 * 1024 * 1024)
}

export function runCaptured(
  limit: CaptureLimit,
  input: Bytes,
  current: Command
): Effect<ChildProcessesEnvironment, ChildProcessError, CapturedProcess> {
  return serviceEffect(async (environment, context) => {
    const result = await environment.childProcesses.runCaptured(
      providerCommand(current),
      toUint8Array(input),
      captureLimitValue(limit),
      context
    )
    return result.kind === "failure"
      ? result
      : serviceSuccess(
          Object.freeze({
            status: result.value.status,
            stdout: fromUint8Array(result.value.stdout),
            stderr: fromUint8Array(result.value.stderr),
          })
        )
  })
}

export function runInherited(
  current: Command
): Effect<ChildProcessesEnvironment, ChildProcessError, ChildExitStatus> {
  return serviceEffect((environment, context) =>
    environment.childProcesses.runInherited(providerCommand(current), context)
  )
}

export function runStreaming<Environment, Failure>(
  input: Stream<Environment, Failure, ChildInput>,
  current: Command
): Stream<
  Environment & ChildProcessesEnvironment,
  Either<Failure, ChildProcessError>,
  ChildEvent
> {
  return Object.freeze({
    open(
      environment: Environment & ChildProcessesEnvironment,
      context: EffectContext
    ) {
      return environment.childProcesses.openStreaming(
        input,
        providerCommand(current),
        environment,
        context
      )
    },
  }) as unknown as Stream<
    Environment & ChildProcessesEnvironment,
    Either<Failure, ChildProcessError>,
    ChildEvent
  >
}

export async function attemptInputPull<Environment, Failure>(
  cursor: StreamCursor<ChildInput>,
  environment: Environment,
  context: EffectContext
): Promise<Either<Failure, IteratorResult<ChildInput>>> {
  return attempt((() => cursor.next()) as Effect<
    Environment,
    Failure,
    IteratorResult<ChildInput>
  >)(environment, context)
}

export function childProcessFailure(
  error: ChildProcessError
): ServiceResult<ChildProcessError, never> {
  return serviceFailure(error)
}

export function providerCommand(current: Command): ProviderCommand {
  const state = commandState(current)
  return Object.freeze({
    executable:
      state.executable.tag === "SearchPath"
        ? Object.freeze({
            tag: "search-path" as const,
            value: state.executable.value,
          })
        : Object.freeze({
            tag: "path" as const,
            value: renderPath(state.executable.value),
          }),
    arguments: Object.freeze([...state.arguments]),
    directory: state.directory === undefined ? "" : renderPath(state.directory),
    hasDirectory: state.directory !== undefined,
    clearEnvironment: state.clearEnvironment,
    environment: Object.freeze(
      [...state.environment].map(([name, value]) =>
        Object.freeze({
          name,
          value: value ?? "",
          unset: value === undefined,
        })
      )
    ),
    terminationGraceMilliseconds: state.terminationGraceMilliseconds,
    outputBufferChunks: state.outputBufferChunks,
  })
}

function withEnvironment(
  name: string,
  value: string | undefined,
  current: Command
): Command {
  const state = commandState(current)
  const environment = new Map(state.environment)
  environment.set(name, value)
  return makeCommand({ ...state, environment })
}

function makeCommand(state: CommandState): Command {
  const value = Object.freeze({ [commandBrand]: true }) as Command
  commandStates.set(
    value,
    Object.freeze({
      ...state,
      arguments: Object.freeze([...state.arguments]),
      environment: new Map(state.environment),
    })
  )
  return value
}

function commandState(value: Command): CommandState {
  const state = commandStates.get(value)
  if (state === undefined) {
    throw new TypeError("Command value does not use the runtime brand")
  }
  return state
}

function makeCaptureLimit(bytes: number): CaptureLimit {
  const value = Object.freeze({ [captureLimitBrand]: true }) as CaptureLimit
  captureLimits.set(value, bytes)
  return value
}

function captureLimitValue(value: CaptureLimit): number {
  const bytes = captureLimits.get(value)
  if (bytes === undefined) {
    throw new TypeError("CaptureLimit value does not use the runtime brand")
  }
  return bytes
}

function variant<Tag extends string>(tag: Tag): Readonly<{ tag: Tag }> {
  return Object.freeze({ tag })
}

function tagged<Tag extends string, Value>(
  tag: Tag,
  value: Value
): Readonly<{ tag: Tag; value: Value }> {
  return Object.freeze({ tag, value })
}
