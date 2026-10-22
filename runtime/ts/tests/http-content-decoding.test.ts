import { expect, test } from "bun:test"
import { mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"

test("Node and Bun decode streaming responses with bounded demand and joined cancellation", async () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-http-content-decoding-")
  )
  try {
    const probe = resolve(import.meta.dir, "../probes/http-content-decoding.ts")
    const built = await Bun.build({
      entrypoints: [probe],
      outdir: directory,
      target: "node",
    })
    expect(built.success).toBe(true)
    for (const [command, script, target] of [
      ["node", join(directory, "http-content-decoding.js"), "node-process"],
      [process.execPath, probe, "bun-process"],
    ]) {
      const child = Bun.spawn([command!, script!, target!], {
        stdout: "pipe",
        stderr: "pipe",
      })
      const stdout = new Response(child.stdout).text()
      const stderr = new Response(child.stderr).text()
      let timer: ReturnType<typeof setTimeout> | undefined
      try {
        const exitCode = await Promise.race([
          child.exited,
          new Promise<never>((_resolve, reject) => {
            timer = setTimeout(
              () =>
                reject(new Error(`${target} content decoding probe timed out`)),
              15_000
            )
          }),
        ])
        expect({ target, exitCode, stderr: await stderr }).toEqual({
          target,
          exitCode: 0,
          stderr: "",
        })
        expect(await stdout).toContain("HTTP content decoding probe passed")
      } finally {
        if (timer !== undefined) clearTimeout(timer)
        if (child.exitCode === null) {
          child.kill("SIGTERM")
          const killed = await Promise.race([
            child.exited.then(() => true),
            Bun.sleep(500).then(() => false),
          ])
          if (!killed) child.kill("SIGKILL")
          await child.exited
        }
        expect(() => process.kill(child.pid, 0)).toThrow()
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
}, 40_000)
