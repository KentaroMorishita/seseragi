import { expect, test } from "bun:test"
import { mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"

test("a browser exchange finishes an empty-chunk upload without setImmediate", async () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-http-empty-browser-"))
  try {
    const probe = resolve(
      import.meta.dir,
      "http-client-empty-chunk-browser.probe.ts"
    )
    const built = await Bun.build({
      entrypoints: [probe],
      outdir: directory,
      target: "node",
    })
    expect(built.success).toBe(true)
    for (const [command, script, runtime] of [
      [
        "node",
        join(directory, "http-client-empty-chunk-browser.probe.js"),
        "node",
      ],
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
                reject(new Error(`${runtime} browser upload probe timed out`)),
              3_000
            )
          }),
        ])
        expect({ runtime, exitCode, stderr: await stderr }).toEqual({
          runtime,
          exitCode: 0,
          stderr: "",
        })
        const output = await stdout
        expect(output).toContain("browser empty chunk upload probe passed")
        expect(JSON.parse(output.split("\n")[0] ?? "")).toEqual({
          status: 200,
          body: "ok",
          received: [7],
        })
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
