import { appendFile } from "node:fs/promises"
import path from "node:path"
import {
  assertReleaseReadiness,
  releaseReadiness,
  repositoryRoot,
} from "./release-readiness"

export type ReleasePromotion = {
  schemaVersion: 1
  action: "release" | "skip"
  tag: string
  releaseSha: string
  createTag: boolean
  reason: "pending-version" | "incomplete-release" | "already-released"
}

function fail(message: string): never {
  throw new Error(`release promotion: ${message}`)
}

function git(args: string[], root: string): string {
  const result = Bun.spawnSync(["git", ...args], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  })
  if (result.exitCode !== 0) {
    fail(
      `git ${args.join(" ")} failed: ${result.stderr.toString().trim() || `exit ${result.exitCode}`}`
    )
  }
  return result.stdout.toString().trim()
}

export async function planReleasePromotion(
  releasePublished: boolean,
  root = repositoryRoot
): Promise<ReleasePromotion> {
  const readiness = await releaseReadiness(root)
  assertReleaseReadiness(readiness)

  if (releasePublished) {
    return {
      schemaVersion: 1,
      action: "skip",
      tag: readiness.tag,
      releaseSha: git(["rev-list", "-n", "1", readiness.tag], root),
      createTag: false,
      reason: "already-released",
    }
  }

  if (readiness.state === "pending-release") {
    return {
      schemaVersion: 1,
      action: "release",
      tag: readiness.tag,
      releaseSha: git(["rev-parse", "HEAD^{commit}"], root),
      createTag: true,
      reason: "pending-version",
    }
  }

  return {
    schemaVersion: 1,
    action: "release",
    tag: readiness.tag,
    releaseSha: git(["rev-list", "-n", "1", readiness.tag], root),
    createTag: false,
    reason: "incomplete-release",
  }
}

async function main(): Promise<void> {
  const command = process.argv[2]
  if (command !== "plan") {
    fail("usage: release-promotion.ts plan --published <true|false>")
  }
  const publishedIndex = process.argv.indexOf("--published")
  const published = process.argv[publishedIndex + 1]
  if (published !== "true" && published !== "false") {
    fail("--published must be true or false")
  }
  const plan = await planReleasePromotion(published === "true")
  console.log(JSON.stringify(plan, null, 2))

  const output = process.env.GITHUB_OUTPUT
  if (output) {
    await appendFile(
      path.resolve(output),
      [
        `should_release=${plan.action === "release"}`,
        `release_tag=${plan.tag}`,
        `release_sha=${plan.releaseSha}`,
        `create_tag=${plan.createTag}`,
        `reason=${plan.reason}`,
        "",
      ].join("\n")
    )
  }
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  })
}
