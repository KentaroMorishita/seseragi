import { existsSync, readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..")
const readmePath = resolve(root, "README.md")
const readme = readFileSync(readmePath, "utf8")

const normalize = (value: string): string => value.replace(/\r\n/gu, "\n").trimEnd()

const examplePattern =
  /<!-- canonical-example: path=([^\s]+) -->\r?\n```seseragi\r?\n([\s\S]*?)\r?\n```\r?\n<!-- \/canonical-example -->/u
const exampleMatch = readme.match(examplePattern)

if (!exampleMatch) {
  throw new Error("README canonical example marker is missing or malformed")
}

const sourceRelativePath = exampleMatch[1]
const sourcePath = resolve(root, sourceRelativePath)

if (!existsSync(sourcePath)) {
  throw new Error(`README canonical example source does not exist: ${sourceRelativePath}`)
}

const embeddedSource = normalize(exampleMatch[2])
const canonicalSource = normalize(readFileSync(sourcePath, "utf8"))

if (embeddedSource !== canonicalSource) {
  throw new Error(
    `README canonical example is stale: update it from ${sourceRelativePath}`,
  )
}

const localTargets = new Set<string>()
const markdownLinkPattern = /\]\((\.\.?\/[^)\s]+)(?:\s+"[^"]*")?\)/gu
const htmlLinkPattern = /\b(?:href|src|srcset)="(\.\.?\/[^"]+)"/gu

for (const pattern of [markdownLinkPattern, htmlLinkPattern]) {
  for (const match of readme.matchAll(pattern)) {
    const rawTarget = match[1].split(/[?#]/u, 1)[0]
    localTargets.add(decodeURIComponent(rawTarget))
  }
}

const missingTargets = [...localTargets]
  .filter((target) => !existsSync(resolve(root, target)))
  .sort()

if (missingTargets.length > 0) {
  throw new Error(`README contains missing local targets:\n${missingTargets.join("\n")}`)
}

console.log(
  `README check passed: canonical example and ${localTargets.size} local targets are current.`,
)
