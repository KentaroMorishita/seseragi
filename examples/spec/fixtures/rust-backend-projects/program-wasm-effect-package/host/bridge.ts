// The foreign task owns WASM loading and execution. Rendering stays in Seseragi.
import {
  loadProgram,
  type OwnedCallback,
  type OwnedEffect,
  type ProgramExecution,
  type ProgramInstance,
  type ProgramOutcome,
  ProgramWasmClosed,
  ProgramWasmDefect,
  programWasmPath,
} from "@seseragi/programs/calculation"

let program: ProgramInstance | undefined
let prepared: OwnedEffect<string, string> | undefined
let callback: OwnedCallback<string, string> | undefined
let generation = 0
let transition: Promise<void> = Promise.resolve()
let currentRun:
  | {
      owner: ProgramInstance
      result: Promise<ProgramOutcome<unknown, unknown>>
    }
  | undefined
let closed: { owner: ProgramInstance; report: string } | undefined
let active:
  | {
      owner: ProgramInstance
      plan: OwnedEffect<never, undefined>
      execution: ProgramExecution<never, undefined>
    }
  | undefined

function changeOwner(action: () => Promise<string>): Promise<string> {
  // Repeated reload/close requests cannot abandon a newly loaded instance.
  const result = transition.then(action, action)
  transition = result.then(
    () => undefined,
    () => undefined
  )
  return result
}

function outcome<E, A>(
  value: ProgramOutcome<E, A>,
  success: (value: A) => string,
  failure: (error: E) => string
): string {
  switch (value.tag) {
    case "Success":
      return `Success: ${success(value.value)}`
    case "Failure":
      return `Failure: ${failure(value.error)}`
    case "Defect":
      return `Defect: ${value.messages.join(" / ")}`
    case "Cancelled":
      return "Cancelled"
    default: {
      const exhaustive: never = value
      return exhaustive
    }
  }
}

function snapshot(owner = program): string {
  return JSON.stringify(owner?.ownership() ?? { loaded: false })
}

function current(): ProgramInstance {
  if (!program) throw new Error("Load the program first")
  return program
}

function detach(plan: OwnedEffect<unknown, unknown>, owner: ProgramInstance) {
  // A closing owner already owns its plans until cancellation/cleanup ends.
  if (!plan.disposed && owner.ownership().alive && !owner.ownership().closing) {
    plan.dispose()
  }
}

function begin<E, A>(
  owner: ProgramInstance,
  plan: OwnedEffect<E, A>
): ProgramExecution<E, A> {
  const execution = plan.run()
  currentRun = { owner, result: execution.result }
  return execution
}

function cleanupDefect(
  error: unknown,
  terminal: ProgramOutcome<unknown, unknown> | undefined
): boolean {
  if (
    terminal?.tag !== "Defect" ||
    !(error instanceof AggregateError) ||
    error.message !== "Program WASM Effect ended with a Defect during disposal"
  ) {
    return false
  }
  const errors: unknown = error.errors
  const messages = terminal.messages
  return (
    Array.isArray(errors) &&
    errors.length === messages.length &&
    errors.every(
      (entry: unknown, index: number) =>
        entry instanceof Error &&
        entry.name === "Error" &&
        entry.message === messages[index]
    )
  )
}

async function evaluate(
  owner: ProgramInstance,
  plan: OwnedEffect<string, string>,
  retain = false
): Promise<string> {
  let execution: ProgramExecution<string, string> | undefined
  try {
    execution = begin(owner, plan)
    return outcome(
      await execution.result,
      (value) => value,
      (error) => error
    )
  } finally {
    if (execution && currentRun?.result === execution.result)
      currentRun = undefined
    if (!retain) detach(plan, owner)
  }
}

async function closeCurrent(): Promise<string> {
  if (!program) return "No instance to dispose"
  const owner = program
  if (closed?.owner === owner) return closed.report
  const running = currentRun?.owner === owner ? currentRun : undefined
  let disposalRejected = false
  let disposalFailure: unknown
  try {
    await owner.dispose()
  } catch (error) {
    disposalRejected = true
    disposalFailure = error
  }
  let terminal: ProgramOutcome<unknown, unknown> | undefined
  let executionRejected = false
  let executionFailure: unknown
  if (running) {
    try {
      terminal = await running.result
    } catch (error) {
      executionRejected = true
      executionFailure = error
    }
  }
  const ownership = owner.ownership()
  if (
    ownership.alive ||
    ownership.retired ||
    ownership.callbacks !== 0 ||
    ownership.plans !== 0 ||
    ownership.running ||
    ownership.managedLiveAtClose !== 0
  ) {
    if (disposalRejected) throw disposalFailure
    if (executionRejected) throw executionFailure
    throw new Error("Normal disposal did not release all managed program roots")
  }
  if (active?.owner === owner) active = undefined
  if (currentRun?.owner === owner) currentRun = undefined
  prepared = undefined
  callback = undefined
  if (executionRejected) throw executionFailure
  // Only the loader's documented disposal error, correlated with this exact
  // execution's Defect messages, is rendered as an observed cleanup failure.
  // A trap, protocol error or unrelated AggregateError remains a task failure.
  if (disposalRejected && !cleanupDefect(disposalFailure, terminal)) {
    throw disposalFailure
  }
  const completed = terminal
    ? outcome(
        terminal,
        (value) => (value === undefined ? "Unit" : "completed"),
        (error) => JSON.stringify(error) ?? "undefined"
      )
    : "No active root"
  const report = `${disposalRejected ? "Cleanup Defect reported; roots released" : "Disposed"} after ${completed}\n${snapshot(owner)}`
  closed = { owner, report }
  return report
}

