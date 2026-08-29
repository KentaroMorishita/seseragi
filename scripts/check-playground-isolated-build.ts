import {
  copyFile,
  lstat,
  mkdir,
  mkdtemp,
  readlink,
  rm,
  symlink,
} from "node:fs/promises"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"

const root = resolve(import.meta.dir, "..")
const isolatedRoot = await mkdtemp(
  join(tmpdir(), "seseragi-playground-isolated-")
)

async function run(command: string[], cwd: string): Promise<void> {
  const process = Bun.spawn(command, {
    cwd,
    stdout: "inherit",
    stderr: "inherit",
  })
  const exitCode = await process.exited
  if (exitCode !== 0) {
    throw new Error(`${command.join(" ")} failed with exit code ${exitCode}`)
  }
}

async function workspaceFiles(): Promise<string[]> {
  const process = Bun.spawn(
    ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
    { cwd: root, stdout: "pipe", stderr: "pipe" }
  )
  const [exitCode, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ])
  if (exitCode !== 0) {
    throw new Error(`git ls-files failed (${exitCode})\n${stderr}`)
  }
  return stdout.split("\0").filter(Boolean)
}

async function copyWorkspaceFile(relative: string): Promise<void> {
  const source = join(root, relative)
  const destination = join(isolatedRoot, relative)
  const metadata = await lstat(source)
  await mkdir(dirname(destination), { recursive: true })
  if (metadata.isSymbolicLink()) {
    await symlink(await readlink(source), destination)
    return
  }
  await copyFile(source, destination)
}

try {
  for (const relative of await workspaceFiles()) {
    await copyWorkspaceFile(relative)
  }

  const playground = join(isolatedRoot, "apps/playground")
  await run(["bun", "install", "--frozen-lockfile"], playground)
  await run(["bun", "run", "build"], playground)
  console.log(
    "Playground isolated build passed without repository root node_modules."
  )
} finally {
  await rm(isolatedRoot, { recursive: true, force: true })
}
