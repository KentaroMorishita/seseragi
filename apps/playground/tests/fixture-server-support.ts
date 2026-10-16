import { mkdtemp, readFile, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { resolve } from "node:path"

export interface FixtureListenerStatus {
  port: number
  pendingRequests: number
  pendingWebSockets: number
}

export interface FixtureServerConfiguration {
  outputs: Record<string, string>
  indexes: Record<string, string>
}

type FixtureResponse =
  | { kind: "ready"; status: FixtureListenerStatus }
  | { kind: "status"; id: number; status: FixtureListenerStatus }

interface FixtureWorkerOptions {
  worker?: string
  startupTimeoutMs?: number
  shutdownTimeoutMs?: number
}

async function bounded<T>(
  promise: Promise<T>,
  message: string,
  milliseconds = 5_000
) {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(() => reject(new Error(message)), milliseconds)
      }),
    ])
  } finally {
    clearTimeout(timer)
  }
}

export async function startFixtureServer(
  configuration: string,
  options: FixtureWorkerOptions = {}
) {
  const directory = await mkdtemp(resolve(tmpdir(), "seseragi-fixture-worker-"))
  const stderrPath = resolve(directory, "stderr.log")
  let resolveReady!: (status: FixtureListenerStatus) => void
  let rejectReady!: (error: Error) => void
  const ready = new Promise<FixtureListenerStatus>((resolve, reject) => {
    resolveReady = resolve
    rejectReady = reject
  })
  const requests = new Map<
    number,
    {
      resolve: (status: FixtureListenerStatus) => void
      reject: (error: Error) => void
      timer: ReturnType<typeof setTimeout>
    }
  >()
  const rejectRequests = (error: Error) => {
    rejectReady(error)
    for (const request of requests.values()) {
      clearTimeout(request.timer)
      request.reject(error)
    }
    requests.clear()
  }
  let child: Bun.Subprocess<"ignore", "ignore", Bun.BunFile>
  try {
    child = Bun.spawn(
      [
        process.execPath,
        options.worker ??
          resolve(import.meta.dir, "fixtures/dom-fixture-server.ts"),
        configuration,
      ],
      {
        stdin: "ignore",
        stdout: "ignore",
        // A pipe's EOF can outlive the worker. Keep diagnostics in an owned file
        // so neither normal exit nor SIGKILL must join a stream reader.
        stderr: Bun.file(stderrPath),
        ipc(value: unknown) {
          if (value === null || typeof value !== "object") {
            rejectRequests(new Error("fixture worker returned invalid IPC"))
            return
          }
          const message = value as FixtureResponse
          if (message.kind === "ready") {
            resolveReady(message.status)
          } else if (message.kind === "status") {
            const request = requests.get(message.id)
            if (request !== undefined) {
              clearTimeout(request.timer)
              requests.delete(message.id)
              request.resolve(message.status)
            }
          }
        },
      }
    )
  } catch (error) {
    await bounded(
      rm(directory, { recursive: true, force: true }),
      "fixture worker log cleanup timed out"
    )
    throw error
  }
  let stderr = ""
  let diagnostics: Promise<void> | undefined
  const captureDiagnostics = () => {
    diagnostics ??= bounded(
      readFile(stderrPath, "utf8"),
      "fixture worker diagnostic read timed out"
    ).then((text) => {
      stderr = text
    })
    return diagnostics
  }
  // This promise represents only the actual worker's exit. Do not attach a
  // diagnostic drain or another process's lifetime to it.
  let joinedExitCode: number | undefined
  const exited = child.exited.then((code) => {
    joinedExitCode = code
    rejectRequests(new Error(`fixture worker exited (${code})`))
    return code
  })
  const running = () =>
    joinedExitCode === undefined &&
    child.exitCode === null &&
    child.signalCode === null
  void exited.then(captureDiagnostics).catch(() => {})
  const finish = async (failure?: unknown, killImmediately = false) => {
    let code: number | undefined
    try {
      if (killImmediately && running()) child.kill("SIGKILL")
      code = await bounded(
        exited,
        "fixture worker did not stop",
        options.shutdownTimeoutMs
      )
    } catch (error) {
      failure ??= error
      try {
        if (running()) child.kill("SIGKILL")
        code = await bounded(
          exited,
          "fixture worker did not exit after SIGKILL"
        )
      } catch (joinError) {
        failure = new AggregateError(
          [failure, joinError],
          "fixture worker exit join failed"
        )
      }
    }
    try {
      await captureDiagnostics()
      if (code !== undefined) {
        await bounded(
          rm(directory, { recursive: true, force: true }),
          "fixture worker log cleanup timed out"
        )
      }
    } catch (error) {
      failure ??= error
    }
    if (failure !== undefined || code !== 0) {
      const message =
        failure instanceof Error
          ? failure.message
          : `fixture worker failed (${code})`
      throw new Error(
        `${message}; worker pid=${child.pid} exit=${child.exitCode} signal=${child.signalCode}; stderr file=${stderrPath}\n${stderr}`,
        { cause: failure }
      )
    }
  }
  let stopping: Promise<void> | undefined
  const stop = () => {
    stopping ??= (async () => {
      let sendError: unknown
      if (running()) {
        try {
          child.send({ kind: "stop" })
        } catch (error) {
          sendError = error
        }
      }
      await finish(sendError)
    })()
    return stopping
  }
  try {
    const initial = await bounded(
      ready,
      "fixture worker did not start",
      options.startupTimeoutMs
    )
    if (
      !Number.isInteger(initial.port) ||
      initial.port < 1 ||
      initial.port > 65535
    ) {
      throw new Error(`invalid fixture worker port: ${initial.port}`)
    }
    let sequence = 0
    return {
      port: initial.port,
      diagnostics: () => ({
        pid: child.pid,
        exitCode: child.exitCode,
        joinedExitCode,
        signalCode: child.signalCode,
        stderr,
      }),
      status: () =>
        new Promise<FixtureListenerStatus>((resolve, reject) => {
          if (!running() || stopping !== undefined) {
            reject(new Error(`fixture worker unavailable (${child.exitCode})`))
            return
          }
          const id = ++sequence
          const timer = setTimeout(() => {
            requests.delete(id)
            reject(new Error("fixture worker did not report listener state"))
          }, 5_000)
          requests.set(id, { resolve, reject, timer })
          try {
            child.send({ kind: "status", id })
          } catch (error) {
            clearTimeout(timer)
            requests.delete(id)
            reject(error)
          }
        }),
      stop,
    }
  } catch (error) {
    await finish(error, true)
    throw error
  }
}
