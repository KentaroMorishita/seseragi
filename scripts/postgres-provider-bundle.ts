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

const rawGenerated = await result.outputs[0]?.text()
if (rawGenerated === undefined) throw new Error("PostgreSQL bundle is missing")

// Bun includes resolved dependency paths in generated section comments. A
// linked node_modules directory in a dedicated worktree makes those comments
// point outside the checkout even though the executable output is identical.
// Keep the committed bundle reproducible across regular and linked installs.
const generated = rawGenerated.replace(
  /^\/\/ (?:\.\.\/[^/\n]+\/)+node_modules\//gm,
  "// node_modules/"
)

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
