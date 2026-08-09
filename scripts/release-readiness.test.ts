import { afterEach, describe, expect, test } from "bun:test"
import { execFileSync } from "node:child_process"
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import {
  assertReleaseReadiness,
  releaseReadiness,
  repositoryRoot,
} from "./release-readiness"

const temporaryRepositories: string[] = []

function git(root: string, ...args: string[]): string {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim()
}

async function repository(version = "0.3.0"): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "seseragi-readiness-"))
  temporaryRepositories.push(root)
  git(root, "init", "-b", "main")
  git(root, "config", "user.name", "Seseragi release test")
  git(root, "config", "user.email", "release-test@example.invalid")
  await mkdir(path.join(root, "crates", "seseragi-lsp"), { recursive: true })
  await writeFile(
    path.join(root, "Cargo.toml"),
    `[workspace.package]\nversion = "${version}"\n`
  )
  await writeFile(
    path.join(root, "crates", "seseragi-lsp", "server.rs"),
    "v1\n"
  )
  git(root, "add", ".")
  git(root, "commit", "-m", "initial release")
  git(root, "tag", "-a", `v${version}`, "-m", `release v${version}`)
  return root
}

async function setVersion(root: string, version: string): Promise<void> {
  await writeFile(
    path.join(root, "Cargo.toml"),
    `[workspace.package]\nversion = "${version}"\n`
  )
}

afterEach(async () => {
  await Promise.all(
    temporaryRepositories
      .splice(0)
      .map((root) => rm(root, { recursive: true, force: true }))
  )
})

describe("release readiness", () => {
  test("reports a version bump with user-visible changes as pending", async () => {
    const root = await repository()
    await setVersion(root, "0.4.0")
    await writeFile(
      path.join(root, "crates", "seseragi-lsp", "server.rs"),
      "v2\n"
    )
    git(root, "commit", "-am", "prepare next release")

    const result = await releaseReadiness(root)
    expect(result.state).toBe("pending-release")
    expect(result.latestReleaseTag).toBe("v0.3.0")
    expect(result.userVisibleSurfaces).toEqual(["lsp"])
    expect(() => assertReleaseReadiness(result)).not.toThrow()
  })

  test("reports the tagged release commit as released", async () => {
    const root = await repository()

    const result = await releaseReadiness(root)
    expect(result.state).toBe("released")
    expect(result.userVisibleFiles).toEqual([])
  })

  test("rejects user-visible changes after the current version tag", async () => {
    const root = await repository()
    await writeFile(
      path.join(root, "crates", "seseragi-lsp", "server.rs"),
      "v2\n"
    )
    git(root, "commit", "-am", "change shipped LSP behavior")

    const result = await releaseReadiness(root)
    expect(result.state).toBe("version-bump-required")
    expect(() => assertReleaseReadiness(result)).toThrow(
      "bump the canonical version"
    )
  })

  test("allows internal release process changes after the current tag", async () => {
    const root = await repository()
    await mkdir(path.join(root, "docs"), { recursive: true })
    await writeFile(path.join(root, "docs", "RELEASE.md"), "process only\n")
    git(root, "add", ".")
    git(root, "commit", "-m", "document release process")

    const result = await releaseReadiness(root)
    expect(result.state).toBe("released")
    expect(result.userVisibleFiles).toEqual([])
  })

  test("rejects a canonical version behind a newer release tag", async () => {
    const root = await repository()
    await setVersion(root, "0.4.0")
    git(root, "commit", "-am", "release 0.4.0")
    git(root, "tag", "-a", "v0.4.0", "-m", "release v0.4.0")
    await setVersion(root, "0.3.0")
    git(root, "commit", "-am", "accidentally lower version")

    await expect(releaseReadiness(root)).rejects.toThrow(
      "is behind an existing release tag"
    )
  })

  test("runs the lightweight detector on main changes and monthly", async () => {
    const workflow = await readFile(
      path.join(repositoryRoot, ".github/workflows/release-readiness.yml"),
      "utf8"
    )

    expect(workflow).toContain("branches: [main]")
    expect(workflow).toContain('cron: "17 0 1 * *"')
    expect(workflow).toContain("fetch-depth: 0")
    expect(workflow).toContain("bun scripts/release-readiness.ts check")
    for (const surface of [
      "crates/seseragi-cli/**",
      "crates/seseragi-lsp/**",
      "runtime/ts/**",
      "apps/playground/**",
      "extensions/seseragi/**",
      "examples/spec/**",
      "docs/spec/**",
    ]) {
      expect(workflow).toContain(surface)
    }
  })
})
