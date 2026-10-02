import { afterAll, beforeAll, expect, test } from "bun:test"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { staticSiteHandler } from "../scripts/static-site-handler"

const configuration = JSON.parse(
  readFileSync(resolve(import.meta.dir, "../vercel.json"), "utf8")
)
let directory: string
let server: ReturnType<typeof Bun.serve>

beforeAll(() => {
  directory = mkdtempSync(join(tmpdir(), "seseragi-static-handler-"))
  mkdirSync(join(directory, "ja"))
  mkdirSync(join(directory, "assets"))
  for (const [path, body] of [
    ["index.html", "<!doctype html><title>Home</title>"],
    ["ja/index.html", "<!doctype html><title>Japanese</title>"],
    ["assets/site.css", "body { color: black; }"],
    ["assets/site.js", "export const ready = true"],
    ["assets/icon.svg", '<svg xmlns="http://www.w3.org/2000/svg"></svg>'],
  ]) {
    writeFileSync(join(directory, path), body)
  }
  server = Bun.serve({
    hostname: "127.0.0.1",
    port: 0,
    fetch: staticSiteHandler(directory, configuration),
  })
})

afterAll(() => {
  server?.stop(true)
  if (directory) rmSync(directory, { recursive: true, force: true })
})

const request = (path: string, method = "GET") =>
  fetch(new URL(path, server.url), { method, redirect: "manual" })

function expectSecurityHeaders(response: Response) {
  for (const { key, value } of configuration.headers[0].headers) {
    expect(response.headers.get(key)).toBe(value)
  }
  expect(response.headers.has("undefined")).toBe(false)
}

test("serves generated routes and assets with MIME and production headers", async () => {
  for (const [path, type] of [
    ["/", "text/html"],
    ["/ja/", "text/html"],
    ["/assets/site.css?version=1", "text/css"],
    ["/assets/site.js", "text/javascript"],
    ["/assets/icon.svg", "image/svg+xml"],
  ]) {
    const response = await request(path)
    expect(response.status).toBe(200)
    expect(response.headers.get("content-type")).toStartWith(type)
    expectSecurityHeaders(response)
    expect((await response.text()).length).toBeGreaterThan(0)
  }
})

test("redirects extensionless paths with queries using configured trailing slash", async () => {
  const response = await request("/ja?language=ja")
  expect(response.status).toBe(308)
  expect(response.headers.get("location")).toBe("/ja/?language=ja")
  expectSecurityHeaders(response)
  expect(await response.text()).toBe("")
  expect((await request("/ja/?language=ja")).status).toBe(200)
})

test("returns 404 for missing pages and assets instead of a server error", async () => {
  for (const path of ["/does-not-exist/", "/assets/missing.css"]) {
    const response = await request(path)
    expect(response.status).toBe(404)
    expectSecurityHeaders(response)
    expect(await response.text()).toBe("Not found")
  }
  expect((await request("/does-not-exist")).status).toBe(308)
})

test("HEAD retains MIME and headers without a body", async () => {
  const response = await request("/ja/", "HEAD")
  expect(response.status).toBe(200)
  expect(response.headers.get("content-type")).toStartWith("text/html")
  expectSecurityHeaders(response)
  expect(await response.text()).toBe("")
})

test("rejects malformed and escaping paths", async () => {
  for (const path of ["/%ZZ", "/%2e%2e%2foutside.txt"]) {
    expect((await request(path)).status).toBe(404)
  }
})
