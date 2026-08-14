import path from "node:path"
import { readReleaseContract } from "./release-contract"

export const repositoryRoot = path.resolve(import.meta.dir, "..")

export const USER_VISIBLE_SURFACES = [
  { name: "cli", prefixes: ["crates/seseragi-cli/"] },
  {
    name: "compiler",
    prefixes: [
      "crates/seseragi-driver/",
      "crates/seseragi-formatter/",
      "crates/seseragi-lowering/",
      "crates/seseragi-project/",
      "crates/seseragi-provider/",
      "crates/seseragi-runtime/",
      "crates/seseragi-semantics/",
      "crates/seseragi-source/",
      "crates/seseragi-syntax/",
    ],
  },
  { name: "lsp", prefixes: ["crates/seseragi-lsp/"] },
  { name: "runtime", prefixes: ["runtime/ts/"] },
  {
    name: "wasm-playground",
    prefixes: ["crates/seseragi-wasm/", "apps/playground/"],
  },
  {
    name: "vscode",
    prefixes: ["extensions/seseragi/", "extensions/seseragi-legacy/"],
  },
  { name: "language-spec", prefixes: ["examples/spec/", "docs/spec/"] },
] as const

const INTERNAL_ONLY_PATHS = [
  /(^|\/)(?:tests?|[^/]*_tests?)\//u,
  /(^|\/)__snapshots__\//u,
  /(?:^|\/)[^/]*(?:_test|_tests|\.test)\.[^/]+$/u,
  /\.(?:snap|test)\.[^/]+$/u,
] as const

export type ReleaseReadinessState =
  | "pending-release"
  | "released"
  | "version-bump-required"

export type ReleaseReadiness = {
  schemaVersion: 1
  version: string
  tag: string
  state: ReleaseReadinessState
  comparisonBase: string | null
  latestReleaseTag: string | null
  userVisibleSurfaces: string[]
  userVisibleFiles: string[]
}

export type PullRequestReleaseBoundary = {
  schemaVersion: 1
  base: string
  head: string
  baseVersion: string
  headVersion: string
  userVisibleSurfaces: string[]
  userVisibleFiles: string[]
}

