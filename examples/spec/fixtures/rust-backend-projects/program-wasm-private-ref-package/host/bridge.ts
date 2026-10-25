import {
  loadProgram,
  type OwnedCallback,
  type OwnedEffect,
  type ProgramInstance,
  type ProgramOutcome,
  ProgramWasmClosed,
  programWasmPath,
} from "@seseragi/programs/calculation"

type Callback = OwnedCallback<string, OwnedEffect<never, string>>
let owner: ProgramInstance | undefined
let factory: OwnedEffect<never, Callback> | undefined
let callback: Callback | undefined
let plan: OwnedEffect<never, string> | undefined
let transition: Promise<void> = Promise.resolve()

function current(): ProgramInstance {
  if (!owner) throw new Error("Load the program first")
  return owner
}

function snapshot(): string {
  return JSON.stringify(owner?.ownership() ?? { loaded: false })
}

function outcome<E, A>(value: ProgramOutcome<E, A>): string {
  switch (value.tag) {
    case "Success":
      return `Success: ${String(value.value)}`
    case "Failure":
      return `Failure: ${String(value.error)}`
    case "Defect":
      return `Defect: ${value.messages.join(" / ")}`
    case "Cancelled":
      return "Cancelled"
  }
}

async function closeCurrent(): Promise<string> {
  const instance = owner
  if (!instance) return "No loaded instance"
  await instance.dispose()
  const closed = instance.ownership()
  if (
    closed.alive ||
    closed.retired ||
    closed.callbacks !== 0 ||
    closed.plans !== 0 ||
    closed.running ||
    closed.managedLiveAtClose !== 0
  ) {
    throw new Error("The completed owner still retains managed roots")
  }
  factory = undefined
  callback = undefined
  plan = undefined
  return "Disposed; managed roots released"
}

async function apply(action: string, input: string): Promise<string> {
  let result: string
  if (action === "reload") {
    await closeCurrent()
    const response = await fetch(new URL(programWasmPath, document.baseURI))
    if (!response.ok) throw new Error(`Program WASM HTTP ${response.status}`)
    owner = await loadProgram(await response.arrayBuffer())
    result = "Loaded"
  } else if (action === "dispose") {
    result = await closeCurrent()
  } else if (action === "finalize") {
    result = "UI finalizer"
    try {
      if (callback && !callback.disposed) {
        const finishing = callback(input)
        try {
          const completed = await finishing.run().result
          result += ` / ${outcome(completed)}`
          if (completed.tag !== "Success") {
            throw new Error("Old callback did not finish before owner close")
          }
        } finally {
          if (!finishing.disposed) finishing.dispose()
        }
      }
    } finally {
      await closeCurrent()
    }
  } else {
    const instance = current()
    try {
      switch (action) {
        case "prepare":
          factory?.dispose()
          factory = instance.exports.create(`${input}:`)
          result = "Cold factory prepared"
          break
        case "factory": {
          factory ??= instance.exports.create(`${input}:`)
          const completed = await factory.run().result
          if (completed.tag !== "Success") throw new Error(outcome(completed))
          callback?.dispose()
          callback = completed.value
          result = "Callback received; no inner Effect has run"
          break
        }
        case "plan":
          if (!callback) throw new Error("Run the factory first")
          plan?.dispose()
          plan = callback(input)
          result = "Cold callback plan prepared"
          break
        case "run":
          if (!plan) throw new Error("Create a callback plan first")
          result = outcome(await plan.run().result)
          break
        case "detach":
          if (!callback) throw new Error("Run the factory first")
          callback.dispose()
          result = "Callback detached; the retained plan stays usable"
          break
        default:
          throw new Error(`Unknown action ${action}`)
      }
    } catch (error) {
      if (!(error instanceof ProgramWasmClosed)) throw error
      result = `${error.name}: ${error.message}`
    }
  }
  return `${result}\n${snapshot()}`
}

export function operate(action: string, input: string): Promise<string> {
  // User operations and the application finalizer share one owner transition.
  const result = transition.then(() => apply(action, input))
  transition = result.then(
    () => undefined,
    () => undefined
  )
  return result
}
