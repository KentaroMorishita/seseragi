import {
  loadProgram,
  type OwnedCallback,
  type ProgramInstance,
  programWasmPath,
} from "@seseragi/programs/predicate"

export async function compare(input: string): Promise<string> {
  const response = await fetch(new URL(programWasmPath, document.baseURI))
  if (!response.ok) throw new Error(`program WASM HTTP ${response.status}`)
  let program: ProgramInstance | undefined
  let callback: OwnedCallback<string, boolean> | undefined
  try {
    program = await loadProgram(await response.arrayBuffer())
    callback = program.exports.saved("ready\n")
    return callback(input) ? "accepted\n" : "rejected\n"
  } finally {
    callback?.dispose()
    program?.dispose()
  }
}
