import { readdir, readFile, writeFile } from "node:fs/promises"
import { basename, dirname, join } from "node:path"

const root = process.cwd()
const packages = ["node_modules/@seseragi/runtime", "node_modules/seseragi"]
const entries: string[] = []
async function collect(dir: string) {
  for (const item of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, item.name)
    if (item.isDirectory()) await collect(path)
    else if (path.endsWith(".ts") && !path.endsWith(".d.ts")) entries.push(path)
  }
}
for (const pkg of packages) await collect(join(root, pkg))
const result = await Bun.build({
  entrypoints: entries,
  root,
  outdir: root,
  target: "node",
  format: "esm",
  splitting: true,
  packages: "external",
  naming: {
    entry: "[dir]/[name].js",
    chunk: "node_modules/@seseragi/runtime/.chunks/[hash].js",
  },
  sourcemap:
    process.env.SESERAGI_PROCESS_SOURCE_MAP === "emit" ? "linked" : "none",
})
if (!result.success)
  throw new AggregateError(result.logs, "runtime JS build failed")
function map(value: unknown, key = ""): unknown {
  if (Array.isArray(value)) return value.map((v) => map(v, key))
  if (value !== null && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([k, v]) => [k, map(v, k)])
    )
  return key !== "types" &&
    typeof value === "string" &&
    value.endsWith(".ts") &&
    !value.endsWith(".d.ts")
    ? `${value.slice(0, -3)}.js`
    : value
}
for (const pkg of packages) {
  const file = join(root, pkg, "package.json")
  const manifest = JSON.parse(await readFile(file, "utf8"))
  manifest.exports = map(manifest.exports)
  await writeFile(file, `${JSON.stringify(manifest, null, 2)}\n`)
}

// Generated application imports use .js extensions. Keep each module in its
// owning directory so relative foreign runtime resolution remains unchanged.
const transpiler = new Bun.Transpiler({ loader: "ts", target: "node" })
const applications: string[] = JSON.parse(
  process.env.SESERAGI_GENERATED_TYPESCRIPT ?? "[]"
)
for (const relative of applications) {
  const path = join(root, relative)
  const javascript = `${path.slice(0, -3)}.js`
  if (await Bun.file(javascript).exists())
    throw new Error(
      `Generated JavaScript output collides with staged input: ${relative}`
    )
  if (process.env.SESERAGI_PROCESS_SOURCE_MAP === "emit") {
    const result = Bun.spawnSync(
      [
        process.execPath,
        "build",
        basename(path),
        "--external=*",
        "--target=node",
        `--outfile=${basename(javascript)}`,
        "--sourcemap=linked",
      ],
      { cwd: dirname(path), stdout: "pipe", stderr: "pipe" }
    )
    if (result.exitCode !== 0)
      throw new Error(
        `Generated JavaScript emission failed: ${new TextDecoder().decode(result.stderr)}`
      )
  } else {
    await writeFile(
      javascript,
      transpiler.transformSync(await readFile(path, "utf8"))
    )
  }
}
