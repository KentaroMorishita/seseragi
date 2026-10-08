import { afterAll, beforeAll, expect, test } from "bun:test"
import { cp, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import {
  createSecureServer,
  type Http2SecureServer,
  type ServerHttp2Session,
  type ServerHttp2Stream,
} from "node:http2"
import { type AddressInfo, createConnection } from "node:net"
import { tmpdir } from "node:os"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import type { Browser } from "playwright"
import { launchTestBrowser } from "./browser-test-support"
import { ensureSeseragiCli, runCommand } from "./cli-test-support"

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..")
const fixture = resolve(
  root,
  "examples/spec/fixtures/projects/file-multipart-browser-e2e"
)
let browser: Browser | undefined
let server: Http2SecureServer | undefined
let temporary = ""
let output = ""
let origin = ""
const sessions = new Set<ServerHttp2Session>()
const transportEvents: string[] = []
function recordTransport(event: string, error: Error) {
  const detail = `${event}: ${(error as NodeJS.ErrnoException).code ?? ""} ${error.message}`
  transportEvents.push(detail)
  console.log(`Multipart transport: ${detail}`)
}
let uploaded!: Promise<UploadedRequest>
let resolveUploaded!: (request: UploadedRequest) => void

type UploadedRequest = Readonly<{
  readonly contentType: string
  readonly body: Uint8Array
}>

beforeAll(async () => {
  temporary = await mkdtemp(resolve(tmpdir(), "seseragi-file-upload-"))
  output = resolve(temporary, "build")
  const key = resolve(temporary, "localhost-key.pem")
  const certificate = resolve(temporary, "localhost-cert.pem")
  await runCommand([
    "openssl",
    "req",
    "-x509",
    "-newkey",
    "rsa:2048",
    "-nodes",
    "-keyout",
    key,
    "-out",
    certificate,
    "-subj",
    "/CN=127.0.0.1",
    "-days",
    "1",
    "-addext",
    "subjectAltName=IP:127.0.0.1",
  ])
  uploaded = new Promise((resolveUpload) => {
    resolveUploaded = resolveUpload
  })
  server = createSecureServer({
    key: await readFile(key),
    cert: await readFile(certificate),
  })
  // A browser may abandon a speculative TLS connection before sending a request.
  // Bun 1.3.x otherwise destroys this socket with the error a second time,
  // emitting an unhandled error and aborting an unrelated in-flight navigation.
  server.on("clientError", (error, socket) => {
    recordTransport("clientError", error)
    socket.destroy()
  })
  server.on("tlsClientError", (error) => {
    recordTransport("tlsClientError", error)
  })
  server.on("sessionError", (error) => {
    recordTransport("sessionError", error)
  })
  server.on("session", (session) => {
    sessions.add(session)
    session.once("close", () => sessions.delete(session))
  })
  server.on("stream", (stream: ServerHttp2Stream, headers) => {
    void (async () => {
      const method = String(headers[":method"] ?? "GET")
      const pathname = String(headers[":path"] ?? "/")
      if (method === "GET" && pathname === "/favicon.ico") {
        stream.respond({ ":status": 204 })
        stream.end()
        return
      }
      if (method === "POST" && pathname === "/upload") {
        const chunks: Buffer[] = []
        for await (const chunk of stream) chunks.push(Buffer.from(chunk))
        resolveUploaded({
          contentType: String(headers["content-type"] ?? ""),
          body: new Uint8Array(Buffer.concat(chunks)),
        })
        stream.respond({ ":status": 201, "content-type": "text/plain" })
        stream.end("uploaded")
        return
      }
      const relative = pathname === "/" ? "index.html" : pathname.slice(1)
      const file = resolve(output, relative)
      if (!file.startsWith(`${output}/`)) {
        stream.respond({ ":status": 403 })
        stream.end("Forbidden")
        return
      }
      try {
        const content = await readFile(file)
        stream.respond({
          ":status": 200,
          "content-type": relative.endsWith(".js")
            ? "text/javascript; charset=utf-8"
            : relative.endsWith(".css")
              ? "text/css; charset=utf-8"
              : relative.endsWith(".html")
                ? "text/html; charset=utf-8"
                : "application/octet-stream",
        })
        stream.end(content)
      } catch {
        stream.respond({ ":status": 404 })
        stream.end("Not found")
      }
    })()
  })
  await new Promise<void>((resolveListen, rejectListen) => {
    server?.once("error", rejectListen)
    server?.listen(0, "127.0.0.1", () => {
      server?.off("error", rejectListen)
      resolveListen()
    })
  })
  const address = server.address() as AddressInfo
  origin = `https://127.0.0.1:${address.port}`
  console.log(`Multipart HTTP/2 listener: ${origin}; pid=${process.pid}`)
  // Configure the normal source before compiling, never patch generated JS.
  const project = resolve(temporary, "project")
  await cp(fixture, project, { recursive: true })
  const sourcePath = resolve(project, "src/main.ssrg")
  const source = await readFile(sourcePath, "utf8")
  const fixtureUrl = "https://127.0.0.1:41289/upload"
  expect(source.split(fixtureUrl)).toHaveLength(2)
  await writeFile(sourcePath, source.replace(fixtureUrl, `${origin}/upload`))
  const cli = await ensureSeseragiCli()
  await runCommand([cli, "lock", "update", project])
  await runCommand([cli, "build", project, "--out-dir", output])
  browser = await launchTestBrowser()
}, 120_000)

afterAll(async () => {
  try {
    await browser?.close()
  } finally {
    for (const session of sessions) session.destroy()
    try {
      await new Promise<void>((resolveClose, rejectClose) => {
        if (server === undefined || !server.listening) resolveClose()
        else
          server.close((error) => (error ? rejectClose(error) : resolveClose()))
      })
    } finally {
      if (temporary !== "")
        await rm(temporary, { recursive: true, force: true })
    }
  }
})

test("records an aborted TLS handshake without poisoning the upload server", async () => {
  if (server === undefined) throw new Error("upload server did not start")
  const aborted = new Promise<Error>((resolveAbort) => {
    server?.once("tlsClientError", resolveAbort)
  })
  const socket = createConnection(Number(new URL(origin).port), "127.0.0.1")
  await new Promise<void>((resolveConnect, rejectConnect) => {
    socket.once("connect", resolveConnect)
    socket.once("error", rejectConnect)
  })
  // Truncated ClientHello: this connection must fail, while later HTTPS still works.
  socket.write(Buffer.from([0x16, 0x03, 0x01, 0, 10]))
  socket.destroy()
  expect(await withTimeout(aborted, 5_000)).toHaveProperty("code", "ECONNRESET")
  expect(server.listening).toBe(true)
})

test("selects a browser File and streams a normal-source multipart POST", async () => {
  if (browser === undefined || server === undefined) {
    throw new Error("file upload browser harness did not start")
  }
  const page = await browser.newPage({ ignoreHTTPSErrors: true })
  const errors: string[] = []
  page.on("pageerror", (error) => errors.push(error.message))
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text())
  })
  page.on("requestfailed", (request) => {
    transportEvents.push(`${request.url()}: ${request.failure()?.errorText}`)
  })
  try {
    const response = await page.goto(origin)
    expect(response?.status()).toBe(200)
  } catch (error) {
    throw new Error(
      `HTTPS navigation failed at ${origin}; ${transportEvents.join("; ")}`,
      { cause: error }
    )
  }
  const payload = Buffer.alloc(128 * 1024, "a")
  await page.locator("#upload").setInputFiles({
    name: "large.txt",
    mimeType: "text/plain",
    buffer: payload,
  })
  let request: UploadedRequest
  try {
    request = await withTimeout(uploaded, 10_000)
  } catch (error) {
    const status = await page
      .locator("html")
      .getAttribute("data-seseragi-status")
    const html = await page.locator("body").innerHTML()
    throw new Error(
      `${String(error)}; status=${status}; errors=${JSON.stringify(errors)}; transport=${JSON.stringify(transportEvents)}; html=${html}`
    )
  }
  const boundary = request.contentType.slice(
    "multipart/form-data; boundary=".length
  )
  const wire = new TextDecoder().decode(request.body)

  expect(request.contentType).toMatch(
    /^multipart\/form-data; boundary=seseragi-[0-9a-f]{36}$/
  )
  expect(wire).toContain('name="size"\r\n')
  expect(wire).toContain("\r\n\r\n131072\r\n")
  expect(wire).toContain(
    'name="upload"; filename="large.txt"\r\nContent-Type: text/plain\r\n'
  )
  expect(wire).toContain(`\r\n${"a".repeat(128 * 1024)}\r\n`)
  expect(wire.endsWith(`--${boundary}--\r\n`)).toBe(true)
  await page.waitForTimeout(100)
  expect(await page.locator("html").getAttribute("data-seseragi-status")).toBe(
    "mounted"
  )
  expect(errors).toEqual([])
  await page.close()
}, 30_000)

async function withTimeout<Value>(
  value: Promise<Value>,
  milliseconds: number
): Promise<Value> {
  let timer: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      value,
      new Promise<Value>((_resolve, reject) => {
        timer = setTimeout(
          () => reject(new Error("timed out waiting for multipart harness")),
          milliseconds
        )
      }),
    ])
  } finally {
    clearTimeout(timer)
  }
}
