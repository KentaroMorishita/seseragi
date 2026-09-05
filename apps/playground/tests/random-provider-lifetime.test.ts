import { expect, test } from "bun:test"
import { createRandomProvider } from "../../../runtime/providers/random"
import { ProviderPackageLoader } from "../../../runtime/ts/src/provider-package"

test("cached Random provider re-seeds after each loader lifetime", async () => {
  let seed = 1n
  let seedReads = 0
  const provider = createRandomProvider(() => {
    seedReads++
    return seed
  })
  const run = async () => {
    const loader = new ProviderPackageLoader("browser", [
      {
        provider: "seseragi/runtime#random",
        service: "std/random::Random",
        target: "browser",
        module: "cached/random",
        exportName: "provider",
        loadMode: "eager",
        importModule: async () => ({ provider }),
      },
    ])
    try {
      await loader.start()
      const loaded = await loader.load("seseragi/runtime#random")
      return await loaded.entry.shuffleIndices!(9)
    } finally {
      await loader.shutdown()
    }
  }
  expect(seedReads).toBe(0)
  await run()
  seed = 42n
  const second = await run()
  const third = await run()
  expect(second).toEqual({
    kind: "success",
    value: [3, 2, 4, 0, 1, 7, 5, 8, 6],
  })
  expect(third).toEqual(second)
  expect(seedReads).toBe(3)
})
