import { test } from "bun:test"
import assert from "node:assert/strict"
import { createServer, type Socket } from "node:net"
import { createHttpProxyConnection } from "../../providers/http-proxy-connection"

test.each(["status", "headers", "information", "upgrade", "cancel"])(
  "bounded CONNECT %s",
  async (mode) => {
    const sockets = new Set<Socket>()
    let authenticated = false
    const server = createServer((socket) => {
      sockets.add(socket)
      socket.on("error", () => {})
      socket.once("close", () => sockets.delete(socket))
      let input = ""
      socket.on("data", function onData(bytes) {
        input += bytes.toString("latin1")
        assert(input.length < 4096)
        if (!input.includes("\r\n\r\n")) return
        socket.off("data", onData)
        // Check the actual fixed fixture credential without retaining it in reports.
        authenticated = input.includes(
          "Basic " + Buffer.from("fixture-user:fixture-pass").toString("base64")
        )
        if (mode === "status")
          socket.end(
            "HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 0\r\n\r\n"
          )
        if (mode === "headers")
          socket.write(`HTTP/1.1 200 OK\r\nX-Large: ${"x".repeat(20000)}`)
        if (mode === "information")
          socket.write("HTTP/1.1 103 Early Hints\r\n\r\n".repeat(17))
        if (mode === "upgrade")
          socket.end(
            "HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: fixture\r\n\r\n"
          )
      })
    })
    await new Promise<void>((r) => server.listen(0, "127.0.0.1", r))
    const address = server.address()
    assert(address && typeof address !== "string")
    const port = address.port
    const connection = createHttpProxyConnection(
      new URL("https://origin.invalid/"),
      new URL(`http://fixture-user:fixture-pass@127.0.0.1:${port}`),
      []
    )
    let timer: ReturnType<typeof setTimeout> | undefined
    let deadlineTimer: ReturnType<typeof setTimeout> | undefined
    try {
      if (mode === "cancel") timer = setTimeout(() => connection.cancel(), 20)
      await assert.rejects(
        Promise.race([
          connection.ready,
          new Promise<never>((_, reject) => {
            deadlineTimer = setTimeout(
              () => reject(new Error("probe timed out")),
              800
            )
          }),
        ]),
        (err) => {
          assert(!String(err).includes("timed out"))
          if (mode === "headers" || mode === "information")
            assert.equal((err as { tag?: string }).tag, "HttpProtocolFailure")
          return true
        }
      )
      assert(authenticated)
    } finally {
      clearTimeout(timer)
      clearTimeout(deadlineTimer)
      connection.cancel()
      for (const socket of sockets) socket.destroy()
      await new Promise<void>((r) => server.close(() => r()))
    }
  },
  4000
)
