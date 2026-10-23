import {
  type HostEffect,
  type HostEffectContext,
  loadProgram,
  type OwnedEffect,
  type ProgramExecution,
  type ProgramInstance,
  type ProgramOutcome,
  programWasmPath,
} from "@seseragi/programs/calculation"

type Phase = "open" | "read" | "release"
type Mode = "normal" | "failure" | "defect" | "cleanup"

// These are host-owned resource records, not OS file descriptors. The actual
// asynchronous read uses fetch and its AbortSignal. Release drops the record
// after its asynchronous acknowledgement; the WASM finalizer must await it.
class HostSession {
  readonly resources = new Map<string, string>()
  readonly trace: string[] = []
  opened = 0
  released = 0
  mode: Mode = "normal"
  hold: Phase | undefined
  pending: { phase: Phase; resume: () => void } | undefined

  async pause(phase: Phase, context: HostEffectContext): Promise<void> {
    if (this.hold !== phase) return
    this.hold = undefined
    let resume = () => {}
    const waiting = new Promise<void>((resolve) => {
      resume = resolve
    })
    const pending = { phase, resume }
    this.pending = pending
    // The loader supplies a masked signal for acquisition/finalization. Only
    // the unmasked read can acknowledge cancellation through this listener.
    const abort = () => {
      this.trace.push(`abort:${phase}`)
      resume()
    }
    context.signal.addEventListener("abort", abort, { once: true })
    if (context.signal.aborted) abort()
    try {
      await waiting
    } finally {
      context.signal.removeEventListener("abort", abort)
      if (this.pending === pending) this.pending = undefined
    }
  }

  readonly request: HostEffect<string, string, string> = async (
    action,
    context
  ) => {
    this.trace.push(action)
    if (action.startsWith("open:")) {
      await this.pause("open", context)
      const token = `resource-${++this.opened}`
      this.resources.set(token, action.slice(5))
      return { tag: "Success", value: token }
    }
    if (!action.startsWith("read:")) throw new Error("Unknown host request")
    const resource = action.slice(5)
    if (!this.resources.has(resource))
      throw new Error("Host resource is closed")
    await this.pause("read", context)
    if (context.signal.aborted) return { tag: "Success", value: "" }
    if (this.mode === "failure") return { tag: "Failure", error: "read failed" }
    if (this.mode === "defect") throw new Error("read defect")
    try {
      const response = await fetch(new URL("./input.txt", document.baseURI), {
        signal: context.signal,
      })
      if (!response.ok)
        return { tag: "Failure", error: `input HTTP ${response.status}` }
      return { tag: "Success", value: await response.text() }
    } catch (error) {
      // A cooperatively cancelled I/O acknowledges completion. The runtime
      // publishes Cancelled after the registered release, without exposing
      // this placeholder as a successful language continuation.
      if (context.signal.aborted) return { tag: "Success", value: "" }
      throw error
    }
  }

  readonly release: HostEffect<string, never, undefined> = async (
    resource,
    context
  ) => {
    this.trace.push(`release:start:${resource}`)
    await this.pause("release", context)
    await new Promise<void>((resolve) => setTimeout(resolve, 0))
    if (!this.resources.delete(resource))
      throw new Error("Host resource released twice")
    this.released++
    this.trace.push(`release:end:${resource}`)
    if (this.mode === "cleanup") throw new Error("release defect")
    return { tag: "Success", value: undefined }
  }
}

interface Current {
  owner: ProgramInstance
  host: HostSession
  plan?: OwnedEffect<string, string>
  execution?: ProgramExecution<string, string>
  terminal?: ProgramOutcome<string, string>
  closing?: Promise<void>
  closeFailure?: unknown
}

let current: Current | undefined
let transition: Promise<void> = Promise.resolve()

function outcome(value: ProgramOutcome<string, string>): string {
  switch (value.tag) {
    case "Success":
      return `Success: ${value.value}`
    case "Failure":
      return `Failure: ${value.error}`
    case "Defect":
      return `Defect: ${value.messages.join(" / ")}`
    case "Cancelled":
      return "Cancelled"
  }
}

function report(state: Current, message: string): string {
  return `${message}\n${JSON.stringify({
    open: state.host.resources.size,
    opened: state.host.opened,
    released: state.host.released,
    pending: state.host.pending?.phase ?? null,
    trace: state.host.trace,
    ownership: state.owner.ownership(),
  })}`
}

function loaded(): Current {
  if (!current) throw new Error("Load the program first")
  return current
}

function dropPlan(state: Current): void {
  if (state.execution) throw new Error("Wait for the running Effect first")
  if (state.plan && !state.plan.disposed) state.plan.dispose()
  state.plan = undefined
}

