import { mock } from "bun:test"
import assert from "node:assert/strict"
import type {
  CompilerReply,
  CompilerRequest,
} from "../../src/compiler/compiler-worker-protocol"
import type { ProjectCompileResponse } from "../../src/compiler/types"

let failure: unknown
let result: ProjectCompileResponse = {
  status: "failure",
  schema: 1,
  diagnostics: [],
  problems: [],
}
mock.module("../../src/wasm/pkg/seseragi_wasm.js", () => ({
  default: async () => ({}),
  compile_project: () => {
    if (failure !== undefined) throw failure
    return JSON.stringify(result)
  },
  analyze_project: () => "{}",
  analyze_single_file: () => "{}",
  convert_workspace_bindings: () => "{}",
  format_project_file: () => "{}",
  format_project_file_with_options: () => "{}",
  resolve_browser_runtime_modules: () => "{}",
}))

const scope = globalThis as unknown as {
  onmessage: ((event: MessageEvent<CompilerRequest>) => void) | null
  postMessage: (reply: CompilerReply) => void
}
const previous = {
  onmessage: scope.onmessage,
  postMessage: scope.postMessage,
}
await import("../../src/compiler/compiler.worker")
const onmessage = scope.onmessage
assert(onmessage)

const request: CompilerRequest = {
  id: 1,
  method: "compileProject",
  args: [
    {
      schema: 1,
      entry: "main.ssrg",
      files: [
        { path: "main.ssrg", source: 'pub effect fn main = println "ok"' },
      ],
    },
  ],
}
const compile = () =>
  new Promise<CompilerReply>((resolve) => {
    scope.postMessage = resolve
    onmessage(new MessageEvent("message", { data: request }))
  })

try {
  assert.deepEqual(await compile(), { id: 1, ok: true, result })

  for (failure of [
    new Error("compiler operation failed"),
    "compiler operation failed",
    Object.assign(new Error("compiler operation failed"), {
      name: "RuntimeError",
    }),
  ]) {
    assert.deepEqual(await compile(), {
      id: 1,
      ok: false,
      error: "compiler operation failed",
      fatal: false,
    })
  }

  failure = undefined
  result = {
    status: "success",
    schema: 1,
    diagnostics: [],
    modules: [],
    entry: { path: "main.ssrg", module: "main" },
  }
  assert.deepEqual(await compile(), { id: 1, ok: true, result })

  failure = new WebAssembly.RuntimeError("unreachable")
  assert.deepEqual(await compile(), {
    id: 1,
    ok: false,
    error: "Internal compiler error: unreachable. Please try again.",
    fatal: true,
  })
  console.log("Compiler worker trap regression passed")
} finally {
  scope.onmessage = previous.onmessage
  scope.postMessage = previous.postMessage
}
