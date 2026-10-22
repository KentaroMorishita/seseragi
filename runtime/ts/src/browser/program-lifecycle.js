// A standard Web build owns program instances for one browser application.
// The raw program ABI remains usable independently of this page lifetime.
export function createProgramLifetime() {
  const owners = new Set()
  const pending = new Set()
  const failures = []
  let closing = false
  let closed

  function load(create, closedError) {
    if (closing) return Promise.reject(closedError())
    // Reserve completion before invoking the factory. Keep the original
    // loader's immediate start, and observe a close during that invocation.
    let complete
    const completion = new Promise((resolve) => {
      complete = resolve
    })
    pending.add(completion)
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
        const dispose = () => {
          if (released) return disposal
          released = true
          try {
            const result = program.dispose()
            if (result !== undefined) {
              disposal = Promise.resolve(result)
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
        return Object.freeze({
          exports: program.exports,
          dispose,
          ownership: () => program.ownership(),
        })
      } finally {
        pending.delete(completion)
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

  return Object.freeze({ load, close })
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
