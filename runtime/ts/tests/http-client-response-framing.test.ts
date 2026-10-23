import { expect, test } from "bun:test"
import { existsSync, mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"

test("native HTTP response framing rejects ambiguous heads and preserves valid bodies", async () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-http-response-framing-")
  )
  const pids: number[] = []
  try {
    const probe = resolve(
      import.meta.dir,
      "../probes/http-client-response-framing.ts"
    )
    const built = await Bun.build({
      entrypoints: [probe],
      outdir: directory,
      target: "node",
    })
    expect(built.success).toBe(true)
    for (const [command, script, target] of [
      [
        "node",
        join(directory, "http-client-response-framing.js"),
        "node-process",
      ],
      [process.execPath, probe, "bun-process"],
    ]) {
      const child = Bun.spawn([command!, script!, target!], {
        stdout: "pipe",
        stderr: "pipe",
      })
      pids.push(child.pid)
      process.stdout.write(
        `${JSON.stringify({ responseFramingProbePid: child.pid, target })}\n`
      )
      const stdout = new Response(child.stdout).text()
      const stderr = new Response(child.stderr).text()
      let timer: ReturnType<typeof setTimeout> | undefined
      try {
        const exitCode = await Promise.race([
          child.exited,
          new Promise<never>((_resolve, reject) => {
            timer = setTimeout(
              () =>
                reject(new Error(`${target} response framing probe timed out`)),
              12000
            )
          }),
        ])
        expect({ target, exitCode, stderr: await stderr }).toEqual({
          target,
          exitCode: 0,
          stderr: "",
        })
        const output = await stdout
        expect(output).toContain("HTTP client response framing probe passed")
        process.stdout.write(output)
      } finally {
        clearTimeout(timer)
        if (child.exitCode === null) {
          child.kill("SIGTERM")
          const stopped = await Promise.race([
            child.exited.then(() => true),
            Bun.sleep(500).then(() => false),
          ])
          if (!stopped) {
            child.kill("SIGKILL")
            const killed = await Promise.race([
              child.exited.then(() => true),
              Bun.sleep(500).then(() => false),
            ])
            expect(killed).toBe(true)
          }
        }
        expect(() => process.kill(child.pid, 0)).toThrow()
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
    for (const pid of pids) expect(() => process.kill(pid, 0)).toThrow()
    expect(existsSync(directory)).toBe(false)
    process.stdout.write(
      `${JSON.stringify({ responseFramingFixtureRoot: directory, removed: !existsSync(directory), pids })}\n`
    )
  }
}, 30000)
