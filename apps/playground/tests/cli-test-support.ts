import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { runSampleCommand } from "../../../scripts/sample-cli-process"

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..")
const targetDirectory = resolve(root, process.env.CARGO_TARGET_DIR ?? "target")

let cliBuild: Promise<string> | undefined

export function ensureSeseragiCli(): Promise<string> {
  cliBuild ??= runCommand(["cargo", "build", "-p", "seseragi-cli"]).then(() =>
    resolve(targetDirectory, "debug/seseragi")
  )
  return cliBuild
}

export async function runCommand(command: string[]): Promise<void> {
  const { status, stdout, stderr } = await runSampleCommand(command, {
    cwd: root,
    label: `CLI fixture ${command.join(" ")}`,
  })
  if (status !== 0) {
    throw new Error(
      `${command.join(" ")} failed (${status})\n${stdout}${stderr}`
    )
  }
}
