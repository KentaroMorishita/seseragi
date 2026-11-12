import { describe, expect, test } from "bun:test"
import { createHash } from "node:crypto"
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { pathToFileURL } from "node:url"

const script = resolve(import.meta.dir, "install.sh")

function hostTarget(): string | undefined {
  const os = { darwin: "darwin", linux: "linux" }[process.platform as string]
  const cpu = { arm64: "arm64", x64: "x64" }[process.arch as string]
  const target = os && cpu ? `${os}-${cpu}` : undefined
  return ["darwin-arm64", "darwin-x64", "linux-x64"].includes(target ?? "")
    ? target
    : undefined
}

const target = hostTarget()

function run(command: string[], cwd?: string): string {
  const result = Bun.spawnSync(command, { cwd, stderr: "pipe" })
  if (result.exitCode !== 0) {
    throw new Error(`${command.join(" ")} failed: ${result.stderr}`)
  }
  return result.stdout.toString()
}

function publishFakeRelease(
  root: string,
  version: string,
  checksum?: string
): string {
  const archiveTarget = target as string
  const contents = join(root, `contents-${version}`)
  mkdirSync(contents, { recursive: true })
  for (const binary of ["seseragi", "seseragi-lsp"]) {
    const path = join(contents, binary)
    writeFileSync(path, `#!/bin/sh\necho "${binary} ${version}"\n`)
    chmodSync(path, 0o755)
  }
  writeFileSync(join(contents, "UNICODE-LICENSE"), `license ${version}\n`)
  const releaseDirectory = join(root, `release-${version}`)
  mkdirSync(releaseDirectory, { recursive: true })
  const archive = `seseragi-v${version}-${archiveTarget}.tar.gz`
  run([
    "tar",
    "-czf",
    join(releaseDirectory, archive),
    "-C",
    contents,
    "seseragi",
    "seseragi-lsp",
    "UNICODE-LICENSE",
  ])
  const digest =
    checksum ??
    createHash("sha256")
      .update(readFileSync(join(releaseDirectory, archive)))
      .digest("hex")
  writeFileSync(
    join(releaseDirectory, `${archive}.sha256`),
    `${digest}  ${archive}\n`
  )
  return pathToFileURL(releaseDirectory).href
}

function install(root: string, version: string, downloadUrl: string) {
  return Bun.spawnSync(["sh", script], {
    env: {
      HOME: root,
      PATH: process.env.PATH ?? "/usr/bin:/bin",
      SESERAGI_VERSION: version,
      SESERAGI_DOWNLOAD_URL: downloadUrl,
      SESERAGI_INSTALL: join(root, "seseragi"),
    },
    stderr: "pipe",
  })
}

describe.skipIf(target === undefined)("install.sh", () => {
  test("installs both binaries and updates them in place", () => {
    const root = mkdtempSync(join(tmpdir(), "seseragi-install-"))
    const binary = join(root, "seseragi", "bin", "seseragi")

    const first = install(root, "1.2.3", publishFakeRelease(root, "1.2.3"))
    expect(first.stderr.toString()).toBe("")
    expect(first.exitCode).toBe(0)
    expect(first.stdout.toString()).toContain("Installed seseragi 1.2.3")
    expect(run([binary])).toBe("seseragi 1.2.3\n")
    expect(run([join(root, "seseragi", "bin", "seseragi-lsp")])).toBe(
      "seseragi-lsp 1.2.3\n"
    )

    const second = install(root, "v1.3.0", publishFakeRelease(root, "1.3.0"))
    expect(second.exitCode).toBe(0)
    expect(run([binary])).toBe("seseragi 1.3.0\n")
    expect(
      readFileSync(join(root, "seseragi", "UNICODE-LICENSE"), "utf8")
    ).toBe("license 1.3.0\n")
  })

  test("refuses an archive whose checksum does not match", () => {
    const root = mkdtempSync(join(tmpdir(), "seseragi-install-"))
    const result = install(
      root,
      "1.2.3",
      publishFakeRelease(root, "1.2.3", "0".repeat(64))
    )
    expect(result.exitCode).toBe(1)
    expect(result.stderr.toString()).toContain("checksum mismatch")
    expect(existsSync(join(root, "seseragi"))).toBe(false)
  })

  test("refuses a missing checksum file", () => {
    const root = mkdtempSync(join(tmpdir(), "seseragi-install-"))
    const url = publishFakeRelease(root, "1.2.3")
    rmSync(
      join(new URL(url).pathname, `seseragi-v1.2.3-${target}.tar.gz.sha256`)
    )
    const result = install(root, "1.2.3", url)
    expect(result.exitCode).toBe(1)
    expect(result.stderr.toString()).toContain(
      "could not download the checksum"
    )
    expect(existsSync(join(root, "seseragi"))).toBe(false)
  })

  test("rejects a version that is not a release number", () => {
    const root = mkdtempSync(join(tmpdir(), "seseragi-install-"))
    const result = install(root, "1.2.3; rm -rf /", "file:///nowhere")
    expect(result.exitCode).toBe(1)
    expect(result.stderr.toString()).toContain("invalid version")
  })
})
