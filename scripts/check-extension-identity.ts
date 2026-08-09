import { existsSync } from "node:fs"
import { readdir, readFile } from "node:fs/promises"
import path from "node:path"

const repositoryRoot = path.resolve(import.meta.dir, "..")
const LEGACY_REFERENCE =
  /seseragi-spec-preview|seseragi-dev\.seseragi-spec-preview/u
const IGNORED_DIRECTORIES = new Set([".git", "dist", "node_modules", "target"])
const TEXT_EXTENSIONS = new Set([
  ".js",
  ".json",
  ".md",
  ".sh",
  ".toml",
  ".ts",
  ".yaml",
  ".yml",
])
const LEGACY_REFERENCE_COUNTS = new Map([
  ["extensions/seseragi/README.md", 1],
  ["extensions/seseragi/extension-core.js", 1],
  ["extensions/seseragi/scripts/verify-legacy-package.ts", 1],
  ["extensions/seseragi-legacy/README.md", 2],
  ["extensions/seseragi-legacy/package.json", 1],
])

async function textFiles(root: string, directory = root): Promise<string[]> {
  const files: string[] = []
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!IGNORED_DIRECTORIES.has(entry.name)) {
        files.push(...(await textFiles(root, path.join(directory, entry.name))))
      }
      continue
    }
    const relative = path.relative(root, path.join(directory, entry.name))
    if (
      TEXT_EXTENSIONS.has(path.extname(entry.name)) ||
      entry.name === ".gitignore" ||
      entry.name === "AGENTS.md"
    ) {
      files.push(relative)
    }
  }
  return files
}

function checkManifest(
  manifest: {
    name?: unknown
    publisher?: unknown
    repository?: { directory?: unknown }
  },
  expected: { id: string; directory: string }
): void {
  if (`${manifest.publisher}.${manifest.name}` !== expected.id) {
    throw new Error(`extension ID must be ${expected.id}`)
  }
  if (manifest.repository?.directory !== expected.directory) {
    throw new Error(`extension directory must be ${expected.directory}`)
  }
}

export async function checkExtensionIdentity(
  root = repositoryRoot
): Promise<void> {
  const oldDirectory = path.join(root, "extensions", "seseragi-spec-preview")
  if (existsSync(oldDirectory)) {
    throw new Error(
      "the legacy extension directory must not remain in the repository"
    )
  }

  const official = JSON.parse(
    await readFile(path.join(root, "extensions/seseragi/package.json"), "utf8")
  )
  checkManifest(official, {
    id: "seseragi-dev.seseragi",
    directory: "extensions/seseragi",
  })
  if (
    official.contributes?.configurationDefaults?.["[seseragi]"]?.[
      "editor.defaultFormatter"
    ] !== "seseragi-dev.seseragi"
  ) {
    throw new Error("the official extension must own the Seseragi formatter ID")
  }

  const legacy = JSON.parse(
    await readFile(
      path.join(root, "extensions/seseragi-legacy/package.json"),
      "utf8"
    )
  )
  checkManifest(legacy, {
    id: "seseragi-dev.seseragi-spec-preview",
    directory: "extensions/seseragi-legacy",
  })
  const legacySource = await readFile(
    path.join(root, "extensions/seseragi-legacy/extension.js"),
    "utf8"
  )
  if (legacySource.includes("seseragi-lsp")) {
    throw new Error("the legacy migration stub must never start seseragi-lsp")
  }

  const unexpected = []
  for (const relative of await textFiles(root)) {
    const source = await readFile(path.join(root, relative), "utf8")
    const references = source.match(
      new RegExp(LEGACY_REFERENCE.source, "gu")
    )?.length
    if (!references) continue
    if (relative === "scripts/check-extension-identity.ts") continue
    if (LEGACY_REFERENCE_COUNTS.get(relative) === references) continue
    unexpected.push(relative)
  }
  if (unexpected.length > 0) {
    throw new Error(
      `unexpected legacy extension references remain in ${unexpected.join(", ")}`
    )
  }
}

if (import.meta.main) {
  await checkExtensionIdentity()
  console.log(
    "Official extension identity and legacy migration boundary are valid."
  )
}
