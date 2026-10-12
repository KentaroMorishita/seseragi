import type { CompilerReply, CompilerRequest } from "./compiler-worker-protocol"
import * as service from "./wasm-service"

const scope = globalThis as unknown as {
  onmessage: (event: MessageEvent<CompilerRequest>) => void
  postMessage: (reply: CompilerReply) => void
}

async function execute(request: CompilerRequest) {
  switch (request.method) {
    case "compileSingleFile":
      return service.compileSingleFile(...request.args)
    case "analyzeSingleFile":
      return service.analyzeSingleFile(...request.args)
    case "formatSingleFile":
      return service.formatSingleFile(...request.args)
    case "compileProject":
      return service.compileProject(...request.args)
    case "analyzeProject":
      return service.analyzeProject(...request.args)
    case "formatProjectFile":
      return service.formatProjectFile(...request.args)
    case "convertWorkspaceBindings":
      return service.convertWorkspaceBindings(...request.args)
  }
}

scope.onmessage = (event) => {
  const request = event.data
  void execute(request).then(
    (result) => scope.postMessage({ id: request.id, ok: true, result }),
    (error: unknown) =>
      scope.postMessage({
        id: request.id,
        ok: false,
        error: error instanceof Error ? error.message : String(error),
      })
  )
}
