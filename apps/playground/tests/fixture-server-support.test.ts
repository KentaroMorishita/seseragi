import { expect, test } from "bun:test"
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { resolve } from "node:path"
import { startFixtureServer } from "./fixture-server-support"

async function configuration(directory: string) {
  const path = resolve(directory, "fixture-server.json")
  await writeFile(
    path,
    JSON.stringify({
      outputs: { fixture: directory },
      indexes: { fixture: "<!doctype html><p>owned listener</p>" },
    })
  )
  return path
}

test("owns the fixture worker until explicit shutdown and awaits its exit", async () => {
  const directory = await mkdtemp(resolve(tmpdir(), "seseragi-listener-"))
  const server = await startFixtureServer(await configuration(directory))
  try {
    const url = `http://127.0.0.1:${server.port}/fixture/`
    expect(await (await fetch(url)).text()).toContain("owned listener")
    expect((await server.status()).port).toBe(server.port)
    await server.stop()
    await server.stop()
    expect(server.diagnostics().exitCode).toBe(0)
    await expect(server.status()).rejects.toThrow("fixture worker unavailable")
    await expect(
      fetch(url, { signal: AbortSignal.timeout(1_000) })
    ).rejects.toThrow()
  } finally {
    await server.stop()
    await rm(directory, { recursive: true, force: true })
  }
}, 15_000)

test("reports an unexpectedly terminated fixture worker instead of restarting it", async () => {
  const directory = await mkdtemp(resolve(tmpdir(), "seseragi-listener-exit-"))
  const server = await startFixtureServer(await configuration(directory))
  try {
    process.kill(server.diagnostics().pid, "SIGTERM")
    await expect(server.status()).rejects.toThrow()
    await expect(server.stop()).rejects.toThrow()
  } finally {
    await server.stop().catch(() => {})
    await rm(directory, { recursive: true, force: true })
  }
}, 15_000)

test("reports fixture startup failure and joins the failed child", async () => {
  const directory = await mkdtemp(resolve(tmpdir(), "seseragi-listener-start-"))
  try {
    await expect(
      startFixtureServer(resolve(directory, "missing.json"))
    ).rejects.toThrow(/fixture worker exited.*ENOENT/s)
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}, 15_000)

const probeWorker = resolve(
  import.meta.dir,
  "fixtures/fixture-worker-lifecycle-probe.ts"
)

test("joins the worker without waiting for an inherited stderr handle", async () => {
  const directory = await mkdtemp(
    resolve(tmpdir(), "seseragi-listener-stderr-")
  )
  const descendant = resolve(directory, "descendant.pid")
  const path = resolve(directory, "configuration.json")
  await writeFile(
    path,
    JSON.stringify({ mode: "inherited-stderr", descendant })
  )
  const server = await startFixtureServer(path, { worker: probeWorker })
  try {
    expect(await (await fetch(`http://127.0.0.1:${server.port}`)).text()).toBe(
      "owned listener"
    )
    const started = performance.now()
    await server.stop()
    expect(performance.now() - started).toBeLessThan(3_000)
    expect(server.diagnostics().exitCode).toBe(0)
    expect(server.diagnostics().stderr).toContain(
      "worker diagnostic: inherited-stderr"
    )
    expect(Number(await readFile(descendant, "utf8"))).toBeGreaterThan(0)
  } finally {
    await server.stop().catch(() => {})
    const pid = Number(await readFile(descendant, "utf8").catch(() => 0))
    if (Number.isInteger(pid) && pid > 0) {
      try {
        process.kill(pid, "SIGKILL")
      } catch (error) {
        expect((error as NodeJS.ErrnoException).code).toBe("ESRCH")
      }
    }
    await rm(directory, { recursive: true, force: true })
  }
}, 15_000)

test("kills an unresponsive worker and retains its stderr diagnostics", async () => {
  const directory = await mkdtemp(resolve(tmpdir(), "seseragi-listener-kill-"))
  const path = resolve(directory, "configuration.json")
  await writeFile(path, JSON.stringify({ mode: "ignore-stop" }))
  const server = await startFixtureServer(path, {
    worker: probeWorker,
    shutdownTimeoutMs: 200,
  })
  try {
    const failure = await server.stop().catch((error: Error) => error)
    expect(failure).toBeInstanceOf(Error)
    if (!(failure instanceof Error)) throw new Error("worker must fail to stop")
    expect(failure.message).toContain("fixture worker did not stop")
    expect(failure.message).toContain("worker diagnostic: ignore-stop")
    expect(server.diagnostics().joinedExitCode).toBeDefined()
    expect(server.diagnostics().signalCode).not.toBeNull()
    expect(() => process.kill(server.diagnostics().pid, 0)).toThrow()
    await expect(server.stop()).rejects.toBe(failure)
  } finally {
    await server.stop().catch(() => {})
    await rm(directory, { recursive: true, force: true })
  }
}, 15_000)

test("bounds startup failure and joins a worker that never becomes ready", async () => {
  const directory = await mkdtemp(resolve(tmpdir(), "seseragi-listener-ready-"))
  const path = resolve(directory, "configuration.json")
  await writeFile(path, JSON.stringify({ mode: "never-ready" }))
  try {
    const failure = await startFixtureServer(path, {
      worker: probeWorker,
      startupTimeoutMs: 1_000,
    }).catch((error: Error) => error)
    expect(failure).toBeInstanceOf(Error)
    if (!(failure instanceof Error))
      throw new Error("worker must fail to start")
    expect(failure.message).toContain("fixture worker did not start")
    expect(failure.message).toContain("worker diagnostic: never-ready")
    const pid = Number(/worker pid=(\d+)/.exec(failure.message)?.[1])
    expect(Number.isInteger(pid) && pid > 0).toBe(true)
    expect(() => process.kill(pid, 0)).toThrow()
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}, 15_000)
