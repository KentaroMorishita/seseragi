import {
  attemptInputPull,
  type ChildEvent,
  type ChildExitStatus,
  type ChildInput,
  type ChildProcesses,
  type ChildProcessError,
  ChildStderr,
  ChildStderrChunk,
  ChildStdout,
  ChildStdoutChunk,
  type ProviderCapturedProcess,
  type ProviderCommand,
  type ProviderStreamingFailure,
} from "./child-process"
import type { EffectContext } from "./effect"
import { pathFromProvider } from "./path"
import type { ProcessSignal } from "./process"
import {
  invokeProviderOperation,
  openProviderSubscription,
  type ProviderBridgeOutcome,
  ProviderCodecRegistry,
  type ProviderLogicalType,
  type ProviderOperationContract,
} from "./provider"
import type { LoadedProviderEntry } from "./provider-package"
import { type ServiceResult, serviceFailure, serviceSuccess } from "./service"
import type { Stream, StreamCursor } from "./stream"
import { type Either, Left, Right } from "./sum"

const named = (identity: string) =>
  Object.freeze({ kind: "named", identity } as const)
const primitive = (name: "bytes" | "int" | "string") =>
  Object.freeze({ kind: "primitive", name } as const)
const record = (
  fields: ReadonlyArray<Readonly<{ name: string; type: ProviderLogicalType }>>
) => Object.freeze({ kind: "record", fields: Object.freeze(fields) } as const)

const commandType = named("std/child-process::CommandRequest")
const errorType = named("std/child-process::ChildProcessError")
const capturedType = named("std/child-process::CapturedProcess")
const statusType = named("std/child-process::ChildExitStatus")
const eventType = named("std/child-process::ChildEvent")
const streamingFailureType = named("std/child-process::StreamingFailure")

const runCapturedContract = operation(
  "runCaptured",
  "one-shot",
  record([
    { name: "command", type: commandType },
    { name: "input", type: primitive("bytes") },
    { name: "limitBytes", type: primitive("int") },
  ]),
  capturedType,
  errorType
)
const runInheritedContract = operation(
  "runInherited",
  "one-shot",
  commandType,
  statusType,
  errorType
)
const runStreamingContract = operation(
  "runStreaming",
  "subscription",
  commandType,
  eventType,
  streamingFailureType
)

export function createProviderChildProcesses(
  loaded: LoadedProviderEntry,
  applicationDirectory?: string
): ChildProcesses {
  if (loaded.service !== "std/child-process::ChildProcesses") {
    throw new TypeError(
      "resolved provider does not implement std/child-process::ChildProcesses"
    )
  }
  const codecs = codecsFor()
  return Object.freeze({
    async runCaptured(
      command: ProviderCommand,
      input: Uint8Array,
      limitBytes: number,
      context: EffectContext
    ) {
      return capturedResult(
        await invokeProviderOperation({
          provider: loaded.provider,
          service: loaded.service,
          operation: runCapturedContract,
          entry: loaded.entry,
          input: {
            command: withApplicationDirectory(command, applicationDirectory),
            input,
            limitBytes,
          },
          codecs,
          context,
        })
      )
    },
    async runInherited(command: ProviderCommand, context: EffectContext) {
      return statusResult(
        await invokeProviderOperation({
          provider: loaded.provider,
          service: loaded.service,
          operation: runInheritedContract,
          entry: loaded.entry,
          input: withApplicationDirectory(command, applicationDirectory),
          codecs,
          context,
        })
      )
    },
    async openStreaming<Environment, Failure>(
      input: Stream<Environment, Failure, ChildInput>,
      command: ProviderCommand,
      environment: Environment,
      context: EffectContext
    ): Promise<StreamCursor<ChildEvent>> {
      const inputCursor = await input.open(environment, context)
      const source = openProviderSubscription({
        provider: loaded.provider,
        service: loaded.service,
        operation: runStreamingContract,
        entry: loaded.entry,
        input: withApplicationDirectory(command, applicationDirectory),
        codecs,
        context,
        attachment: Object.freeze({
          next: () =>
            attemptInputPull(inputCursor, environment, context).then(
              (result) =>
                result.tag === "Left"
                  ? Object.freeze({
                      kind: "failure" as const,
                      error: result.value,
                    })
                  : Object.freeze({
                      kind: "result" as const,
                      value: result.value,
                    })
            ),
          close: inputCursor.close,
        }),
      })
      let closing: Promise<void> | undefined
      const close = (): Promise<void> => {
        closing ??= Promise.allSettled([
          source.close(),
          inputCursor.close(),
        ]).then((outcomes) => {
          const failed = outcomes.find(
            (outcome): outcome is PromiseRejectedResult =>
              outcome.status === "rejected"
          )
          if (failed !== undefined) throw failed.reason
        })
        return closing
      }
      return Object.freeze({
        next: () => source.pull(context) as Promise<IteratorResult<ChildEvent>>,
        close,
      })
    },
  }) as unknown as ChildProcesses
}

