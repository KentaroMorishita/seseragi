import { mkdtemp, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"

const DEFAULT_TIMEOUT_MS = 120_000

interface SampleCommandOptions {
  cwd: string
  label: string
  stdin?: string
  timeoutMs?: number
}

interface SampleCommandResult {
  status: number
  stdout: string
  stderr: string
}

export async function runSampleCommand(
  command: string[],
  {
    cwd,
    label,
    stdin = "",
    timeoutMs = DEFAULT_TIMEOUT_MS,
  }: SampleCommandOptions
): Promise<SampleCommandResult> {
  console.log(`Checking ${label}...`)
  const started = performance.now()
  const controller = new AbortController()
  const child = Bun.spawn({
    cmd: command,
    cwd,
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
    signal: controller.signal,
    killSignal: "SIGKILL",
  })
  let timedOut = false
  const timeout = setTimeout(() => {
    timedOut = true
    controller.abort()
  }, timeoutMs)

  try {
    if (stdin !== "") child.stdin.write(stdin)
    child.stdin.end()
    const [status, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ])
    if (timedOut) {
      const elapsed = Math.round(performance.now() - started)
      throw new Error(
        `${label} timed out after ${elapsed}ms (limit ${timeoutMs}ms)${stderr ? `:\n${stderr}` : ""}`
      )
    }
    return { status, stdout, stderr }
  } finally {
    clearTimeout(timeout)
    if (child.exitCode === null && child.signalCode === null) {
      child.kill("SIGKILL")
      await child.exited
    }
  }
}

export async function withTemporaryDirectory<T>(
  prefix: string,
  run: (directory: string) => Promise<T>
): Promise<T> {
  const directory = await mkdtemp(join(tmpdir(), prefix))
  try {
    return await run(directory)
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}
