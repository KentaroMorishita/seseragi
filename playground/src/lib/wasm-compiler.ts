import init, { compile_single_file } from "../wasm/pkg/seseragi_wasm"
import type { CompileResponse, Diagnostic } from "./playground-types"

let initialization: Promise<unknown> | undefined

export async function compileWithSharedDriver(
  source: string
): Promise<CompileResponse> {
  initialization ??= init()
  await initialization
  return JSON.parse(
    compile_single_file("playground.ssrg", "playground/main", source)
  ) as CompileResponse
}

export function renderDiagnostics(diagnostics: readonly Diagnostic[]): string {
  return diagnostics
    .map(
      (diagnostic) =>
        `${diagnostic.code}: ${diagnostic.messageKey} ` +
        `[${diagnostic.primary.start}, ${diagnostic.primary.end})`
    )
    .join("\n")
}
