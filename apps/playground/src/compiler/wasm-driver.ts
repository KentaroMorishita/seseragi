import { createCompilerWorkerClient } from "./compiler-worker-client"
import type { CompilerService } from "./compiler-worker-protocol"

const compiler = createCompilerWorkerClient(
  () =>
    new Worker(new URL("./compiler.worker.ts", import.meta.url), {
      type: "module",
    })
)

// Keep the browser-facing API identical; only the execution thread changes.
export const compileSingleFile: CompilerService["compileSingleFile"] = (
  source
) => compiler.call("compileSingleFile", source)
export const analyzeSingleFile: CompilerService["analyzeSingleFile"] = (
  source
) => compiler.call("analyzeSingleFile", source)
export const formatSingleFile: CompilerService["formatSingleFile"] = (
  source,
  options
) => compiler.call("formatSingleFile", source, options)
export const compileProject: CompilerService["compileProject"] = (request) =>
  compiler.call("compileProject", request)
export const analyzeProject: CompilerService["analyzeProject"] = (request) =>
  compiler.call("analyzeProject", request)
export const formatProjectFile: CompilerService["formatProjectFile"] = (
  request,
  path,
  options
) => compiler.call("formatProjectFile", request, path, options)
export const convertWorkspaceBindings: CompilerService["convertWorkspaceBindings"] =
  (request) => compiler.call("convertWorkspaceBindings", request)
