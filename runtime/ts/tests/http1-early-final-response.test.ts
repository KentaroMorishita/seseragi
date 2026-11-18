import { expect, test } from "bun:test"
import { mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"

test("Bun and Node stop an endless HTTP/1 body when the server answers early", async () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-http1-early-final-"))
  try {
    const probe = resolve(
      import.meta.dir,
      "http1-early-final-response.probe.ts"
    )
    const built = await Bun.build({
      entrypoints: [probe],
      outdir: directory,
      target: "node",
    })
    expect(built.success).toBe(true)
    for (const [command, script, target] of [
      ["node", join(directory, "http1-early-final-response.probe.js"), "node"],
      [process.execPath, probe, "bun"],
    ]) {
      const child = Bun.spawn([command!, script!], {
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
                reject(new Error(`${target} early response probe timed out`)),
              3_000
            )
          }),
        ])
        expect({ target, exitCode, stderr: await stderr }).toEqual({
          target,
          exitCode: 0,
          stderr: "",
        })
        const output = await stdout
        expect(output).toContain("early final response probe passed")
        const report = JSON.parse(output.split("\n")[0] ?? "") as {
          status: number
          body: string
          pulls: number
          cancelled: boolean
        }
        expect(report).toMatchObject({
          status: 413,
          body: "too large",
          cancelled: true,
        })
        expect(report.pulls).toBeLessThan(1_000)
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
}, 15_000)
