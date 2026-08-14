import { afterEach, describe, expect, test } from "bun:test"
import { execFileSync } from "node:child_process"
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import { planReleasePromotion } from "./release-promotion"

const temporaryRepositories: string[] = []

function git(root: string, ...args: string[]): string {
  return execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim()
}

async function repository(): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "seseragi-promotion-"))
  temporaryRepositories.push(root)
  git(root, "init", "-b", "main")
  git(root, "config", "user.name", "Seseragi release test")
  git(root, "config", "user.email", "release-test@example.invalid")
  await mkdir(path.join(root, "crates", "seseragi-cli"), { recursive: true })
  await writeFile(
    path.join(root, "Cargo.toml"),
    '[workspace.package]\nversion = "0.4.0"\n'
  )
  await writeFile(path.join(root, "crates", "seseragi-cli", "main.rs"), "v1\n")
  git(root, "add", ".")
  git(root, "commit", "-m", "release 0.4.0")
  git(root, "tag", "-a", "v0.4.0", "-m", "release v0.4.0")
  return root
}

async function preparePending(root: string): Promise<string> {
  await writeFile(
    path.join(root, "Cargo.toml"),
    '[workspace.package]\nversion = "0.4.1"\n'
  )
  await writeFile(path.join(root, "crates", "seseragi-cli", "main.rs"), "v2\n")
  await writeFile(path.join(root, "CHANGELOG.md"), "## [0.4.1]\n\n- CLI.\n")
  git(root, "add", ".")
  git(root, "commit", "-m", "prepare 0.4.1")
  return git(root, "rev-parse", "HEAD")
}

afterEach(async () => {
  await Promise.all(
    temporaryRepositories
      .splice(0)
      .map((root) => rm(root, { recursive: true, force: true }))
  )
})

describe("release promotion", () => {
  test("promotes a pending canonical version from its main commit", async () => {
    const root = await repository()
    const sha = await preparePending(root)

    expect(await planReleasePromotion(false, root)).toEqual({
      schemaVersion: 1,
      action: "release",
      tag: "v0.4.1",
      releaseSha: sha,
      createTag: true,
      reason: "pending-version",
    })
  })

  test("promotes an explicit internal recovery version", async () => {
    const root = await repository()
    await writeFile(
      path.join(root, "Cargo.toml"),
      '[workspace.package]\nversion = "0.4.1"\n'
    )
    await writeFile(
      path.join(root, "CHANGELOG.md"),
      "## [0.4.1]\n\n- Release recovery.\n"
    )
    git(root, "add", ".")
    git(root, "commit", "-m", "prepare internal recovery")

    const plan = await planReleasePromotion(false, root)
    expect(plan.action).toBe("release")
    expect(plan.tag).toBe("v0.4.1")
    expect(plan.reason).toBe("pending-version")
  })

  test("retries an incomplete release from the existing tag commit", async () => {
    const root = await repository()
    const sha = await preparePending(root)
    git(root, "tag", "-a", "v0.4.1", "-m", "release v0.4.1")
    await writeFile(path.join(root, "internal.txt"), "after tag\n")
    git(root, "add", ".")
    git(root, "commit", "-m", "internal follow-up")

    const plan = await planReleasePromotion(false, root)
    expect(plan.reason).toBe("incomplete-release")
    expect(plan.releaseSha).toBe(sha)
    expect(plan.createTag).toBe(false)
  })

  test("does not release the same version twice", async () => {
    const root = await repository()

    const plan = await planReleasePromotion(true, root)
    expect(plan.action).toBe("skip")
    expect(plan.reason).toBe("already-released")
    expect(plan.createTag).toBe(false)
  })
})
