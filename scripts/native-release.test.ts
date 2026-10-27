import { expect, test } from "bun:test"
import { chmod, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import {
  hostNativeReleaseTarget,
  nativeArchiveName,
  nativeChecksumName,
  nativeReleaseTargets,
  packageNativeRelease,
  verifyNativeRelease,
} from "./native-release"
import { repositoryRoot } from "./release-contract"
import {
  NativeReleaseCleanupError,
  NativeReleaseCommandError,
  NativeReleaseTemporaryDirectoryError,
  removeNativeReleaseDirectory,
  runNativeReleaseCommand,
  withNativeReleaseDirectory,
} from "./native-release-process"

function errno(code: string): Error & { code: string } {
  return Object.assign(new Error(`${code}: fixture cleanup lock`), { code })
}

function expectExited(pid: number | undefined): void {
  expect(pid).toBeDefined()
  expect(() => process.kill(pid!, 0)).toThrow()
}

test("closes the exited command and its stdio before removing its temporary files", async () => {
  let directory = ""
  const output = "日本語🙂".repeat(8192)
  const result = await withNativeReleaseDirectory(
    "verification",
    async (temporary, execute) => {
      directory = temporary
      const script = `const fs = require("node:fs"); fs.openSync("held-until-exit", "w"); process.stdout.write("日本語🙂".repeat(8192)); process.stderr.write("diagnostic");`
      const command = await execute([process.execPath, "-e", script], temporary)
      expect(command.stdout).toBe(output)
      expect(command.stderr).toBe("diagnostic")
      expect(command.exitCode).toBe(0)
      expect(command.signal).toBeNull()
      expect(command.exitObserved).toBe(true)
      expect(command.closeObserved).toBe(true)
      expectExited(command.pid)
      return "smoke passed"
    }
  )
  expect(result).toBe("smoke passed")
  await expect(stat(directory)).rejects.toThrow()
})

test("preserves a nonzero process result after exit and stdio close", async () => {
  await withNativeReleaseDirectory("verification", async (directory) => {
    try {
      await runNativeReleaseCommand(
        [
          process.execPath,
          "-e",
          'process.stdout.write("partial"); process.stderr.write("original smoke error"); process.exit(23)',
        ],
        directory
      )
      throw new Error("nonzero smoke unexpectedly passed")
    } catch (error) {
      expect(error).toBeInstanceOf(NativeReleaseCommandError)
      const result = (error as NativeReleaseCommandError).result
      expect(result.stdout).toBe("partial")
      expect(result.stderr).toBe("original smoke error")
      expect(result.exitCode).toBe(23)
      expect(result.exitObserved).toBe(true)
      expect(result.closeObserved).toBe(true)
      expectExited(result.pid)
    }
  })
})

test("preserves spawn failure and observes close without inventing an exit", async () => {
  await withNativeReleaseDirectory("verification", async (directory) => {
    try {
      await runNativeReleaseCommand(
        [path.join(directory, "missing-executable")],
        directory
      )
      throw new Error("missing command unexpectedly passed")
    } catch (error) {
      expect(error).toBeInstanceOf(NativeReleaseCommandError)
      const result = (error as NativeReleaseCommandError).result
      expect(result.pid).toBeUndefined()
      expect(result.exitObserved).toBe(false)
      expect(result.closeObserved).toBe(true)
      expect((error as Error).cause).toHaveProperty("code", "ENOENT")
    }
  })
})

test("retains the original validation error when cleanup succeeds", async () => {
  const original = new Error("archive metadata does not match")
  let directory = ""
  await expect(
    withNativeReleaseDirectory("verification", async (temporary) => {
      directory = temporary
      throw original
    })
  ).rejects.toBe(original)
  await expect(stat(directory)).rejects.toThrow()
})

test("retries only transient cleanup failures with finite waits", async () => {
  const waits: number[] = []
  let attempts = 0
  await removeNativeReleaseDirectory("owned-fixture", {
    remove: async () => {
      if (++attempts < 3) throw errno("EBUSY")
    },
    wait: async (milliseconds) => {
      waits.push(milliseconds)
    },
  })
  expect(attempts).toBe(3)
  expect(waits).toEqual([100, 200])
})

test("stops cleanup after six attempts and preserves its final filesystem error", async () => {
  const waits: number[] = []
  const failure = errno("EPERM")
  let attempts = 0
  try {
    await removeNativeReleaseDirectory("owned-fixture", {
      remove: async () => {
        attempts += 1
        throw failure
      },
      wait: async (milliseconds) => {
        waits.push(milliseconds)
      },
    })
    throw new Error("locked cleanup unexpectedly passed")
  } catch (error) {
    expect(error).toBeInstanceOf(NativeReleaseCleanupError)
    expect(error).toHaveProperty("attempts", 6)
    expect(error).toHaveProperty("waitedMs", 1500)
    expect((error as Error).cause).toBe(failure)
  }
  expect(attempts).toBe(6)
  expect(waits).toEqual([100, 200, 300, 400, 500])
})

test("does not retry a permanent cleanup error", async () => {
  let attempts = 0
  let waits = 0
  const failure = errno("EACCES")
  await expect(
    removeNativeReleaseDirectory("owned-fixture", {
      remove: async () => {
        attempts += 1
        throw failure
      },
      wait: async () => {
        waits += 1
      },
    })
  ).rejects.toHaveProperty("cause", failure)
  expect(attempts).toBe(1)
  expect(waits).toBe(0)
})

test("a passing smoke still fails if cleanup fails", async () => {
  let directory = ""
  let executions = 0
  try {
    await withNativeReleaseDirectory(
      "verification",
      async (temporary, execute) => {
        directory = temporary
        executions += 1
        await execute([
          process.execPath,
          "-e",
          'process.stdout.write("smoke passed")',
        ])
        return "smoke passed"
      },
      {
        remove: async () => {
          throw errno("EBUSY")
        },
        wait: async () => {},
      }
    )
    throw new Error("cleanup failure unexpectedly passed")
  } catch (error) {
    expect(error).toBeInstanceOf(NativeReleaseTemporaryDirectoryError)
    expect(error).toHaveProperty("operationSucceeded", true)
    expect((error as AggregateError).errors).toHaveLength(1)
    expect((error as Error).message).toContain("verification succeeded")
    expect((error as Error).message).toContain("cleanup also failed")
    expect((error as Error).message).toContain("EBUSY")
    const commands = (error as NativeReleaseTemporaryDirectoryError)
      .commandResults
    expect(commands).toHaveLength(1)
    expect(commands[0]?.exitCode).toBe(0)
    expect(commands[0]?.exitObserved).toBe(true)
    expect(commands[0]?.closeObserved).toBe(true)
    expectExited(commands[0]?.pid)
    expect((error as Error).message).toContain(
      "exitObserved=true closeObserved=true"
    )
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
  expect(executions).toBe(1)
})

test("a validation failure and cleanup failure retain both original error objects", async () => {
  const original = new Error("checksum or release metadata differs")
  let directory = ""
  try {
    await withNativeReleaseDirectory(
      "verification",
      async (temporary) => {
        directory = temporary
        throw original
      },
      {
        remove: async () => {
          throw errno("EBUSY")
        },
        wait: async () => {},
      }
    )
    throw new Error("combined failure unexpectedly passed")
  } catch (error) {
    expect(error).toBeInstanceOf(NativeReleaseTemporaryDirectoryError)
    expect(error).toHaveProperty("operationSucceeded", false)
    expect((error as Error).cause).toBe(original)
    const errors = (error as AggregateError).errors
    expect(errors[0]).toBe(original)
    expect(errors[1]).toBeInstanceOf(NativeReleaseCleanupError)
    expect((error as Error).message).toContain(original.message)
    expect((error as Error).message).toContain("EBUSY")
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
})

test("a nonzero smoke and cleanup failure retain PID, exit code, stderr and cleanup cause", async () => {
  let directory = ""
  try {
    await withNativeReleaseDirectory(
      "verification",
      async (temporary, execute) => {
        directory = temporary
        await execute(
          [
            process.execPath,
            "-e",
            'process.stderr.write("binary smoke refused"); process.exit(17)',
          ],
          temporary
        )
      },
      {
        remove: async () => {
          throw errno("EBUSY")
        },
        wait: async () => {},
      }
    )
    throw new Error("combined process failure unexpectedly passed")
  } catch (error) {
    expect(error).toBeInstanceOf(NativeReleaseTemporaryDirectoryError)
    const errors = (error as AggregateError).errors
    expect(errors[0]).toBeInstanceOf(NativeReleaseCommandError)
    const command = (errors[0] as NativeReleaseCommandError).result
    expect(command.exitCode).toBe(17)
    expect(command.stderr).toBe("binary smoke refused")
    expect(command.closeObserved).toBe(true)
    expectExited(command.pid)
    expect(
      (error as NativeReleaseTemporaryDirectoryError).commandResults
    ).toEqual([command])
    expect(errors[1]).toBeInstanceOf(NativeReleaseCleanupError)
    expect((error as Error).message).toContain("binary smoke refused")
    expect((error as Error).message).toContain("EBUSY")
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
})

test("uses one fixed native archive contract per release target", () => {
  expect(hostNativeReleaseTarget("win32", "x64")).toBe("win32-x64")
  expect(() => hostNativeReleaseTarget("linux", "arm64")).toThrow(
    "unsupported host"
  )
  expect(nativeArchiveName("0.4.0", "darwin-arm64")).toBe(
    "seseragi-v0.4.0-darwin-arm64.tar.gz"
  )
  expect(nativeArchiveName("0.4.0", "linux-x64")).toBe(
    "seseragi-v0.4.0-linux-x64.tar.gz"
  )
  expect(nativeArchiveName("0.4.0", "win32-x64")).toBe(
    "seseragi-v0.4.0-win32-x64.zip"
  )
  expect(nativeChecksumName("0.4.0", "win32-x64")).toBe(
    "seseragi-v0.4.0-win32-x64.zip.sha256"
  )
  expect(nativeReleaseTargets["darwin-x64"].rustTarget).toBe(
    "x86_64-apple-darwin"
  )
})

test("packages and re-verifies every native target before publish", async () => {
  const workflow = await readFile(
    path.join(repositoryRoot, ".github/workflows/release.yml"),
    "utf8"
  )
  for (const target of Object.keys(nativeReleaseTargets).filter(
    (target) => target !== "linux-x64"
  )) {
    expect(
      workflow.match(new RegExp(`target: ${target}`, "gu"))?.length ?? 0
    ).toBeGreaterThanOrEqual(2)
  }
  expect(workflow).toContain("uses: ./.github/workflows/linux-native.yml")
  expect(workflow).toContain("local-web-product-e2e, linux-native]")
  expect(workflow).toContain("native-release.ts smoke")
  expect(workflow).toContain("native-release.ts verify")
  expect(workflow).toContain("needs: [gate, native-verify, vscode")
  expect(workflow).toContain("matrix.archive }}.sha256")
  expect(workflow).not.toContain("seseragi-lsp-v*")
})

test("packages, checksums, extracts and executes the host CLI and LSP", async () => {
  if (process.platform === "win32") return
  const temporary = await mkdtemp(path.join(tmpdir(), "seseragi-native-test-"))
  const target = hostNativeReleaseTarget()
  const rustTarget = nativeReleaseTargets[target].rustTarget
  const cli = path.join(temporary, "source-cli")
  const lsp = path.join(temporary, "source-lsp")
  const output = path.join(temporary, "output")

  try {
    await writeFile(
      cli,
      `#!/bin/sh\nif [ "$1" = "--version-json" ]; then\n  printf '%s\\n' '{"name":"seseragi","version":"0.4.0","target":"${rustTarget}","channel":"development","dirty":true,"releaseTag":null}'\nelse\n  printf '%s\\n' 'seseragi 0.4.0 (development, commit fixture, target ${rustTarget})'\nfi\n`
    )
    await writeFile(
      lsp,
      `#!/bin/sh\nprintf '%s\\n' '{"name":"seseragi-lsp","version":"0.4.0","target":"${rustTarget}","channel":"development","dirty":true,"releaseTag":null}'\n`
    )
    await chmod(cli, 0o755)
    await chmod(lsp, 0o755)

    const artifact = await packageNativeRelease({
      version: "0.4.0",
      target,
      cli,
      lsp,
      output,
    })
    await verifyNativeRelease({
      version: "0.4.0",
      target,
      archive: artifact.archive,
      output,
    })

    const checksum = await readFile(artifact.checksum, "utf8")
    expect(checksum).toMatch(
      new RegExp(`^[0-9a-f]{64}  ${path.basename(artifact.archive)}\\n$`, "u")
    )
    await expect(
      verifyNativeRelease({
        version: "0.4.0",
        target,
        archive: artifact.archive,
        output,
        release: true,
      })
    ).rejects.toThrow("not a clean v0.4.0 release build")

    await writeFile(artifact.checksum, `${"0".repeat(64)}  invalid.tar.gz\n`)
    await expect(
      verifyNativeRelease({
        version: "0.4.0",
        target,
        archive: artifact.archive,
        output,
      })
    ).rejects.toThrow("does not match")
  } finally {
    await rm(temporary, { recursive: true, force: true })
  }
})
