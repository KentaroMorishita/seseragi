import { expect, test } from "bun:test"
import { mkdtemp, rm } from "node:fs/promises"
import { type ClientRequest, request as httpRequest } from "node:http"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { createBunHttpServerProvider } from "../../../runtime/providers/bun/http-server"
import type { ProviderResult } from "../../../runtime/ts/src/provider"

test("bounds HTTP uploads and releases incomplete requests on Node", async () => {
  const directory = await mkdtemp(join(tmpdir(), "seseragi-http-body-bounds-"))
  let child: Bun.Subprocess<"ignore", "pipe", "pipe"> | undefined
  let timeout: ReturnType<typeof setTimeout> | undefined
  try {
    const build = await Bun.build({
      entrypoints: [
        new URL(
          "../../../runtime/ts/probes/http-server-request-bounds.ts",
          import.meta.url
        ).pathname,
      ],
      target: "node",
      outdir: directory,
      naming: "probe.mjs",
    })
    expect(build.success).toBe(true)
    child = Bun.spawn(["node", join(directory, "probe.mjs")], {
      stdin: "ignore",
      stdout: "pipe",
      stderr: "pipe",
    })
    const stdout = new Response(child.stdout).text()
    const stderr = new Response(child.stderr).text()
    const exited = await Promise.race([
      child.exited,
      new Promise<never>((_resolve, reject) => {
        timeout = setTimeout(
          () => reject(new Error("Node probe timed out")),
          12_000
        )
      }),
    ])
    expect(await stderr).toBe("")
    expect(await stdout).toBe("HTTP request body bounds probe passed\n")
    expect(exited).toBe(0)
  } finally {
    clearTimeout(timeout)
    if (child !== undefined && child.exitCode === null) {
      child.kill("SIGKILL")
      await child.exited
    }
    if (child !== undefined) {
      const childPid = child.pid
      expect(() => process.kill(childPid, 0)).toThrow()
    }
    await rm(directory, { recursive: true, force: true })
  }
}, 15_000)

test("keeps Bun's native body cap before the application handler", async () => {
  const provider = createBunHttpServerProvider()
  let handled = 0
  const started = (await provider.listen?.({
    hostname: "127.0.0.1",
    port: 0,
    async handler() {
      handled += 1
      return { status: 200, headers: [], body: new Uint8Array() }
    },
  })) as ProviderResult
  if (started.kind !== "success") throw new Error("Bun listener did not start")
  const token = started.value as { server: { port: number } }
  let request: ClientRequest | undefined
  let timeout: ReturnType<typeof setTimeout> | undefined
  try {
    const status = await new Promise<number | undefined>((resolve, reject) => {
      timeout = setTimeout(() => reject(new Error("Bun cap timed out")), 2_000)
      request = httpRequest(
        {
          host: "127.0.0.1",
          port: token.server.port,
          method: "POST",
          headers: {
            connection: "close",
            "content-length": 128 * 1024 * 1024 + 1,
          },
        },
        (response) => {
          response.on("error", reject)
          response.once("end", () => resolve(response.statusCode))
          response.resume()
        }
      )
      request.once("error", reject)
      // Only send headers: no large body or allocation is needed to verify
      // the host's existing 128 MiB early-rejection boundary.
      request.flushHeaders()
    })
    expect(status).toBe(413)
    expect(handled).toBe(0)
  } finally {
    clearTimeout(timeout)
    request?.destroy()
    await provider.close?.(started.value)
  }
})
