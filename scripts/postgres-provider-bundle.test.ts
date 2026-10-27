import { expect, test } from "bun:test"
import {
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import { pathToFileURL } from "node:url"
import { runNativeReleaseCommand } from "./native-release-process"
import { normalizePostgresBundleSectionPaths } from "./postgres-provider-bundle"

test("normalizes only the resolved dependency section prefix at every worktree depth", () => {
  for (const depth of [0, 1, 4, 8]) {
    const working = path.resolve("fixture", ...Array(depth).fill("worktree"))
    const dependency = path.resolve("dependencies/node_modules")
    const prefix = path.relative(working, dependency).split(path.sep).join("/")
    const section = `${prefix}/pg/lib/client.js`
    const source = `// ${section}\nconst path = "${section}";\nconst template = \`${section}\`;\n/* ${section} */\n// unrelated/node_modules/pg/lib/client.js\n// ${prefix}/../other.js\n`
    expect(
      normalizePostgresBundleSectionPaths(source, working, dependency)
    ).toBe(
      source.replace(`// ${section}\n`, "// node_modules/pg/lib/client.js\n")
    )
  }
})

test("keeps a regular dependency section and all executable bytes unchanged", () => {
  const root = path.resolve("fixture")
  const source = `// node_modules/pg/lib/client.js\nconst value = "node_modules/pg/lib/client.js";\n`
  expect(
    normalizePostgresBundleSectionPaths(
      source,
      root,
      path.join(root, "node_modules")
    )
  ).toBe(source)
})

test("leaves section-like text inside templates, expressions and nested comments intact", () => {
  const working = path.resolve("worktrees/deep/repository")
  const dependency = path.resolve("installed/node_modules")
  const section = `${path.relative(working, dependency).split(path.sep).join("/")}/pg/index.js`
  const source = `// ${section}\nconst template = \`\n// ${section}\nvar value = 1;\n\`;\nconst expression = \`\n\${(() => {\n// ${section}\nreturn 2;\n})()}\n\`;\nfunction nested() {\n// ${section}\nreturn /[\\/]/.test("/");\n}\n/*\n// ${section}\n*/\n`
  expect(normalizePostgresBundleSectionPaths(source, working, dependency)).toBe(
    source.replace(`// ${section}\n`, "// node_modules/pg/index.js\n")
  )
})

test("real Bun bundles agree for regular and deep linked dependencies and reject changed code", async () => {
  const temporary = await realpath(
    await mkdtemp(path.join(tmpdir(), "seseragi-postgres-bundle-test-"))
  )
  const normal = path.join(temporary, "installed")
  const linked = path.join(
    temporary,
    "worktrees",
    "one",
    "two",
    "three",
    "linked"
  )
  const sourcePath = "runtime/providers/postgres/pg.ts"
  const bundlePath = "runtime/providers/postgres/pg.bundle.js"
  const implementation = pathToFileURL(
    path.join(import.meta.dir, "postgres-provider-bundle.ts")
  ).href
  const execute = async (
    directory: string,
    check: boolean
  ): Promise<string> => {
    const script = check
      ? `import { checkPostgresProviderBundle } from ${JSON.stringify(implementation)}; await checkPostgresProviderBundle(process.cwd());`
      : `import { buildPostgresProviderBundle } from ${JSON.stringify(implementation)}; console.log(JSON.stringify(await buildPostgresProviderBundle(process.cwd())));`
    return (
      await runNativeReleaseCommand([process.execPath, "-e", script], directory)
    ).stdout.trim()
  }
  try {
    for (const directory of [normal, linked]) {
      await mkdir(path.dirname(path.join(directory, sourcePath)), {
        recursive: true,
      })
      await writeFile(
        path.join(directory, sourcePath),
        'export { value } from "pg-fixture"\n'
      )
    }
    const dependency = path.join(normal, "node_modules", "pg-fixture")
    await mkdir(dependency, { recursive: true })
    await writeFile(
      path.join(dependency, "package.json"),
      JSON.stringify({
        name: "pg-fixture",
        version: "1.0.0",
        type: "module",
        main: "index.js",
      })
    )
    await writeFile(
      path.join(dependency, "index.js"),
      "export const value = 7\n"
    )
    await symlink(
      path.join(normal, "node_modules"),
      path.join(linked, "node_modules"),
      "junction"
    )
    const committed = JSON.parse(await execute(normal, false)) as string
    expect(JSON.parse(await execute(linked, false))).toBe(committed)
    expect(committed).toContain("// node_modules/pg-fixture/index.js")
    for (const directory of [normal, linked]) {
      await writeFile(path.join(directory, bundlePath), committed)
      await execute(directory, true)
    }

    await writeFile(
      path.join(dependency, "index.js"),
      "export const value = 9\n"
    )
    const changed = JSON.parse(await execute(linked, false)) as string
    expect(changed).not.toBe(committed)
    expect(changed).toContain("var value = 9;")
    await expect(execute(linked, true)).rejects.toThrow(
      "PostgreSQL Provider bundle is stale"
    )
    expect(await readFile(path.join(linked, bundlePath), "utf8")).toBe(
      committed
    )
  } finally {
    await rm(temporary, { recursive: true, force: true })
  }
})
