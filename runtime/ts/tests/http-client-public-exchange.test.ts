import { expect, test } from "bun:test"
import { existsSync, mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"

test("native public exchange retains gzip bytes for proxies and explicit decoders while helpers limit decoded bodies", async () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-public-exchange-contract-")
  )
  const pids: number[] = []
  try {
    const build = await Bun.build({
      entrypoints: [
        join(import.meta.dir, "../probes/http-client-public-exchange.ts"),
      ],
      outdir: directory,
      target: "node",
    })
    expect(build.success).toBe(true)
    for (const [command, target] of [
      ["node", "node-process"],
      [process.execPath, "bun-process"],
    ]) {
      const child = Bun.spawn(
        [command!, join(directory, "http-client-public-exchange.js"), target!],
        { stdout: "pipe", stderr: "pipe" }
      )
      pids.push(child.pid)
      process.stdout.write(
        `${JSON.stringify({ compatibilityPid: child.pid, command, target })}\n`
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
                reject(new Error("public exchange contract probe timed out")),
              10000
            )
          }),
        ])
        expect({ exitCode, stderr: await stderr }).toEqual({
          exitCode: 0,
          stderr: "",
        })
        process.stdout.write(await stdout)
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
            expect(
              await Promise.race([
                child.exited.then(() => true),
                Bun.sleep(500).then(() => false),
              ])
            ).toBe(true)
          }
        }
        expect(() => process.kill(child.pid, 0)).toThrow()
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
    expect(existsSync(directory)).toBe(false)
    for (const pid of pids) expect(() => process.kill(pid, 0)).toThrow()
    process.stdout.write(
      `${JSON.stringify({
        compatibilityFixtureRoot: directory,
        removed: true,
        pids,
      })}\n`
    )
  }
}, 30000)
