import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import { highlightSeseragi } from "../../playground/src/editor/seseragi-language"
import { playgroundUrlForSource } from "../../playground/src/workspace/source-link"

const app = resolve(import.meta.dir, "..")
const root = resolve(app, "../..")
const referencePath = join(
  root,
  "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
)
const styleFiles = [
  "tokens.css",
  "base.css",
  "shell.css",
  "home.css",
  "docs.css",
  "article.css",
  "code.css",
  "responsive.css",
]

type BuildOptions = {
  output: string
  origin: string
  playgroundUrl?: string
  profile?: "development" | "release"
}

type RenderedPage = {
  route: string
  html: string
}

const sha256 = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")

function canonicalExample(
  id: string,
  sourcePath: string,
  playgroundUrl: string
) {
  const source = readFileSync(join(root, sourcePath), "utf8")
  return {
    id,
    sourcePath,
    source,
    sha256: sha256(source),
    playgroundUrl: playgroundUrlForSource(playgroundUrl, source),
    highlighted: highlightSeseragi(source).map(({ text, classes }) => ({
      text,
      className: classes,
    })),
  }
}

function compilerReference(identity: string) {
  const artifact = JSON.parse(readFileSync(referencePath, "utf8"))
  assert.equal(artifact.schema, 1)
  const item = artifact.modules
    .flatMap((module: { items: unknown[] }) => module.items)
    .find((entry: { identity: string }) => entry.identity === identity)
  assert.ok(item, `Missing compiler reference: ${identity}`)
  return {
    identity: item.identity,
    name: item.name,
    namespace: item.namespace,
    itemKind: item.kind,
    signature: item.signature,
    description: item.description,
    typeParameters: item.typeParameters,
    constraints: item.constraints,
    highlighted: highlightSeseragi(item.signature).map(({ text, classes }) => ({
      text,
      className: classes,
    })),
  }
}

function generatorInput(playgroundUrl: string) {
  return {
    schema: 1,
    origin: "",
    playgroundUrl,
    examples: [
      canonicalExample(
        "hello-world",
        "examples/spec/lessons/01-hello-world.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "function-application",
        "examples/spec/lessons/02-values-and-functions.ssrg",
        playgroundUrl
      ),
    ],
    references: [compilerReference("std/array::get")],
  }
}

function routeFile(output: string, route: string): string {
  assert.match(route, /^\/(?:[a-z0-9-]+\/)*$/u)
  return join(output, route.slice(1), "index.html")
}

function compileGenerator(
  directory: string,
  profile: "development" | "release"
): string {
  const cli =
    process.env.SESERAGI_BIN ?? join(root, "target", "debug", "seseragi")
  const result = spawnSync(
    cli,
    [
      "build",
      app,
      "--target",
      "process",
      "--profile",
      profile,
      "--out-dir",
      directory,
    ],
    { cwd: root, encoding: "utf8" }
  )
  assert.equal(result.status, 0, result.stderr || result.stdout)
  const entry = join(directory, profile === "release" ? "entry.js" : "entry.ts")
  assert.ok(existsSync(entry), `Missing ${profile} generator entry: ${entry}`)
  return entry
}

function renderGenerator(entry: string, input: object): RenderedPage[] {
  const result = spawnSync("bun", [entry], {
    cwd: dirname(entry),
    encoding: "utf8",
    input: `${JSON.stringify(input)}\n`,
  })
  assert.equal(result.status, 0, result.stderr || result.stdout)
  const decoded: unknown = JSON.parse(result.stdout)
  assert.ok(Array.isArray(decoded), "Site generator must return page records")
  return decoded as RenderedPage[]
}

function publishAssets(output: string): string[] {
  const assets = join(output, "assets")
  mkdirSync(assets, { recursive: true })
  const css = styleFiles
    .map((name) => readFileSync(join(app, "styles", name), "utf8").trim())
    .join("\n\n")
  writeFileSync(join(assets, "site.css"), `${css}\n`)
  copyFileSync(
    join(root, "assets/brand/public/brand/seseragi-icon.svg"),
    join(assets, "seseragi-icon.svg")
  )
  return ["assets/seseragi-icon.svg", "assets/site.css"]
}

export function buildSite(options: BuildOptions) {
  const output = resolve(options.output)
  assert.ok(!existsSync(output), `Output already exists: ${output}`)
  const origin = new URL(options.origin)
  assert.equal(origin.pathname, "/", "Site origin must not contain a path")
  assert.equal(origin.protocol, "https:", "Site origin must use HTTPS")
  const playgroundUrl = options.playgroundUrl ?? "https://seseragi.vercel.app/"
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-site-"))
  try {
    const generator = join(temporary, "generator")
    const entry = compileGenerator(generator, options.profile ?? "development")
    const input = { ...generatorInput(playgroundUrl), origin: origin.origin }
    const pages = renderGenerator(entry, input)
    assert.equal(pages.length, 16, "Expected eight bilingual page definitions")
    assert.equal(
      new Set(pages.map(({ route }) => route)).size,
      pages.length,
      "Duplicate generated route"
    )
    mkdirSync(output, { recursive: true })
    for (const page of pages) {
      assert.ok(!page.html.includes("site-build-error"), page.route)
      const path = routeFile(output, page.route)
      mkdirSync(dirname(path), { recursive: true })
      writeFileSync(path, page.html)
    }
    const assets = publishAssets(output)
    const files = [
      ...pages.map(({ route }) =>
        route === "/" ? "index.html" : `${route.slice(1)}index.html`
      ),
      ...assets,
    ].sort()
    const manifest = {
      schema: 1,
      generator: "seseragi/official-site",
      pages: pages.map(({ route }) => route).sort(),
      examples: input.examples.map(({ id, sourcePath, sha256 }) => ({
        id,
        sourcePath,
        sha256,
      })),
      references: input.references.map(({ identity }) => identity),
      files: files.map((path) => ({
        path,
        sha256: sha256(readFileSync(join(output, path))),
      })),
    }
    writeFileSync(
      join(output, "site-manifest.json"),
      `${JSON.stringify(manifest, null, 2)}\n`
    )
    return manifest
  } catch (error) {
    rmSync(output, { recursive: true, force: true })
    throw error
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
}

if (import.meta.main) {
  const [output, originArgument] = process.argv.slice(2)
  const origin =
    originArgument ??
    process.env.SESERAGI_SITE_ORIGIN ??
    "https://seseragi-docs.vercel.app"
  assert.ok(output, "Usage: build.ts OUTPUT [ORIGIN]")
  const finalOutput = resolve(output)
  const stagingOutput = `${finalOutput}.staging-${process.pid}`
  try {
    const manifest = buildSite({
      output: stagingOutput,
      origin,
      profile:
        process.env.NODE_ENV === "production" ? "release" : "development",
    })
    rmSync(finalOutput, { recursive: true, force: true })
    renameSync(stagingOutput, finalOutput)
    console.log(
      JSON.stringify(
        { output: finalOutput, pages: manifest.pages.length },
        null,
        2
      )
    )
  } catch (error) {
    rmSync(stagingOutput, { recursive: true, force: true })
    throw error
  }
}
