import { expect, test } from "bun:test"
import { runCommand } from "./cli-test-support"

test("drains both pipes when CLI fixture commands exit immediately", async () => {
  for (let index = 0; index < 20; index += 1) {
    await runCommand([
      process.execPath,
      "-e",
      'process.stdout.write("o".repeat(256 * 1024)); process.stderr.write("e".repeat(256 * 1024))',
    ])
  }
}, 15_000)

test("reports the failed CLI fixture command and both output streams", async () => {
  await expect(
    runCommand([
      process.execPath,
      "-e",
      'console.log("fixture output"); console.error("fixture error"); process.exit(7)',
    ])
  ).rejects.toThrow("failed (7)\nfixture output\nfixture error")
})
