import { resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")
for (const command of [
  ["node_modules/.bin/biome", "check", "apps/docs"],
  [
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
    ...["build.ts", "build.test.ts", "browser.ts", "check.ts"].map(
      (name) => `apps/docs/scripts/${name}`
    ),
  ],
]) {
  const checked = Bun.spawnSync(command, {
    cwd: root,
    stdout: "inherit",
    stderr: "inherit",
  })
  if (checked.exitCode !== 0) process.exit(checked.exitCode)
}
const result = Bun.spawnSync(["cargo", "build", "-p", "seseragi-cli"], {
  cwd: root,
  stdout: "inherit",
  stderr: "inherit",
})
if (result.exitCode !== 0) process.exit(result.exitCode)
const cli = resolve(
  root,
  process.env.CARGO_TARGET_DIR ?? "target",
  "debug",
  "seseragi"
)
const test = Bun.spawnSync(["bun", "test", "apps/docs/scripts/build.test.ts"], {
  cwd: root,
  env: { ...process.env, SESERAGI_BIN: cli },
  stdout: "inherit",
  stderr: "inherit",
})
process.exit(test.exitCode)
