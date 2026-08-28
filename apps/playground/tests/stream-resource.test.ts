import { describe, expect, test } from "bun:test"
import {
  createEffectExecution,
  EffectCancellation,
  registerResourceFinalizer,
} from "../../../runtime/ts/src/effect"
import {
  buffer,
  bufferCapacity,
  merge,
  runCollect,
  runForEach,
  type Stream,
  type StreamCursor,
  zip,
} from "../../../runtime/ts/src/stream"

function customStream<Value>(
  open: Stream<unknown, never, Value>["open"]
): Stream<unknown, never, Value> {
  return Object.freeze({ open }) as Stream<unknown, never, Value>
}

function onceClose(action: () => void | Promise<void>): () => Promise<void> {
  let closing: Promise<void> | undefined
  return () => {
    closing ??= Promise.resolve().then(action)
    return closing
  }
}

function emptyCursor(close: () => void | Promise<void>): StreamCursor<never> {
  return Object.freeze({
    next: async (): Promise<IteratorResult<never>> => ({
      done: true,
      value: undefined,
    }),
    close: onceClose(close),
  })
}

describe("Stream resource ownership", () => {
  test("zip drains the left branch when the right branch cannot open", async () => {
    const events: string[] = []
    const left = customStream((_environment, context) => {
      registerResourceFinalizer(context, () => {
        events.push("left-finalizer")
      })
      return emptyCursor(() => {
        events.push("left-cursor")
      })
    })
    const openDefect = new Error("right open defect")
    const right = customStream<never>(() => {
      throw openDefect
    })
    const execution = createEffectExecution()

    await expect(zip(right, left).open({}, execution.context)).rejects.toBe(
      openDefect
    )
    expect(events).toEqual(["left-cursor", "left-finalizer"])
    await execution.close()
  })

  test("merge cancels an opened sibling when the next branch cannot open", async () => {
    const events: string[] = []
    const left = customStream((_environment, context) => {
      registerResourceFinalizer(context, () => {
        events.push("left-finalizer")
      })
      return emptyCursor(() => {
        events.push("left-cursor")
      })
    })
    const right = customStream<never>(() => {
      throw new Error("right open defect")
    })
    const execution = createEffectExecution()

    await expect(
      merge(right, left).open({}, execution.context)
    ).rejects.toThrow("right open defect")
    expect(events).toEqual(["left-cursor", "left-finalizer"])
    await execution.close()
  })

  test("terminal consumer defects still close the cursor and execution scope", async () => {
    const events: string[] = []
    let emitted = false
    const source = customStream<number>((_environment, context) => {
      registerResourceFinalizer(context, () => {
        events.push("scope-finalizer")
      })
      return Object.freeze({
        next: async (): Promise<IteratorResult<number>> => {
          if (emitted) return { done: true, value: undefined }
          emitted = true
          return { done: false, value: 1 }
        },
        close: onceClose(() => {
          events.push("cursor-close")
        }),
      })
    })
    const consumerDefect = new Error("consumer defect")

    await expect(
      runForEach(
        () => async () => {
          throw consumerDefect
        },
        source
      )({}, undefined)
    ).rejects.toBe(consumerDefect)
    expect(events).toEqual(["cursor-close", "scope-finalizer"])
  })

  test("cleanup defects outrank the terminal failure without skipping scope cleanup", async () => {
    const events: string[] = []
    const cursorDefect = new Error("cursor close defect")
    const scopeDefect = new Error("scope cleanup defect")
    const consumerDefect = new Error("consumer defect")
    let emitted = false
    const source = customStream<number>((_environment, context) => {
      registerResourceFinalizer(context, () => {
        events.push("scope-finalizer")
        throw scopeDefect
      })
      return Object.freeze({
        next: async (): Promise<IteratorResult<number>> => {
          if (emitted) return { done: true, value: undefined }
          emitted = true
          return { done: false, value: 1 }
        },
        close: onceClose(() => {
          events.push("cursor-close")
          throw cursorDefect
        }),
      })
    })

    await expect(
      runForEach(
        () => async () => {
          throw consumerDefect
        },
        source
      )({}, undefined)
    ).rejects.toBe(cursorDefect)
    expect(events).toEqual(["cursor-close", "scope-finalizer"])
    expect(
      (cursorDefect as Error & { readonly suppressed?: unknown[] }).suppressed
    ).toEqual([scopeDefect, consumerDefect])
  })

  test("terminal cleanup retains earlier suppressed defects in order", async () => {
    const firstScopeDefect = new Error("first scope defect")
    const secondScopeDefect = new Error("second scope defect")
    const consumerDefect = new Error("consumer defect")
    let emitted = false
    const source = customStream<number>((_environment, context) => {
      registerResourceFinalizer(context, () => {
        throw firstScopeDefect
      })
      registerResourceFinalizer(context, () => {
        throw secondScopeDefect
      })
      return Object.freeze({
        next: async (): Promise<IteratorResult<number>> => {
          if (emitted) return { done: true, value: undefined }
          emitted = true
          return { done: false, value: 1 }
        },
        close: onceClose(() => undefined),
      })
    })

    await expect(
      runForEach(
        () => async () => {
          throw consumerDefect
        },
        source
      )({}, undefined)
    ).rejects.toBe(secondScopeDefect)
    expect(
      (secondScopeDefect as Error & { readonly suppressed?: unknown[] })
        .suppressed
    ).toEqual([firstScopeDefect, consumerDefect])
  })

  test("buffer drains its child scope after upstream close defects", async () => {
    const events: string[] = []
    const cursorDefect = new Error("buffer cursor close defect")
    const source = customStream((_environment, context) => {
      registerResourceFinalizer(context, () => {
        events.push("scope-finalizer")
      })
      return emptyCursor(() => {
        events.push("cursor-close")
        throw cursorDefect
      })
    })
    const capacity = bufferCapacity(1)
    if (capacity.tag === "Left") throw new Error("expected valid capacity")

    await expect(
      runCollect(buffer(capacity.value, source))({}, undefined)
    ).rejects.toBe(cursorDefect)
    expect(events).toEqual(["cursor-close", "scope-finalizer"])
  })

  test("cancellation preserves scope LIFO and exactly-once cleanup", async () => {
    const events: string[] = []
    const source = customStream<never>((_environment, context) => {
      registerResourceFinalizer(context, () => {
        events.push("first")
      })
      registerResourceFinalizer(context, () => {
        events.push("second")
      })
      return Object.freeze({
        next: () => new Promise<IteratorResult<never>>(() => undefined),
        close: onceClose(() => {
          events.push("cursor-close")
        }),
      })
    })
    const execution = createEffectExecution()
    const running = Promise.resolve(
      runCollect(source)({}, execution.context)
    ).catch((error: unknown) => error)
    await Promise.resolve()

    await execution.cancel()
    expect(await running).toBeInstanceOf(EffectCancellation)
    expect(events.filter((event) => event === "second")).toHaveLength(1)
    expect(events.filter((event) => event === "first")).toHaveLength(1)
    expect(events.filter((event) => event === "cursor-close")).toHaveLength(1)
    expect(events.indexOf("second")).toBeLessThan(events.indexOf("first"))
  })
})
