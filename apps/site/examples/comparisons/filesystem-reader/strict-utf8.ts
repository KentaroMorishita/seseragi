import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"

const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true })

async function describe(file: string): Promise<string> {
  let data: Uint8Array
  try {
    data = await readFile(file)
  } catch (error) {
    if (
      !(error instanceof Error) ||
      !("code" in error) ||
      error.code !== "ENOENT"
    ) {
      throw error
    }
    return "access: ENOENT"
  }
  try {
    return `decoded: ${decoder.decode(data)}`
  } catch (error) {
    if (!(error instanceof TypeError)) throw error
    return "decode: TypeError (no byte offset)"
  }
}

const directory = await mkdtemp(join(tmpdir(), "fs16-ts-utf8-"))
try {
  const file = join(directory, "report.txt")
  console.log(await describe(file))
  await writeFile(file, new Uint8Array([65, 195, 40, 66]), { flag: "wx" })
  console.log(await describe(file))
  await writeFile(file, "\uFEFFA", { flag: "w" })
  const text = decoder.decode(await readFile(file))
  console.log(`BOM preserved: ${text === "\uFEFFA"}`)
} finally {
  await rm(directory, { recursive: true, force: true })
}
