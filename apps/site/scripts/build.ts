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
  playgroundUrl: string,
  standalone = true
) {
  const source = readFileSync(join(root, sourcePath), "utf8")
  return {
    id,
    sourcePath,
    source,
    sha256: sha256(source),
    playgroundUrl: standalone
      ? playgroundUrlForSource(playgroundUrl, source)
      : "",
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
        "modules-domain",
        "apps/site/examples/src/language/modules-domain.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-identity",
        "apps/site/examples/src/language/modules-identity.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "modules-imports",
        "apps/site/examples/src/language/modules-imports.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-top-level",
        "apps/site/examples/src/language/modules-top-level.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-private-domain",
        "apps/site/examples/invalid/visibility/src/domain.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "modules-invalid-private-import",
        "apps/site/examples/invalid/visibility/src/main.ssrg",
        playgroundUrl,
        false
      ),
      canonicalExample(
        "modules-invalid-import",
        "apps/site/examples/invalid/import/src/main.ssrg",
        playgroundUrl
      ),
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
        "expressions-evaluation",
        "apps/site/examples/src/language/expressions-evaluation.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-blocks",
        "apps/site/examples/src/language/expressions-blocks.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-conditionals",
        "apps/site/examples/src/language/expressions-conditionals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-invalid-conditional",
        "apps/site/examples/invalid/src/language/invalid-conditional.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-ranges-comprehensions",
        "apps/site/examples/src/language/expressions-ranges-comprehensions.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "expressions-lambdas",
        "apps/site/examples/src/language/expressions-lambdas.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-algebraic-data-types",
        "apps/site/examples/src/language/data-algebraic-data-types.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-structs",
        "apps/site/examples/src/language/data-structs.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-invalid-struct",
        "apps/site/examples/invalid/src/language/invalid-struct-literal.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-newtypes",
        "apps/site/examples/src/language/data-newtypes.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-records",
        "apps/site/examples/src/language/data-records.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-invalid-record",
        "apps/site/examples/invalid/src/language/invalid-record-literal.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-tuples-arrays-lists",
        "apps/site/examples/src/language/data-tuples-arrays-lists.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-impls-and-methods",
        "apps/site/examples/src/language/data-impls-and-methods.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-invalid-impl-owner",
        "apps/site/examples/invalid/src/language/invalid-impl-owner.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-operator-overloads",
        "apps/site/examples/src/language/data-operator-overloads.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "data-invalid-operator-overload",
        "apps/site/examples/invalid/src/language/invalid-operator-overload.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-binding-rules",
        "apps/site/examples/src/language/patterns-binding-rules.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-invalid-binding",
        "apps/site/examples/invalid/src/language/invalid-binding-pattern.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-irrefutable",
        "apps/site/examples/src/language/patterns-irrefutable.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-invalid-irrefutable",
        "apps/site/examples/invalid/src/language/invalid-irrefutable-pattern.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-match",
        "apps/site/examples/src/language/patterns-match.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "patterns-invalid-match",
        "apps/site/examples/invalid/src/language/invalid-match.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-model",
        "apps/site/examples/src/language/traits-model.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-declarations",
        "apps/site/examples/src/language/traits-declarations.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-invalid-declaration",
        "apps/site/examples/invalid/src/language/invalid-trait-declaration.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-instances",
        "apps/site/examples/src/language/traits-instances.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-invalid-instance",
        "apps/site/examples/invalid/src/language/invalid-trait-instance.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-constraints",
        "apps/site/examples/src/language/traits-constraints.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-invalid-constraint",
        "apps/site/examples/invalid/src/language/invalid-trait-constraint.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-method-calls",
        "apps/site/examples/src/language/traits-method-calls.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-invalid-method",
        "apps/site/examples/invalid/src/language/invalid-trait-method.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-coherence",
        "apps/site/examples/src/language/traits-coherence.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-invalid-coherence",
        "apps/site/examples/invalid/src/language/invalid-trait-coherence.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-standard-operators",
        "apps/site/examples/src/language/traits-standard-operators.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-laws",
        "apps/site/examples/src/language/traits-laws.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-deriving",
        "apps/site/examples/src/language/traits-deriving.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-invalid-deriving",
        "apps/site/examples/invalid/src/language/invalid-trait-deriving.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-methods-versus-traits",
        "apps/site/examples/src/language/traits-methods-versus-traits.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-do-notation",
        "apps/site/examples/src/language/traits-do-notation.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-do-desugaring",
        "apps/site/examples/src/language/traits-do-desugaring.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-do-block-typing",
        "apps/site/examples/src/language/traits-do-block-typing.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "traits-invalid-do",
        "apps/site/examples/invalid/src/language/invalid-trait-do.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-pure-expressions",
        "apps/site/examples/src/language/effects-pure-expressions.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-maybe",
        "apps/site/examples/src/language/effects-maybe.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-either",
        "apps/site/examples/src/language/effects-either.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-cold-value",
        "apps/site/examples/src/language/effects-cold-value.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-contract-form",
        "apps/site/examples/src/language/effects-contract-form.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-inferred-form",
        "apps/site/examples/src/language/effects-inferred-form.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-effectful-for",
        "apps/site/examples/src/language/effects-effectful-for.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-environment",
        "apps/site/examples/src/language/effects-environment.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-error-channels",
        "apps/site/examples/src/language/effects-error-channels.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-task",
        "apps/site/examples/src/language/effects-task.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-execution-order",
        "apps/site/examples/src/language/effects-execution-order.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-runtime-boundary",
        "apps/site/examples/src/language/effects-runtime-boundary.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-defects",
        "apps/site/examples/src/language/effects-defects.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-pure-body",
        "apps/site/examples/invalid/src/language/invalid-effect-pure-body.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-maybe",
        "apps/site/examples/invalid/src/language/invalid-effect-maybe.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-contract",
        "apps/site/examples/invalid/src/language/invalid-effect-contract.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-errors",
        "apps/site/examples/invalid/src/language/invalid-effect-errors.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-task",
        "apps/site/examples/invalid/src/language/invalid-effect-task.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-cancellation-resources",
        "apps/site/examples/src/language/effects-cancellation-resources.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-scheduler-fairness",
        "apps/site/examples/src/language/effects-scheduler-fairness.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-fiber-supervision",
        "apps/site/examples/src/language/effects-fiber-supervision.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-signal-transactions",
        "apps/site/examples/src/language/effects-signal-transactions.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-derived-signals",
        "apps/site/examples/src/language/effects-derived-signals.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-signal-operators",
        "apps/site/examples/src/language/effects-signal-operators.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-signal-subscription",
        "apps/site/examples/src/language/effects-signal-subscription.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-foreign-failure",
        "apps/site/examples/src/language/effects-foreign-failure.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-finalizer",
        "apps/site/examples/invalid/src/language/invalid-effect-finalizer.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-signal-read",
        "apps/site/examples/invalid/src/language/invalid-signal-read.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-signal-distinct",
        "apps/site/examples/invalid/src/language/invalid-signal-distinct.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-signal-write",
        "apps/site/examples/invalid/src/language/invalid-signal-write.ssrg",
        playgroundUrl
      ),
      canonicalExample(
        "effects-invalid-throw",
        "apps/site/examples/invalid/src/language/invalid-effect-throw.ssrg",
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
      2 * (94 + referencePageCount),
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
