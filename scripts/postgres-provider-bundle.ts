import { resolve } from "node:path"

const root = resolve(import.meta.dir, "..")
const entry = resolve(root, "runtime/providers/postgres/pg.ts")
const outputPath = resolve(root, "runtime/providers/postgres/pg.bundle.js")
const result = await Bun.build({
  entrypoints: [entry],
  target: "bun",
  external: [
    "@seseragi/runtime/provider",
    "@seseragi/runtime/provider-package",
  ],
})

if (!result.success || result.outputs.length !== 1) {
  for (const log of result.logs) console.error(log)
  throw new Error("failed to bundle the PostgreSQL Provider")
}

const generated = await result.outputs[0]?.text()
if (generated === undefined) throw new Error("PostgreSQL bundle is missing")

if (process.argv.includes("--write")) {
  await Bun.write(outputPath, generated)
  console.log("Updated runtime/providers/postgres/pg.bundle.js")
} else {
  const committed = await Bun.file(outputPath).text()
  if (committed !== generated) {
    console.error(
      "PostgreSQL Provider bundle is stale; run `bun run postgres:bundle`"
    )
    process.exitCode = 1
  }
}
