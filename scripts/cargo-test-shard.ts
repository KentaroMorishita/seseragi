// Runs one shard of a `cargo test` invocation so CI can split a long suite
// across parallel jobs. Every test of the selected targets runs in exactly
// one shard, and every job computes the same split on its own.
//
//   bun scripts/cargo-test-shard.ts 2/4 --locked -p seseragi-cli --test a --test b
import { spawnSync } from "node:child_process"

export type Shard = { index: number; count: number }

export type TestTarget = { name: string; tests: string[] }

export const parseShard = (text: string): Shard => {
  const match = /^(\d+)\/(\d+)$/.exec(text)
  const index = Number(match?.[1])
  const count = Number(match?.[2])
  if (!match || count < 1 || index < 1 || index > count) {
    throw new Error(`shard must be <index>/<count> with 1 <= index <= count`)
  }
  return { index, count }
}

/** Test names from `--list --format terse` output. */
export const parseTestList = (output: string): string[] =>
  output
    .split(/\r?\n/)
    .filter((line) => line.endsWith(": test"))
    .map((line) => line.slice(0, -": test".length))

/** Executables of the test targets in a `--message-format=json` stream. */
export const testExecutables = (
  output: string
): { name: string; executable: string }[] => {
  const found = new Map<string, string>()
  for (const line of output.split(/\r?\n/)) {
    if (!line.startsWith("{")) continue
    const message = JSON.parse(line) as {
      reason?: string
      executable?: string | null
      profile?: { test?: boolean }
      target?: { name?: string; kind?: string[] }
    }
    if (
      message.reason === "compiler-artifact" &&
      message.profile?.test === true &&
      message.executable &&
      message.target?.name
    ) {
      found.set(
        `${message.target.kind?.join(",")}:${message.target.name}`,
        message.executable
      )
    }
  }
  return [...found]
    .sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))
    .map(([name, executable]) => ({ name, executable }))
}

/**
 * The test names for one shard. Tests are dealt in turn across the shards,
 * target by target with the turn carried over, so each shard gets a similar
 * share of every target. libtest filters apply to every target, so a name
 * that several targets share is dealt once and runs in all of them.
 */
export const shardTests = (targets: TestTarget[], shard: Shard): string[] => {
  const seen = new Set<string>()
  const selected: string[] = []
  let turn = 0
  for (const target of targets) {
    for (const test of [...new Set(target.tests)].sort()) {
      if (seen.has(test)) continue
      seen.add(test)
      if (turn % shard.count === shard.index - 1) selected.push(test)
      turn += 1
    }
  }
  return selected
}

const run = (
  command: string,
  args: string[],
  capture: boolean
): { status: number; stdout: string } => {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
    stdio: capture ? ["ignore", "pipe", "inherit"] : "inherit",
  })
  if (result.error) throw result.error
  return { status: result.status ?? 1, stdout: result.stdout ?? "" }
}

const main = (argv: string[]): number => {
  const [shardText, ...cargoArgs] = argv
  if (!shardText || cargoArgs.length === 0) {
    throw new Error(
      "usage: bun scripts/cargo-test-shard.ts <index>/<count> <cargo test arguments>"
    )
  }
  if (cargoArgs.includes("--")) {
    throw new Error("test binary arguments are chosen by the shard")
  }
  const shard = parseShard(shardText)
  const build = run(
    "cargo",
    [
      "test",
      ...cargoArgs,
      "--no-run",
      "--message-format=json-render-diagnostics",
    ],
    true
  )
  if (build.status !== 0) return build.status
  const targets = testExecutables(build.stdout).map(({ name, executable }) => {
    const listed = run(executable, ["--list", "--format", "terse"], true)
    if (listed.status !== 0) {
      throw new Error(`could not list the tests of ${name}`)
    }
    return { name, tests: parseTestList(listed.stdout) }
  })
  const total = new Set(targets.flatMap((target) => target.tests)).size
  if (total === 0) throw new Error("no tests found; nothing was executed")
  const selected = shardTests(targets, shard)
  console.log(
    `Shard ${shard.index}/${shard.count}: ${selected.length} of ${total} tests from ${targets.length} targets`
  )
  if (selected.length === 0) return 0
  return run(
    "cargo",
    ["test", ...cargoArgs, "--", "--exact", ...selected],
    false
  ).status
}

if (import.meta.main) {
  process.exitCode = main(process.argv.slice(2))
}