function withApplicationDirectory(
  command: ProviderCommand,
  applicationDirectory?: string
): ProviderCommand {
  if (
    command.hasDirectory ||
    applicationDirectory === undefined ||
    applicationDirectory.length === 0
  ) {
    return command
  }
  return Object.freeze({
    ...command,
    directory: applicationDirectory,
    hasDirectory: true,
  })
}

function operation(
  name: string,
  kind: ProviderOperationContract["kind"],
  input: ProviderLogicalType,
  success: ProviderLogicalType,
  failure: ProviderLogicalType
): ProviderOperationContract {
  return Object.freeze({
    identity: `std/child-process::ChildProcesses#${name}`,
    kind,
    input,
    success,
    failure,
  })
}

function codecsFor(): ProviderCodecRegistry {
  return new ProviderCodecRegistry([
    namedCodec(commandType.identity, validateCommand),
    namedCodec(errorType.identity, validateError),
    namedCodec(capturedType.identity, validateCaptured),
    namedCodec(statusType.identity, validateStatus),
    namedCodec(eventType.identity, validateEvent),
    namedCodec(streamingFailureType.identity, validateStreamingFailure),
  ])
}

function namedCodec(identity: string, validate: (value: unknown) => unknown) {
  return { identity, encode: validate, decode: validate }
}

function capturedResult(
  outcome: ProviderBridgeOutcome
): ServiceResult<ChildProcessError, ProviderCapturedProcess> {
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    return serviceFailure(validateError(outcome.failure))
  }
  return serviceSuccess(validateCaptured(outcome.value))
}

function statusResult(
  outcome: ProviderBridgeOutcome
): ServiceResult<ChildProcessError, ChildExitStatus> {
  if (outcome.kind === "defect") throw outcome.defect
  if (outcome.kind === "failure") {
    return serviceFailure(validateError(outcome.failure))
  }
  return serviceSuccess(validateStatus(outcome.value))
}

function validateCommand(value: unknown): ProviderCommand {
  if (typeof value !== "object" || value === null) invalid("command")
  const command = value as ProviderCommand
  if (
    (command.executable?.tag !== "search-path" &&
      command.executable?.tag !== "path") ||
    typeof command.executable.value !== "string" ||
    !Array.isArray(command.arguments) ||
    command.arguments.some((argument) => typeof argument !== "string") ||
    typeof command.directory !== "string" ||
    typeof command.hasDirectory !== "boolean" ||
    typeof command.clearEnvironment !== "boolean" ||
    !Array.isArray(command.environment) ||
    command.environment.some(
      (entry) =>
        typeof entry !== "object" ||
        entry === null ||
        typeof entry.name !== "string" ||
        typeof entry.value !== "string" ||
        typeof entry.unset !== "boolean"
    ) ||
    !Number.isSafeInteger(command.terminationGraceMilliseconds) ||
    command.terminationGraceMilliseconds < 0 ||
    !Number.isSafeInteger(command.outputBufferChunks) ||
    command.outputBufferChunks <= 0
  ) {
    invalid("command")
  }
  return Object.freeze({
    executable: Object.freeze({ ...command.executable }),
    arguments: Object.freeze([...command.arguments]),
    directory: command.directory,
    hasDirectory: command.hasDirectory,
    clearEnvironment: command.clearEnvironment,
    environment: Object.freeze(
      command.environment.map((entry) => Object.freeze({ ...entry }))
    ),
    terminationGraceMilliseconds: command.terminationGraceMilliseconds,
    outputBufferChunks: command.outputBufferChunks,
  })
}

function validateCaptured(value: unknown): ProviderCapturedProcess {
  if (typeof value !== "object" || value === null) invalid("captured result")
  const captured = value as ProviderCapturedProcess
  if (
    !(captured.stdout instanceof Uint8Array) ||
    !(captured.stderr instanceof Uint8Array)
  ) {
    invalid("captured result")
  }
  return Object.freeze({
    status: validateStatus(captured.status),
    stdout: new Uint8Array(captured.stdout),
    stderr: new Uint8Array(captured.stderr),
  })
}

