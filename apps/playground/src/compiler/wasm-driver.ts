import init, {
  analyze_single_file,
  compile_single_file,
} from "../wasm/pkg/seseragi_wasm"
import type { AnalysisDocument, CompileResponse } from "./types"

let initialization: Promise<unknown> | undefined

export async function compileSingleFile(
  source: string
): Promise<CompileResponse> {
  initialization ??= init()
  await initialization
  return JSON.parse(
    compile_single_file("playground.ssrg", "playground/main", source)
  ) as CompileResponse
}

export async function analyzeSingleFile(
  source: string
): Promise<AnalysisDocument> {
  initialization ??= init()
  await initialization
  return JSON.parse(
    analyze_single_file("playground.ssrg", "playground/main", source)
  ) as AnalysisDocument
}
