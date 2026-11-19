import path from "node:path"
import { readReleaseContract } from "./release-contract"

export const repositoryRoot = path.resolve(import.meta.dir, "..")

export const USER_VISIBLE_SURFACES = [
  { name: "cli", prefixes: ["crates/seseragi-cli/"] },
  {
    name: "compiler",
    prefixes: [
      "crates/seseragi-backend-rust/src/",
      "crates/seseragi-backend-rust/Cargo.toml",
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
  { name: "runtime", prefixes: ["runtime/ts/", "runtime/providers/"] },
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
  /^examples\/spec\/fixtures\/production\/budgets\.json$/u,
  /(^|\/)(?:tests?|[^/]*_tests?|e2e)\//u,
  /(^|\/)__snapshots__\//u,
  /(?:^|\/)[^/]*(?:_test|_tests|\.test|\.spec)\.[^/]+$/u,
  /(?:^|\/)tests\.rs$/u,
  /\.(?:snap|spec|test)\.[^/]+$/u,
  /^apps\/playground\/src\/generated\/(?:sample|tour|deep-dive)-manifest\.ts$/u,
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

function fileAtOptional(
  ref: string,
  file: string,
  root: string
): string | null {
  const result = git(["show", `${ref}:${file}`], root, { allowFailure: true })
  return result.exitCode === 0 ? result.stdout : null
}

function findMatchingRustBrace(source: string, openBraceIndex: number): number {
  let depth = 0
  let i = openBraceIndex
  while (i < source.length) {
    const ch = source[i]
    const next = source[i + 1]
    if (ch === "/" && next === "/") {
      i += 2
      while (i < source.length && source[i] !== "\n") i += 1
      continue
    }
    if (ch === "/" && next === "*") {
      let commentDepth = 1
      i += 2
      while (i < source.length && commentDepth > 0) {
        if (source[i] === "/" && source[i + 1] === "*") {
          commentDepth += 1
          i += 2
        } else if (source[i] === "*" && source[i + 1] === "/") {
          commentDepth -= 1
          i += 2
        } else {
          i += 1
        }
      }
      continue
    }
    if (ch === "r" || (ch === "b" && next === "r")) {
      let cursor = ch === "b" ? i + 2 : i + 1
      let hashes = 0
      while (source[cursor] === "#") {
        hashes += 1
        cursor += 1
      }
      if (source[cursor] === '"') {
        cursor += 1
        const closing = `"${"#".repeat(hashes)}`
        const end = source.indexOf(closing, cursor)
        if (end === -1) return -1
        i = end + closing.length
        continue
      }
    }
    if (ch === '"') {
      i += 1
      while (i < source.length && source[i] !== '"') {
        i += source[i] === "\\" ? 2 : 1
      }
      i += 1
      continue
    }
    if (ch === "'") {
      const charMatch = /^'(?:\\.|[^'\\\r\n])'/u.exec(source.slice(i))
      if (charMatch) {
        i += charMatch[0].length
        continue
      }
    }
    if (ch === "{") depth += 1
    else if (ch === "}") {
      depth -= 1
      if (depth === 0) return i
    }
    i += 1
  }
  return -1
}

function isOnlyWhitespaceOrRustComments(suffix: string): boolean {
  let i = 0
  while (i < suffix.length) {
    if (/\s/u.test(suffix[i]!)) {
      i += 1
      continue
    }
    if (suffix[i] === "/" && suffix[i + 1] === "/") {
      i += 2
      while (i < suffix.length && suffix[i] !== "\n") i += 1
      continue
    }
    if (suffix[i] === "/" && suffix[i + 1] === "*") {
      let commentDepth = 1
      i += 2
      while (i < suffix.length && commentDepth > 0) {
        if (suffix[i] === "/" && suffix[i + 1] === "*") {
          commentDepth += 1
          i += 2
        } else if (suffix[i] === "*" && suffix[i + 1] === "/") {
          commentDepth -= 1
          i += 2
        } else {
          i += 1
        }
      }
      if (commentDepth > 0) return false
      continue
    }
    return false
  }
  return true
}

export function stripRustInlineTestModule(source: string): string {
  const marker = /\r?\n#\[cfg\(test\)\][ \t]*\r?\nmod tests[ \t]*\{/u.exec(
    source
  )
  if (!marker) return source
  const openBraceIndex = marker.index + marker[0].length - 1
  const closeBraceIndex = findMatchingRustBrace(source, openBraceIndex)
  if (closeBraceIndex === -1) return source
  const trailing = source.slice(closeBraceIndex + 1)
  if (!isOnlyWhitespaceOrRustComments(trailing)) return source
  return source.slice(0, marker.index).trimEnd()
}

function isRustInlineTestOnlyChange(
  baseRef: string,
  headRef: string,
  file: string,
  root: string
): boolean {
  if (!file.endsWith(".rs")) return false
  const baseContent = fileAtOptional(baseRef, file, root)
  const headContent = fileAtOptional(headRef, file, root)
  if (baseContent === null || headContent === null) return false
  return (
    stripRustInlineTestModule(baseContent) ===
    stripRustInlineTestModule(headContent)
  )
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
  const mergeBase =
    git(["merge-base", base, head], root, { allowFailure: true }).stdout || base
  const visibleFiles = userVisibleFiles(
    files.filter(
      (file) => !isRustInlineTestOnlyChange(mergeBase, head, file, root)
    )
  )
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
  const rawChangedFiles = changedFiles(comparisonBase, root)
  const filteredChangedFiles = comparisonBase
    ? rawChangedFiles.filter(
        (file) =>
          !isRustInlineTestOnlyChange(comparisonBase, "HEAD", file, root)
      )
    : rawChangedFiles
  const visibleFiles = userVisibleFiles(filteredChangedFiles)
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
