import { expect, test } from "bun:test"
import { stderr } from "node:process"
import { liveConsole } from "../src/console"
import { unit } from "../src/effect"

async function withStderrWrite(
  write: typeof stderr.write,
  check: () => Promise<void>
) {
  const original = stderr.write
  const listeners = stderr.listeners("error")
  stderr.write = write
  try {
    await check()
    await new Promise<void>((resolve) => setImmediate(resolve))
    expect(stderr.listeners("error")).toEqual(listeners)
  } finally {
    stderr.write = original
  }
}

for (const emitError of [true, false]) {
  test(`stderr callback failures settle concurrent writes and release listeners (event=${emitError})`, async () => {
    const error = new Error("write failed")
    const listeners = stderr.listenerCount("error")
    let calls = 0
    await withStderrWrite(
      (_text, callback) => {
        if (typeof callback !== "function") {
          throw new Error("expected a write callback")
        }
        calls++
        process.nextTick(() => {
          callback(error)
          if (emitError) stderr.emit("error", error)
        })
        return false
      },
      async () => {
        const writes = Array.from({ length: 32 }, () =>
          liveConsole.errorLine("x")
        )
        expect(stderr.listenerCount("error")).toBe(listeners + 1)
        const results = await Promise.all(writes)
        expect(calls).toBe(32)
        for (const result of results) {
          expect(result).toEqual({
            kind: "failure",
            error: { kind: "console-error", message: "write failed" },
          })
        }
      }
    )
  })
}

test("stderr success preserves bytes and existing error listeners", async () => {
  const writes: unknown[] = []
  const observed: Error[] = []
  const onError = (error: Error) => observed.push(error)
  stderr.on("error", onError)
  try {
    await withStderrWrite(
      (text, callback) => {
        if (typeof callback !== "function") {
          throw new Error("expected a write callback")
        }
        writes.push(text)
        process.nextTick(() => callback(null))
        return true
      },
      async () => {
        for (const write of [
          liveConsole.error(""),
          liveConsole.error("雪🙂\0"),
          liveConsole.errorLine(""),
        ]) {
          expect(await write).toEqual({ kind: "success", value: unit })
        }
        expect(writes).toEqual(["", "雪🙂\0", "\n"])
      }
    )
    const unrelated = new Error("unrelated stream error")
    stderr.emit("error", unrelated)
    expect(observed).toEqual([unrelated])
  } finally {
    stderr.off("error", onError)
  }
})

test("stderr synchronous Error remains ConsoleError and releases its listener", async () => {
  await withStderrWrite(
    () => {
      throw new Error("synchronous failure")
    },
    async () => {
      expect(await liveConsole.error("x")).toEqual({
        kind: "failure",
        error: { kind: "console-error", message: "synchronous failure" },
      })
    }
  )
})

test("stderr non-Error throws remain defects and release their listener", async () => {
  const defect = Symbol("write defect")
  await withStderrWrite(
    () => {
      throw defect
    },
    async () => {
      await expect(liveConsole.error("x")).rejects.toBe(defect)
    }
  )
})
