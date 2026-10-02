import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"

async function withWorkspace(
  action: (directory: string) => Promise<string>
): Promise<string> {
  const directory = await mkdtemp(join(tmpdir(), "fs16-ts-workspace-"))
  try {
    return await action(directory)
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}

const text = await withWorkspace(async (directory) => {
  const file = join(directory, "report.txt")
  await writeFile(file, "ready", { flag: "wx" })
  return await readFile(file, "utf8")
})
console.log(`success: ${text}`)

try {
  await withWorkspace(async () => {
    throw new Error("report rejected")
  })
} catch (error) {
  if (!(error instanceof Error)) throw error
  console.log(`callback failure: ${error.message}`)
}
