import init, { compile_single_file } from "../wasm/pkg/seseragi_wasm"
import type { CompileResponse } from "./types"

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
