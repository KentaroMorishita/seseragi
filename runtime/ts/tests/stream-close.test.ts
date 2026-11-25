import { expect, test } from "bun:test"
import {
  createEffectExecution,
  type EffectContext,
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

test("an inner pull stays stopped while the outer pull is released", async () => {
  const log: string[] = []
  const inner = streams.zip(waitingSource(log), streams.fromArray([1, 2]))
  // The outer pull holds a resource that takes a while to release, so the
  // inner pull would resume during that release unless it is stopped too.
  const outer = streams.fromPull<unknown, never, number>(
    async (environment, context) => {
      const cursor = await inner.open(environment, context)
      return {
        pull: async (pullContext) => {
          registerResourceFinalizer(pullContext, async () => {
            await new Promise((resolve) => setTimeout(resolve, 5))
            log.push("outer-pull-released")
          })
          return await cursor.next()
        },
        close: () => cursor.close(),
      }
    }
  )
  const result = await run(
    streams.runCollect(streams.zip(outer, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
  // The inner zip closes its stopped side while the outer resource is being
  // released; the inner side never resumes and keeps the native order.
  expect(log).toContain("outer-pull-released")
  expect(log.filter((entry) => entry !== "outer-pull-released")).toEqual(
    nativeOrder
  )
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

const delay = (milliseconds: number) =>
  new Promise((resolve) => setTimeout(resolve, milliseconds))

test("a parent cancellation closes the source before its open resource", async () => {
  const log: string[] = []
  let opened = true
  const source = streams.fromPull<unknown, never, number>(
    (_environment, context) => {
      registerResourceFinalizer(context, () => {
        opened = false
        log.push("open-released")
      })
      return {
        pull: (pullContext) =>
          withResourceAcquisition(pullContext, async () => {
            await delay(5)
            return { done: false, value: 1 } as const
          }),
        close: () => {
          log.push(opened ? "closed" : "closed-after-release")
        },
      }
    }
  )
  const root = createEffectExecution()
  const running = run(
    streams.runCollect(streams.zip(source, streams.fromArray([1, 2]))),
    {},
    root.context
  ).catch(() => undefined)
  await delay(1)
  await root.cancel()
  await running
  expect(log).toEqual(["closed", "open-released"])
})

test("a pull acquiring until its source closes does not block zip", async () => {
  const source = streams.fromPull<unknown, never, number>(() => {
    let release = (): void => undefined
    return {
      pull: (context) =>
        withResourceAcquisition(
          context,
          () =>
            new Promise<IteratorResult<number>>((resolve) => {
              release = () => resolve({ done: true, value: undefined })
            })
        ),
      close: () => release(),
    }
  })
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
})

// A source that finishes at once. Its close waits for a cancellation, and
// for a while after it still uses the resource it acquired while opening.
function closedOnCancel(log: string[]) {
  return streams.fromPull<unknown, never, number>((_environment, context) => {
    let opened = true
    registerResourceFinalizer(context, () => {
      opened = false
      log.push("open-released")
    })
    let cancelled = (): void => undefined
    const waiting = new Promise<void>((resolve) => {
      cancelled = resolve
    })
    context.onCancel(() => cancelled())
    return {
      pull: async () => ({ done: true, value: undefined }),
      close: async () => {
        await waiting
        await delay(5)
        log.push(opened ? "closed" : "closed-after-release")
      },
    }
  })
}

test("a buffer closing its finished source still passes a cancellation on", async () => {
  const log: string[] = []
  const capacity = streams.bufferCapacity(1)
  if (capacity.tag !== "Right") throw new Error("capacity")
  const later = streams.fromPull<unknown, never, number>(() => ({
    pull: async () => {
      await delay(10)
      return { done: true, value: undefined }
    },
    close: () => undefined,
  }))
  const result = await run(
    streams.runCollect(
      streams.zip(streams.buffer(capacity.value, closedOnCancel(log)), later)
    ),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual(["closed", "open-released"])
})

test("a parent cancellation reaches a buffer closing its source", async () => {
  const log: string[] = []
  const capacity = streams.bufferCapacity(1)
  if (capacity.tag !== "Right") throw new Error("capacity")
  const root = createEffectExecution()
  const running = run(
    streams.runCollect(streams.buffer(capacity.value, closedOnCancel(log))),
    {},
    root.context
  ).catch(() => undefined)
  await delay(5)
  await root.cancel()
  await running
  expect(log).toEqual(["closed", "open-released"])
})

test("a pull whose cleanup waits for the close does not block zip", async () => {
  const source = streams.fromPull<unknown, never, number>(() => {
    let closed = (): void => undefined
    const closing = new Promise<void>((resolve) => {
      closed = resolve
    })
    return {
      pull: async (context) => {
        context.onCancel(() => closing)
        await yieldNow()(undefined, context)
        return { done: false, value: 1 }
      },
      close: () => closed(),
    }
  })
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
})

test("an inner buffer closes before the resource it was opened with", async () => {
  const log: string[] = []
  const capacity = streams.bufferCapacity(1)
  if (capacity.tag !== "Right") throw new Error("capacity")
  const parent = streams.fromEffect<unknown, never, number>(
    async (_environment, context) => {
      if (context === undefined) throw new Error("context")
      registerResourceFinalizer(context, () => {
        log.push("parent-released")
      })
      return 0
    }
  )
  const source = streams.flatMap(
    () => streams.buffer(capacity.value, waitingSource(log)),
    parent
  )
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual([...nativeOrder, "parent-released"])
})

test("a cleanup started by the interrupt runs before the pull is released", async () => {
  const log: string[] = []
  const source = streams.fromPull<unknown, never, number>(() => ({
    pull: async (context) => {
      registerResourceFinalizer(context, () => {
        log.push("pull-released")
      })
      context.signal.addEventListener("abort", () => {
        context.onCancel(async () => {
          await delay(5)
          log.push("interrupt-cleanup")
        })
      })
      await yieldNow()(undefined, context)
      return { done: false, value: 1 }
    },
    close: () => undefined,
  }))
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual(["interrupt-cleanup", "pull-released"])
})

test("a pull's resources wait for cleanups its cleanups add", async () => {
  const log: string[] = []
  const source = streams.fromPull<unknown, never, number>(() => ({
    pull: async (context) => {
      registerResourceFinalizer(context, () => {
        log.push("pull-released")
      })
      context.onCancel(() => {
        context.onCancel(async () => {
          await delay(5)
          log.push("added-cleanup")
        })
      })
      await yieldNow()(undefined, context)
      return { done: false, value: 1 }
    },
    close: () => undefined,
  }))
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual(["added-cleanup", "pull-released"])
})

test("a cleanup kept by one stream's open is not kept in another run", async () => {
  const log: string[] = []
  const shared = (): void => {
    log.push("shared-cleanup")
  }
  const keeping = streams.fromPull<unknown, never, number>(
    (_environment, context) => {
      context.onCancel(shared)
      return {
        pull: async () => ({ done: true, value: undefined }),
        close: () => undefined,
      }
    }
  )
  expect((await run(streams.runCollect(keeping), {})).kind).toBe("success")
  const source = streams.fromPull<unknown, never, number>(() => ({
    pull: async (context) => {
      registerResourceFinalizer(context, () => {
        log.push("pull-released")
      })
      context.onCancel(shared)
      await yieldNow()(undefined, context)
      return { done: false, value: 1 }
    },
    close: () => undefined,
  }))
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual(["shared-cleanup", "pull-released"])
})

// A pull that waits until it is stopped.
async function untilStopped(context: EffectContext): Promise<never> {
  while (true) await yieldNow()(undefined, context)
}

test("a parent cancellation reaches a close that waits for its notice", async () => {
  const log: string[] = []
  const source = streams.fromPull<unknown, never, number>(
    (_environment, context) => {
      let opened = true
      registerResourceFinalizer(context, () => {
        opened = false
        log.push("open-released")
      })
      let noticed = (): void => undefined
      const notice = new Promise<void>((resolve) => {
        noticed = resolve
      })
      context.onCancel(() => noticed())
      return {
        pull: async (pullContext) => {
          await untilStopped(pullContext)
          return { done: true, value: undefined }
        },
        close: async () => {
          await notice
          log.push(opened ? "closed" : "closed-after-release")
        },
      }
    }
  )
  const root = createEffectExecution()
  const running = run(
    streams.runCollect(streams.zip(source, streams.fromArray([1, 2]))),
    {},
    root.context
  ).catch(() => undefined)
  await delay(5)
  await root.cancel()
  await running
  expect(log).toEqual(["closed", "open-released"])
})

test("a pull finalizer that waits for the close does not block zip", async () => {
  const source = streams.fromPull<unknown, never, number>(() => {
    let closed = (): void => undefined
    const closing = new Promise<void>((resolve) => {
      closed = resolve
    })
    return {
      pull: async (context) => {
        registerResourceFinalizer(context, () => closing)
        await untilStopped(context)
        return { done: true, value: undefined }
      },
      close: () => closed(),
    }
  })
  const result = await run(
    streams.runCollect(streams.zip(source, streams.empty)),
    {}
  )
  expect(result.kind).toBe("success")
})

test("an inner stream keeps what it borrows from the outer pull", async () => {
  const log: string[] = []
  let borrowed = false
  const outer = streams.fromEffect<unknown, never, number>(
    async (_environment, context) => {
      if (context === undefined) throw new Error("context")
      borrowed = true
      registerResourceFinalizer(context, () => {
        borrowed = false
        log.push("outer-released")
      })
      return 0
    }
  )
  const inner = streams.fromPull<unknown, never, number>(() => ({
    pull: async (context) => {
      await untilStopped(context)
      return { done: true, value: undefined }
    },
    close: () => {
      log.push(borrowed ? "closed" : "closed-after-release")
    },
  }))
  const result = await run(
    streams.runCollect(
      streams.zip(
        streams.flatMap(() => inner, outer),
        streams.empty
      )
    ),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual(["closed", "outer-released"])
})

test("an inner stream keeps what it borrows past a finished scope", async () => {
  const log: string[] = []
  let borrowed = false
  const outer = streams.fromEffect<unknown, never, number>(
    async (_environment, context) => {
      if (context === undefined) throw new Error("context")
      borrowed = true
      registerResourceFinalizer(context, () => {
        borrowed = false
        log.push("outer-released")
      })
      // A finished scope leaves an unregistered entry after the resource.
      registerResourceFinalizer(context, () => undefined).unregister()
      return 0
    }
  )
  const inner = streams.fromPull<unknown, never, number>(() => ({
    pull: async (context) => {
      await untilStopped(context)
      return { done: true, value: undefined }
    },
    close: () => {
      log.push(borrowed ? "closed" : "closed-after-release")
    },
  }))
  const result = await run(
    streams.runCollect(
      streams.zip(
        streams.flatMap(() => inner, outer),
        streams.empty
      )
    ),
    {}
  )
  expect(result.kind).toBe("success")
  expect(log).toEqual(["closed", "outer-released"])
})

test("a buffer that finished normally does not cancel its closing source", async () => {
  const capacity = streams.bufferCapacity(1)
  if (capacity.tag !== "Right") throw new Error("capacity")
  // The consumer is still handling the value when the source finishes, so
  // it sees the end while the source is closing.
  const source = streams.fromPull<unknown, never, number>(
    (_environment, context) => {
      let sent = false
      return {
        pull: async () => {
          if (sent) return { done: true, value: undefined }
          sent = true
          return { done: false, value: 1 }
        },
        close: async () => {
          await delay(0)
          if (context.cancelled) throw new Error("closed as cancelled")
        },
      }
    }
  )
  const result = await run(
    streams.runForEach(
      () => async () => {
        await delay(2)
      },
      streams.buffer(capacity.value, source)
    ),
    {}
  )
  expect(result.kind).toBe("success")
})

test("a defect is never suppressed by itself", async () => {
  const capacity = streams.bufferCapacity(1)
  if (capacity.tag !== "Right") throw new Error("capacity")
  const source = streams.fromPull<unknown, never, number>(() => ({
    pull: async () => {
      throw new Error("pull failed")
    },
    close: async () => {
      throw new Error("close failed")
    },
  }))
  const error = await run(
    streams.runCollect(streams.buffer(capacity.value, source)),
    {}
  ).then(
    () => undefined,
    (cause: unknown) => cause
  )
  expect(error).toBeInstanceOf(Error)
  const suppressed = (error as { suppressed?: unknown[] }).suppressed ?? []
  expect(suppressed).not.toContain(error)
  expect(new Set(suppressed).size).toBe(suppressed.length)
})
