import { expect, test } from "bun:test"
import { resetProcessHashSeedForTest } from "../../../runtime/ts/src/hash"
import type { EntryContract } from "../src/compiler/types"
import {
  executeGeneratedModule,
  executeGeneratedProject,
} from "../src/runtime/browser-execution"

const entry: EntryContract = {
  environment: [],
  failureRenderer: { kind: "never" },
}

test("hash startup fails before single-file and project application evaluation", async () => {
  const crypto = Object.getOwnPropertyDescriptor(globalThis, "crypto")
  const host = globalThis as typeof globalThis & {
    __SESERAGI_HASH_SEED__?: number
  }
  const fixed = host.__SESERAGI_HASH_SEED__
  const environment = process.env.SESERAGI_HASH_SEED
  try {
    Object.defineProperty(globalThis, "crypto", {
      value: undefined,
      configurable: true,
    })
    delete host.__SESERAGI_HASH_SEED__
    delete process.env.SESERAGI_HASH_SEED
    resetProcessHashSeedForTest()
    const source = 'throw new Error("application evaluated before startup")'
    await expect(executeGeneratedModule(source, entry)).rejects.toThrow(
      "secure entropy is unavailable"
    )
    await expect(
      executeGeneratedProject(
        [{ path: "main.ssrg", typescript: source }],
        "main.ssrg",
        entry
      )
    ).rejects.toThrow("secure entropy is unavailable")
    host.__SESERAGI_HASH_SEED__ = -13
    const seeded = `
      import { empty } from "@seseragi/runtime/map"
      import { processHashSeed } from "@seseragi/runtime/hash"
      const values = empty()
      throw new Error("application seed:" + processHashSeed())
    `
    await expect(executeGeneratedModule(seeded, entry)).rejects.toThrow(
      "application seed:-13"
    )
  } finally {
    if (crypto !== undefined)
      Object.defineProperty(globalThis, "crypto", crypto)
    else Reflect.deleteProperty(globalThis, "crypto")
    if (fixed === undefined) delete host.__SESERAGI_HASH_SEED__
    else host.__SESERAGI_HASH_SEED__ = fixed
    if (environment === undefined) delete process.env.SESERAGI_HASH_SEED
    else process.env.SESERAGI_HASH_SEED = environment
    resetProcessHashSeedForTest()
  }
})
