import { readFile, writeFile } from "node:fs/promises"

const configuration: { mode: string; descendant?: string } = JSON.parse(
  await readFile(process.argv[2]!, "utf8")
)
console.error(`worker diagnostic: ${configuration.mode}`)
const server = Bun.serve({
  hostname: "127.0.0.1",
  port: 0,
  fetch: () => new Response("owned listener"),
})
process.on("message", async (value: unknown) => {
  const message = value as { kind: string }
  if (message.kind !== "stop" || configuration.mode === "ignore-stop") return
  if (configuration.mode === "inherited-stderr") {
    const descendant = Bun.spawn(
      [process.execPath, "-e", "setTimeout(() => {}, 20000)"],
      { stdin: "ignore", stdout: "ignore", stderr: "inherit" }
    )
    await writeFile(configuration.descendant!, String(descendant.pid))
  }
  await server.stop(true)
  process.disconnect?.()
  process.exit(0)
})
if (configuration.mode !== "never-ready") {
  process.send?.({
    kind: "ready",
    status: {
      port: server.port,
      pendingRequests: server.pendingRequests,
      pendingWebSockets: server.pendingWebSockets,
    },
  })
}
