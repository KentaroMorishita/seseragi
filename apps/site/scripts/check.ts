import assert from "node:assert/strict"
import { mkdirSync, rmSync, writeFileSync } from "node:fs"
import { resolve } from "node:path"
import { buildSite } from "./build"
import { verifyPublication } from "./verify-publication"

const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/release/seseragi")
// The second build runs in this process, rather than through run(). Use the
// same optimized CLI in both processes, including on a fresh CI runner.
process.env.SESERAGI_BIN = cli
const output = resolve(root, process.env.SITE_OUTPUT ?? "target/site")

function run(command: string[]) {
  const result = Bun.spawnSync(command, {
    cwd: root,
    env: {
      ...process.env,
      CARGO_INCREMENTAL: "0",
      NODE_ENV: "production",
      SESERAGI_BIN: cli,
      SITE_OUTPUT: output,
      SITE_SCREENSHOTS: resolve(root, "target/site-verification/screenshots"),
    },
    stdout: "inherit",
    stderr: "inherit",
  })
  assert.equal(result.exitCode, 0, command.join(" "))
}

run(["node_modules/.bin/biome", "check", "apps/site"])
run([
  "node_modules/.bin/tsc",
  "--noEmit",
  "--skipLibCheck",
  "--types",
  "bun",
  "--target",
  "ES2022",
  "--module",
  "Preserve",
  "--moduleResolution",
  "Bundler",
  "apps/site/scripts/check.ts",
  "apps/site/tests/reboot-browser.ts",
  "apps/site/tests/render-generator.test.ts",
  "apps/site/tests/static-site-handler.test.ts",
])
run(["cargo", "build", "--locked", "--release", "-p", "seseragi-cli"])
// Check compiler metadata at its existing conformance boundary.
run([
  "cargo",
  "test",
  "--locked",
  "-p",
  "seseragi-conformance",
  "stdlib_surface::tests::canonical_",
])
run(["bun", "apps/site/scripts/check-published-examples.ts"])
run(["python3", "-B", "apps/site/tests/profile-process.test.py"])
run([
  "bun",
  "test",
  "apps/site/tests/static-site-handler.test.ts",
  "apps/site/tests/render-generator.test.ts",
])
run(["bun", "apps/site/scripts/build.ts", output])
const manifest = verifyPublication(output)
const duplicateOutput = `${output}-determinism-${process.pid}`
const reports = resolve(root, "target/site-verification")
mkdirSync(reports, { recursive: true })
writeFileSync(
  `${reports}/manifest-first.json`,
  `${JSON.stringify(manifest, null, 2)}\n`
)
try {
  const duplicate = buildSite({
    output: duplicateOutput,
    origin: "https://seseragi-docs.vercel.app",
    profile: "release",
  })
  writeFileSync(
    `${reports}/manifest-second.json`,
    `${JSON.stringify(duplicate, null, 2)}\n`
  )
  assert.deepEqual(
    duplicate,
    manifest,
    "Two complete builds must have identical routes and artifact hashes"
  )
} finally {
  rmSync(duplicateOutput, { recursive: true, force: true })
}
run(["bun", "apps/site/tests/reboot-browser.ts"])
console.info(
  "Docs Reboot: publication, execution, compiler metadata, determinism, and browser checks passed"
)
