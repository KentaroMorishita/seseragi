import { expect, test } from "bun:test"
import {
  registerResourceFinalizer,
  run,
  withResourceAcquisition,
  yieldNow,
} from "../src/effect"
import * as streams from "../src/stream"

// A source with one value whose close takes a while and may reject.
function slowlyClosed(log: string[], failure?: Error) {
  return streams.fromPull<unknown, never, number>(() => {
    let sent = false
    return {
      pull: async () => {
        if (sent) return { done: true, value: undefined }
        sent = true
        return { done: false, value: 1 }
      },
      close: async () => {
        await new Promise((resolve) => setTimeout(resolve, 5))
        log.push("closed")
        if (failure !== undefined) throw failure
      },
    }
  })
}

function nestedZip(source: streams.Stream<unknown, never, number>) {
  return streams.runCollect(
    streams.zip(streams.empty, streams.zip(streams.empty, source))
  )
}

test("an outer zip waits for the inner zip to close its source", async () => {
  const log: string[] = []
  const result = await run(nestedZip(slowlyClosed(log)), {})
  log.push("finished")
  expect(result.kind).toBe("success")
  expect(log).toEqual(["closed", "finished"])
})

test("an outer zip reports the inner close failure", async () => {
  const log: string[] = []
  await expect(
    run(nestedZip(slowlyClosed(log, new Error("close failed"))), {})
  ).rejects.toThrow("close failed")
  expect(log).toEqual(["closed"])
})

// A source whose pull is still waiting when the other side stops. Opening it
// registers a resource that its close still uses; the pull registers one of
// its own before it waits.
function waitingSource(
  log: string[],
  options: Readonly<{ closeFailure?: Error }> = {}
) {
  return streams.fromPull<unknown, never, number>((_environment, context) => {
    let opened = true
    registerResourceFinalizer(context, () => {
      opened = false
      log.push("open-released")
    })
    let closed = false
    const close = async (): Promise<void> => {
      if (closed) return
      closed = true
      log.push(opened ? "close-start" : "close-after-release")
      await new Promise((resolve) => setTimeout(resolve, 5))
      log.push("close-end")
      if (options.closeFailure !== undefined) throw options.closeFailure
    }
    // Like a host source, a cancellation also closes the source.
    const releaseCancellation = context.onCancel(close)
    return {
      pull: async (pullContext) => {
        registerResourceFinalizer(pullContext, () => {
          log.push("pull-released")
        })
        await yieldNow()(undefined, pullContext)
        log.push("resumed")
        return { done: false, value: 1 }
      },
      close: async () => {
        releaseCancellation()
        await close()
      },
    }
  })
}

// The native order: stop the pending pull, release what it acquired, close
// the cursor, then release what opening the source acquired.
const nativeOrder = [
  "pull-released",
  "close-start",
  "close-end",
  "open-released",
]

const failing = streams.fromPull<unknown, never, number>(() => ({
  pull: async () => {
    throw new Error("left failed")
  },
  close: () => undefined,
}))

test("zip cancels a pending pull before closing that source", async () => {
  const log: string[] = []
  const result = await run(
    streams.runCollect(streams.zip(waitingSource(log), streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual(nativeOrder)
})

test("merge cancels a pending pull when the other side fails", async () => {
  const log: string[] = []
  await expect(
    run(streams.runCollect(streams.merge(waitingSource(log), failing)), {})
  ).rejects.toThrow("left failed")
  expect(log).toEqual(nativeOrder)
})

test("an inner zip stops its pending pull when the outer zip stops", async () => {
  const log: string[] = []
  const inner = streams.zip(waitingSource(log), streams.fromArray([1, 2]))
  const result = await run(
    streams.runCollect(streams.zip(inner, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual(nativeOrder)
})

test("zip keeps the close failure of a source it cancels", async () => {
  const log: string[] = []
  const source = waitingSource(log, {
    closeFailure: new Error("close failed"),
  })
  await expect(
    run(streams.runCollect(streams.zip(source, streams.empty)), {})
  ).rejects.toThrow("close failed")
  expect(log).toEqual(nativeOrder)
})

test("a buffer cancelled by its parent closes its source once", async () => {
  const log: string[] = []
  const capacity = streams.bufferCapacity(1)
  if (capacity.tag !== "Right") throw new Error("capacity")
  const source = streams.buffer(
    capacity.value,
    waitingSource(log, { closeFailure: new Error("close failed") })
  )
  await expect(
    run(streams.runCollect(streams.zip(source, streams.empty)), {})
  ).rejects.toThrow("close failed")
  expect(log).toEqual(nativeOrder)
})

test("a stream opened during a pull keeps what its close uses", async () => {
  for (const wrap of [
    (source: streams.Stream<unknown, never, number>) =>
      streams.flatMap(() => source, streams.singleton(0)),
    (source: streams.Stream<unknown, never, number>) =>
      streams.concat(source, streams.empty),
  ]) {
    const log: string[] = []
    const result = await run(
      streams.runCollect(streams.zip(wrap(waitingSource(log)), streams.empty)),
      {}
    )
    expect(result.kind).toBe("success")
    expect(log).toEqual(nativeOrder)
  }
})

test("an inner stream closes before the resource it was opened with", async () => {
  const log: string[] = []
  const parent = streams.fromEffect<unknown, never, number>(
    async (_environment, context) => {
      if (context === undefined) throw new Error("context")
      registerResourceFinalizer(context, () => {
        log.push("parent-released")
      })
      return 0
    }
  )
  const source = streams.flatMap(() => waitingSource(log), parent)
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual([...nativeOrder, "parent-released"])
})

test("a cancelled pull may register a cleanup while acquiring", async () => {
  const source = streams.fromPull<unknown, never, number>(() => ({
    pull: (context) =>
      withResourceAcquisition(context, async () => {
        await new Promise((resolve) => setTimeout(resolve, 5))
        await new Promise<void>((resolve) => {
          context.onCancel(resolve)
        })
        return { done: false, value: 1 } as const
      }),
    close: () => undefined,
  }))
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
})

test("an inner stream still opening is cancelled with the pull", async () => {
  const inner = streams.fromPull<unknown, never, number>(
    async (_environment, context) => {
      await withResourceAcquisition(
        context,
        () =>
          new Promise<void>((resolve) => {
            context.onCancel(resolve)
          })
      )
      return {
        pull: async () => ({ done: true, value: undefined }),
        close: () => undefined,
      }
    }
  )
  const source = streams.flatMap(() => inner, streams.singleton(0))
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
})
