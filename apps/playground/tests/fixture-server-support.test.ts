import { expect, test } from "bun:test"
import { mkdtemp, rm, writeFile } from "node:fs/promises"
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
    ).rejects.toThrow("fixture worker exited")
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}, 15_000)
