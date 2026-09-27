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
    examples: [
      canonicalExample(
        "hello-world",
        "examples/samples/hello-world/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "function-application",
        "examples/spec/lessons/02-values-and-functions.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-method-calls",
        "examples/spec/lessons/11-generics.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-methods",
        "examples/spec/artifacts/semantic-diagnostics-schema-1/inherent-method-invalid/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-pipelines",
        "examples/spec/lessons/04-collections-and-pipelines.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-operator-precedence",
        "examples/spec/fixtures/compile/grammar-terminal-coverage.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-custom-operator",
        "examples/spec/lessons/13-traits-and-custom-operators.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-custom-operator",
        "examples/spec/artifacts/semantic-diagnostics-schema-1/custom-operator-invalid-declaration/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-reserved-words",
        "examples/spec/fixtures/compile/grammar-terminal-coverage.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-optional-record-field",
        "examples/spec/fixtures/compile/optional-record-field.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-record-fields",
        "examples/spec/artifacts/semantic-diagnostics-schema-1/record-field-errors/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-system",
        "examples/spec/lessons/07-domain-types.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-constructors",
        "examples/spec/fixtures/compile/generic-struct.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-polymorphism",
        "examples/spec/fixtures/projects/local-rec/src/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-nominal-structural",
        "examples/spec/lessons/19-recursive-data-and-spread.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-structural",
        "examples/spec/artifacts/semantic-diagnostics-schema-1/structured-type-differences/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-requirement-merge",
        "examples/spec/fixtures/compile/requirement-merge.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-requirement-merge",
        "examples/spec/fixtures/diagnostics/invalid-requirement-merge.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-coercion",
        "examples/spec/artifacts/semantic-diagnostics-schema-1/newtype-no-coercion/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-recursion",
        "examples/spec/fixtures/compile/local-recursion.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "types-invalid-recursion",
        "examples/spec/fixtures/diagnostics/local-forward-reference.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "language-program-entry",
        "examples/spec/lessons/01-hello-world.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-source-text",
        "examples/spec/fixtures/projects/char-literal/src/domain.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-literals",
        "examples/spec/fixtures/compile/literals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-numeric",
        "examples/spec/fixtures/diagnostics/invalid-numeric-literal.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-char-valid",
        "examples/spec/fixtures/projects/char-literal/src/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-char-invalid",
        "examples/spec/artifacts/schema-1/char-literal-invalid/main.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "syntax-invalid-escape",
        "examples/spec/fixtures/diagnostics/invalid-escape.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "test-discovery",
        "examples/spec/fixtures/projects/test-discovery/tests/basic.ssrg",
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
      2 * (51 + referencePageCount),
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