async function start(owner: ProgramInstance, input: string, cleanup: boolean) {
  if (active) throw new Error("Cancel the active root before starting another")
  const plan = cleanup
    ? owner.exports.cancelCleanup(input)
    : owner.exports.pending(input)
  let execution: ProgramExecution<never, undefined>
  try {
    execution = begin(owner, plan)
  } catch (error) {
    detach(plan, owner)
    throw error
  }
  active = { owner, plan, execution }
  // One actual task turn lets the first bounded WASM poll acquire/register its
  // resource before the UI offers cancellation or owner disposal.
  await new Promise<void>((resolve) => setTimeout(resolve, 0))
  return cleanup ? "Running; release will report cancel released" : "Running"
}

export async function operate(action: string, input: string): Promise<string> {
  if (action === "reload") {
    return changeOwner(async () => {
      const previous = await closeCurrent()
      const response = await fetch(new URL(programWasmPath, document.baseURI))
      if (!response.ok) throw new Error(`program WASM HTTP ${response.status}`)
      program = await loadProgram(await response.arrayBuffer())
      closed = undefined
      generation++
      return `Loaded generation ${generation}\n${previous}\n${snapshot()}`
    })
  }
  if (action === "dispose") return changeOwner(closeCurrent)
  if (action === "finalize") {
    return changeOwner(async () => {
      let result = "UI finalizer"
      try {
        // Preserve the application finalizer's ability to use a retained pure
        // callback while its program is still alive, then close the owner.
        if (callback && !callback.disposed) result = callback(" / UI finalizer")
      } finally {
        result += `\n${await closeCurrent()}`
      }
      return result
    })
  }
  const owner = current()
  let result: string
  try {
    switch (action) {
      case "prepare":
        if (prepared) detach(prepared, owner)
        prepared = owner.exports.value(input)
        result = "Cold plan prepared; run has not started"
        break
      case "run":
        prepared ??= owner.exports.value(input)
        result = await evaluate(owner, prepared, true)
        break
      case "failure":
        result = await evaluate(owner, owner.exports.rejection(input))
        break
      case "defect":
        result = await evaluate(owner, owner.exports.defect(input))
        break
      case "cleanup":
        result = await evaluate(owner, owner.exports.cleanup(input))
        break
      case "callback": {
        if (callback && !callback.disposed) callback.dispose()
        const plan = owner.exports.callback(input)
        let execution:
          | ProgramExecution<never, OwnedCallback<string, string>>
          | undefined
        try {
          execution = begin(owner, plan)
          result = outcome(
            await execution.result,
            (value) => {
              callback = value
              return value(" / callback")
            },
            (error) => error
          )
        } finally {
          if (execution && currentRun?.result === execution.result)
            currentRun = undefined
          detach(plan, owner)
        }
        break
      }
      case "call-callback":
        if (!callback) throw new Error("Create the callback first")
        result = callback(input)
        break
      case "detach":
        if (!callback) throw new Error("Create the callback first")
        callback.dispose()
        result = "Callback detached"
        break
      case "start":
      case "start-cleanup":
        result = await start(owner, input, action === "start-cleanup")
        break
      case "cancel": {
        const running = active
        if (!running) throw new Error("Start an Effect before cancelling it")
        running.execution.cancel()
        try {
          result = outcome(
            await running.execution.result,
            () => "Unit",
            (error) => error
          )
        } finally {
          if (currentRun?.result === running.execution.result)
            currentRun = undefined
          detach(running.plan, running.owner)
          if (active === running) active = undefined
        }
        break
      }
      default:
        throw new Error(`Unknown action ${action}`)
    }
  } catch (error) {
    if (
      !(
        error instanceof ProgramWasmClosed || error instanceof ProgramWasmDefect
      )
    ) {
      throw error
    }
    result = `${error.name}: ${error.message}`
  }
  return `${result}\n${snapshot(owner)}`
}
