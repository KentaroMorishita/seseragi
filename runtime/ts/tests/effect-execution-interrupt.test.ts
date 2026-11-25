import { expect, test } from "bun:test"
import {
  createEffectExecution,
  interruptExecution,
  markExecution,
  openingStream,
  registerResourceFinalizer,
  releaseExecutionAfter,
} from "../src/effect"

test("an interrupt reaches child executions but releases nothing", async () => {
  const log: string[] = []
  const parent = createEffectExecution()
  const child = createEffectExecution(parent.context)
  const grandchild = createEffectExecution(child.context)
  grandchild.context.onCancel(() => {
    log.push("cleanup")
  })
  registerResourceFinalizer(grandchild.context, () => {
    log.push("released")
  })
  interruptExecution(parent)
  expect(child.context.cancelled).toBe(true)
  expect(grandchild.context.cancelled).toBe(true)
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(log).toEqual([])
  await parent.cancel()
  expect(log).toEqual(["cleanup", "released"])
})

test("a child made after an interrupt starts interrupted", () => {
  const parent = createEffectExecution()
  interruptExecution(parent)
  expect(createEffectExecution(parent.context).context.cancelled).toBe(true)
})

test("a partial release keeps what came before the mark", async () => {
  const log: string[] = []
  const execution = createEffectExecution()
  registerResourceFinalizer(execution.context, () => {
    log.push("before")
  })
  execution.context.onCancel(() => {
    log.push("cleanup-before")
  })
  const mark = markExecution(execution)
  registerResourceFinalizer(execution.context, () => {
    log.push("after")
  })
  execution.context.onCancel(() => {
    log.push("cleanup-after")
  })
  interruptExecution(execution)
  await releaseExecutionAfter(execution, mark)
  expect(log).toEqual(["cleanup-after", "after"])
  await execution.cancel()
  expect(log).toEqual(["cleanup-after", "after", "cleanup-before", "before"])
})

test("closing an interrupted execution still runs its cleanups", async () => {
  const log: string[] = []
  const execution = createEffectExecution()
  execution.context.onCancel(() => {
    log.push("cleanup")
  })
  interruptExecution(execution)
  await execution.close()
  expect(log).toEqual(["cleanup"])
})

test("a cleanup that registers itself again runs once", async () => {
  let runs = 0
  const execution = createEffectExecution()
  const cleanup = (): void => {
    runs += 1
    execution.context.onCancel(cleanup)
  }
  execution.context.onCancel(cleanup)
  await execution.cancel()
  expect(runs).toBe(1)
})

test("a partial release starts a cleanup that another one waits for", async () => {
  const execution = createEffectExecution()
  const mark = markExecution(execution)
  execution.context.onCancel(
    () =>
      new Promise<void>((resolve) => {
        execution.context.onCancel(resolve)
      })
  )
  interruptExecution(execution)
  await releaseExecutionAfter(execution, mark)
  await execution.cancel()
})

test("a cancellation waits for a partial release to finish", async () => {
  const log: string[] = []
  const execution = createEffectExecution()
  registerResourceFinalizer(execution.context, () => {
    log.push("old")
  })
  const mark = markExecution(execution)
  registerResourceFinalizer(execution.context, async () => {
    await new Promise((resolve) => setTimeout(resolve, 5))
    log.push("new")
  })
  interruptExecution(execution)
  const partial = releaseExecutionAfter(execution, mark)
  await execution.cancel()
  await partial
  expect(log).toEqual(["new", "old"])
})

test("registering a kept cleanup again keeps it for the close", async () => {
  const log: string[] = []
  const execution = createEffectExecution()
  const cleanup = (): void => {
    log.push("cleanup")
  }
  const mark = markExecution(execution)
  await openingStream(execution.context, () => {
    execution.context.onCancel(cleanup)
  })
  execution.context.onCancel(cleanup)
  interruptExecution(execution)
  await releaseExecutionAfter(execution, mark)
  expect(log).toEqual([])
  await execution.cancel()
  expect(log).toEqual(["cleanup"])
})
