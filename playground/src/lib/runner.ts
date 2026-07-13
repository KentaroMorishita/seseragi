import { executeGeneratedModule } from "./browser-execution"
import { compileWithSharedDriver, renderDiagnostics } from "./wasm-compiler"

export async function compileAndRun(seseragiCode: string): Promise<string> {
  const compiled = await compileWithSharedDriver(seseragiCode)
  if (compiled.status === "failure") {
    throw new Error(renderDiagnostics(compiled.diagnostics.diagnostics))
  }
  if (compiled.entry === undefined) {
    throw new Error(compiled.entryError ?? "program has no executable main")
  }
  return executeGeneratedModule(compiled.generated.typescript, compiled.entry)
}