function validateStatus(value: unknown): ChildExitStatus {
  if (typeof value !== "object" || value === null) invalid("exit status")
  const status = value as ChildExitStatus
  if (
    status.tag === "ChildExited" &&
    Number.isSafeInteger(status.value) &&
    status.value >= 0
  ) {
    return Object.freeze({ tag: status.tag, value: status.value })
  }
  if (status.tag === "ChildSignaled" && validSignal(status.value)) {
    return Object.freeze({
      tag: status.tag,
      value: Object.freeze({ tag: status.value.tag }),
    })
  }
  if (
    status.tag === "ChildHostTerminated" &&
    typeof status.value === "string"
  ) {
    return Object.freeze({ tag: status.tag, value: status.value })
  }
  return invalid("exit status")
}

function validateEvent(value: unknown): ChildEvent {
  if (typeof value !== "object" || value === null) invalid("child event")
  const event = value as ChildEvent
  if (
    (event.tag === "ChildStdoutChunk" || event.tag === "ChildStderrChunk") &&
    event.value instanceof Uint8Array
  ) {
    return event.tag === "ChildStdoutChunk"
      ? ChildStdoutChunk(event.value)
      : ChildStderrChunk(event.value)
  }
  if (event.tag === "ChildExitedWith") {
    return Object.freeze({ tag: event.tag, value: validateStatus(event.value) })
  }
  return invalid("child event")
}

function validateStreamingFailure(
  value: unknown
): Either<unknown, ChildProcessError> {
  if (typeof value !== "object" || value === null) {
    return Right(validateError(value))
  }
  const failure = value as ProviderStreamingFailure<unknown>
  if (failure.kind === "input" && "error" in failure) {
    return Left(failure.error)
  }
  if (failure.kind === "child" && "error" in failure) {
    return Right(validateError(failure.error))
  }
  return invalid("streaming failure")
}

function validateError(value: unknown): ChildProcessError {
  if (typeof value !== "object" || value === null) invalid("child failure")
  const error = value as ChildProcessError
  switch (error.tag) {
    case "ChildInputAfterClose":
      return Object.freeze({ tag: error.tag })
    case "UnsupportedChildSignal":
      if (!validSignal(error.value)) invalid("child failure")
      return Object.freeze({
        tag: error.tag,
        value: Object.freeze({ tag: error.value.tag }),
      })
    case "ChildInputFailed":
    case "ChildWaitFailed":
    case "ChildTerminationFailed":
      if (typeof error.value !== "string") invalid("child failure")
      return Object.freeze({ tag: error.tag, value: error.value })
    case "ChildSpawnFailed":
      if (
        typeof error.value?.detail !== "string" ||
        typeof error.value?.executable !== "object" ||
        error.value.executable === null ||
        (error.value.executable.tag !== "SearchPath" &&
          error.value.executable.tag !== "ExecutablePath") ||
        typeof error.value.executable.value !== "string"
      ) {
        invalid("child failure")
      }
      return Object.freeze({
        tag: error.tag,
        value: Object.freeze({
          executable:
            error.value.executable.tag === "SearchPath"
              ? Object.freeze({
                  tag: "SearchPath" as const,
                  value: error.value.executable.value,
                })
              : Object.freeze({
                  tag: "ExecutablePath" as const,
                  value: pathFromProvider(error.value.executable.value),
                }),
          detail: error.value.detail,
        }),
      })
    case "ChildOutputReadFailed":
      if (
        typeof error.value?.detail !== "string" ||
        (error.value?.channel?.tag !== "ChildStdout" &&
          error.value?.channel?.tag !== "ChildStderr")
      ) {
        invalid("child failure")
      }
      return Object.freeze({
        tag: error.tag,
        value: Object.freeze({
          channel:
            error.value.channel.tag === "ChildStdout"
              ? ChildStdout
              : ChildStderr,
          detail: error.value.detail,
        }),
      })
    case "ChildOutputLimitExceeded":
      if (
        !Number.isSafeInteger(error.value?.limitBytes) ||
        error.value.limitBytes <= 0 ||
        (error.value?.channel?.tag !== "ChildStdout" &&
          error.value?.channel?.tag !== "ChildStderr")
      ) {
        invalid("child failure")
      }
      return Object.freeze({
        tag: error.tag,
        value: Object.freeze({
          channel:
            error.value.channel.tag === "ChildStdout"
              ? ChildStdout
              : ChildStderr,
          limitBytes: error.value.limitBytes,
        }),
      })
  }
}

function validSignal(value: unknown): value is ProcessSignal {
  const tag = (value as { tag?: unknown } | null)?.tag
  return (
    typeof value === "object" &&
    value !== null &&
    (tag === "Interrupt" ||
      tag === "Terminate" ||
      tag === "Hangup" ||
      tag === "Quit" ||
      tag === "User1" ||
      tag === "User2")
  )
}

function invalid(subject: string): never {
  throw new TypeError(`${subject} is invalid`)
}
