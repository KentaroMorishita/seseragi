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
import { canonicalExample } from "./canonical-example"
import { chapterExamples } from "./function-chapter-examples"
import { sitePhase } from "./profile"
import { articleExecutions } from "./published-examples"
import { compilerReferenceModules } from "./reference"
import { type RenderedPage, renderGenerator } from "./render-generator"

const app = resolve(import.meta.dir, "..")
const root = resolve(app, "../..")
const styleFiles = [
  "tokens.css",
  "base.css",
  "shell.css",
  "language-menu.css",
  "home.css",
  "docs.css",
  "mobile-navigation.css",
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

const sha256 = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")

export function generatorInput(playgroundUrl: string) {
  return {
    schema: 1,
    origin: "",
    playgroundUrl,
    tourUrl: new URL("tour/", playgroundUrl).href,
    grammar: "",
    examples: [
      ...chapterExamples(playgroundUrl),
      ...articleExecutions.map(({ id, sourcePath }) =>
        canonicalExample(id, sourcePath, playgroundUrl)
      ),
      ...[
        [
          "pilot-function-application",
          "apps/site/examples/src/language/pilot-function-application.ssrg",
        ],
        [
          "pilot-function-application-invalid",
          "apps/site/examples/invalid/src/language/pilot-function-application.ssrg",
        ],
        [
          "pilot-currying",
          "apps/site/examples/src/language/pilot-currying.ssrg",
        ],
        [
          "pilot-currying-invalid",
          "apps/site/examples/invalid/src/language/pilot-currying.ssrg",
        ],
        [
          "syntax-reader-pipelines",
          "apps/site/examples/src/language/syntax-reader-pipelines.ssrg",
        ],
        [
          "syntax-reader-pipelines-invalid",
          "apps/site/examples/invalid/src/language/syntax-reader-pipelines.ssrg",
        ],
        ["first-run-hello", "examples/samples/hello-world/main.ssrg"],
      ].map(([id, path]) => canonicalExample(id, path, playgroundUrl)),
    ],
    referenceModules: compilerReferenceModules(),
  }
}

function routeFile(output: string, route: string): string {
  assert.match(route, /^\/(?:[a-z0-9-]+\/)*$/u)
  return join(output, route.slice(1), "index.html")
}

export function validateInternalLinks(pages: RenderedPage[]) {
  const byRoute = new Map<string, { ids: Set<string>; links: string[] }>()
  for (const page of pages) {
    const ids: string[] = []
    const links: string[] = []
    // Read elements, not escaped example output, comments or script text.
    new HTMLRewriter()
      .on("[id]", {
        element(element) {
          const id = element.getAttribute("id")
          if (id !== null) ids.push(id)
        },
      })
      .on("a[href]", {
        element(element) {
          const href = element.getAttribute("href")
          if (href !== null) links.push(href)
        },
      })
      .transform(page.html)
    assert.equal(
      new Set(ids).size,
      ids.length,
      `Duplicate HTML id in ${page.route}`
    )
    byRoute.set(page.route, { ids: new Set(ids), links })
  }
  for (const page of pages) {
    for (const href of byRoute.get(page.route)?.links ?? []) {
      if (href.startsWith("http://") || href.startsWith("https://")) continue
      const [path, fragment] = href.split("#", 2)
      const route = path || page.route
      const target = byRoute.get(route)
      assert.ok(target, `Unresolved internal link from ${page.route}: ${href}`)
      if (fragment)
        assert.ok(
          target.ids.has(fragment),
          `Unresolved fragment from ${page.route}: ${href}`
        )
    }
  }
}

export function compileGenerator(
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
  assert.ifError(result.error)
  assert.equal(result.status, 0, result.stderr || result.stdout)
  const entry = join(directory, profile === "release" ? "entry.js" : "entry.ts")
  assert.ok(existsSync(entry), `Missing ${profile} generator entry: ${entry}`)
  return entry
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
  const client = new Bun.Transpiler({ loader: "ts" }).transformSync(
    readFileSync(join(app, "client/mobile-navigation.ts"), "utf8")
  )
  writeFileSync(join(assets, "mobile-navigation.js"), client)
  copyFileSync(join(app, "public/language.svg"), join(assets, "language.svg"))
  writeFileSync(
    join(assets, "language-menu.js"),
    new Bun.Transpiler({ loader: "ts" }).transformSync(
      readFileSync(join(app, "client/language-menu.ts"), "utf8")
    )
  )
  copyFileSync(
    join(root, "assets/brand/public/brand/favicon.ico"),
    join(output, "favicon.ico")
  )
  return [
    "favicon.ico",
    "assets/seseragi-icon.svg",
    "assets/site.css",
    "assets/mobile-navigation.js",
    "assets/language.svg",
    "assets/language-menu.js",
  ]
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
    const entry = sitePhase("compile", () =>
      compileGenerator(generator, options.profile ?? "development")
    )
    const input = sitePhase("input", () => ({
      ...generatorInput(playgroundUrl),
      origin: origin.origin,
    }))
    const pages = sitePhase("generation", () => renderGenerator(entry, input))
    const coverage = sitePhase("validation", () => {
      const routes = new Set(pages.map(({ route }) => route))
      assert.ok(pages.length > 0, "The published catalog must not be empty")
      for (const route of routes) {
        if (!route.startsWith("/ja/"))
          assert.ok(
            routes.has(route === "/" ? "/ja/" : `/ja${route}`),
            `Missing Japanese page: ${route}`
          )
      }
      assert.equal(
        new Set(pages.map(({ route }) => route)).size,
        pages.length,
        "Duplicate generated route"
      )
      sitePhase("links", () => validateInternalLinks(pages))
      const coverage = {
        edition: "docs-reboot",
        pages: routes.size / 2,
        locales: ["en", "ja"],
      }
      for (const { route, html } of pages) {
        if (!route.startsWith("/ja/")) continue
        const main = html.match(/<main\b[^>]*>([\s\S]*?)<\/main>/u)?.[1]
        assert.ok(main, `Missing article main: ${route}`)
        assert.ok(
          !/準備中|整備中|詳しい日本語解説|日本語本文は#[0-9]+/u.test(main),
          `Untranslated article placeholder: ${route}`
        )
      }
      return coverage
    })
    const assets = sitePhase("publication", () => {
      mkdirSync(output, { recursive: true })
      for (const page of pages) {
        assert.ok(!page.html.includes("site-build-error"), page.route)
        const path = routeFile(output, page.route)
        mkdirSync(dirname(path), { recursive: true })
        // Browser-only enhancement is linked by the host publisher, not embedded
        // in page prose. Content, navigation and component markup stay in Seseragi.
        const html = page.html.includes('class="mobile-docs-navigation"')
          ? page.html.replace(
              "</body>",
              '<script type="module" src="/assets/mobile-navigation.js"></script></body>'
            )
          : page.html
        writeFileSync(
          path,
          html.replace(
            "</body>",
            '<script type="module" src="/assets/language-menu.js"></script></body>'
          )
        )
      }
      const assets = publishAssets(output)
      return assets
    })
    const files = [
      ...pages.map(({ route }) =>
        route === "/" ? "index.html" : `${route.slice(1)}index.html`
      ),
      ...assets,
    ].sort()
    const manifest = {
      schema: 1,
      generator: "seseragi/official-site",
      publication: coverage,
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
      files: sitePhase("hashes", () =>
        files.map((path) => ({
          path,
          sha256: sha256(readFileSync(join(output, path))),
        }))
      ),
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
    sitePhase("cleanup", () =>
      rmSync(temporary, { recursive: true, force: true })
    )
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
