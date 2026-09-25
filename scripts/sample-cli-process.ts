import {
  type ChildProcessWithoutNullStreams,
  spawn,
  spawnSync,
} from "node:child_process"
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

function processIsRunning(child: ChildProcessWithoutNullStreams): boolean {
  return (
    child.pid !== undefined &&
    child.exitCode === null &&
    child.signalCode === null
  )
}

function terminateProcessTree(child: ChildProcessWithoutNullStreams): void {
  if (!processIsRunning(child)) return
  if (process.platform === "win32") {
    const killed = spawnSync(
      "taskkill",
      ["/pid", String(child.pid), "/T", "/F"],
      {
        stdio: "ignore",
      }
    )
    if (killed.status !== 0 && processIsRunning(child)) child.kill("SIGKILL")
    return
  }
  try {
    // detached makes the direct child a process-group leader on POSIX, so the
    // negative PID reaches CLI adapters and other descendants as one unit.
    process.kill(-child.pid!, "SIGKILL")
  } catch {
    child.kill("SIGKILL")
  }
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
  const [executable, ...commandArguments] = command
  if (executable === undefined) throw new Error(`${label} has no command`)
  const child = spawn(executable, commandArguments, {
    cwd,
    detached: process.platform !== "win32",
    stdio: ["pipe", "pipe", "pipe"],
  })
  let stdout = ""
  let stderr = ""
  child.stdout.setEncoding("utf8")
  child.stderr.setEncoding("utf8")
  child.stdout.on("data", (chunk: string) => {
    stdout += chunk
  })
  child.stderr.on("data", (chunk: string) => {
    stderr += chunk
  })
  const completed = new Promise<number>((resolve, reject) => {
    child.once("error", reject)
    child.once("close", (code) => resolve(code ?? 1))
  })
  let timedOut = false
  const timeout = setTimeout(() => {
    timedOut = true
    terminateProcessTree(child)
  }, timeoutMs)

  try {
    child.stdin.end(stdin)
    const status = await completed
    if (timedOut) {
      const elapsed = Math.round(performance.now() - started)
      throw new Error(
        `${label} timed out after ${elapsed}ms (limit ${timeoutMs}ms)${stderr ? `:\n${stderr}` : ""}`
      )
    }
    return { status, stdout, stderr }
  } finally {
    clearTimeout(timeout)
    if (processIsRunning(child)) {
      terminateProcessTree(child)
      await completed.catch(() => {})
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
