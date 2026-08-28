import {
  spawn,
  type ChildProcess,
  type ChildProcessWithoutNullStreams,
  type SpawnOptions,
} from "node:child_process"
import nodeProcess from "node:process"
import { isAbsolute, resolve } from "node:path"
import {
  type ProviderResult,
  type ProviderSubscriptionObserver,
  providerRuntimeAbi,
  withProviderCancellation,
} from "@seseragi/runtime/provider"
import {
  defineProviderPackage,
  type ProviderPackageEntry,
  type ProviderRuntimeTarget,
} from "@seseragi/runtime/provider-package"

type ProviderExecutable =
  | Readonly<{ tag: "search-path"; value: string }>
  | Readonly<{ tag: "path"; value: string }>

type CommandRequest = Readonly<{
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

type ChildSignal =
  | Readonly<{ tag: "Interrupt" }>
  | Readonly<{ tag: "Terminate" }>
  | Readonly<{ tag: "Hangup" }>
  | Readonly<{ tag: "Quit" }>
  | Readonly<{ tag: "User1" }>
  | Readonly<{ tag: "User2" }>

type ChildStatus =
  | Readonly<{ tag: "ChildExited"; value: number }>
  | Readonly<{ tag: "ChildSignaled"; value: ChildSignal }>
  | Readonly<{ tag: "ChildHostTerminated"; value: string }>

type ChildChannel =
  | Readonly<{ tag: "ChildStdout" }>
  | Readonly<{ tag: "ChildStderr" }>

type ChildFailure = Readonly<{
  tag: string
  value?: unknown
}>

type ChildInput =
  | Readonly<{ tag: "WriteChildStdin"; value: Uint8Array }>
  | Readonly<{ tag: "CloseChildStdin" }>
  | Readonly<{ tag: "SignalChild"; value: ChildSignal }>
  | Readonly<{ tag: "KillChild" }>

type InputAttachment = Readonly<{
  next: () => Promise<
    | Readonly<{ kind: "failure"; error: unknown }>
    | Readonly<{
        kind: "result"
        value: IteratorResult<ChildInput>
      }>
  >
  close: () => Promise<void>
}>

type RunState = {
  readonly child: ChildProcess
  readonly command: CommandRequest
  readonly exited: Promise<void>
  resolveExited: () => void
  settled: boolean
  terminating?: Promise<void>
}

type StreamQueueEntry = Readonly<{
  event: unknown
  channel?: "stdout" | "stderr"
}>

const stdoutChannel: ChildChannel = Object.freeze({ tag: "ChildStdout" })
const stderrChannel: ChildChannel = Object.freeze({ tag: "ChildStderr" })

export function createChildProcessesProvider(
  provider: string,
  target: ProviderRuntimeTarget
): ProviderPackageEntry {
  const active = new Set<RunState>()
  return defineProviderPackage({
    abi: providerRuntimeAbi,
    provider,
    service: "std/child-process::ChildProcesses",
    targets: [target],
    operations: {
      runCaptured(value) {
        const request = capturedRequest(value)
        let state: RunState | undefined
        const completion = (async (): Promise<ProviderResult> => {
          try {
            state = start(request.command, "pipe", active) as RunState & {
              child: ChildProcessWithoutNullStreams
            }
            const child = state.child as ChildProcessWithoutNullStreams
            const [stdout, stderr, status] = await Promise.all([
              captureChannel(
                child.stdout,
                stdoutChannel,
                request.limitBytes
              ),
              captureChannel(
                child.stderr,
                stderrChannel,
                request.limitBytes
              ),
              waitStatus(state),
              writeAll(child, request.input),
            ])
            return success(
              Object.freeze({ status, stdout, stderr })
            )
          } catch (cause) {
            if (state !== undefined) {
              await terminateAndReap(state).catch(() => undefined)
            }
            return failure(childFailure(cause, state?.command ?? request.command))
          } finally {
            if (state !== undefined) active.delete(state)
          }
        })()
        return withProviderCancellation(completion, async () => {
          if (state !== undefined) await terminateAndReap(state)
        })
      },
      runInherited(value) {
        const command = commandRequest(value)
        let state: RunState | undefined
        const completion = (async (): Promise<ProviderResult> => {
          try {
            state = start(command, "inherit", active)
            return success(await waitStatus(state))
          } catch (cause) {
            if (state !== undefined) {
              await terminateAndReap(state).catch(() => undefined)
            }
            return failure(childFailure(cause, command))
          } finally {
            if (state !== undefined) active.delete(state)
          }
        })()
        return withProviderCancellation(completion, async () => {
          if (state !== undefined) await terminateAndReap(state)
        })
      },
      runStreaming(value, observer, attachment) {
        return streamChild(
          commandRequest(value),
          subscriptionObserver(observer),
          inputAttachment(attachment),
          active
        )
      },
    },
    async shutdown() {
      await Promise.allSettled([...active].map(terminateAndReap))
    },
  })
}

function capturedRequest(value: unknown): Readonly<{
  command: CommandRequest
  input: Uint8Array
  limitBytes: number
}> {
  const request = dataRecord(value, ["command", "input", "limitBytes"])
  if (!(request.input instanceof Uint8Array)) {
    throw new TypeError("child input must be Bytes")
  }
  if (
    !Number.isSafeInteger(request.limitBytes) ||
    (request.limitBytes as number) <= 0
  ) {
    throw new TypeError("child capture limit must be positive")
  }
  return Object.freeze({
    command: commandRequest(request.command),
    input: new Uint8Array(request.input),
    limitBytes: request.limitBytes as number,
  })
}

function commandRequest(value: unknown): CommandRequest {
  const request = dataRecord(value, [
    "arguments",
    "clearEnvironment",
    "directory",
    "environment",
    "executable",
    "hasDirectory",
    "outputBufferChunks",
    "terminationGraceMilliseconds",
  ])
  const executable = dataRecord(request.executable, ["tag", "value"])
  if (
    (executable.tag !== "search-path" && executable.tag !== "path") ||
    typeof executable.value !== "string" ||
    !Array.isArray(request.arguments) ||
    request.arguments.some((argument) => typeof argument !== "string") ||
    typeof request.directory !== "string" ||
    typeof request.hasDirectory !== "boolean" ||
    typeof request.clearEnvironment !== "boolean" ||
    !Array.isArray(request.environment) ||
    !Number.isSafeInteger(request.terminationGraceMilliseconds) ||
    (request.terminationGraceMilliseconds as number) < 0 ||
    !Number.isSafeInteger(request.outputBufferChunks) ||
    (request.outputBufferChunks as number) <= 0
  ) {
    throw new TypeError("child command is invalid")
  }
  const environment = request.environment.map((entry) => {
    const item = dataRecord(entry, ["name", "unset", "value"])
    if (
      typeof item.name !== "string" ||
      typeof item.value !== "string" ||
      typeof item.unset !== "boolean"
    ) {
      throw new TypeError("child environment entry is invalid")
    }
    return Object.freeze({
      name: item.name,
      value: item.value,
      unset: item.unset,
    })
  })
  return Object.freeze({
    executable: Object.freeze({
      tag: executable.tag,
      value: executable.value,
    }) as ProviderExecutable,
    arguments: Object.freeze([...(request.arguments as string[])]),
    directory: request.directory,
    hasDirectory: request.hasDirectory,
    clearEnvironment: request.clearEnvironment,
    environment: Object.freeze(environment),
    terminationGraceMilliseconds:
      request.terminationGraceMilliseconds as number,
    outputBufferChunks: request.outputBufferChunks as number,
  })
}

function start(
  command: CommandRequest,
  stdio: "inherit" | "pipe",
  active: Set<RunState>
): RunState {
  const options: SpawnOptions = {
    stdio,
    env: environmentFor(command),
    ...(command.hasDirectory ? { cwd: command.directory } : {}),
  }
  let resolveExited = (): void => undefined
  const exited = new Promise<void>((resolveExit) => {
    resolveExited = resolveExit
  })
  let child: ChildProcess
  try {
    child = spawn(executableFor(command), [...command.arguments], options)
  } catch (cause) {
    throw new SpawnFailure(errorMessage(cause))
  }
  const state: RunState = {
    child,
    command,
    exited,
    resolveExited,
    settled: false,
  }
  active.add(state)
  const settle = (): void => {
    if (state.settled) return
    state.settled = true
    state.resolveExited()
  }
  child.once("close", settle)
  child.once("error", settle)
  return state
}

function executableFor(command: CommandRequest): string {
  if (command.executable.tag === "search-path") {
    return command.executable.value
  }
  if (isAbsolute(command.executable.value)) return command.executable.value
  return resolve(
    command.hasDirectory ? command.directory : nodeProcess.cwd(),
    command.executable.value
  )
}

function environmentFor(command: CommandRequest): NodeJS.ProcessEnv {
  const environment: NodeJS.ProcessEnv = command.clearEnvironment
    ? {}
    : { ...nodeProcess.env }
  for (const entry of command.environment) {
    if (entry.unset) delete environment[entry.name]
    else environment[entry.name] = entry.value
  }
  return environment
}

function waitStatus(state: RunState): Promise<ChildStatus> {
  return new Promise((resolveStatus, reject) => {
    let spawned = false
    state.child.once("spawn", () => {
      spawned = true
    })
    state.child.once("error", (cause) => {
      reject(
        spawned
          ? new WaitFailure(errorMessage(cause))
          : new SpawnFailure(errorMessage(cause))
      )
    })
    state.child.once("close", (code, signal) => {
      if (code !== null) {
        resolveStatus(
          Object.freeze({
            tag: "ChildExited",
            value: Math.max(0, code),
          })
        )
        return
      }
      resolveStatus(statusForSignal(signal))
    })
  })
}

async function writeAll(
  child: ChildProcessWithoutNullStreams,
  input: Uint8Array
): Promise<void> {
  if (input.length > 0) await writeChunk(child, input)
  await closeInput(child)
}

function writeChunk(
  child: ChildProcessWithoutNullStreams,
  input: Uint8Array
): Promise<void> {
  return new Promise((resolveWrite, reject) => {
    const onError = (cause: Error): void => {
      cleanup()
      reject(new InputFailure(errorMessage(cause)))
    }
    const onDrain = (): void => {
      cleanup()
      resolveWrite()
    }
    const cleanup = (): void => {
      child.stdin.off("error", onError)
      child.stdin.off("drain", onDrain)
    }
    child.stdin.once("error", onError)
    const accepted = child.stdin.write(new Uint8Array(input))
    if (accepted) {
      cleanup()
      resolveWrite()
    } else {
      child.stdin.once("drain", onDrain)
    }
  })
}

function closeInput(child: ChildProcessWithoutNullStreams): Promise<void> {
  if (child.stdin.destroyed || child.stdin.writableEnded) {
    return Promise.resolve()
  }
  return new Promise((resolveClose, reject) => {
    const onError = (cause: Error): void => {
      child.stdin.off("error", onError)
      reject(new InputFailure(errorMessage(cause)))
    }
    child.stdin.once("error", onError)
    child.stdin.end(() => {
      child.stdin.off("error", onError)
      resolveClose()
    })
  })
}

function captureChannel(
  stream: NodeJS.ReadableStream,
  channel: ChildChannel,
  limitBytes: number
): Promise<Uint8Array> {
  return new Promise((resolveCapture, reject) => {
    const chunks: Uint8Array[] = []
    let length = 0
    stream.on("data", (value: unknown) => {
      const chunk = bytes(value)
      length += chunk.length
      if (length > limitBytes) {
        reject(new OutputLimitFailure(channel, limitBytes))
        return
      }
      chunks.push(chunk)
    })
    stream.once("error", (cause) => {
      reject(new OutputReadFailure(channel, errorMessage(cause)))
    })
    stream.once("end", () => resolveCapture(concat(chunks, length)))
  })
}

function streamChild(
  command: CommandRequest,
  observer: ProviderSubscriptionObserver,
  input: InputAttachment,
  active: Set<RunState>
): Readonly<{
  demand: (count: number) => void
  unsubscribe: () => Promise<void>
}> {
  let state: RunState | undefined
  let child: ChildProcessWithoutNullStreams | undefined
  let demand = 0
  let closed = false
  let terminal = false
  let stdinClosed = false
  let stdoutQueued = 0
  let stderrQueued = 0
  const queue: StreamQueueEntry[] = []

  const flush = (): void => {
    if (terminal || closed) return
    while (demand > 0 && queue.length > 0) {
      const next = queue.shift() as StreamQueueEntry
      demand -= 1
      if (next.channel === "stdout") stdoutQueued -= 1
      if (next.channel === "stderr") stderrQueued -= 1
      observer.next(next.event)
    }
    if (stdoutQueued < command.outputBufferChunks) child?.stdout.resume()
    if (stderrQueued < command.outputBufferChunks) child?.stderr.resume()
    if (state?.settled && queue.length === 0) {
      terminal = true
      observer.complete()
    }
  }

  const enqueue = (
    event: unknown,
    channel?: "stdout" | "stderr"
  ): void => {
    if (terminal || closed) return
    if (channel === "stdout") stdoutQueued += 1
    if (channel === "stderr") stderrQueued += 1
    queue.push(Object.freeze({ event, ...(channel === undefined ? {} : { channel }) }))
    if (stdoutQueued >= command.outputBufferChunks) child?.stdout.pause()
    if (stderrQueued >= command.outputBufferChunks) child?.stderr.pause()
    flush()
  }

  const fail = async (failureValue: unknown): Promise<void> => {
    if (terminal || closed) return
    terminal = true
    queue.length = 0
    await input.close().catch(() => undefined)
    if (state !== undefined) {
      await terminateAndReap(state).catch(() => undefined)
      active.delete(state)
    }
    observer.failure(failureValue)
  }

  try {
    state = start(command, "pipe", active)
    child = state.child as ChildProcessWithoutNullStreams
    child.stdout.on("data", (value: unknown) => {
      for (const chunk of split(bytes(value))) {
        enqueue(
          Object.freeze({ tag: "ChildStdoutChunk", value: chunk }),
          "stdout"
        )
      }
    })
    child.stderr.on("data", (value: unknown) => {
      for (const chunk of split(bytes(value))) {
        enqueue(
          Object.freeze({ tag: "ChildStderrChunk", value: chunk }),
          "stderr"
        )
      }
    })
    child.stdout.once("error", (cause) => {
      void fail(
        streamingChildFailure(
          new OutputReadFailure(stdoutChannel, errorMessage(cause)),
          command
        )
      )
    })
    child.stderr.once("error", (cause) => {
      void fail(
        streamingChildFailure(
          new OutputReadFailure(stderrChannel, errorMessage(cause)),
          command
        )
      )
    })
    void waitStatus(state).then(
      async (status) => {
        await input.close().catch(() => undefined)
        enqueue(Object.freeze({ tag: "ChildExitedWith", value: status }))
        active.delete(state as RunState)
        flush()
      },
      (cause: unknown) => {
        void fail(streamingChildFailure(cause, command))
      }
    )
    void pumpInput(child, input, () => stdinClosed, () => {
      stdinClosed = true
    }).then(
      () => undefined,
      (cause: unknown) => {
        if (cause instanceof InputStreamFailure) {
          void fail(Object.freeze({ kind: "input", error: cause.error }))
        } else {
          void fail(streamingChildFailure(cause, command))
        }
      }
    )
  } catch (cause) {
    terminal = true
    observer.failure(streamingChildFailure(cause, command))
  }

  return Object.freeze({
    demand(count: number) {
      if (terminal || closed) return
      demand += count
      flush()
    },
    async unsubscribe() {
      if (closed) return
      closed = true
      queue.length = 0
      await input.close().catch(() => undefined)
      if (state !== undefined) {
        await terminateAndReap(state)
        active.delete(state)
      }
    },
  })
}

async function pumpInput(
  child: ChildProcessWithoutNullStreams,
  input: InputAttachment,
  isClosed: () => boolean,
  markClosed: () => void
): Promise<void> {
  while (true) {
    const pulled = await input.next()
    if (pulled.kind === "failure") {
      throw new InputStreamFailure(pulled.error)
    }
    if (pulled.value.done) {
      if (!isClosed()) {
        await closeInput(child)
        markClosed()
      }
      return
    }
    const event = childInput(pulled.value.value)
    switch (event.tag) {
      case "WriteChildStdin":
        if (isClosed()) throw new InputAfterCloseFailure()
        await writeChunk(child, event.value)
        break
      case "CloseChildStdin":
        if (!isClosed()) {
          await closeInput(child)
          markClosed()
        }
        break
      case "SignalChild":
        signalChild(child, event.value)
        break
      case "KillChild":
        if (!child.kill("SIGKILL") && child.exitCode === null) {
          throw new TerminationFailure("forced child termination failed")
        }
        break
    }
  }
}

function childInput(value: unknown): ChildInput {
  if (typeof value !== "object" || value === null) {
    throw new InputFailure("child input event is invalid")
  }
  const input = value as ChildInput
  if (
    input.tag === "WriteChildStdin" &&
    input.value instanceof Uint8Array
  ) {
    return Object.freeze({
      tag: input.tag,
      value: new Uint8Array(input.value),
    })
  }
  if (input.tag === "CloseChildStdin" || input.tag === "KillChild") {
    return Object.freeze({ tag: input.tag })
  }
  if (input.tag === "SignalChild" && validSignal(input.value)) {
    return Object.freeze({
      tag: input.tag,
      value: Object.freeze({ tag: input.value.tag }),
    })
  }
  throw new InputFailure("child input event is invalid")
}

function signalChild(child: ChildProcess, signal: ChildSignal): void {
  const hostSignal = hostSignalFor(signal)
  if (
    nodeProcess.platform === "win32" &&
    hostSignal !== "SIGINT" &&
    hostSignal !== "SIGTERM"
  ) {
    throw new UnsupportedSignalFailure(signal)
  }
  if (!child.kill(hostSignal) && child.exitCode === null) {
    throw new InputFailure(`child signal ${hostSignal} failed`)
  }
}

async function terminateAndReap(state: RunState): Promise<void> {
  if (state.terminating !== undefined) return state.terminating
  state.terminating = (async () => {
    if (state.settled) return
    state.child.stdin?.end()
    try {
      state.child.kill("SIGTERM")
    } catch {
      // A target that cannot terminate gracefully proceeds to force kill.
    }
    const graceful = await waitForExit(
      state.exited,
      state.command.terminationGraceMilliseconds
    )
    if (!graceful) {
      try {
        if (!state.child.kill("SIGKILL") && !state.settled) {
          throw new Error("host rejected forced child termination")
        }
      } catch (cause) {
        throw new TerminationFailure(errorMessage(cause))
      }
      await state.exited
    }
  })()
  return state.terminating
}

async function waitForExit(
  exited: Promise<void>,
  graceMilliseconds: number
): Promise<boolean> {
  let timer: ReturnType<typeof setTimeout> | undefined
  const timedOut = await Promise.race([
    exited.then(() => false),
    new Promise<boolean>((resolveTimeout) => {
      timer = setTimeout(() => resolveTimeout(true), graceMilliseconds)
    }),
  ])
  if (timer !== undefined) clearTimeout(timer)
  return !timedOut
}

function statusForSignal(signal: NodeJS.Signals | null): ChildStatus {
  const portable = portableSignal(signal)
  return portable === undefined
    ? Object.freeze({
        tag: "ChildHostTerminated",
        value: signal ?? "host terminated",
      })
    : Object.freeze({ tag: "ChildSignaled", value: portable })
}

function portableSignal(signal: NodeJS.Signals | null): ChildSignal | undefined {
  switch (signal) {
    case "SIGINT":
      return Object.freeze({ tag: "Interrupt" })
    case "SIGTERM":
      return Object.freeze({ tag: "Terminate" })
    case "SIGHUP":
      return Object.freeze({ tag: "Hangup" })
    case "SIGQUIT":
      return Object.freeze({ tag: "Quit" })
    case "SIGUSR1":
      return Object.freeze({ tag: "User1" })
    case "SIGUSR2":
      return Object.freeze({ tag: "User2" })
    default:
      return undefined
  }
}

function hostSignalFor(signal: ChildSignal): NodeJS.Signals {
  switch (signal.tag) {
    case "Interrupt":
      return "SIGINT"
    case "Terminate":
      return "SIGTERM"
    case "Hangup":
      return "SIGHUP"
    case "Quit":
      return "SIGQUIT"
    case "User1":
      return "SIGUSR1"
    case "User2":
      return "SIGUSR2"
  }
}

function validSignal(value: unknown): value is ChildSignal {
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

function childFailure(
  cause: unknown,
  command: CommandRequest
): ChildFailure {
  if (cause instanceof SpawnFailure) {
    return Object.freeze({
      tag: "ChildSpawnFailed",
      value: Object.freeze({
        executable: applicationExecutable(command.executable),
        detail: cause.message,
      }),
    })
  }
  if (cause instanceof OutputLimitFailure) {
    return Object.freeze({
      tag: "ChildOutputLimitExceeded",
      value: Object.freeze({
        channel: cause.channel,
        limitBytes: cause.limitBytes,
      }),
    })
  }
  if (cause instanceof OutputReadFailure) {
    return Object.freeze({
      tag: "ChildOutputReadFailed",
      value: Object.freeze({
        channel: cause.channel,
        detail: cause.message,
      }),
    })
  }
  if (cause instanceof UnsupportedSignalFailure) {
    return Object.freeze({
      tag: "UnsupportedChildSignal",
      value: cause.signal,
    })
  }
  if (cause instanceof InputAfterCloseFailure) {
    return Object.freeze({ tag: "ChildInputAfterClose" })
  }
  if (cause instanceof InputFailure) {
    return Object.freeze({ tag: "ChildInputFailed", value: cause.message })
  }
  if (cause instanceof TerminationFailure) {
    return Object.freeze({
      tag: "ChildTerminationFailed",
      value: cause.message,
    })
  }
  return Object.freeze({
    tag: "ChildWaitFailed",
    value: errorMessage(cause),
  })
}

function streamingChildFailure(
  cause: unknown,
  command: CommandRequest
): Readonly<{ kind: "child"; error: ChildFailure }> {
  return Object.freeze({ kind: "child", error: childFailure(cause, command) })
}

function applicationExecutable(executable: ProviderExecutable): unknown {
  return executable.tag === "search-path"
    ? Object.freeze({ tag: "SearchPath", value: executable.value })
    : Object.freeze({ tag: "ExecutablePath", value: executable.value })
}

function bytes(value: unknown): Uint8Array {
  if (!(value instanceof Uint8Array)) {
    throw new TypeError("child output chunk must be Bytes")
  }
  return new Uint8Array(value)
}

function split(value: Uint8Array): ReadonlyArray<Uint8Array> {
  const chunks: Uint8Array[] = []
  for (let offset = 0; offset < value.length; offset += 65_536) {
    chunks.push(new Uint8Array(value.subarray(offset, offset + 65_536)))
  }
  return chunks
}

function concat(chunks: ReadonlyArray<Uint8Array>, length: number): Uint8Array {
  const result = new Uint8Array(length)
  let offset = 0
  for (const chunk of chunks) {
    result.set(chunk, offset)
    offset += chunk.length
  }
  return result
}

function inputAttachment(value: unknown): InputAttachment {
  if (
    typeof value !== "object" ||
    value === null ||
    typeof (value as InputAttachment).next !== "function" ||
    typeof (value as InputAttachment).close !== "function"
  ) {
    throw new TypeError("child input attachment is invalid")
  }
  return value as InputAttachment
}

function subscriptionObserver(value: unknown): ProviderSubscriptionObserver {
  if (
    typeof value !== "object" ||
    value === null ||
    typeof (value as ProviderSubscriptionObserver).next !== "function" ||
    typeof (value as ProviderSubscriptionObserver).complete !== "function" ||
    typeof (value as ProviderSubscriptionObserver).failure !== "function" ||
    typeof (value as ProviderSubscriptionObserver).defect !== "function"
  ) {
    throw new TypeError("child subscription observer is invalid")
  }
  return value as ProviderSubscriptionObserver
}

function dataRecord(
  value: unknown,
  fields: ReadonlyArray<string>
): Record<string, unknown> {
  if (
    typeof value !== "object" ||
    value === null ||
    Object.getPrototypeOf(value) !== Object.prototype
  ) {
    throw new TypeError("child provider boundary value must be a plain record")
  }
  const names = Object.keys(value)
  if (
    names.length !== fields.length ||
    names.some((name) => !fields.includes(name))
  ) {
    throw new TypeError("child provider boundary record shape is invalid")
  }
  return value as Record<string, unknown>
}

function success(value: unknown): ProviderResult {
  return Object.freeze({ kind: "success", value })
}

function failure(value: unknown): ProviderResult {
  return Object.freeze({ kind: "failure", failure: value })
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause)
}

class SpawnFailure extends Error {}
class WaitFailure extends Error {}
class InputFailure extends Error {}
class InputAfterCloseFailure extends Error {}
class TerminationFailure extends Error {}
class InputStreamFailure extends Error {
  constructor(readonly error: unknown) {
    super("child input stream failed")
  }
}
class UnsupportedSignalFailure extends Error {
  constructor(readonly signal: ChildSignal) {
    super(`child signal is unsupported: ${signal.tag}`)
  }
}
class OutputReadFailure extends Error {
  constructor(
    readonly channel: ChildChannel,
    message: string
  ) {
    super(message)
  }
}
class OutputLimitFailure extends Error {
  constructor(
    readonly channel: ChildChannel,
    readonly limitBytes: number
  ) {
    super(`child output exceeded ${limitBytes} bytes`)
  }
}
