import { expect, test } from "bun:test"
import { existsSync, mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"

async function assertProbe(command: string[], marker: string): Promise<void> {
  const child = Bun.spawn(command, { stdout: "pipe", stderr: "pipe" })
  const stdout = new Response(child.stdout).text()
  const stderr = new Response(child.stderr).text()
  const mode = command.at(-1)
  console.info(
    JSON.stringify({ fixture: "HTTP lifecycle", mode, pid: child.pid })
  )
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    const exitCode = await Promise.race([
      child.exited,
      new Promise<never>((_resolve, reject) => {
        timer = setTimeout(
          () => reject(new Error(`${mode} probe timed out`)),
          7_000
        )
      }),
    ])
    expect({ mode, exitCode, stderr: await stderr }).toEqual({
      mode,
      exitCode: 0,
      stderr: "",
    })
    expect(await stdout).toContain(marker)
  } finally {
    clearTimeout(timer)
    if (child.exitCode === null) {
      child.kill("SIGTERM")
      const exited = await Promise.race([
        child.exited.then(() => true),
        Bun.sleep(500).then(() => false),
      ])
      if (!exited) {
        child.kill("SIGKILL")
        expect(
          await Promise.race([
            child.exited.then(() => true),
            Bun.sleep(500).then(() => false),
          ])
        ).toBe(true)
      }
    }
    expect(() => process.kill(child.pid, 0)).toThrow()
    console.info(
      JSON.stringify({
        fixture: "HTTP lifecycle",
        mode,
        pid: child.pid,
        exited: true,
      })
    )
  }
}

test("Node serveOnce drains delayed responses and escalates cancellation during native backpressure", async () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-node-http-lifecycle-"))
  try {
    const build = await Bun.build({
      entrypoints: [
        resolve(import.meta.dir, "../probes/http-server-node-lifecycle.ts"),
      ],
      outdir: directory,
      target: "node",
    })
    expect(build.success).toBe(true)
    for (const mode of [
      "delayed-buffer",
      "delayed-stream",
      "buffered-backpressure",
      "cancel-drain",
      "close-drain",
      "shutdown-late-stream",
    ]) {
      await assertProbe(
        ["node", join(directory, "http-server-node-lifecycle.js"), mode],
        `Node HTTP server lifecycle probe passed: ${mode}`
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
    expect(existsSync(directory)).toBe(false)
    console.info(
      JSON.stringify({ fixture: "HTTP lifecycle", directory, removed: true })
    )
  }
}, 30_000)

test("Bun serveOnce drains delayed responses and escalates close, cancellation and shutdown", async () => {
  const probe = resolve(
    import.meta.dir,
    "../probes/http-server-bun-lifecycle.ts"
  )
  for (const mode of [
    "delayed-buffer",
    "delayed-stream",
    "cancel-drain",
    "close-drain",
    "shutdown-drain",
    "cancel-slow-finalizer",
    "shutdown-slow-finalizer",
    "shutdown-late-stream",
  ]) {
    await assertProbe(
      [process.execPath, probe, mode],
      `Bun HTTP server lifecycle probe passed: ${mode}`
    )
  }
}, 30_000)
