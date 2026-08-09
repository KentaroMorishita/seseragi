import path from "node:path"

export const repositoryRoot = path.resolve(import.meta.dir, "..")

function fail(message: string): never {
  throw new Error(`release gate: ${message}`)
}

function git(
  args: string[],
  root: string,
  options: { allowFailure?: boolean } = {}
): { exitCode: number; stdout: string; stderr: string } {
  const result = Bun.spawnSync(["git", ...args], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  })
  const output = {
    exitCode: result.exitCode,
    stdout: result.stdout.toString().trim(),
    stderr: result.stderr.toString().trim(),
  }
  if (output.exitCode !== 0 && !options.allowFailure) {
    fail(
      `git ${args.join(" ")} failed: ${output.stderr || `exit ${output.exitCode}`}`
    )
  }
  return output
}

export function verifyReleaseCommit(
  tag: string,
  mainRef = "refs/remotes/origin/main",
  root = repositoryRoot
): string {
  if (!tag) fail("a release tag is required")

  const releaseSha = git(["rev-parse", "HEAD^{commit}"], root).stdout
  const tagSha = git(["rev-list", "-n", "1", `refs/tags/${tag}`], root).stdout
  if (tagSha !== releaseSha) {
    fail(
      `tag ${tag} resolves to ${tagSha}, but the checked-out release commit is ${releaseSha}`
    )
  }

  const containment = git(
    ["merge-base", "--is-ancestor", releaseSha, mainRef],
    root,
    { allowFailure: true }
  )
  if (containment.exitCode !== 0) {
    fail(`${releaseSha} is not contained in ${mainRef}`)
  }

  return releaseSha
}

function option(name: string): string | undefined {
  const index = process.argv.indexOf(name)
  return index === -1 ? undefined : process.argv[index + 1]
}

function main(): void {
  const command = process.argv[2]
  if (command !== "check-main") {
    fail("usage: release-gate.ts check-main --tag TAG [--main-ref REF]")
  }
  const tag = option("--tag") || process.env.GITHUB_REF_NAME
  if (!tag) fail("--tag or GITHUB_REF_NAME is required")
  console.log(
    verifyReleaseCommit(tag, option("--main-ref") || "refs/remotes/origin/main")
  )
}

if (import.meta.main) main()
