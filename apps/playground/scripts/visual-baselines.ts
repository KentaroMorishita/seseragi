import { createHash } from "node:crypto"
import { readFile, writeFile } from "node:fs/promises"
import path from "node:path"
import process from "node:process"

type Review = Readonly<{
  schema: 1
  reason: string
  snapshots: Readonly<Record<string, string>>
}>

const playground = path.resolve(import.meta.dir, "..")
const snapshotRoot = path.join(
  playground,
  "e2e",
  "web-ui-regression.spec.ts-snapshots"
)
const reviewPath = path.join(playground, "e2e", "visual-baselines.review.json")

async function snapshots(): Promise<Record<string, string>> {
  const hashes: Record<string, string> = {}
  const glob = new Bun.Glob("*.png")
  const files = Array.from(glob.scanSync({ cwd: snapshotRoot })).sort()
  for (const file of files) {
    const contents = await readFile(path.join(snapshotRoot, file))
    hashes[file] = createHash("sha256").update(contents).digest("hex")
  }
  return hashes
}

function fail(message: string): never {
  console.error(`visual baselines: ${message}`)
  process.exit(1)
}

async function check(): Promise<void> {
  const review = JSON.parse(await readFile(reviewPath, "utf8")) as Review
  const current = await snapshots()
  if (review.reason.trim() === "") fail("review reason is empty")
  if (JSON.stringify(current) !== JSON.stringify(review.snapshots)) {
    fail(
      "snapshot files do not match the reviewed manifest; " +
        'run `bun run test:visual:update -- "<reason>"`'
    )
  }
}

async function update(): Promise<void> {
  const reason = process.argv.slice(3).join(" ").trim()
  if (reason === "") {
    fail('provide a review reason: bun run test:visual:update -- "<reason>"')
  }

  const executable = path.join(playground, "node_modules", ".bin", "playwright")
  const child = Bun.spawn(
    [
      executable,
      "test",
      "--config",
      "playwright.config.ts",
      "--update-snapshots",
    ],
    {
      cwd: playground,
      stdin: "inherit",
      stdout: "inherit",
      stderr: "inherit",
    }
  )
  const exitCode = await child.exited
  if (exitCode !== 0) process.exit(exitCode)

  const review: Review = { schema: 1, reason, snapshots: await snapshots() }
  await writeFile(reviewPath, `${JSON.stringify(review, null, 2)}\n`)
  console.log(`visual baselines: recorded review reason: ${reason}`)
}

const command = process.argv[2]
if (command === "check") await check()
else if (command === "update") await update()
else fail("expected `check` or `update`")
