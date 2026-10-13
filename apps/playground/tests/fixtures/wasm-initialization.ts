import { mock } from "bun:test"
import assert from "node:assert/strict"

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: Error) => void
  const promise = new Promise<T>((yes, no) => {
    resolve = yes
    reject = no
  })
  return { promise, resolve, reject }
}

const first = deferred<object>()
const retry = deferred<object>()
let starts = 0
let analyses = 0
let operationFails = false
const calls: string[] = []
const result = { status: "success", schema: 1, documents: [] }
mock.module("../../src/wasm/pkg/seseragi_wasm.js", () => ({
  default: () => {
    starts++
    return starts === 1 ? first.promise : retry.promise
  },
  analyze_project: () => {
    analyses++
    if (operationFails) throw new Error("compiler operation failed")
    return JSON.stringify(result)
  },
  compile_project: () => {
    calls.push("compile")
    return JSON.stringify(result)
  },
  analyze_single_file: () => {
    calls.push("recovery")
    return "{}"
  },
  convert_workspace_bindings: () => {
    calls.push("convert")
    return "{}"
  },
  format_project_file: () => {
    calls.push("format")
    return "{}"
  },
  format_project_file_with_options: () => {
    calls.push("format options")
    return "{}"
  },
  resolve_browser_runtime_modules: () => {
    calls.push("runtime")
    return "{}"
  },
}))

const service = await import("../../src/compiler/wasm-service")
const project = {
  schema: 1 as const,
  entry: "main.ssrg",
  files: [{ path: "main.ssrg", source: "pub fn main = 1" }],
}
const rejected = Promise.allSettled([
  service.analyzeProject(project),
  service.compileProject(project),
  service.analyzeProject(project),
])
assert.equal(starts, 1)
first.reject(new Error("transient WASM fetch failure"))
const failures = await rejected
assert(
  failures.every(
    (value) =>
      value.status === "rejected" &&
      String(value.reason).includes("transient WASM fetch failure")
  )
)
assert.equal(starts, 1)
assert.equal(analyses, 0)
assert.equal(calls.length, 0)

const waiting = Promise.all([
  service.analyzeProject(project),
  service.analyzeProject(project),
  service.compileProject(project),
])
assert.equal(starts, 2)
assert.equal(analyses, 0)
retry.resolve({})
assert.deepEqual(await waiting, [result, result, result])
assert.equal(analyses, 2)
assert.deepEqual(calls, ["compile"])

operationFails = true
await assert.rejects(
  service.analyzeProject(project),
  /compiler operation failed/
)
operationFails = false
assert.deepEqual(await service.analyzeProject(project), result)
assert.equal(starts, 2)
assert.equal(analyses, 4)
await service.formatProjectFile(project, "main.ssrg")
await service.formatProjectFile(project, "main.ssrg", { lineWidth: 80 })
await service.convertWorkspaceBindings({} as never)
await service.resolveBrowserRuntimeModules({} as never)
assert.equal(starts, 2)
assert.deepEqual(calls, [
  "compile",
  "format",
  "format options",
  "convert",
  "runtime",
])
console.log("WASM initialization regression passed")
