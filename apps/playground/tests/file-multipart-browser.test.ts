import { afterAll, beforeAll, expect, test } from "bun:test"
import { mkdtemp, readFile, rm } from "node:fs/promises"
import {
  createSecureServer,
  type Http2SecureServer,
  type ServerHttp2Stream,
} from "node:http2"
import { tmpdir } from "node:os"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { type Browser, chromium } from "playwright"
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
  const cli = await ensureSeseragiCli()
  await runCommand([
    cli,
    "build",
    fixture,
    "--out-dir",
    output,
  ])
  uploaded = new Promise((resolveUpload) => {
    resolveUploaded = resolveUpload
  })
  server = createSecureServer({
    key: await readFile(key),
    cert: await readFile(certificate),
  })
  server.on("stream", (stream: ServerHttp2Stream, headers) => {
    void (async () => {
      const method = String(headers[":method"] ?? "GET")
      const pathname = String(headers[":path"] ?? "/")
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
  await new Promise<void>((resolveListen) => {
    server?.listen(41289, "127.0.0.1", resolveListen)
  })
  browser = await chromium.launch()
}, 120_000)

afterAll(async () => {
  await browser?.close()
  await new Promise<void>((resolveClose) => {
    if (server === undefined) resolveClose()
    else server.close(() => resolveClose())
  })
  if (temporary !== "") await rm(temporary, { recursive: true, force: true })
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
  await page.goto("https://127.0.0.1:41289")
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
      `${String(error)}; status=${status}; errors=${JSON.stringify(errors)}; html=${html}`
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
  return await Promise.race([
    value,
    new Promise<Value>((_resolve, reject) => {
      setTimeout(
        () => reject(new Error("timed out waiting for upload")),
        milliseconds
      )
    }),
  ])
}
