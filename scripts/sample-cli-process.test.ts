import { expect, test } from "bun:test"
import { existsSync } from "node:fs"
import { runSampleCommand, withTemporaryDirectory } from "./sample-cli-process"

async function processExited(pid: number): Promise<boolean> {
  for (let attempt = 0; attempt < 40; attempt += 1) {
    try {
      process.kill(pid, 0)
    } catch {
      return true
    }
    await Bun.sleep(50)
  }
  return false
}

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

test("timed-out sample commands terminate their process tree and clean their output directory", async () => {
  let outputDirectory = ""
  let timeoutError = ""
  const started = performance.now()
  const hangingProcessTree = [
    'const grandchild = Bun.spawn({ cmd: [process.execPath, "-e", "await Bun.sleep(60_000)"], stdout: "inherit", stderr: "inherit" });',
    'console.error("child pid=" + process.pid + " grandchild pid=" + grandchild.pid + " before hang");',
    "await Bun.sleep(60_000);",
  ].join(" ")
  try {
    await withTemporaryDirectory(
      "seseragi-sample-timeout-",
      async (directory) => {
        outputDirectory = directory
        await runSampleCommand([process.execPath, "-e", hangingProcessTree], {
          cwd: directory,
          label: "fixture timeout run",
          timeoutMs: 1_500,
        })
      }
    )
  } catch (error) {
    timeoutError = String(error)
  }
  expect(timeoutError).toMatch(
    /fixture timeout run timed out.*limit 1500ms.*before hang/su
  )
  const childPid = Number(timeoutError.match(/child pid=(\d+)/u)?.[1])
  const grandchildPid = Number(timeoutError.match(/grandchild pid=(\d+)/u)?.[1])
  expect(childPid).toBeGreaterThan(0)
  expect(grandchildPid).toBeGreaterThan(0)
  expect(await processExited(childPid)).toBe(true)
  expect(await processExited(grandchildPid)).toBe(true)
  expect(performance.now() - started).toBeLessThan(10_000)
  expect(existsSync(outputDirectory)).toBe(false)
}, 15_000)