function fail(message: string): never {
  throw new Error(`release readiness: ${message}`)
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

function semverParts(tag: string): number[] | null {
  const match = /^v(\d+)\.(\d+)\.(\d+)$/u.exec(tag)
  return match ? match.slice(1).map(Number) : null
}

function versionFromCargoToml(contents: string, ref: string): string {
  const workspace = /\[workspace\.package\]([\s\S]*?)(?:\n\[|$)/u.exec(
    contents
  )?.[1]
  const version = workspace
    ? /^version\s*=\s*"(\d+\.\d+\.\d+)"\s*$/mu.exec(workspace)?.[1]
    : undefined
  if (!version) fail(`cannot read canonical version from ${ref}:Cargo.toml`)
  return version
}

function fileAt(ref: string, file: string, root: string): string {
  return git(["show", `${ref}:${file}`], root).stdout
}

function compareTags(left: string, right: string): number {
  const leftParts = semverParts(left)
  const rightParts = semverParts(right)
  if (!leftParts || !rightParts) return left.localeCompare(right)
  for (let index = 0; index < leftParts.length; index += 1) {
    const difference = leftParts[index] - rightParts[index]
    if (difference !== 0) return difference
  }
  return 0
}

function releaseTags(root: string): string[] {
  return git(["tag", "--list", "v[0-9]*"], root)
    .stdout.split("\n")
    .filter((tag) => semverParts(tag) !== null)
    .sort(compareTags)
}

function tagCommit(tag: string, root: string): string | null {
  const result = git(["rev-list", "-n", "1", `refs/tags/${tag}`], root, {
    allowFailure: true,
  })
  return result.exitCode === 0 ? result.stdout : null
}

function changedFiles(base: string | null, root: string): string[] {
  const args = base
    ? ["diff", "--name-only", `${base}..HEAD`]
    : ["ls-tree", "-r", "--name-only", "HEAD"]
  return git(args, root).stdout.split("\n").filter(Boolean)
}

function userVisibleFiles(files: string[]): string[] {
  return files.filter(
    (file) =>
      !INTERNAL_ONLY_PATHS.some((pattern) => pattern.test(file)) &&
      USER_VISIBLE_SURFACES.some(({ prefixes }) =>
        prefixes.some((prefix) => file.startsWith(prefix))
      )
  )
}

function userVisibleSurfaces(files: string[]): string[] {
  return USER_VISIBLE_SURFACES.filter(({ prefixes }) =>
    files.some((file) => prefixes.some((prefix) => file.startsWith(prefix)))
  ).map(({ name }) => name)
}

export function pullRequestReleaseBoundary(
  base: string,
  head: string,
  root = repositoryRoot
): PullRequestReleaseBoundary {
  if (!base || !head) fail("both PR base and head refs are required")
  const files = git(["diff", "--name-only", `${base}...${head}`], root)
    .stdout.split("\n")
    .filter(Boolean)
  const visibleFiles = userVisibleFiles(files)
  const baseVersion = versionFromCargoToml(
    fileAt(base, "Cargo.toml", root),
    base
  )
  const headVersion = versionFromCargoToml(
    fileAt(head, "Cargo.toml", root),
    head
  )

  if (visibleFiles.length > 0) {
    if (compareTags(`v${headVersion}`, `v${baseVersion}`) <= 0) {
      fail(
        `PR changes ${visibleFiles.length} user-visible file(s), but canonical version ${headVersion} is not newer than base version ${baseVersion}`
      )
    }
    if (!files.includes("CHANGELOG.md")) {
      fail(`PR version ${headVersion} requires a CHANGELOG.md update`)
    }
    const heading = new RegExp(
      `^## \\[${headVersion.replaceAll(".", "\\.")}\\]`,
      "mu"
    )
    if (!heading.test(fileAt(head, "CHANGELOG.md", root))) {
      fail(`CHANGELOG.md has no entry for PR version ${headVersion}`)
    }
  }

  return {
    schemaVersion: 1,
    base,
    head,
    baseVersion,
    headVersion,
    userVisibleSurfaces: userVisibleSurfaces(visibleFiles),
    userVisibleFiles: visibleFiles,
  }
}

export async function releaseReadiness(
  root = repositoryRoot
): Promise<ReleaseReadiness> {
  const { version, tag } = await readReleaseContract(root)
  const tags = releaseTags(root)
  const currentTagCommit = tagCommit(tag, root)
  const latestReleaseTag = tags.at(-1) ?? null

  if (tags.some((candidate) => compareTags(candidate, tag) > 0)) {
    fail(`${tag} is behind an existing release tag`)
  }

  const comparisonBase = currentTagCommit ? tag : latestReleaseTag
  const visibleFiles = userVisibleFiles(changedFiles(comparisonBase, root))
  const visibleSurfaces = userVisibleSurfaces(visibleFiles)
  let state: ReleaseReadinessState
  if (!currentTagCommit) {
    state = "pending-release"
  } else if (visibleFiles.length > 0) {
    state = "version-bump-required"
  } else {
    state = "released"
  }

  return {
    schemaVersion: 1,
    version,
    tag,
    state,
    comparisonBase,
    latestReleaseTag,
    userVisibleSurfaces: visibleSurfaces,
    userVisibleFiles: visibleFiles,
  }
}

export function assertReleaseReadiness(result: ReleaseReadiness): void {
  if (result.state === "version-bump-required") {
    fail(
      `${result.tag} already exists, but ${result.userVisibleFiles.length} user-visible file(s) changed afterward; bump the canonical version and update CHANGELOG.md`
    )
  }
}

async function main(): Promise<void> {
  const command = process.argv[2] || "check"
  if (command === "check-pr") {
    const base = process.argv[3]
    const head = process.argv[4]
    const result = pullRequestReleaseBoundary(base, head)
    const surfaces = result.userVisibleSurfaces.join(", ") || "none"
    console.log(
      `PR release boundary: ${result.baseVersion} -> ${result.headVersion} (surfaces: ${surfaces}).`
    )
    return
  }
  if (command !== "check" && command !== "info") {
    fail("usage: release-readiness.ts <check|info|check-pr BASE HEAD>")
  }
  const result = await releaseReadiness()
  if (command === "info") {
    console.log(
      JSON.stringify(
        {
          schemaVersion: result.schemaVersion,
          version: result.version,
          tag: result.tag,
          state: result.state,
          comparisonBase: result.comparisonBase,
          latestReleaseTag: result.latestReleaseTag,
          userVisibleSurfaces: result.userVisibleSurfaces,
          userVisibleFileCount: result.userVisibleFiles.length,
        },
        null,
        2
      )
    )
    return
  }
  assertReleaseReadiness(result)
  const surfaces = result.userVisibleSurfaces.join(", ") || "none"
  console.log(
    `Release readiness: ${result.state} (${result.tag}; surfaces: ${surfaces}).`
  )
}

if (import.meta.main) await main()
