import { fromUint8Array } from "@seseragi/runtime/bytes"
import {
  addArguments,
  captureLimit,
  type CaptureLimit,
  CloseChildStdin,
  command,
  type Command,
  type ChildEvent,
  type ChildProcessesEnvironment,
  type ChildProcessError,
  type CapturedProcess,
  runCaptured,
  runStreaming,
  SearchPath,
  WriteChildStdin,
} from "@seseragi/runtime/child-process"
import type { Effect } from "@seseragi/runtime/effect"
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

function nodeCommand(arguments_: ReadonlyArray<string>): Command {
  const base = command(SearchPath("node"))
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
