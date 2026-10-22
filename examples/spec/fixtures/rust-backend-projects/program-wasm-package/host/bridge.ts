// The task import owns loading and lifetime changes. UI rendering stays pure.
import {
  loadProgram,
  type OwnedCallback,
  type ProgramInstance,
  ProgramWasmClosed,
  ProgramWasmDefect,
  programWasmPath,
} from "@seseragi/programs/calculation"

let program: ProgramInstance | undefined
let callback: OwnedCallback<number, string> | undefined

export async function operate(action: string, input: number): Promise<string> {
  if (action === "reload") {
    program?.dispose()
    program = undefined
    callback = undefined
    const response = await fetch(new URL(programWasmPath, document.baseURI))
    if (!response.ok) throw new Error(`program WASM HTTP ${response.status}`)
    program = await loadProgram(await response.arrayBuffer())
    const prefix = program.exports.label("合計: ")
    try {
      callback = prefix(40)
    } finally {
      prefix.dispose()
    }
  }
  if (!program || !callback) throw new Error("Program has not been loaded")
  let result = ""
  try {
    switch (action) {
      case "reload":
      case "call":
        result = callback(input)
        break
      case "validate":
      case "reject":
        result = JSON.stringify(
          program.exports.validate(action === "reject" ? -1 : input)
        )
        break
      case "detach":
        callback.dispose()
        result = "detached"
        break
      case "dispose":
        program.dispose()
        result = "disposed"
        break
      case "finalize":
        try {
          result = callback(input)
        } finally {
          program.dispose()
        }
        break
      case "overflow":
        program.exports.staged(1)
        throw new Error("The first source stage did not raise its Defect")
      default:
        throw new Error(`Unknown action ${action}`)
    }
  } catch (error) {
    if (
      !(
        error instanceof ProgramWasmClosed || error instanceof ProgramWasmDefect
      )
    ) {
      program.dispose()
      throw error
    }
    result = error.name
  }
  return `${result}\n${JSON.stringify(program.ownership())}`
}
