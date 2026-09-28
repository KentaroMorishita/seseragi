import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import {
  closeSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import { highlightSeseragi } from "../../playground/src/editor/seseragi-language"
import { playgroundUrlForSource } from "../../playground/src/workspace/source-link"
import { compilerReferenceModules } from "./reference"

const app = resolve(import.meta.dir, "..")
const root = resolve(app, "../..")
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

function generatorInput(playgroundUrl: string) {
  return {
    schema: 1,
    origin: "",
    playgroundUrl,
    tourUrl: new URL("tour/", playgroundUrl).href,
    examples: [
      canonicalExample(
        "hello-world",
        "examples/samples/hello-world/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "function-application",
        "apps/site/examples/src/language/function-application.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-method-calls",
        "apps/site/examples/src/language/method-calls.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-methods",
        "apps/site/examples/invalid/src/language/invalid-methods.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-pipelines",
        "apps/site/examples/src/language/pipelines.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-operator-precedence",
        "apps/site/examples/src/language/operator-precedence.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-custom-operator",
        "apps/site/examples/src/language/custom-operator.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-custom-operator",
        "apps/site/examples/invalid/src/language/invalid-custom-operator.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-reserved-words",
        "apps/site/examples/src/language/reserved-words.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-optional-record-field",
        "apps/site/examples/src/language/optional-record-field.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-record-fields",
        "apps/site/examples/invalid/src/language/invalid-record-fields.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-system",
        "apps/site/examples/src/language/type-system.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-constructors",
        "apps/site/examples/src/language/type-constructors.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-polymorphism",
        "apps/site/examples/src/language/polymorphism.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-nominal-structural",
        "apps/site/examples/src/language/nominal-and-structural.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-structural",
        "apps/site/examples/invalid/src/language/invalid-structural.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-requirement-merge",
        "apps/site/examples/src/language/requirement-merge.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-requirement-merge",
        "apps/site/examples/invalid/src/language/invalid-requirement-merge.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-coercion",
        "apps/site/examples/invalid/src/language/invalid-coercion.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-recursion",
        "apps/site/examples/src/language/recursion.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-recursion",
        "apps/site/examples/invalid/src/language/invalid-recursion.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "language-program-entry",
        "apps/site/examples/src/language/program-entry.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-source-text",
        "apps/site/examples/src/language/source-text.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-literals",
        "apps/site/examples/src/language/literals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-numeric",
        "apps/site/examples/invalid/src/language/invalid-numeric.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-char-valid",
        "apps/site/examples/src/language/character-literals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-char-invalid",
        "apps/site/examples/invalid/src/language/invalid-character-literals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-escape",
        "apps/site/examples/invalid/src/language/invalid-escape.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "web-starter-main",
        "examples/samples/web-starter/src/main.ssrg",
        playgroundUrl
      ),
    ],
    referenceModules: compilerReferenceModules(),
  }
}

function routeFile(output: string, route: string): string {
  assert.match(route, /^\/(?:[a-z0-9-]+\/)*$/u)
  return join(output, route.slice(1), "index.html")
}

function validateInternalLinks(pages: RenderedPage[]) {
  const byRoute = new Map(pages.map((page) => [page.route, page.html]))
  for (const page of pages) {
    const links = page.html.matchAll(/<a\b[^>]*\bhref="([^"]+)"/gu)
    for (const match of links) {
      const href = match[1]
      if (href.startsWith("http://") || href.startsWith("https://")) continue
      const [path, fragment] = href.split("#", 2)
      const route = path || page.route
      const target = byRoute.get(route)
      assert.ok(target, `Unresolved internal link from ${page.route}: ${href}`)
      if (fragment)
        assert.ok(
          target.includes(`id="${fragment}"`),
          `Unresolved fragment from ${page.route}: ${href}`
        )
    }
  }
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
  const encodedInput = join(dirname(entry), "render-input.jsonl")
  const output = join(dirname(entry), "rendered-pages.json")
  writeFileSync(encodedInput, `${JSON.stringify(input)}\n`)
  const inputDescriptor = openSync(encodedInput, "r")
  const outputDescriptor = openSync(output, "wx")
  let result: ReturnType<typeof spawnSync>
  try {
    result = spawnSync("bun", [entry], {
      cwd: dirname(entry),
      encoding: "utf8",
      stdio: [inputDescriptor, outputDescriptor, "pipe"],
    })
  } finally {
    closeSync(inputDescriptor)
    closeSync(outputDescriptor)
    rmSync(encodedInput)
  }
  assert.equal(
    result.status,
    0,
    result.stderr?.toString() ||
      result.error?.message ||
      `Site generator failed (status ${result.status}, signal ${result.signal})`
  )
  const decoded: unknown = JSON.parse(readFileSync(output, "utf8"))
  rmSync(output)
  assert.ok(Array.isArray(decoded), "Site generator must return page records")
  return decoded as RenderedPage[]
}

function publishAssets(output: string): string[] {
  const assets = join(output, "assets")
  mkdirSync(assets, { recursive: true })
  const css = [
    readFileSync(
      join(root, "apps/playground/src/editor/syntax-theme.css"),
      "utf8"
    ).trim(),
    ...styleFiles.map((name) =>
      readFileSync(join(app, "styles", name), "utf8").trim()
    ),
  ].join("\n\n")
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
    const referencePageCount = input.referenceModules.reduce(
      (count, module) => count + 1 + module.items.length,
      0
    )
    assert.equal(
      pages.length,
      2 * (40 + referencePageCount),
      "Unexpected bilingual page count"
    )
    assert.equal(
      new Set(pages.map(({ route }) => route)).size,
      pages.length,
      "Duplicate generated route"
    )
    validateInternalLinks(pages)
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
      referenceModules: input.referenceModules.map(
        ({ specifier, availability, targets, items }) => ({
          specifier,
          availability,
          targets,
          symbols: items.map(({ identity, namespace, itemKind }) => ({
            identity,
            namespace,
            itemKind,
          })),
        })
      ),
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
