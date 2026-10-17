import { spawnSync } from "node:child_process"
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

function terminateProcessTree(
  child: Pick<Bun.Subprocess, "pid" | "kill">
): void {
  if (process.platform === "win32") {
    const killed = spawnSync(
      "taskkill",
      ["/pid", String(child.pid), "/T", "/F"],
      {
        stdio: "ignore",
        timeout: 1_000,
      }
    )
    if (killed.status !== 0) child.kill("SIGKILL")
    return
  }
  try {
    // detached makes the direct child a process-group leader on POSIX, so the
    // negative PID reaches CLI adapters and other descendants as one unit.
    // Descendants may still own the pipes after the group leader has exited.
    process.kill(-child.pid, "SIGKILL")
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
  const child = Bun.spawn({
    cmd: [executable, ...commandArguments],
    cwd,
    env: process.env,
    detached: process.platform !== "win32",
    stdin: new TextEncoder().encode(stdin),
    stdout: "pipe",
    stderr: "pipe",
  })
  let stdout = ""
  let stderr = ""
  const readers = [child.stdout.getReader(), child.stderr.getReader()] as const
  const capture = async (
    reader: ReadableStreamDefaultReader<Uint8Array>,
    append: (text: string) => void
  ): Promise<void> => {
    const decoder = new TextDecoder()
    try {
      for (;;) {
        const { done, value } = await reader.read()
        if (done) break
        append(decoder.decode(value, { stream: true }))
      }
      append(decoder.decode())
    } finally {
      reader.releaseLock()
    }
  }
  // Process exit and stream EOF are separate observations. Neither the
  // completion nor the deadline depends on a Node compatibility close event.
  const completed = Promise.all([
    child.exited,
    capture(readers[0], (text) => {
      stdout += text
    }),
    capture(readers[1], (text) => {
      stderr += text
    }),
  ])
  let completedNormally = false
  let timeout: ReturnType<typeof setTimeout> | undefined
  const deadline = new Promise<never>((_resolve, reject) => {
    timeout = setTimeout(() => {
      const elapsed = Math.round(performance.now() - started)
      reject(
        new Error(
          `${label} timed out after ${elapsed}ms (limit ${timeoutMs}ms)${stderr ? `:\n${stderr}` : ""}`
        )
      )
    }, timeoutMs)
  })

  try {
    const [status] = await Promise.race([completed, deadline])
    completedNormally = true
    return { status, stdout, stderr }
  } finally {
    clearTimeout(timeout)
    if (!completedNormally) {
      try {
        terminateProcessTree(child)
      } finally {
        // Do not turn a missed deadline into another unbounded close/drain wait.
        for (const reader of readers) void reader.cancel().catch(() => {})
        child.unref()
      }
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
