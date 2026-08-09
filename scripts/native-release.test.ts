import { expect, test } from "bun:test"
import { chmod, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import { repositoryRoot } from "./release-contract"
import {
  hostNativeReleaseTarget,
  nativeArchiveName,
  nativeChecksumName,
  nativeReleaseTargets,
  packageNativeRelease,
  verifyNativeRelease,
} from "./native-release"

test("uses one fixed native archive contract per release target", () => {
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
  for (const target of Object.keys(nativeReleaseTargets)) {
    expect(
      workflow.match(new RegExp(`target: ${target}`, "gu"))?.length ?? 0
    ).toBeGreaterThanOrEqual(2)
  }
  expect(workflow).toContain("native-release.ts smoke")
  expect(workflow).toContain("native-release.ts verify")
  expect(workflow).toContain("needs: [native-verify, vscode")
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
      `#!/bin/sh\nprintf '%s\\n' 'seseragi 0.4.0 (development, commit fixture, target ${rustTarget})'\n`
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
