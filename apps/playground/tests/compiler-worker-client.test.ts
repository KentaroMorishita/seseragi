import { expect, test } from "bun:test"
import { createCompilerWorkerClient } from "../src/compiler/compiler-worker-client"
import type {
  CompilerReply,
  CompilerRequest,
} from "../src/compiler/compiler-worker-protocol"

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
  worker.reply({ id: 1, ok: false, error: "WASM failed" })
  await expect(request).rejects.toThrow("WASM failed")
  const next = client.call("analyzeProject", project)
  worker.reply({ id: 2, ok: true, result: analyzed })
  expect(await next).toEqual(analyzed)
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
