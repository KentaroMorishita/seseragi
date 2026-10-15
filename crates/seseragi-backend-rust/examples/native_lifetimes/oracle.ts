// Runtime-state oracle. Checked Seseragi source parity is verified separately.
import {
  acquireRelease,
  createEffectExecution,
  defer,
  type Effect,
  EffectCancellation,
  fail,
  flatMap,
  run,
  scoped,
  succeed,
} from "../../../../runtime/ts/src/effect"

type Exit = { kind: string; value?: string | string[] }
type Trace = { events: string[]; exit: Exit }
const traces: Record<string, Trace> = {}

function defect(error: unknown): Exit {
  const primary = error as Error & { suppressed?: Error[] }
  return {
    kind: "defect",
    value: [
      primary.message,
      ...(primary.suppressed ?? []).map((e) => e.message),
    ],
  }
}

for (const name of [
  "success",
  "failure",
  "defect",
  "cancel",
  "finalizer-defects",
  "explicit-close",
]) {
  const events: string[] = []
  const closed = new Set<string>()
  const close = (label: string) =>
    defer(() => {
      if (!closed.has(label)) {
        closed.add(label)
        events.push(`release:${label}`)
        if (name === "finalizer-defects") throw new Error(`${label}-defect`)
      }
      return succeed(undefined)
    })
  const acquire = (label: string) =>
    acquireRelease(
      defer(() => {
        events.push(`acquire:${label}`)
        return succeed(label)
      }),
      close
    )
  const execution = createEffectExecution()
  let started = (): void => undefined
  const pending = new Promise<void>((resolve) => {
    started = resolve
  })
  const use: Effect<unknown, string, string> =
    name === "failure" || name === "finalizer-defects"
      ? fail("expected")
      : name === "defect"
        ? () => {
            throw new Error("use-defect")
          }
        : name === "cancel"
          ? (_env, ctx) =>
              new Promise<string>(() => {
                started()
                ctx?.onCancel(() => undefined)
              })
          : succeed("done")
  const effect = scoped(
    name === "explicit-close"
      ? flatMap(acquire("explicit"), () =>
          flatMap(close("explicit"), () => use)
        )
      : flatMap(acquire("a"), () => flatMap(acquire("b"), () => use))
  )
  const result = run(effect, {}, execution.context).then(
    (result) =>
      result.kind === "success"
        ? { kind: "success", value: result.value }
        : { kind: "failure", value: result.error },
    (error) =>
      error instanceof EffectCancellation
        ? { kind: "cancelled" }
        : defect(error)
  )
  if (name === "cancel") {
    await pending
    await execution.cancel()
  }
  traces[name] = { events, exit: await result }
}

{
  const events: string[] = []
  const execution = createEffectExecution()
  let started = (): void => undefined
  const pending = new Promise<void>((resolve) => {
    started = resolve
  })
  const acquire: Effect<unknown, never, string> = (_env, ctx) =>
    new Promise((resolve) => {
      events.push("acquire:start")
      started()
      ctx?.onCancel(() => {
        events.push("acquire:late")
        resolve("late")
      })
    })
  const effect = scoped(
    acquireRelease(acquire, (name) =>
      defer(() => {
        events.push(`release:${name}`)
        return succeed(undefined)
      })
    )
  )
  const result = run(effect, {}, execution.context).then(
    () => ({ kind: "success", value: "unexpected" }),
    (error) =>
      error instanceof EffectCancellation
        ? { kind: "cancelled" }
        : defect(error)
  )
  await pending
  events.push("cancel")
  await execution.cancel()
  traces["late-acquire"] = { events, exit: await result }
}
console.log(JSON.stringify(traces))
