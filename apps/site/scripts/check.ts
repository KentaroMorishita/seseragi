import { resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")

for (const command of [
  ["node_modules/.bin/biome", "check", "apps/site"],
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
    "apps/site/scripts/build.ts",
    "apps/site/scripts/check-examples.ts",
    "apps/site/scripts/check-content-map.ts",
    "apps/site/tests/build.test.ts",
    "apps/site/tests/browser.test.ts",
    "apps/site/tests/deployment.test.ts",
  ],
  ["cargo", "build", "-p", "seseragi-cli"],
  ["bun", "apps/site/scripts/check-examples.ts"],
  [
    "cargo",
    "test",
    "-p",
    "seseragi-conformance",
    "stdlib_surface::tests::canonical_",
  ],
]) {
  const result = Bun.spawnSync(command, {
    cwd: root,
    stdout: "inherit",
    stderr: "inherit",
  })
  if (result.exitCode !== 0) process.exit(result.exitCode)
}

const contentMap = Bun.spawnSync(
  ["bun", "apps/site/scripts/check-content-map.ts"],
  {
    cwd: root,
    stdout: "inherit",
    stderr: "inherit",
  }
)
if (contentMap.exitCode !== 0) process.exit(contentMap.exitCode)

const test = Bun.spawnSync(
  [
    "bun",
    "test",
    "apps/site/tests/build.test.ts",
    "apps/site/tests/deployment.test.ts",
  ],
  {
    cwd: root,
    env: {
      ...process.env,
      SESERAGI_BIN: resolve(root, "target/debug/seseragi"),
    },
    stdout: "inherit",
    stderr: "inherit",
  }
)
if (test.exitCode !== 0) process.exit(test.exitCode)

const browser = Bun.spawnSync(["bun", "apps/site/tests/browser.test.ts"], {
  cwd: root,
  env: {
    ...process.env,
    SESERAGI_BIN: resolve(root, "target/debug/seseragi"),
  },
  stdout: "inherit",
  stderr: "inherit",
})
process.exit(browser.exitCode)
