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

export async function startFixtureServer(configuration: string) {
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
  const child = Bun.spawn(
    [
      process.execPath,
      resolve(import.meta.dir, "fixtures/dom-fixture-server.ts"),
      configuration,
    ],
    {
      stdin: "ignore",
      stdout: "ignore",
      stderr: "pipe",
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
  let stderr = ""
  const errors = new Response(child.stderr).text().then((text) => {
    stderr = text
  })
  const exited = child.exited.then(async (code) => {
    await errors
    rejectRequests(new Error(`fixture worker exited (${code}): ${stderr}`))
    return code
  })
  const bounded = async <T>(promise: Promise<T>, message: string) => {
    let timer: ReturnType<typeof setTimeout> | undefined
    try {
      return await Promise.race([
        promise,
        new Promise<never>((_resolve, reject) => {
          timer = setTimeout(() => reject(new Error(message)), 5_000)
        }),
      ])
    } finally {
      clearTimeout(timer)
    }
  }
  let stopping: Promise<void> | undefined
  const stop = () => {
    stopping ??= (async () => {
      let sendError: unknown
      if (child.exitCode === null) {
        try {
          child.send({ kind: "stop" })
        } catch (error) {
          sendError = error
        }
      }
      let code: number
      try {
        code = await bounded(exited, "fixture worker did not stop")
      } catch (error) {
        child.kill("SIGKILL")
        await exited
        throw error
      }
      if (sendError !== undefined) throw sendError
      if (code !== 0) {
        throw new Error(`fixture worker failed (${code}): ${stderr}`)
      }
    })()
    return stopping
  }
  try {
    const initial = await bounded(ready, "fixture worker did not start")
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
        signalCode: child.signalCode,
        stderr,
      }),
      status: () =>
        new Promise<FixtureListenerStatus>((resolve, reject) => {
          if (child.exitCode !== null || stopping !== undefined) {
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
    if (child.exitCode === null) child.kill("SIGKILL")
    await exited
    throw error
  }
}
