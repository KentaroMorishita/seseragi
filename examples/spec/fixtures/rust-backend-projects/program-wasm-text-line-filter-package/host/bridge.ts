import {
  loadProgram,
  type OwnedCallback,
  type ProgramInstance,
  programWasmPath,
} from "@seseragi/programs/processor"

export async function normalize(input: string): Promise<string> {
  const response = await fetch(new URL(programWasmPath, document.baseURI))
  if (!response.ok) throw new Error(`program WASM HTTP ${response.status}`)
  let program: ProgramInstance | undefined
  let callback: OwnedCallback<string, string> | undefined
  try {
    program = await loadProgram(await response.arrayBuffer())
    callback = program.exports.factory("")
    return callback(input)
  } finally {
    callback?.dispose()
    program?.dispose()
  }
}
