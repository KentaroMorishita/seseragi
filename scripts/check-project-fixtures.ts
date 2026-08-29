import { readdir } from "node:fs/promises"
import { isAbsolute, join, relative, resolve, sep } from "node:path"

const root = resolve(import.meta.dir, "..")
const projects = join(root, "examples/spec/fixtures/projects")
const inventoryPath = join(projects, "inventory.json")
const statusPath = join(projects, "STATUS.md")
const settingsPath = join(root, ".vscode/settings.json")

const phases = new Set([
  "compile",
  "convert",
  "diagnostic",
  "run",
  "test",
  "tooling",
])
const productRunners = new Set([
  "cli-build",
  "cli-doc",
  "cli-run",
  "cli-test",
  "lsp-project",
  "project-loader",
  "wasm-project",
])
const plannedRunners = new Set([
  "planned-conformance",
  "planned-converter",
  "planned-tooling",
])

type Availability = "contract-only" | "current"
type Fixture = {
  availability: Availability
  phase: string
  runners: string[]
  evidence?: string[]
}
type Inventory = {
  schema: number
  kind: string
  fixtures: Record<string, Fixture>
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

function expectKeys(
  value: Record<string, unknown>,
  expected: readonly string[],
  label: string
): void {
  const actual = Object.keys(value).sort()
  const canonical = [...expected].sort()
  assert(
    JSON.stringify(actual) === JSON.stringify(canonical),
    `${label} keys must be ${canonical.join(", ")}; received ${actual.join(", ")}`
  )
}

async function readJson(path: string): Promise<unknown> {
  return JSON.parse(await Bun.file(path).text())
}

async function validateInventory(): Promise<Inventory> {
  const raw = (await readJson(inventoryPath)) as Inventory
  expectKeys(
    raw as unknown as Record<string, unknown>,
    ["schema", "kind", "fixtures"],
    "inventory"
  )
  assert(raw.schema === 1, "project fixture inventory schema must be 1")
  assert(
    raw.kind === "project-fixture-inventory",
    "project fixture inventory kind is invalid"
  )

  const directories = (await readdir(projects, { withFileTypes: true }))
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort()
  const names = Object.keys(raw.fixtures).sort()
  assert(
    JSON.stringify(names) === JSON.stringify(directories),
    `inventory must cover every project fixture directory\nexpected: ${directories.join(", ")}\nreceived: ${names.join(", ")}`
  )

  for (const name of names) {
    const fixture = raw.fixtures[name]
    assert(
      fixture.availability === "current" ||
        fixture.availability === "contract-only",
      `${name} has invalid availability ${fixture.availability}`
    )
    const keys =
      fixture.availability === "current"
        ? ["availability", "phase", "runners", "evidence"]
        : ["availability", "phase", "runners"]
    expectKeys(fixture as unknown as Record<string, unknown>, keys, name)
    assert(
      phases.has(fixture.phase),
      `${name} has invalid phase ${fixture.phase}`
    )
    assert(
      Array.isArray(fixture.runners) && fixture.runners.length > 0,
      `${name} must declare at least one runner`
    )
    assert(
      new Set(fixture.runners).size === fixture.runners.length,
      `${name} runner entries must be unique`
    )

    const expectationPath = join(projects, name, "project.expect.json")
    const hasExpectation = await Bun.file(expectationPath).exists()
    if (hasExpectation) {
      const expectation = (await readJson(expectationPath)) as {
        phase?: unknown
      }
      assert(
        expectation.phase === fixture.phase,
        `${name} inventory phase must match project.expect.json`
      )
    }

    if (fixture.availability === "current") {
      assert(
        fixture.runners.every((runner) => productRunners.has(runner)),
        `${name} current fixture must use only normal product runners`
      )
      assert(
        fixture.evidence !== undefined && fixture.evidence.length > 0,
        `${name} current fixture requires product-route evidence`
      )
      assert(
        new Set(fixture.evidence).size === fixture.evidence.length,
        `${name} evidence entries must be unique`
      )
      for (const evidence of fixture.evidence ?? []) {
        const path = resolve(root, evidence)
        const repositoryRelative = relative(root, path)
        assert(
          repositoryRelative !== ".." &&
            !repositoryRelative.startsWith(`..${sep}`) &&
            !isAbsolute(repositoryRelative),
          `${name} evidence escapes the repository`
        )
        assert(
          await Bun.file(path).exists(),
          `${name} evidence is missing: ${evidence}`
        )
        assert(
          (await Bun.file(path).text()).includes(name),
          `${name} evidence does not reference the fixture: ${evidence}`
        )
      }
    } else {
      assert(
        hasExpectation,
        `${name} contract-only fixture requires project.expect.json`
      )
      assert(
        fixture.runners.length === 1 && plannedRunners.has(fixture.runners[0]),
        `${name} contract-only fixture must select one planned runner`
      )
    }
  }
  return raw
}

function statusDocument(inventory: Inventory): string {
  const entries = Object.entries(inventory.fixtures)
  const sections: Array<[Availability, string]> = [
    ["current", "Current product-route fixtures"],
    ["contract-only", "Contract-only fixtures"],
  ]
  const lines = [
    "# Project fixture status",
    "",
    "このfileは `inventory.json` から生成します。directoryの存在だけを実装済みの根拠にせず、",
    "`current` は通常product routeのtest evidenceを持つfixtureだけを表します。",
    "",
  ]
  for (const [availability, heading] of sections) {
    const selected = entries.filter(
      ([, fixture]) => fixture.availability === availability
    )
    lines.push(`## ${heading} (${selected.length})`, "")
    lines.push(
      "| Fixture | Phase | Runner | Evidence |",
      "| --- | --- | --- | --- |"
    )
    for (const [name, fixture] of selected) {
      const runners = fixture.runners
        .map((runner) => `\`${runner}\``)
        .join(", ")
      const evidence =
        (fixture.evidence ?? []).map((path) => `\`${path}\``).join("<br>") ||
        "-"
      lines.push(
        `| \`${name}\` | \`${fixture.phase}\` | ${runners} | ${evidence} |`
      )
    }
    lines.push("")
  }
  lines.push(
    "## Promotion rule",
    "",
    "`contract-only` を `current` へ昇格する変更は、planned runnerを通常の CLI / LSP / project loader / WASM routeへ置き換え、",
    "fixture directoryを直接参照するtest evidenceを同じ変更で追加します。inventory checkerはevidence fileの存在とfixture参照を検証します。",
    ""
  )
  return lines.join("\n")
}

function workspaceSettings(inventory: Inventory): string {
  const contractOnly = Object.entries(inventory.fixtures)
    .filter(([, fixture]) => fixture.availability === "contract-only")
    .map(([name]) => name)
    .sort()
  const pattern = `examples/spec/fixtures/projects/{${contractOnly.join(",")}}/**/*.ssrg`
  return `${JSON.stringify(
    {
      "files.associations": { [pattern]: "plaintext" },
      "files.watcherExclude": { [pattern]: true },
    },
    null,
    2
  )}\n`
}

async function checkGenerated(path: string, expected: string): Promise<void> {
  const actual = await Bun.file(path).text()
  assert(
    actual === expected,
    `${relative(root, path)} is stale; run \`bun run fixtures:generate\``
  )
}

const write = process.argv.includes("--write")
assert(
  process.argv.length === (write ? 3 : 2),
  "usage: bun scripts/check-project-fixtures.ts [--write]"
)
const inventory = await validateInventory()
const status = statusDocument(inventory)
const settings = workspaceSettings(inventory)
if (write) {
  await Bun.write(statusPath, status)
  await Bun.write(settingsPath, settings)
} else {
  await checkGenerated(statusPath, status)
  await checkGenerated(settingsPath, settings)
}
const current = Object.values(inventory.fixtures).filter(
  (fixture) => fixture.availability === "current"
).length
console.log(
  `Validated ${Object.keys(inventory.fixtures).length} project fixture roles (${current} current, ${Object.keys(inventory.fixtures).length - current} contract-only).`
)
