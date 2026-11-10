// A standard Web build owns program instances for one browser application.
// The raw program ABI remains usable independently of this page lifetime.
// How often a retired instance is checked for its last callback and run.
const RETIRE_POLL_MS = 50

// An instance is idle once no run, Effect plan, callback or host request of
// its own remains. Exports themselves are disposed when it is retired.
function idle(ownership) {
  return (
    ownership.alive === false ||
    (!ownership.running &&
      !(ownership.plans > 0) &&
      ownership.callbacks === 0 &&
      !ownership.waitingHost)
  )
}

// The host calls the current instance through these stable functions, so a
// swapped instance takes over without the UI holding a new object.
function forwardingExports(owned) {
  const view = Object.create(null)
  for (const key of Object.keys(owned.current.exports)) {
    const initial = owned.current.exports[key]
    if (typeof initial !== "function") {
      Object.defineProperty(view, key, {
        enumerable: true,
        get: () => owned.current.exports[key],
      })
      continue
    }
    const forward = (...values) => owned.current.exports[key](...values)
    Object.defineProperties(forward, {
      dispose: {
        value() {
          owned.disposedKeys.add(key)
          owned.current.exports[key].dispose?.()
        },
      },
      disposed: { get: () => Boolean(owned.current.exports[key].disposed) },
    })
    view[key] = forward
  }
  return Object.freeze(view)
}

function sameKeys(left, right) {
  const leftKeys = Object.keys(left).sort()
  const rightKeys = Object.keys(right).sort()
  return (
    leftKeys.length === rightKeys.length &&
    leftKeys.every((key, index) => key === rightKeys[index])
  )
}

