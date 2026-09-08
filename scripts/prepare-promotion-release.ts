import { readFile, writeFile } from "node:fs/promises"
import path from "node:path"
import {
  readReleaseContract,
  replaceWorkspaceVersion,
  syncReleaseVersions,
} from "./release-contract"

export const repositoryRoot = path.resolve(import.meta.dir, "..")

function fail(message: string): never {
  throw new Error(`promotion release preparation: ${message}`)
}

export function nextPatchVersion(version: string): string {
  const match = /^(\d+)\.(\d+)\.(\d+)$/u.exec(version)
  if (!match) fail(`stable x.y.z version required, received ${version}`)
  return `${Number(match[1])}.${Number(match[2])}.${Number(match[3]) + 1}`
}

export function normalizeIssueTitle(title: string): string {
  const normalized = title.replace(/[\r\n]+/gu, " ").replace(/\s+/gu, " ").trim()
  if (!normalized) fail("issue title must not be empty")
  return normalized
}

export function changelogEntry(
  version: string,
  date: string,
  issueNumber: number,
  issueTitle: string
): string {
  if (!/^\d{4}-\d{2}-\d{2}$/u.test(date)) fail(`invalid date ${date}`)
  if (!Number.isInteger(issueNumber) || issueNumber <= 0) {
    fail(`invalid issue number ${issueNumber}`)
  }
  const title = normalizeIssueTitle(issueTitle)
  return [
    `## [${version}] - ${date}`,
    "",
    "### Changed",
    "",
    `- #${issueNumber} ${title}`,
    "",
  ].join("\n")
}

export function prependChangelogEntry(
  changelog: string,
  entry: string,
  version: string
): string {
  if (new RegExp(`^## \\[${version.replaceAll(".", "\\.")}\\]`, "mu").test(changelog)) {
    fail(`CHANGELOG.md already contains ${version}`)
  }
  const header = "# Change Log\n\n"
  if (!changelog.startsWith(header)) {
    fail("CHANGELOG.md must start with '# Change Log'")
  }
  return `${header}${entry}${changelog.slice(header.length)}`
}

export async function preparePromotionRelease(options: {
  issueNumber: number
  issueTitle: string
  date: string
  root?: string
}): Promise<{ version: string; tag: string }> {
  const root = options.root ?? repositoryRoot
  const current = await readReleaseContract(root)
  const version = nextPatchVersion(current.version)

  const cargoPath = path.join(root, "Cargo.toml")
  const cargo = await readFile(cargoPath, "utf8")
  await writeFile(cargoPath, replaceWorkspaceVersion(cargo, version))
  await syncReleaseVersions(root)

  const changelogPath = path.join(root, "CHANGELOG.md")
  const changelog = await readFile(changelogPath, "utf8")
  const entry = changelogEntry(
    version,
    options.date,
    options.issueNumber,
    options.issueTitle
  )
  await writeFile(
    changelogPath,
    prependChangelogEntry(changelog, entry, version)
  )

  return { version, tag: `v${version}` }
}

function argument(name: string): string | undefined {
  const index = process.argv.indexOf(name)
  return index === -1 ? undefined : process.argv[index + 1]
}

async function main(): Promise<void> {
  const issue = Number(argument("--issue"))
  const title = argument("--title")
  const date = argument("--date")
  if (!Number.isInteger(issue) || issue <= 0 || !title || !date) {
    fail(
      "usage: prepare-promotion-release.ts --issue N --title TITLE --date YYYY-MM-DD"
    )
  }
  console.log(
    JSON.stringify(
      await preparePromotionRelease({
        issueNumber: issue,
        issueTitle: title,
        date,
      })
    )
  )
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  })
}
