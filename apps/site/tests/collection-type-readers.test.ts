import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  collectionTypeCases,
  collectionTypeExamples,
  collectionTypeInvalidCases,
  collectionTypePrograms,
} from "../scripts/collection-type-readers"
import { compilerReferenceModules } from "../scripts/reference"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const tsc = resolve(root, "node_modules/typescript/bin/tsc")
const examples = collectionTypeExamples("https://seseragi.vercel.app/")

function run(command: string, args: string[], cwd = root) {
  return spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 16 * 1024 * 1024,
  })
}

const typecheckArguments = [
  tsc,
  "--noEmit",
  "--strict",
  "--noUncheckedIndexedAccess",
  "--skipLibCheck",
  "--target",
  "ES2022",
  "--lib",
  "ESNext,DOM",
  "--module",
  "ESNext",
  "--moduleResolution",
  "bundler",
]

test("nine exact collection type and constructor identities keep their module owners and signatures", () => {
  const modules = compilerReferenceModules()
  expect(collectionTypeCases).toHaveLength(9)
  expect(new Set(collectionTypeCases.map((item) => item.identity)).size).toBe(9)
  for (const item of collectionTypeCases) {
    const module = modules.find((module) => module.specifier === item.module)!
    expect(module.availability).toBe("available")
    expect(module.targets).toEqual(["process", "browser"])
    const matches = module.items.filter(
      (symbol) =>
        symbol.identity === item.identity &&
        symbol.namespace === item.namespace &&
        symbol.itemKind === item.itemKind
    )
    expect(matches, item.identity).toHaveLength(1)
    expect(matches[0]?.name).toBe(item.name)
    expect(matches[0]?.signature).toBe(item.signature)
    expect(matches[0]?.constraints).toEqual([])
  }
  const iterator = collectionTypeCases.find((item) => item.name === "Iterator")!
  expect(iterator.module).toBe("std/iterator")
  expect(iterator.identity).toBe("std/prelude::Iterator")
  expect(
    collectionTypeCases.some((item) => /\/(array|list)$/u.test(item.module))
  ).toBe(false)
})

test("six native and six TypeScript sources plus eight rejected sources have unique canonical IDs", () => {
  expect(collectionTypePrograms).toHaveLength(6)
  expect(collectionTypeInvalidCases).toHaveLength(8)
  expect(examples).toHaveLength(20)
  expect(new Set(examples.map((sample) => sample.id)).size).toBe(20)
  for (const sample of examples) {
    const source = readFileSync(join(root, sample.sourcePath), "utf8")
    expect(sample.source).toBe(source)
    expect(sample.sha256).toBe(
      createHash("sha256").update(source).digest("hex")
    )
    expect(sample.highlighted.map((token) => token.text).join("")).toBe(source)
    if (sample.sourcePath.startsWith("apps/site/examples/src/")) {
      expect(new URL(sample.playgroundUrl).searchParams.get("source")).toBe(
        source
      )
    } else {
      expect(sample.playgroundUrl).toBe("")
    }
  }
})

