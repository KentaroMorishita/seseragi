import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import type { FixtureServerConfiguration } from "../fixture-server-support"

const path = process.argv[2]
if (path === undefined) throw new Error("missing fixture configuration")
const configuration: FixtureServerConfiguration = JSON.parse(
  await readFile(path, "utf8")
)
// This process owns the listener until its parent explicitly stops it. Page and
// Bun:test lifecycle operations in the parent cannot dispose this server.
const server = Bun.serve({
  hostname: "127.0.0.1",
  port: 0,
  async fetch(request) {
    const match = /^\/([^/]+)(\/.*)?$/.exec(new URL(request.url).pathname)
    const fixture = match?.[1]
    const output =
      fixture === undefined ? undefined : configuration.outputs[fixture]
    const index =
      fixture === undefined ? undefined : configuration.indexes[fixture]
    if (output === undefined || index === undefined) {
      return new Response("Not found", { status: 404 })
    }
    const relative =
      match?.[2] === undefined || match[2] === "/"
        ? "index.html"
        : match[2].slice(1)
    if (relative === "index.html") {
      return new Response(index, {
        headers: { "content-type": "text/html; charset=utf-8" },
      })
    }
    const file = resolve(output, relative)
    if (!file.startsWith(`${output}/`)) {
      return new Response("Forbidden", { status: 403 })
    }
    return new Response(await readFile(file), {
      headers: {
        "content-type": relative.endsWith(".js")
          ? "text/javascript; charset=utf-8"
          : relative.endsWith(".css")
            ? "text/css; charset=utf-8"
            : "application/octet-stream",
      },
    })
  },
})
const status = () => ({
  port: server.port ?? -1,
  pendingRequests: server.pendingRequests,
  pendingWebSockets: server.pendingWebSockets,
})
let stopping: Promise<void> | undefined
const stop = () => {
  stopping ??= (async () => {
    await server.stop(true)
    if (process.connected) process.disconnect?.()
  })()
  return stopping
}
process.on("message", (value: unknown) => {
  const message = value as { kind: string; id?: number }
  if (message.kind === "stop") {
    void stop()
  } else if (message.kind === "status") {
    process.send?.({ kind: "status", id: message.id, status: status() })
  }
})
process.on("disconnect", () => void stop())
process.send?.({ kind: "ready", status: status() })