export function createProgramLifetime() {
  const owners = new Set()
  const pending = new Set()
  const failures = []
  // Owners that `seseragi dev` may swap to a rebuilt module.
  const swappable = new Set()
  // Loads of swappable programs still running their factory.
  const swapLoads = new Set()
  let closing = false
  let closed

  function track(task) {
    pending.add(task)
    const settle = () => pending.delete(task)
    task.then(settle, (error) => {
      failures.push(error)
      settle()
    })
    return task
  }

  // A retired instance keeps serving the callbacks, runs and finalizers it
  // already started. It is disposed once none remains, or with its owner.
  function retire(owned, previous) {
    owned.retiring.add(previous)
    for (const value of Object.values(previous.exports)) {
      try {
        value?.dispose?.()
      } catch (error) {
        failures.push(error)
      }
    }
    void track(
      (async () => {
        while (owned.retiring.has(previous) && !idle(previous.ownership()))
          await new Promise((resolve) => setTimeout(resolve, RETIRE_POLL_MS))
        if (!owned.retiring.delete(previous)) return
        await previous.dispose()
      })()
    )
  }

  // Replaces each named program's instances with ones built from its new
  // bytes. Nothing is swapped unless every new instance is ready.
  function swap(names, version, fetchBytes = fetchProgram) {
    if (closing) return Promise.reject(new Error("Program lifetime closed"))
    // Close waits for a swap in flight. Its failure belongs to the caller,
    // which reloads instead, not to the instances' own disposal.
    const task = (async () => {
      // A load that began before this update joins it once it completes.
      await Promise.allSettled([...swapLoads])
      const targets = [...swappable].filter((owned) =>
        names.includes(owned.swap.name)
      )
      const bytes = new Map()
      for (const owned of targets) {
        if (!bytes.has(owned.swap.name))
          bytes.set(owned.swap.name, await fetchBytes(owned.swap.path, version))
      }
      const prepared = []
      try {
        for (const owned of targets) {
          const next = await owned.swap.instantiate(bytes.get(owned.swap.name))
          prepared.push([owned, next])
          if (!sameKeys(owned.current.exports, next.exports))
            throw new Error("Program exports changed; reload required")
        }
        if (closing) throw new Error("Program lifetime closed")
      } catch (error) {
        for (const [, next] of prepared) {
          try {
            await next.dispose()
          } catch (cleanup) {
            failures.push(cleanup)
          }
        }
        throw error
      }
      for (const [owned, next] of prepared) {
        if (!swappable.has(owned)) {
          await next.dispose()
          continue
        }
        const previous = owned.current
        owned.current = next
        for (const key of owned.disposedKeys) next.exports[key].dispose?.()
        retire(owned, previous)
      }
    })()
    pending.add(task)
    const settle = () => pending.delete(task)
    task.then(settle, settle)
    return task
  }

  function load(create, closedError, swapInfo) {
    if (closing) return Promise.reject(closedError())
    // Reserve completion before invoking the factory. Keep the original
    // loader's immediate start, and observe a close during that invocation.
    let complete
    const completion = new Promise((resolve) => {
      complete = resolve
    })
    pending.add(completion)
    if (swapInfo) swapLoads.add(completion)
    return (async () => {
      try {
        const program = await create()
        if (closing) {
          try {
            await program.dispose()
          } catch (error) {
            failures.push(error)
            throw error
          }
          throw closedError()
        }
        let released = false
        let disposal
        const owned = {
          current: program,
          retiring: new Set(),
          disposedKeys: new Set(),
          swap: swapInfo,
        }
        const dispose = () => {
          if (released) return disposal
          released = true
          swappable.delete(owned)
          const retired = [...owned.retiring]
          owned.retiring.clear()
          // The owner's disposal also waits for instances it retired.
          const draining = []
          for (const previous of retired) {
            try {
              const result = previous.dispose()
              if (result !== undefined) draining.push(Promise.resolve(result))
            } catch (error) {
              failures.push(error)
            }
          }
          try {
            const result = owned.current.dispose()
            if (result !== undefined || draining.length !== 0) {
              disposal = Promise.all([result, ...draining]).then(() => {})
              pending.add(disposal)
              const settle = () => {
                pending.delete(disposal)
                owners.delete(dispose)
              }
              // Observe rejection even when pagehide cannot await. A controlled
              // close reports it after every other owner has finished cleanup.
              disposal.then(settle, (error) => {
                failures.push(error)
                settle()
              })
              return disposal
            }
            owners.delete(dispose)
          } catch (error) {
            owners.delete(dispose)
            throw error
          }
        }
        owners.add(dispose)
        if (swapInfo) swappable.add(owned)
        return Object.freeze({
          exports: swapInfo ? forwardingExports(owned) : program.exports,
          dispose,
          ownership: () => owned.current.ownership(),
        })
      } finally {
        pending.delete(completion)
        swapLoads.delete(completion)
        complete()
      }
    })()
  }

  function close() {
    if (closed) return closed
    closing = true
    let resolve
    let reject
    closed = new Promise((success, failure) => {
      resolve = success
      reject = failure
    })
    // Begin every close synchronously, including on pagehide. Controlled reload
    // also waits for asynchronous cancellation/finalizers and late factories.
    for (const dispose of [...owners]) {
      try {
        dispose()
      } catch (error) {
        failures.push(error)
      }
    }
    void Promise.allSettled([...pending])
      .then(() => {
        if (failures.length) {
          throw new AggregateError(failures, "program disposal failed")
        }
      })
      .then(resolve, reject)
    return closed
  }

  return Object.freeze({ load, close, swap })
}

async function fetchProgram(path, version) {
  const url = `${path}?v=${encodeURIComponent(version)}`
  const response = await fetch(url, { cache: "no-store" })
  if (!response.ok)
    throw new Error(`Failed to fetch ${url}: ${response.status}`)
  return new Uint8Array(await response.arrayBuffer())
}

