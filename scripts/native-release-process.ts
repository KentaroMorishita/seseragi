import { spawn } from "node:child_process"
import { mkdtemp, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"

export type NativeReleaseCommandResult = {
  command: string[]
  pid: number | undefined
  exitCode: number | null
  signal: NodeJS.Signals | null
  exitObserved: boolean
  closeObserved: boolean
  stdout: string
  stderr: string
}

export type NativeReleaseCommandRunner = (
  command: string[],
  cwd?: string
) => Promise<NativeReleaseCommandResult>

export class NativeReleaseCommandError extends Error {
  constructor(
    readonly result: NativeReleaseCommandResult,
    cause?: Error
  ) {
    super(
      `native release: ${result.command.join(" ")} failed (pid ${result.pid ?? "not started"}, exit ${result.exitCode}, signal ${result.signal ?? "none"}, close ${result.closeObserved})${result.stderr ? `:\n${result.stderr.trim()}` : ""}${cause ? `: ${cause.message}` : ""}`,
      { cause }
    )
    this.name = "NativeReleaseCommandError"
  }
}

// A Windows executable's image and redirected stdio must be released before
// removing its extracted directory. Wait for close, which follows process exit
// and closure of all child stdio, rather than starting cleanup from an exit event.
export function runNativeReleaseCommand(
  command: string[],
  cwd: string
): Promise<NativeReleaseCommandResult> {
  return new Promise((resolve, reject) => {
    const [executable, ...arguments_] = command
    if (!executable) {
      reject(new Error("native release: command has no executable"))
      return
    }
    const child = spawn(executable, arguments_, {
      cwd,
      env: { ...process.env, COPYFILE_DISABLE: "1" },
      stdio: ["ignore", "pipe", "pipe"],
      windowsHide: true,
    })
    const result: NativeReleaseCommandResult = {
      command,
      pid: child.pid,
      exitCode: null,
      signal: null,
      exitObserved: false,
      closeObserved: false,
      stdout: "",
      stderr: "",
    }
    let spawnError: Error | undefined
    child.stdout.setEncoding("utf8")
    child.stderr.setEncoding("utf8")
    child.stdout.on("data", (text: string) => {
      result.stdout += text
    })
    child.stderr.on("data", (text: string) => {
      result.stderr += text
    })
    child.on("error", (error) => {
      spawnError = error
    })
    child.on("exit", (code, signal) => {
      result.exitObserved = true
      result.exitCode = code
      result.signal = signal
    })
    child.on("close", (code, signal) => {
      result.closeObserved = true
      result.exitCode = code
      result.signal = signal
      if (spawnError || code !== 0 || !result.exitObserved) {
        reject(new NativeReleaseCommandError(result, spawnError))
      } else {
        resolve(result)
      }
    })
  })
}

type CleanupOperations = {
  remove: (directory: string) => Promise<void>
  wait: (milliseconds: number) => Promise<void>
}

const cleanupOperations: CleanupOperations = {
  remove: (directory) =>
    rm(directory, { recursive: true, force: true, maxRetries: 0 }),
  wait: (milliseconds) =>
    new Promise((resolve) => setTimeout(resolve, milliseconds)),
}

export class NativeReleaseCleanupError extends Error {
  constructor(
    readonly directory: string,
    readonly attempts: number,
    readonly waitedMs: number,
    cause: unknown
  ) {
    super(
      `native release: failed to remove temporary directory ${directory} after ${attempts} attempt(s) and ${waitedMs}ms of retry waits: ${describeError(cause)}`,
      { cause }
    )
    this.name = "NativeReleaseCleanupError"
  }
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

export async function removeNativeReleaseDirectory(
  directory: string,
  operations: CleanupOperations = cleanupOperations
): Promise<void> {
  // Six attempts, with 100+200+300+400+500ms between them. Never rerun the
  // smoke or retry a permanent filesystem error to hide a validation failure.
  const transientCodes = new Set([
    "EBUSY",
    "ENOTEMPTY",
    "EPERM",
    "EMFILE",
    "ENFILE",
  ])
  let waitedMs = 0
  for (let attempt = 1; ; attempt += 1) {
    try {
      await operations.remove(directory)
      return
    } catch (error) {
      const code =
        error && typeof error === "object" && "code" in error
          ? error.code
          : undefined
      if (
        attempt === 6 ||
        typeof code !== "string" ||
        !transientCodes.has(code)
      ) {
        throw new NativeReleaseCleanupError(directory, attempt, waitedMs, error)
      }
      const delay = 100 * attempt
      await operations.wait(delay)
      waitedMs += delay
    }
  }
}

export class NativeReleaseTemporaryDirectoryError extends AggregateError {
  constructor(
    readonly operation: string,
    readonly operationSucceeded: boolean,
    operationError: unknown,
    cleanupError: unknown,
    readonly commandResults: NativeReleaseCommandResult[]
  ) {
    super(
      operationSucceeded ? [cleanupError] : [operationError, cleanupError],
      `native release: ${operation} ${operationSucceeded ? "succeeded" : `failed: ${describeError(operationError)}`}; temporary directory cleanup also failed: ${describeError(cleanupError)}; owned commands: ${commandResults.map((result) => `${path.basename(result.command[0] ?? "<missing>")} pid=${result.pid ?? "not started"} exit=${result.exitCode} signal=${result.signal ?? "none"} exitObserved=${result.exitObserved} closeObserved=${result.closeObserved}`).join(", ") || "none started"}`,
      { cause: operationSucceeded ? cleanupError : operationError }
    )
    this.name = "NativeReleaseTemporaryDirectoryError"
  }
}

export async function withNativeReleaseDirectory<T>(
  operation: "package" | "verification",
  run: (directory: string, execute: NativeReleaseCommandRunner) => Promise<T>,
  operations: CleanupOperations = cleanupOperations
): Promise<T> {
  const directory = await mkdtemp(
    path.join(tmpdir(), `seseragi-native-${operation}-`)
  )
  const commands: NativeReleaseCommandResult[] = []
  const execute: NativeReleaseCommandRunner = async (
    command,
    cwd = directory
  ) => {
    try {
      const result = await runNativeReleaseCommand(command, cwd)
      commands.push(result)
      return result
    } catch (error) {
      if (error instanceof NativeReleaseCommandError)
        commands.push(error.result)
      throw error
    }
  }
  let result: { success: true; value: T } | { success: false; error: unknown }
  try {
    result = { success: true, value: await run(directory, execute) }
  } catch (error) {
    result = { success: false, error }
  }
  try {
    await removeNativeReleaseDirectory(directory, operations)
  } catch (error) {
    throw new NativeReleaseTemporaryDirectoryError(
      operation,
      result.success,
      result.success ? undefined : result.error,
      error,
      commands
    )
  }
  if (!result.success) throw result.error
  return result.value
}