test("six native canonical programs lint and execute with exact documented output in isolation", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-collection-types-"))
  try {
    for (const item of collectionTypePrograms) {
      const sample = examples.find(
        (sample) => sample.id === `collection-type-${item.nativeSlug}`
      )!
      writeFileSync(join(directory, "main.ssrg"), sample.source)
      const lint = run(cli, ["lint", "main.ssrg"], directory)
      expect(lint.status, `${item.nativeSlug}: ${lint.stderr}`).toBe(0)
      expect(lint.stdout).toBe("")
      expect(lint.stderr).toBe("")
      const result = run(cli, ["run", "main.ssrg"], directory)
      expect(result.status, `${item.nativeSlug}: ${result.stderr}`).toBe(0)
      expect(result.stderr).toBe("")
      expect(result.stdout).toBe(item.output)
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("six TypeScript task comparisons strictly typecheck locally and execute their own documented output", () => {
  const comparisons = collectionTypePrograms.map((item) => ({
    item,
    sample: examples.find(
      (sample) => sample.id === `collection-type-${item.nativeSlug}-ts`
    )!,
  }))
  // Use the installed compiler directly: no registry lookup or installation.
  const checked = run("bun", [
    ...typecheckArguments,
    ...comparisons.map(({ sample }) => join(root, sample.sourcePath)),
  ])
  expect(checked.status, checked.stdout + checked.stderr).toBe(0)
  expect(checked.stdout).toBe("")
  expect(checked.stderr).toBe("")
  for (const { item, sample } of comparisons) {
    const result = run("bun", [join(root, sample.sourcePath)])
    expect(result.status, `${item.nativeSlug}: ${result.stderr}`).toBe(0)
    expect(result.stderr).toBe("")
    expect(result.stdout).toBe(item.typescriptOutput)
  }
})

test("the exact six Playground seeds compile through committed WASM and execute in the browser runtime harness", async () => {
  const bindings = await import(
    new URL("../../playground/src/wasm/pkg/seseragi_wasm.js", import.meta.url)
      .href
  )
  await bindings.default({
    module_or_path: await Bun.file(
      new URL(
        "../../playground/src/wasm/pkg/seseragi_wasm_bg.wasm",
        import.meta.url
      )
    ).arrayBuffer(),
  })
  const runtime = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  for (const item of collectionTypePrograms) {
    const sample = examples.find(
      (sample) => sample.id === `collection-type-${item.nativeSlug}`
    )!
    const seed = new URL(sample.playgroundUrl).searchParams.get("source")!
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    )
    expect(compiled.status, JSON.stringify(compiled)).toBe("success")
    expect(compiled.entry).toBeDefined()
    if (compiled.status !== "success" || !compiled.entry) continue
    const executed = await runtime.executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(executed.stdout, item.nativeSlug).toBe(item.output.trimEnd())
    expect(executed.debug).toBe("()")
  }
})

test("eight plausible collection type mismatches reject with SES-T0101", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-collection-invalid-"))
  try {
    for (const slug of collectionTypeInvalidCases) {
      const sample = examples.find(
        (sample) => sample.id === `collection-type-invalid-${slug}`
      )!
      writeFileSync(join(directory, "main.ssrg"), sample.source)
      const result = run(cli, ["lint", "main.ssrg"], directory)
      expect(result.status, `${slug}: ${result.stderr}`).toBe(2)
      expect(result.stdout).toBe("")
      expect(result.stderr, slug).toContain("SES-T0101")
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("the TypeScript nonempty tuple rejects an empty input at typecheck time", () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-collection-ts-invalid-")
  )
  try {
    const source = examples.find(
      (sample) => sample.id === "collection-type-nonempty-ts"
    )!.source
    const path = join(directory, "nonempty.ts")
    writeFileSync(path, `${source}\nconst empty: NonEmptyScores = []\n`)
    const result = run("bun", [...typecheckArguments, path])
    expect(result.status).not.toBe(0)
    expect(result.stdout).toContain("TS2322")
    expect(result.stderr).toBe("")
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("an ordinary JavaScript iterator advances while retained Seseragi positions are tested separately", () => {
  function* countdown(remaining: number): Iterator<number> {
    while (remaining > 0) {
      yield remaining
      remaining -= 1
    }
  }
  const cursor = countdown(3)
  expect(cursor.next()).toEqual({ value: 3, done: false })
  expect(cursor.next()).toEqual({ value: 2, done: false })
  expect(cursor.next()).toEqual({ value: 1, done: false })
  expect(cursor.next()).toEqual({ value: undefined, done: true })
})

// Production rendering is deliberately bounded to this batch and its linked
// destinations. It is not the full site, layout/browser or reader-acceptance gate.
import { mkdirSync } from "node:fs"
import { generatorInput } from "../scripts/build"
import { assertAuthoredLibraryTitles } from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

function plain(html: string) {
  return html
    .replace(/<[^>]*>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function routeFor(symbol: { module: string; itemKind: string; name: string }) {
  const slug = symbol.name
    .toLowerCase()
    .replaceAll("<", "-")
    .replaceAll(">", "")
    .replaceAll(", ", "-")
  return `/docs/library/${symbol.module.slice(4)}/${symbol.itemKind}/${slug}/`
}

test("exact type/constructor dispatch rejects altered ownership, namespaces, kinds, identities and all 57 instance leaves", () => {
  const all = compilerReferenceModules()
  const chosen = collectionTypeCases.map(
    (item) =>
      all
        .find((m) => m.specifier === item.module)!
        .items.find(
          (x) =>
            x.identity === item.identity &&
            x.namespace === item.namespace &&
            x.itemKind === item.itemKind
        )!
  )
  const probes = chosen.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/list" },
    { ...symbol, namespace: symbol.namespace === "type" ? "value" : "type" },
    { ...symbol, itemKind: "function" },
    { ...symbol, identity: `${symbol.identity}Extra` },
  ])
  const instances = all
    .filter((m) =>
      [
        "std/array",
        "std/list",
        "std/collection",
        "std/iterator",
        "std/map",
        "std/non-empty-list",
        "std/set",
      ].includes(m.specifier)
    )
    .flatMap((m) => m.items.filter((x) => x.namespace === "instance"))
  expect(instances).toHaveLength(57)
  const input = {
    schema: 1,
    origin: "",
    playgroundUrl: "",
    tourUrl: "",
    grammar: "",
    examples: [],
    referenceModules: [{ ...all[0], items: [...probes, ...instances] }],
  }
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-collection-type-dispatch-")
  )
  try {
    const result = renderPageClosure<boolean[]>({
      directory,
      modules: ["reference/editorial/collection-types/catalog"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { collectionTypeEditorialFor } from "./reference/editorial/collection-types/catalog"
fn present value: Maybe<Editorial> -> Bool = match value {
  Just _ -> True
  Nothing -> False
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [[present (collectionTypeEditorialFor symbol) | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(result).toEqual([
      ...chosen.flatMap(() => [true, false, false, false, false]),
      ...instances.map(() => false),
    ])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all eighteen type/constructor bodies render exact localized prose, declarations, outputs, source seeds and title-correct links", () => {
  const all = compilerReferenceModules()
  const selectedIds = new Set<string>(
    collectionTypeCases.map((x) => x.identity)
  )
  const linked = new Set([
    "std/map::fromEntries",
    "std/map::get",
    "std/map::insert",
    "std/set::fromIterable",
    "std/set::insert",
    "std/non-empty-list::fromList",
    "std/non-empty-list::cons",
    "std/non-empty-list::tail",
    "std/iterator::unfold",
    "std/iterator::next",
    "std/collection::reduceUntil",
    "std/array::chunksOf",
    "std/list::windows",
    "std/non-empty-list::Hash",
    "std/iterator::Iterable",
    "std/collection::Eq",
  ])
  const modules = all
    .map((m) => ({
      ...m,
      items: m.items.filter(
        (x) => selectedIds.has(x.identity) || linked.has(x.identity)
      ),
    }))
    .filter((m) => m.items.length)
  const seed = generatorInput("https://seseragi.vercel.app/")
  const input = {
    ...seed,
    origin: "https://seseragi.example",
    examples: [
      ...new Map(
        [...seed.examples, ...examples].map((x) => [x.id, x])
      ).values(),
    ],
    referenceModules: modules,
  }
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-collection-type-render-")
  )
  try {
    const output = renderPageClosure<Array<{ route: string; html: string }>>({
      directory,
      modules: [
        "reference/catalog",
        "render/document",
        "pages/language/data/tuples-arrays-and-lists/page",
      ],
      stdin: `${JSON.stringify(input)}\n`,
      entry: `import * as effects from "std/effect"
import * as inputs from "std/stdin"
import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { En, Ja, localized, localizedRoute } from "./model/locale"
import { Reference, SiteCatalog, PageDefinition, NavigationArea } from "./model/page"
import { referenceGroups } from "./reference/catalog"
import { renderDocument } from "./render/document"
import { page as sequences } from "./pages/language/data/tuples-arrays-and-lists/page"
type Failure deriving Show = | InputFailure | DecodeFailure | ConsoleFailure ConsoleError
struct Output deriving JsonEncode { route: String, html: String }
fn requireInput value: Maybe<String> -> Either<Failure, String> = match value {
  Nothing -> Left InputFailure
  Just encoded -> Right encoded
}
pub effect fn main -> Unit with Console, Stdin fails Failure = do {
  limit <- inputs.lineLimit 67108864 |> effects.fromEither |> effects.mapError (\\_ -> InputFailure)
  line <- inputs.readLineWith limit |> effects.mapError (\\_ -> InputFailure)
  encoded <- requireInput line |> effects.fromEither
  input <- decodeBuildInput encoded |> effects.fromEither |> effects.mapError (\\_ -> DecodeFailure)
  let groups = referenceGroups input.referenceModules
  let sections = arrays.concat [group.sections | group <- groups]
  let pages = arrays.concat [arrays.concat [arrays.concat [[section.overview], section.pages] | section <- sections], [sequences ()]]
  let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference, title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
  let site = SiteCatalog {home, pages, areas: [NavigationArea {id:"library",title:home.title,description:home.summary,landing:home,groups}]}
  println (json.encodeString [Output {route:localizedRoute locale page.path,html:renderDocument input site locale page} | locale <- [En,Ja], page <- pages]) |> effects.mapError ConsoleFailure
}`,
    })
    const pages = new Map(output.map((p) => [p.route, p.html]))
    const titles = new Map(output.map((p) => [p.route, pageTitle(p.html)]))
    let titleLinks = 0
    let bodies = 0
    for (const item of collectionTypeCases)
      for (const locale of ["en", "ja"]) {
        const prefix = locale === "en" ? "" : "/ja"
        const route = prefix + routeFor(item)
        const html = pages.get(route)!
        expect(html, route).toBeDefined()
        bodies++
        const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0]
        const text = plain(body)
        expect(pageTitle(html)).toBe(item.name)
        for (const heading of [
          "using-this-type",
          "typescript-comparison",
          "type-rules",
          "canonical-declaration",
        ])
          expect(body).toContain(`id="${heading}"`)
        expect(body).not.toContain("Missing canonical example")
        expect(text).toContain(item.signature)
        expect(html).toContain(
          `href="${locale === "en" ? "/ja" : ""}${routeFor(item)}"`
        )
        expect(
          pages.get(`${prefix}/docs/library/${item.module.slice(4)}/`)
        ).toContain(`href="${route}"`)
        const terminals = [
          ...body.matchAll(
            /<section class="code-panel terminal-panel"><div class="code-panel-header"><span class="code-panel-title">([^<]*)<\/span><\/div><pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
          ),
        ].map((m) => [plain(m[1]), plain(m[2])])
        expect(terminals, route).toEqual([
          [
            locale === "en" ? "Seseragi output" : "Seseragiの実行結果",
            item.output.trimEnd(),
          ],
          [
            locale === "en" ? "TypeScript output" : "TypeScriptの実行結果",
            item.typescriptOutput.trimEnd(),
          ],
        ])
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((m) => plain(m[1]))
        for (const suffix of ["", "-ts"]) {
          const sample = examples.find(
            (x) => x.id === `collection-type-${item.nativeSlug}${suffix}`
          )!
          expect(
            panels.filter((x) => x === sample.source),
            `${route}:${suffix}`
          ).toHaveLength(1)
        }
        const playgroundSources = [...body.matchAll(/href="([^"]+)"/gu)]
          .map((m) => new URL(plain(m[1]), input.origin))
          .filter(
            (u) =>
              u.origin === "https://seseragi.vercel.app" &&
              u.searchParams.has("source")
          )
          .map((u) => u.searchParams.get("source"))
        expect(playgroundSources).toEqual([
          examples.find((x) => x.id === `collection-type-${item.nativeSlug}`)!
            .source,
        ])
        const sourceSlug = item.slug === "nonemptylist" ? "nonempty" : item.slug
        const own = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/collection-types/${sourceSlug}/${locale}.ssrg`
          ),
          "utf8"
        )
        const other = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/collection-types/${sourceSlug}/${locale === "en" ? "ja" : "en"}.ssrg`
          ),
          "utf8"
        )
        const paragraphs = (s: string) =>
          [...s.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        expect(paragraphs(own).length).toBe(paragraphs(other).length)
        expect(paragraphs(own).length).toBeGreaterThanOrEqual(6)
        for (const p of paragraphs(own)) expect(text, route).toContain(p)
        for (const p of paragraphs(other)) expect(text, route).not.toContain(p)
        for (const [, field, literal] of own.matchAll(
          /^ {2}(\w+): ("(?:[^"\\]|\\.)*")/gmu
        ))
          if (field !== "outputTitle")
            expect(plain(html), `${route}:${field}`).toContain(
              JSON.parse(literal)
            )
        titleLinks += assertAuthoredLibraryTitles(html, titles, {
          route,
          kind: "api",
        })
        const mutated = html.replace(
          /(<p><a\b[^>]*href="[^"#]+">)([^<]+)(<\/a><span>：|<\/a><span>: )/u,
          "$1incorrect title$3"
        )
        expect(mutated).not.toBe(html)
        expect(() =>
          assertAuthoredLibraryTitles(mutated, titles, { route, kind: "api" })
        ).toThrow()
        // Every authored internal link resolves within this bounded real render.
        for (const [, raw] of body.matchAll(/href="([^"]+)"/gu)) {
          const u = new URL(plain(raw), input.origin)
          if (
            u.origin !== input.origin ||
            ["/docs/language/", "/ja/docs/language/"].includes(u.pathname)
          )
            continue
          const target = pages.get(u.pathname)
          expect(target, `${route} -> ${u.pathname}`).toBeDefined()
          if (u.hash) expect(target).toContain(`id="${u.hash.slice(1)}"`)
        }
      }
    expect(bodies).toBe(18)
    expect(titleLinks).toBe(74)
    for (const m of modules)
      for (const x of m.items.filter((x) => x.namespace === "instance"))
        for (const prefix of ["", "/ja"]) {
          const h = pages.get(prefix + routeFor(x))!
          expect(h).not.toContain('id="using-this-type"')
          expect(h).not.toContain('id="typescript-comparison"')
          expect(plain(h)).toContain(x.signature)
          expect(plain(h)).toContain(prefix ? x.reading.ja : x.reading.en)
        }
    if (process.env.COLLECTION_TYPE_RENDER_DIR) {
      for (const page of output) {
        const path = join(
          process.env.COLLECTION_TYPE_RENDER_DIR,
          page.route.slice(1)
        )
        mkdirSync(path, { recursive: true })
        writeFileSync(join(path, "index.html"), page.html)
      }
      writeFileSync(
        join(process.env.COLLECTION_TYPE_RENDER_DIR, "scope.json"),
        JSON.stringify(
          {
            identities: collectionTypeCases.map((x) => x.identity),
            bodies,
            titleLinks,
            routes: collectionTypeCases.flatMap((x) => [
              routeFor(x),
              `/ja${routeFor(x)}`,
            ]),
            allRoutes: output.map((x) => x.route),
          },
          null,
          2
        )
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