function prepare(state: Current, input: string, nested = false): void {
  dropPlan(state)
  // Dispose temporary source stages. The resulting cold plan retains its
  // own callback leases until detach or owner disposal; no host runs here.
  const withRequest = (
    nested ? state.owner.exports.nested : state.owner.exports.prepared
  )(state.host.request)
  try {
    const execute = withRequest(state.host.release)
    try {
      state.plan = execute(input)
    } finally {
      execute.dispose()
    }
  } finally {
    withRequest.dispose()
  }
  state.terminal = undefined
}

function start(state: Current): void {
  if (!state.plan) throw new Error("Prepare a plan first")
  if (state.execution) throw new Error("An Effect is already running")
  state.execution = state.plan.run()
}

async function finish(state: Current): Promise<string> {
  const execution = state.execution
  if (execution) {
    state.terminal = await execution.result
    if (state.execution === execution) state.execution = undefined
  }
  if (state.closing) {
    await state.closing
    const ownership = state.owner.ownership()
    if (
      ownership.alive ||
      ownership.retired ||
      ownership.callbacks !== 0 ||
      ownership.plans !== 0 ||
      ownership.hostCallbacks !== 0 ||
      ownership.running ||
      ownership.waitingHost ||
      ownership.managedLiveAtClose !== 0 ||
      state.host.resources.size !== 0
    )
      throw new Error("Normal close retained a program or host resource")
    // Cleanup failure is still reported after all roots and host resources
    // have been released. It does not become a successful Effect outcome.
    if (state.closeFailure) throw state.closeFailure
  }
  return report(
    state,
    `${state.closing ? "Disposed; " : ""}${state.terminal ? outcome(state.terminal) : "Idle"}`
  )
}

function close(state: Current): void {
  if (state.closing) return
  // Observe a rejection immediately, then rethrow it from finish(). A caller
  // may need to resume a masked host release before awaiting normal close.
  state.closing = state.owner.dispose().catch((error: unknown) => {
    state.closeFailure = error
  })
}

async function untilPending(state: Current): Promise<void> {
  const deadline = performance.now() + 5000
  while (!state.host.pending) {
    if (performance.now() > deadline)
      throw new Error("Host request did not wait")
    if (!state.owner.ownership().running) {
      await finish(state)
      throw new Error("Effect ended before the requested host wait")
    }
    await new Promise<void>((resolve) => setTimeout(resolve, 0))
  }
}

function changeOwner(action: () => Promise<string>): Promise<string> {
  const result = transition.then(action, action)
  transition = result.then(
    () => undefined,
    () => undefined
  )
  return result
}

export async function operate(action: string, input: string): Promise<string> {
  if (action === "reload" || action === "finalize") {
    return changeOwner(async () => {
      if (current) {
        // UI teardown makes this fixture's manually held host cooperative.
        // Interactive dispose deliberately keeps the hold until Resume.
        current.host.hold = undefined
        current.host.pending?.resume()
        close(current)
        await finish(current)
        if (action === "finalize") return report(current, "UI finalized")
      }
      if (action === "finalize") return "No program loaded"
      const response = await fetch(new URL(programWasmPath, document.baseURI))
      if (!response.ok) throw new Error(`program WASM HTTP ${response.status}`)
      const owner = await loadProgram(await response.arrayBuffer())
      current = { owner, host: new HostSession() }
      return report(current, "Loaded")
    })
  }
  const state = loaded()
  if (action === "resume") {
    state.host.pending?.resume()
    return report(state, "Host wait released")
  }
  if (action === "cancel") {
    state.execution?.cancel()
    return report(
      state,
      "Cancellation requested; completion is awaited by Finish"
    )
  }
  if (action === "finish") return finish(state)
  if (action === "dispose") {
    close(state)
    return report(state, "Close requested; completion is awaited by Finish")
  }
  if (action === "detach") {
    dropPlan(state)
    return report(state, "Plan detached")
  }
  if (state.execution || state.closing)
    throw new Error("Wait for the active execution or owner close first")
  if (action === "prepare") {
    state.host.mode = "normal"
    prepare(state, input)
    return report(state, "Cold plan prepared; no host operation ran")
  }
  if (action === "run") {
    if (!state.plan) prepare(state, input)
    start(state)
    return finish(state)
  }
  if (action === "open" || action === "read" || action === "release") {
    state.host.mode = "normal"
    state.host.hold = action
    prepare(state, input)
    start(state)
    await untilPending(state)
    return report(state, `Waiting for host ${action}`)
  }
  if (
    action === "failure" ||
    action === "defect" ||
    action === "cleanup" ||
    action === "nested"
  ) {
    state.host.mode = action === "nested" ? "normal" : action
    prepare(state, input, action === "nested")
    start(state)
    return finish(state)
  }
  throw new Error(`Unknown action ${action}`)
}
