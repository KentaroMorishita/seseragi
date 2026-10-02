import { access, mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"

const directory = await mkdtemp(join(tmpdir(), "fs16-ts-modes-"))
try {
  const file = join(directory, "report.txt")
  const before = await access(file).then(
    () => true,
    () => false
  )
  console.log(`before: ${before}`)
  await writeFile(file, "original", { flag: "wx" })
  try {
    await writeFile(file, "replacement", { flag: "wx" })
    console.log("unexpected success")
  } catch (error) {
    if (
      !(error instanceof Error) ||
      !("code" in error) ||
      error.code !== "EEXIST"
    ) {
      throw error
    }
    console.log("duplicate: EEXIST")
  }
  console.log(`original: ${await readFile(file, "utf8")}`)
  await writeFile(file, "OK", { flag: "w" })
  await writeFile(file, "!", { flag: "a" })
  console.log(`updated: ${await readFile(file, "utf8")}`)
} finally {
  await rm(directory, { recursive: true, force: true })
}
