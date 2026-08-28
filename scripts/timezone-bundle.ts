import { resolve } from "node:path"

const root = resolve(import.meta.dir, "..")
const target = resolve(root, "runtime/ts/src/timezone-rules.js")
const result = await Bun.build({
  entrypoints: [resolve(root, "runtime/timezones/rules.ts")],
  format: "esm",
  minify: true,
  target: "browser",
  write: false,
})
if (!result.success || result.outputs.length !== 1) {
  for (const log of result.logs) console.error(log)
  throw new Error("failed to build the pinned timezone rules bundle")
}
const contents = await result.outputs[0]?.text()
if (contents === undefined || !contents.includes("2025b")) {
  throw new Error("timezone rules bundle does not contain tzdb 2025b")
}
if (process.argv.includes("--write")) {
  await Bun.write(target, contents)
  console.log("updated runtime/ts/src/timezone-rules.js (tzdb 2025b)")
} else {
  const current = await Bun.file(target).text()
  if (current !== contents) {
    throw new Error(
      "timezone rules bundle is stale; run `bun run timezones:bundle`"
    )
  }
  console.log("timezone rules bundle is current (tzdb 2025b)")
}
