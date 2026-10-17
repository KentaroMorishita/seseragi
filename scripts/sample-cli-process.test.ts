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

// POSIX process groups remain killable after their leader exits.
test.skipIf(process.platform === "win32")(
  "sample deadline covers inherited pipes after the command exits",
  async () => {
    const started = performance.now()
    const exitingParent = [
      'const { spawn } = require("node:child_process");',
      'const holder = spawn(process.execPath, ["-e", "setTimeout(() => {}, 5_000)"], { stdio: ["ignore", "inherit", "inherit"] });',
      'console.error("parent pid=" + process.pid + " holder pid=" + holder.pid);',
      "holder.unref(); process.exit(0);",
    ].join(" ")
    let failure = ""
    try {
      await runSampleCommand(["node", "-e", exitingParent], {
        cwd: import.meta.dir,
        label: "fixture exited parent",
        timeoutMs: 1_000,
      })
    } catch (error) {
      failure = String(error)
    }
    expect(failure).toMatch(/fixture exited parent timed out.*limit 1000ms/su)
    expect(performance.now() - started).toBeLessThan(3_000)
    const holder = Number(failure.match(/holder pid=(\d+)/u)?.[1])
    expect(holder).toBeGreaterThan(0)
    expect(await processExited(holder)).toBe(true)
  },
  8_000
)

test("sample command preserves a nonzero exit and Unicode output", async () => {
  const result = await runSampleCommand(
    [
      "node",
      "-e",
      'console.log("日本語"); console.error("失敗"); process.exit(7)',
    ],
    { cwd: import.meta.dir, label: "fixture nonzero" }
  )
  expect(result).toEqual({ status: 7, stdout: "日本語\n", stderr: "失敗\n" })
})

test("sample command waits for inherited output after its parent exits", async () => {
  const later = "setTimeout(() => console.log('later'), 150)"
  const parent = [
    'const { spawn } = require("node:child_process");',
    `const writer = spawn(process.execPath, ["-e", ${JSON.stringify(later)}], { stdio: ["ignore", "inherit", "inherit"] });`,
    'console.log("first"); writer.unref(); process.exit(0);',
  ].join(" ")
  const result = await runSampleCommand(["node", "-e", parent], {
    cwd: import.meta.dir,
    label: "fixture inherited output",
  })
  expect(result).toEqual({ status: 0, stdout: "first\nlater\n", stderr: "" })
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
