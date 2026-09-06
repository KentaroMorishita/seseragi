import { readFile } from "node:fs/promises"
import path from "node:path"
import { spawnSync } from "node:child_process"

export const repositoryRoot = path.resolve(import.meta.dir, "..")

interface FixtureInventoryEntry {
  availability: "current" | "contract-only"
}

interface FixtureInventory {
  fixtures: Record<string, FixtureInventoryEntry>
}

export async function providerFixtureDirectories(
  root = repositoryRoot
): Promise<string[]> {
  const projectsRoot = path.join(root, "examples/spec/fixtures/projects")
  const inventory = JSON.parse(
    await readFile(path.join(projectsRoot, "inventory.json"), "utf8")
  ) as FixtureInventory

  const selected: string[] = []
  for (const [name, metadata] of Object.entries(inventory.fixtures)) {
    if (metadata.availability !== "current") continue
    const directory = path.join(projectsRoot, name)
    let lock: string
    try {
      lock = await readFile(path.join(directory, "seseragi.lock"), "utf8")
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") continue
      throw error
    }
    if (!lock.includes("[[providers]]")) continue
    selected.push(directory)
  }
  return selected.sort()
}

export async function refreshProviderFixtureLocks(options: {
  cli: string
  root?: string
}): Promise<string[]> {
  const root = options.root ?? repositoryRoot
  const directories = await providerFixtureDirectories(root)
  for (const directory of directories) {
    const result = spawnSync(options.cli, ["lock", "update"], {
      cwd: directory,
      encoding: "utf8",
      env: process.env,
    })
    if (result.error) throw result.error
    if (result.status !== 0) {
      throw new Error(
        [
          `provider lock refresh failed for ${path.relative(root, directory)}`,
          result.stdout.trim(),
          result.stderr.trim(),
        ]
          .filter(Boolean)
          .join("\n")
      )
    }
  }
  return directories.map((directory) => path.relative(root, directory))
}

function argument(name: string): string | undefined {
  const index = process.argv.indexOf(name)
  return index === -1 ? undefined : process.argv[index + 1]
}

async function main(): Promise<void> {
  const cli = argument("--cli")
  if (!cli) {
    throw new Error(
      "usage: refresh-provider-fixture-locks.ts --cli /path/to/seseragi"
    )
  }
  const refreshed = await refreshProviderFixtureLocks({
    cli: path.resolve(cli),
  })
  console.log(`Refreshed ${refreshed.length} provider-backed fixture locks.`)
  for (const fixture of refreshed) console.log(`- ${fixture}`)
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  })
}
