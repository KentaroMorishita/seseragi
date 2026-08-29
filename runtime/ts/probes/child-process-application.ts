import { fromUint8Array } from "@seseragi/runtime/bytes"
import {
  addArguments,
  captureLimit,
  type CaptureLimit,
  clearEnvironment,
  CloseChildStdin,
  command,
  type Command,
  type ChildEvent,
  type ChildProcessesEnvironment,
  type ChildProcessError,
  type CapturedProcess,
  type Executable,
  ExecutablePath,
  inDirectory,
  runCaptured,
  runStreaming,
  SearchPath,
  setEnvironment,
  WriteChildStdin,
} from "@seseragi/runtime/child-process"
import type { Effect } from "@seseragi/runtime/effect"
import { parse } from "@seseragi/runtime/path"
import { fromArray, type Stream } from "@seseragi/runtime/stream"

export function capturedFixture(): Effect<
  ChildProcessesEnvironment,
  ChildProcessError,
  CapturedProcess
> {
  const current = nodeCommand([
    "-e",
    "let b=[];process.stdin.on('data',c=>b.push(c));process.stdin.on('end',()=>{process.stdout.write(Buffer.concat(b).toString().toUpperCase());process.stderr.write('warn');process.exitCode=7})",
  ])
  return runCaptured(limit(1024), bytes("hello"), current)
}

export function limitedFixture(): Effect<
  ChildProcessesEnvironment,
  ChildProcessError,
  CapturedProcess
> {
  return runCaptured(
    limit(4),
    bytes(""),
    nodeCommand(["-e", "process.stdout.write('12345')"])
  )
}

export function streamingFixture(): Stream<
  ChildProcessesEnvironment,
  ChildProcessError,
  ChildEvent
> {
  const input = fromArray([WriteChildStdin(bytes("stream")), CloseChildStdin])
  return runStreaming(
    input,
    nodeCommand([
      "-e",
      "let b=[];process.stdin.on('data',c=>b.push(c));process.stdin.on('end',()=>{process.stdout.write(Buffer.concat(b).toString().toUpperCase());process.stderr.write('err');process.exitCode=3})",
    ])
  )
}

export function cancellableFixture(): Stream<
  ChildProcessesEnvironment,
  ChildProcessError,
  ChildEvent
> {
  return runStreaming(
    fromArray([]),
    nodeCommand([
      "-e",
      "process.stdout.write(String(process.pid)+'\\n');setInterval(()=>{},1000)",
    ])
  )
}

export function missingSearchPathFixture(
  executable: string
): Effect<ChildProcessesEnvironment, ChildProcessError, CapturedProcess> {
  return runCaptured(
    limit(1024),
    bytes(""),
    clearEnvironment(configuredCommand(SearchPath(executable), []))
  )
}

export function explicitSearchPathFixture(
  executable: string,
  path: string
): Effect<ChildProcessesEnvironment, ChildProcessError, CapturedProcess> {
  const cleared = clearEnvironment(
    configuredCommand(SearchPath(executable), [
      "-e",
      "process.stdout.write('EXPLICIT_PATH')",
    ])
  )
  const configured = setEnvironment("PATH", path, cleared)
  if (configured.tag === "Left") {
    throw new Error("static PATH environment must be valid")
  }
  return runCaptured(limit(1024), bytes(""), configured.value)
}

export function executablePathFixture(
  executable: string
): Effect<ChildProcessesEnvironment, ChildProcessError, CapturedProcess> {
  const parsed = parse(executable)
  if (parsed.tag === "Left") {
    throw new Error("host executable path must be portable")
  }
  return runCaptured(
    limit(1024),
    bytes(""),
    clearEnvironment(
      configuredCommand(ExecutablePath(parsed.value), [
        "-e",
        "process.stdout.write('EXECUTABLE_PATH')",
      ])
    )
  )
}

export function relativeExecutablePathFixture(
  executable: string,
  directory: string
): Effect<ChildProcessesEnvironment, ChildProcessError, CapturedProcess> {
  const executablePath = parse(executable)
  const workingDirectory = parse(directory)
  if (executablePath.tag === "Left" || workingDirectory.tag === "Left") {
    throw new Error("host executable path and directory must be portable")
  }
  return runCaptured(
    limit(1024),
    bytes(""),
    clearEnvironment(
      inDirectory(
        workingDirectory.value,
        configuredCommand(ExecutablePath(executablePath.value), [
          "-e",
          "process.stdout.write('RELATIVE_EXECUTABLE_PATH')",
        ])
      )
    )
  )
}

function nodeCommand(arguments_: ReadonlyArray<string>): Command {
  return configuredCommand(SearchPath("node"), arguments_)
}

function configuredCommand(
  executable: Executable,
  arguments_: ReadonlyArray<string>
): Command {
  const base = command(executable)
  if (base.tag === "Left") throw new Error("node command must be valid")
  const configured = addArguments(arguments_, base.value)
  if (configured.tag === "Left") {
    throw new Error("static node arguments must be valid")
  }
  return configured.value
}

function limit(value: number): CaptureLimit {
  const result = captureLimit(value)
  if (result.tag === "Left") throw new Error("static limit must be valid")
  return result.value
}

function bytes(value: string) {
  return fromUint8Array(new TextEncoder().encode(value))
}
