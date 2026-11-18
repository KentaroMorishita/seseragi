import { expect, test } from "bun:test"
import { run } from "../src/effect"
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
