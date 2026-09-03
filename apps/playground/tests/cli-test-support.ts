import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..")
const targetDirectory = resolve(
  root,
  process.env.CARGO_TARGET_DIR ?? "target"
)

let cliBuild: Promise<string> | undefined

export function ensureSeseragiCli(): Promise<string> {
  cliBuild ??= runCommand(["cargo", "build", "-p", "seseragi-cli"]).then(
    () => resolve(targetDirectory, "debug/seseragi")
  )
  return cliBuild
}

export async function runCommand(command: string[]): Promise<void> {
  const child = Bun.spawn(command, {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  })
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ])
  if (exitCode !== 0) {
    throw new Error(
      `${command.join(" ")} failed (${exitCode})\n${stdout}${stderr}`
    )
  }
}
