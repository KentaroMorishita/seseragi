import { expect, test } from "bun:test"

test("WASM service retries failed initialization without repeating operations", () => {
  // Isolate the generated-module transport mock from other real WASM tests.
  const result = Bun.spawnSync([
    process.execPath,
    new URL("./fixtures/wasm-initialization.ts", import.meta.url).pathname,
  ])
  expect(result.stderr.toString()).toBe("")
  expect(result.exitCode).toBe(0)
  expect(result.stdout.toString()).toContain(
    "WASM initialization regression passed"
  )
})
