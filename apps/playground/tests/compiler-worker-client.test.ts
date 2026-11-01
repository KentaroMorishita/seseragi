import { expect, test } from "bun:test"
import { createCompilerWorkerClient } from "../src/compiler/compiler-worker-client"
import type {
  CompilerReply,
  CompilerRequest,
} from "../src/compiler/compiler-worker-protocol"
import type { ProjectCompileResponse } from "../src/compiler/types"

class TestWorker extends EventTarget {
  requests: CompilerRequest[] = []
  terminated = false
  postMessage(request: CompilerRequest) {
    this.requests.push(request)
  }
  terminate() {
    this.terminated = true
  }
  reply(reply: CompilerReply) {
    this.dispatchEvent(new MessageEvent("message", { data: reply }))
  }
}

const project = {
  schema: 1 as const,
  entry: "main.ssrg",
  files: [{ path: "main.ssrg", source: "pub fn main = 1" }],
}
const analyzed = { status: "success" as const, schema: 1, documents: [] }
const compiled = {
  status: "failure" as const,
  schema: 1,
  diagnostics: [],
  problems: [],
}

test("compiler worker matches out-of-order replies to their callers", async () => {
  const worker = new TestWorker()
  const client = createCompilerWorkerClient(() => worker)
  const analysis = client.call("analyzeProject", project)
  const compilation = client.call("compileProject", project)
  expect(worker.requests.map(({ method }) => method)).toEqual([
    "analyzeProject",
    "compileProject",
  ])
  worker.reply({ id: 2, ok: true, result: compiled })
  worker.reply({ id: 1, ok: true, result: analyzed })
  expect(await analysis).toEqual(analyzed)
  expect(await compilation).toEqual(compiled)
})

test("compiler operation errors reject their request and allow another operation", async () => {
  const worker = new TestWorker()
  const client = createCompilerWorkerClient(() => worker)
  const request = client.call("analyzeProject", project)
  worker.reply({ id: 1, ok: false, error: "WASM failed", fatal: false })
  await expect(request).rejects.toThrow("WASM failed")
  expect(worker.terminated).toBe(false)
  const next = client.call("analyzeProject", project)
  worker.reply({ id: 2, ok: true, result: analyzed })
  expect(await next).toEqual(analyzed)
})

test("source diagnostics keep the compiler worker available", async () => {
  const worker = new TestWorker()
  let starts = 0
  const client = createCompilerWorkerClient(() => {
    starts++
    return worker
  })
  const request = client.call("compileProject", project)
  worker.reply({ id: 1, ok: true, result: compiled })
  expect(await request).toEqual(compiled)
  const next = client.call("analyzeProject", project)
  worker.reply({ id: 2, ok: true, result: analyzed })
  expect(await next).toEqual(analyzed)
  expect(worker.terminated).toBe(false)
  expect(starts).toBe(1)
})

test("a WASM trap rejects all pending requests and the next compile uses a fresh worker", async () => {
  const first = new TestWorker()
  const second = new TestWorker()
  let starts = 0
  const client = createCompilerWorkerClient(() =>
    ++starts === 1 ? first : second
  )
  const analysis = client.call("analyzeProject", project)
  const compilation = client.call("compileProject", project)
  const results = Promise.allSettled([analysis, compilation])
  const error = "Internal compiler error: unreachable. Please try again."
  first.reply({ id: 2, ok: false, error, fatal: true })
  for (const result of await results) {
    expect(result.status).toBe("rejected")
    if (result.status === "rejected") expect(result.reason.message).toBe(error)
  }
  expect(first.terminated).toBe(true)

  const validProject = {
    ...project,
    files: [{ path: "main.ssrg", source: 'pub effect fn main = println "ok"' }],
  }
  const next = client.call("compileProject", validProject)
  expect(second.requests).toEqual([
    { id: 3, method: "compileProject", args: [validProject] },
  ])
  first.reply({ id: 3, ok: false, error, fatal: true })
  first.reply({ id: 3, ok: true, result: compiled })
  expect(second.terminated).toBe(false)
  const success: ProjectCompileResponse = {
    status: "success",
    schema: 1,
    diagnostics: [],
    modules: [],
    entry: { path: "main.ssrg", module: "main" },
  }
  second.reply({ id: 3, ok: true, result: success })
  expect(await next).toEqual(success)
  expect(starts).toBe(2)
})

test("compiler worker reports WASM traps as internal errors", () => {
  const result = Bun.spawnSync([
    process.execPath,
    new URL("./fixtures/compiler-worker-trap.ts", import.meta.url).pathname,
  ])
  expect(result.stderr.toString()).toBe("")
  expect(result.exitCode).toBe(0)
  expect(result.stdout.toString()).toContain(
    "Compiler worker trap regression passed"
  )
})

test("worker failure rejects pending requests and a later request starts a fresh worker", async () => {
  const first = new TestWorker()
  const second = new TestWorker()
  let starts = 0
  const client = createCompilerWorkerClient(() =>
    ++starts === 1 ? first : second
  )
  const analysis = client.call("analyzeProject", project)
  const compilation = client.call("compileProject", project)
  first.dispatchEvent(new Event("messageerror"))
  await expect(analysis).rejects.toThrow("could not be read")
  await expect(compilation).rejects.toThrow("could not be read")
  expect(first.terminated).toBe(true)
  const next = client.call("analyzeProject", project)
  first.dispatchEvent(new Event("messageerror"))
  expect(second.terminated).toBe(false)
  second.reply({ id: 3, ok: true, result: analyzed })
  expect(await next).toEqual(analyzed)
  expect(starts).toBe(2)
})

test("worker startup failures reject without leaving analysis pending", async () => {
  const client = createCompilerWorkerClient(() => {
    throw new Error("Worker unavailable")
  })
  await expect(client.call("analyzeProject", project)).rejects.toThrow(
    "Worker unavailable"
  )
})

test("a hung compiler is terminated and the next request uses a fresh worker", async () => {
  const first = new TestWorker()
  const second = new TestWorker()
  let starts = 0
  const client = createCompilerWorkerClient(
    () => (++starts === 1 ? first : second),
    20
  )
  const request = client.call("analyzeProject", project)
  const queued = client.call("compileProject", project)
  const results = await Promise.allSettled([request, queued])
  for (const result of results) {
    expect(result.status).toBe("rejected")
    if (result.status === "rejected")
      expect(result.reason.message).toContain("Compiler time limit exceeded")
  }
  expect(first.terminated).toBe(true)
  const next = client.call("analyzeProject", project)
  first.reply({ id: 3, ok: true, result: compiled })
  second.reply({ id: 3, ok: true, result: analyzed })
  expect(await next).toEqual(analyzed)
  await new Promise((resolve) => setTimeout(resolve, 30))
  expect(second.terminated).toBe(false)
})
