import { expect, test } from "bun:test"
import { existsSync } from "node:fs"
import { runSampleCommand, withTemporaryDirectory } from "./sample-cli-process"

test("sample command reports its output and exit status", async () => {
  const result = await runSampleCommand(
    [
      process.execPath,
      "-e",
      'const input = await Bun.stdin.text(); console.log(input.trim().toUpperCase()); console.error("diagnostic")',
    ],
    {
      cwd: import.meta.dir,
      label: "fixture success",
      stdin: "hello\n",
    }
  )
  expect(result.status).toBe(0)
  expect(result.stdout).toBe("HELLO\n")
  expect(result.stderr).toBe("diagnostic\n")
})

test("timed-out sample commands terminate and clean their output directory", async () => {
  let outputDirectory = ""
  let timeoutError = ""
  const started = performance.now()
  try {
    await withTemporaryDirectory(
      "seseragi-sample-timeout-",
      async (directory) => {
        outputDirectory = directory
        await runSampleCommand(
          [
            process.execPath,
            "-e",
            'console.error("child pid=" + process.pid + " before hang"); await Bun.sleep(60_000)',
          ],
          {
            cwd: directory,
            label: "fixture timeout run",
            timeoutMs: 1_500,
          }
        )
      }
    )
  } catch (error) {
    timeoutError = String(error)
  }
  expect(timeoutError).toMatch(
    /fixture timeout run timed out.*limit 1500ms.*before hang/su
  )
  const childPid = Number(timeoutError.match(/child pid=(\d+)/u)?.[1])
  expect(childPid).toBeGreaterThan(0)
  expect(() => process.kill(childPid, 0)).toThrow()
  expect(performance.now() - started).toBeLessThan(10_000)
  expect(existsSync(outputDirectory)).toBe(false)
}, 15_000)
