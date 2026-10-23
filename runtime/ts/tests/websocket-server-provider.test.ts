import { expect, test } from "bun:test"
import { mkdtempSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { ServerConnectionToken } from "../../providers/websocket-server-common"

test("WebSocket server isolates malformed input and handler failures in real Node and Bun processes", async () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-websocket-isolation-"))
  try {
    const probe = resolve(
      import.meta.dir,
      "../probes/websocket-server-provider.ts"
    )
    const built = await Bun.build({
      entrypoints: [probe],
      outdir: directory,
      target: "node",
    })
    expect(built.success).toBe(true)
    for (const [command, script, target, scenario] of [
      ["node", join(directory, "websocket-server-provider.js"), "node", "all"],
      [process.execPath, probe, "bun", "all"],
      [process.execPath, probe, "bun", "drain"],
    ]) {
      const child = Bun.spawn([command!, script!, target!, scenario!], {
        stdout: "pipe",
        stderr: "pipe",
      })
      const stdout = new Response(child.stdout).text()
      const stderr = new Response(child.stderr).text()
      let timeout: ReturnType<typeof setTimeout> | undefined
      try {
        const exited = await Promise.race([
          child.exited,
          new Promise<never>((_resolve, reject) => {
            timeout = setTimeout(
              () => reject(new Error("WebSocket probe timed out")),
              12_000
            )
          }),
        ])
        expect({ target, exited, stderr: await stderr }).toEqual({
          target,
          exited: 0,
          stderr: "",
        })
        expect(await stdout).toContain(
          `WebSocket server isolation probe passed: ${target}`
        )
      } finally {
        if (timeout !== undefined) clearTimeout(timeout)
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
}, 30_000)

test("a failed WebSocket releases pending send slots and drops queued sends", async () => {
  let rejectWrite = (_cause: Error): void => undefined
  let writes = 0
  const write = new Promise<void>((_resolve, reject) => {
    rejectWrite = reject
  })
  const connection = new ServerConnectionToken(
    {
      protocol: "",
      write() {
        writes += 1
        return write
      },
      close() {},
    },
    4
  )
  const sends = Array.from({ length: 16 }, () =>
    connection.send({
      tag: "TextMessage",
      text: "small",
    })
  )
  await Promise.resolve()
  expect(writes).toBe(1)
  expect(
    await connection.send({ tag: "TextMessage", text: "full" })
  ).toMatchObject({
    kind: "failure",
    failure: {
      tag: "WebSocketSendFailed",
      message: expect.stringContaining("backpressure"),
    },
  })
  connection.fail(new Error("fixture transport failure"))
  rejectWrite(new Error("fixture closed transport"))
  const outcomes = await Promise.all(sends)
  expect(outcomes.every((outcome) => outcome.kind === "failure")).toBe(true)
  expect(writes).toBe(1)
  expect(await connection.send({ tag: "TextMessage", text: "closed" })).toEqual(
    {
      kind: "failure",
      failure: { tag: "WebSocketClosed", message: "WebSocket is closed" },
    }
  )
})