// One generated browser application owns its Effect execution and initialized
// services. Its finalizers may still call pure WASM while cancellation drains.
export function createProgramApplication(programs) {
  let requested = false
  let started = false
  let active
  let closing
  const stopFailures = []
  const cleanupFailures = []

  function stop(entry) {
    if (entry.stopping || !entry.shutdown) return
    entry.stopping = true
    try {
      entry.stopped = Promise.resolve(entry.shutdown()).catch((error) => {
        stopFailures.push(error)
      })
    } catch (error) {
      stopFailures.push(error)
      entry.stopped = Promise.resolve()
    }
  }

  async function run(body) {
    if (started || requested) throw new Error("Browser application is closed")
    started = true
    let complete
    const entry = {
      done: new Promise((resolve) => {
        complete = resolve
      }),
      shutdown: undefined,
      stopping: false,
      stopped: undefined,
    }
    active = entry
    const cleanups = []
    let cleaning = false
    const frame = Object.freeze({
      get closing() {
        return requested
      },
      onShutdown(shutdown) {
        if (entry.shutdown) throw new Error("Application shutdown already set")
        entry.shutdown = shutdown
        if (requested) stop(entry)
      },
      cleanup(action) {
        if (cleaning) throw new Error("Application setup has finished")
        cleanups.push(action)
      },
    })
    let failed = false
    let failure
    try {
      await body(frame)
    } catch (error) {
      failed = true
      failure = error
    }
    cleaning = true
    const failures = []
    // Registration order is explicit: Effect close, DOM dispose, providers.
    for (const cleanup of cleanups) {
      try {
        await entry.stopped
        await cleanup()
      } catch (error) {
        failures.push(error)
      }
    }
    await entry.stopped
    cleanups.length = 0
    try {
      await programs.close()
    } catch (error) {
      failures.push(error)
    }
    cleanupFailures.push(...failures)
    active = undefined
    complete()
    if (failed && failures.length) {
      throw new AggregateError(
        [failure, ...failures],
        "Browser application and cleanup failed"
      )
    }
    if (failed) throw failure
    if (failures.length === 1) throw failures[0]
    if (failures.length) {
      throw new AggregateError(failures, "Browser application cleanup failed")
    }
  }

  function close() {
    if (closing) return closing
    requested = true
    let resolve
    let reject
    closing = new Promise((success, failure) => {
      resolve = success
      reject = failure
    })
    const entry = active
    if (entry) stop(entry)
    void (async () => {
      if (entry) {
        await entry.done
        await entry.stopped
      }
      try {
        await programs.close()
      } catch (error) {
        stopFailures.push(error)
      }
      const failures = [...new Set([...stopFailures, ...cleanupFailures])]
      if (failures.length) {
        throw new AggregateError(failures, "Browser application close failed")
      }
    })().then(resolve, reject)
    return closing
  }

  return Object.freeze({ run, close })
}

const lifetime = createProgramLifetime()
const application = createProgramApplication(lifetime)
export const loadOwnedProgram = lifetime.load
export const closePrograms = lifetime.close
export const swapPrograms = lifetime.swap
export const runProgramApplication = application.run
export const closeProgramApplication = application.close
let installed = false

export function installProgramReload(
  host = globalThis,
  reload = () => host.location.reload()
) {
  if (installed) return
  installed = true
  let reloading = false
  host.addEventListener("seseragi:before-reload", (event) => {
    event.preventDefault()
    if (reloading) return
    reloading = true
    void closeProgramApplication()
      .then(reload)
      .catch((error) => {
        // A failed close is observable; do not hide it by navigating away.
        console.error("seseragi: program cleanup failed before reload", error)
      })
  })
  // `seseragi dev` asks to swap rebuilt program modules in place. A listener
  // that takes the update hands back its completion; a failed swap reloads.
  host.addEventListener("seseragi:program-update", (event) => {
    if (reloading) return
    event.preventDefault()
    event.detail.respond(
      swapPrograms(event.detail.programs, event.detail.version)
    )
  })
  host.addEventListener(
    "pagehide",
    () => {
      void closeProgramApplication().catch((error) => {
        console.error("seseragi: program cleanup failed on pagehide", error)
      })
    },
    { once: true }
  )
}
