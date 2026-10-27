import { realpath } from "node:fs/promises"
import { relative, resolve, sep } from "node:path"
import ts from "typescript"

const root = resolve(import.meta.dir, "..")

// Bun includes resolved dependency paths in generated section comments. A
// linked node_modules directory in a dedicated worktree makes those comments
// point outside the checkout even though the executable output is identical.
// Normalize only the known dependency root, not arbitrary node_modules paths
// or executable text. Bun prints section paths relative to its process cwd.
export function normalizePostgresBundleSectionPaths(
  source: string,
  workingDirectory: string,
  dependencyRoot: string
): string {
  const prefix = `${relative(workingDirectory, dependencyRoot).split(sep).join("/")}/`
  const file = ts.createSourceFile(
    "postgres-bundle.js",
    source,
    ts.ScriptTarget.Latest,
    false,
    ts.ScriptKind.JS
  )
  const replacements: { start: number; end: number; text: string }[] = []
  // Only Bun's top-level section comments are eligible. A regexp over lines
  // would also rewrite executable text inside multiline template literals.
  for (const statement of file.statements) {
    for (const comment of ts.getLeadingCommentRanges(source, statement.pos) ??
      []) {
      if (comment.kind !== ts.SyntaxKind.SingleLineCommentTrivia) continue
      const section = /^\/\/ ([^\r\n]+)$/.exec(
        source.slice(comment.pos, comment.end)
      )?.[1]
      if (!section?.startsWith(prefix)) continue
      const suffix = section.slice(prefix.length)
      if (!suffix || suffix.split("/").some((part) => part === "..")) continue
      replacements.push({
        start: comment.pos,
        end: comment.end,
        text: `// node_modules/${suffix}`,
      })
    }
  }
  for (const replacement of replacements.reverse()) {
    source =
      source.slice(0, replacement.start) +
      replacement.text +
      source.slice(replacement.end)
  }
  return source
}

export async function buildPostgresProviderBundle(
  repository = root
): Promise<string> {
  const result = await Bun.build({
    entrypoints: [resolve(repository, "runtime/providers/postgres/pg.ts")],
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
  return normalizePostgresBundleSectionPaths(
    generated,
    process.cwd(),
    await realpath(resolve(repository, "node_modules"))
  )
}

export async function checkPostgresProviderBundle(
  repository = root
): Promise<void> {
  const generated = await buildPostgresProviderBundle(repository)
  const committed = await Bun.file(
    resolve(repository, "runtime/providers/postgres/pg.bundle.js")
  ).text()
  if (committed !== generated) {
    throw new Error(
      "PostgreSQL Provider bundle is stale; run `bun run postgres:bundle`"
    )
  }
}

if (import.meta.main) {
  try {
    if (process.argv.includes("--write")) {
      await Bun.write(
        resolve(root, "runtime/providers/postgres/pg.bundle.js"),
        await buildPostgresProviderBundle()
      )
      console.log("Updated runtime/providers/postgres/pg.bundle.js")
    } else {
      await checkPostgresProviderBundle()
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  }
}
