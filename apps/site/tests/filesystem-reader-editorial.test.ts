import { expect, setDefaultTimeout, test } from "bun:test"
import { createHash } from "node:crypto"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  filesystemReaderCases,
  filesystemReaderExamples,
  filesystemReaderRoutes,
} from "../scripts/filesystem-reader"
import { filesystemReaderReading } from "../scripts/filesystem-reader-reading"
import { compilerReferenceModules } from "../scripts/reference"
import { assertAuthoredLibraryTitles } from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(300_000)
const root = resolve(import.meta.dir, "../../..")
const examples = filesystemReaderExamples("https://seseragi.vercel.app/")
const hash = (value: string) => createHash("sha256").update(value).digest("hex")
function plain(html: string) {
  return html
    .replace(/<[^>]*>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function input(referenceModules: ReturnType<typeof compilerReferenceModules>) {
  return {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "",
    grammar: "",
    examples,
    referenceModules,
  }
}

test("FileSystem declaration reading is exact and does not suggest a service constructor", () => {
  const symbol = {
    identity: "std/fs::FileSystem",
    module: "std/fs",
    namespace: "type",
    kind: "opaque-type",
  }
  const fallback = { en: "unchanged English", ja: "変更しない日本語" }
  const reading = filesystemReaderReading(symbol, fallback)
  expect(reading.en).toContain("execution environment supplies")
  expect(reading.ja).toContain("実行環境からサービスを受け取ります")
  const canonical = compilerReferenceModules()
    .find((m) => m.specifier === "std/fs")!
    .items.find((s) => s.identity === symbol.identity)!
  expect(canonical.signature).toBe("FileSystem")
  expect(canonical.typeParameters).toEqual([])
  expect(canonical.reading).toEqual(reading)
  for (const wrong of [
    { ...symbol, identity: "std/fs::FileSystem-extra" },
    { ...symbol, module: "std/path" },
    { ...symbol, namespace: "value" },
    { ...symbol, kind: "struct" },
    { ...symbol, identity: "std/path::Path", module: "std/path" },
  ])
    expect(filesystemReaderReading(wrong, fallback)).toBe(fallback)
})

test("FileSystemError reading describes its verified fields without invented type parameters", () => {
  const symbol = {
    identity: "std/fs::FileSystemError",
    module: "std/fs",
    namespace: "type",
    kind: "struct",
  }
  const fallback = { en: "unchanged English", ja: "変更しない日本語" }
  const reading = filesystemReaderReading(symbol, fallback)
  expect(reading.en).toContain("operation, path, otherPath, and kind")
  expect(reading.en).toContain("This declaration has no type parameters")
  expect(reading.ja).toContain("この宣言に型引数はありません")
  for (const wrong of [
    { ...symbol, identity: "std/fs::FileSystemError-extra" },
    { ...symbol, module: "std/path" },
    { ...symbol, namespace: "value" },
    { ...symbol, kind: "opaque-type" },
    { ...symbol, identity: "std/fs::FileMetadata" },
  ])
    expect(filesystemReaderReading(wrong, fallback)).toBe(fallback)
  const canonical = JSON.parse(
    readFileSync(
      join(
        root,
        "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
      ),
      "utf8"
    )
  )
    .modules.find((m: { specifier: string }) => m.specifier === "std/fs")
    .items.find((s: { identity: string }) => s.identity === symbol.identity)
  expect(canonical.signature).toBe("FileSystemError")
  expect(canonical.typeParameters).toEqual([])
  const projected = compilerReferenceModules()
    .find((m) => m.specifier === "std/fs")!
    .items.find((s) => s.identity === symbol.identity)!
  expect(projected.reading).toEqual(reading)
  const fieldMetadata = readFileSync(
    join(root, "crates/seseragi-project/src/standard.rs"),
    "utf8"
  )
  const shape = fieldMetadata
    .slice(fieldMetadata.indexOf('"FileSystemError",\n            ['))
    .split("],")[0]
  for (const field of ["operation", "path", "otherPath", "kind"])
    expect(shape).toContain(`required("${field}",`)
})

test("filesystem editorial selects only exact existing identities and module owners", () => {
  const all = compilerReferenceModules()
  const leaves = filesystemReaderRoutes.filter((item) => item.kind !== "module")
  expect(filesystemReaderRoutes).toHaveLength(16)
  expect(leaves).toHaveLength(14)
  const symbols = leaves.map((item) => {
    const matches = all
      .find((m) => m.specifier === item.module)!
      .items.filter(
        (s) =>
          s.identity === item.identity &&
          s.namespace === item.namespace &&
          s.itemKind === item.kind
      )
    expect(matches).toHaveLength(1)
    return matches[0]
  })
  const probes = symbols.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/process" },
    { ...symbol, namespace: "wrong" },
    { ...symbol, itemKind: "wrong" },
    { ...symbol, identity: `${symbol.identity}-wrong` },
  ])
  const controls = all
    .filter((m) => ["std/path", "std/fs"].includes(m.specifier))
    .flatMap((m) =>
      m.items.filter((s) => !symbols.some((x) => x.identity === s.identity))
    )
  const data = input([{ ...all[0], items: [...probes, ...controls] }])
  const directory = mkdtempSync(join(tmpdir(), "seseragi-filesystem-guards-"))
  try {
    const result = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/filesystem-reader/catalog",
      ],
      timeoutMs: 240_000,
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Block } from "./model/page"
import { Editorial } from "./reference/editorial/model"
import { editorialFor, moduleEditorial } from "./reference/editorial/catalog"
import { filesystemReaderEditorialFor, filesystemReaderModuleBlocks } from "./reference/editorial/filesystem-reader/catalog"
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [
    arrays.concat [[(present (filesystemReaderEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules],
    [(arrays.length (filesystemReaderModuleBlocks name) > 0, arrays.length (moduleEditorial name) > 0) | name <- ["std/path", "std/fs", "std/process", "std/fs-extra", "std/path::Path"]]
  ]))
}`,
    })
    expect(result).toHaveLength(probes.length + controls.length + 5)
    for (let i = 0; i < probes.length; i++)
      expect(result[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    for (let i = probes.length; i < probes.length + controls.length; i++)
      expect(result[i]).toEqual([false, false])
    expect(result.slice(-5)).toEqual([
      [true, true],
      [true, true],
      [false, false],
      [false, false],
      [false, false],
    ])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all thirty-two filesystem locale bodies preserve prose, sources, results, declarations and exact title links", () => {
  const all = compilerReferenceModules()
  const modules = all.filter((m) =>
    ["std/path", "std/fs"].includes(m.specifier)
  )
  const data = input(modules)
  const directory = mkdtempSync(join(tmpdir(), "seseragi-filesystem-render-"))
  try {
    const result = renderPageClosure<Array<{ route: string; html: string }>>({
      directory,
      modules: ["reference/catalog", "render/document"],
      timeoutMs: 240_000,
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { En, Ja, localized, localizedRoute } from "./model/locale"
import { Reference, SiteCatalog, PageDefinition, NavigationArea } from "./model/page"
import { referenceGroups } from "./reference/catalog"
import { renderDocument } from "./render/document"
struct Output deriving JsonEncode { route: String, html: String }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "[]"
  Right input -> {
    let groups = referenceGroups input.referenceModules
    let sections = arrays.concat [group.sections | group <- groups]
    let pages = arrays.concat [arrays.concat [[section.overview], section.pages] | section <- sections]
    let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference, title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
    let site = SiteCatalog { home, pages, areas: [NavigationArea { id: "library", title: home.title, description: home.summary, landing: home, groups }] }
    println (json.encodeString [Output {route: localizedRoute locale page.path, html: renderDocument input site locale page} | locale <- [En, Ja], page <- pages])
  }
}`,
    })
    const pages = new Map(result.map((p) => [p.route, p.html]))
    const titles = new Map(result.map((p) => [p.route, pageTitle(p.html)]))
    const bodies: Array<{
      route: string
      html: string
      body: string
      sha256: string
    }> = []
    let titleLinks = 0
    for (const item of filesystemReaderRoutes) {
      for (const locale of ["en", "ja"] as const) {
        const prefix = locale === "en" ? "" : "/ja"
        const route = prefix + item.route
        const html = pages.get(route)!
        expect(html, route).toBeDefined()
        const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0]
        const text = plain(body)
        for (const id of [
          "using-this-operation",
          "reading-the-example",
          "typescript-comparison",
          "operation-rules",
          "choose-next",
        ])
          expect(body).toContain(`id="${id}"`)
        expect(body).not.toContain("Missing canonical example")
        expect(body).not.toContain("/docs/applications/")
        expect(body).not.toContain("/tmp/")
        const copy = (lang: string) =>
          readFileSync(
            join(
              root,
              `apps/site/src/reference/editorial/filesystem-reader/${item.slug}/${lang}.ssrg`
            ),
            "utf8"
          )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const own = copy(locale)
        const other = copy(locale === "en" ? "ja" : "en")
        expect(paragraphs(own)).toHaveLength(paragraphs(other).length)
        for (const paragraph of paragraphs(own))
          expect(text, `${route}: own paragraph`).toContain(paragraph)
        for (const paragraph of paragraphs(other))
          expect(text, `${route}: locale leakage`).not.toContain(paragraph)
        for (const [, field, value] of own.matchAll(
          /^ {2}(\w+): ("(?:[^"\\]|\\.)*")/gmu
        ))
          if (field !== "summary" || item.kind !== "module")
            expect(plain(html), `${route}: ${field}`).toContain(
              JSON.parse(value)
            )
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((m) => plain(m[1]))
        const native = examples.find(
          (e) => e.id === `filesystem-reader-${item.example}`
        )!
        expect(
          panels.filter((p) => p === native.source),
          `${route}: exact native source`
        ).toHaveLength(1)
        const sourceIds = [native.id]
        if (item.comparison) {
          const comparison = examples.find(
            (e) => e.id === `filesystem-reader-${item.comparison}-ts`
          )!
          expect(
            panels.filter((p) => p === comparison.source),
            `${route}: exact TS source`
          ).toHaveLength(1)
          sourceIds.push(comparison.id)
        }
        expect(panels).toHaveLength(
          sourceIds.length + (item.kind === "module" ? 0 : 1)
        )
        const source = filesystemReaderCases.find(
          (c) => c.slug === item.example
        )!
        const terminals = [
          ...body.matchAll(
            /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
          ),
        ].map((m) => plain(m[1]))
        const comparisonCase = filesystemReaderCases.find(
          (c) => c.slug === item.comparison
        )
        const comparisonOutput =
          comparisonCase && "typescriptOutput" in comparisonCase
            ? comparisonCase.typescriptOutput.trimEnd()
            : ""
        expect(terminals).toEqual([
          `seseragi run ${item.example}.ssrg --target process`,
          source.output.trimEnd(),
          ...(comparisonOutput ? [comparisonOutput] : []),
        ])
        const seeds = [...body.matchAll(/href="([^"]+)"/gu)]
          .map((m) => new URL(plain(m[1]), data.origin))
          .filter(
            (u) =>
              u.origin === "https://seseragi.vercel.app" &&
              u.searchParams.has("source")
          )
          .map((u) => u.searchParams.get("source"))
        expect(seeds).toEqual(native.playgroundUrl ? [native.source] : [])
        if (item.module === "std/fs") {
          expect(seeds).toHaveLength(0)
          expect(text).toContain(
            locale === "en" ? "process runtime" : "プロセスの実行環境"
          )
        }
        if (item.kind !== "module") {
          const symbol = modules
            .find((m) => m.specifier === item.module)!
            .items.find((s) => s.identity === item.identity)!
          expect(panels.at(-1)).toBe(symbol.signature)
          expect(text).toContain(
            locale === "en" ? symbol.reading.en : symbol.reading.ja
          )
          expect(body).toContain('id="canonical-declaration"')
          const topic = `${prefix}/docs/library/${item.module.slice(4)}/`
          for (const [, destination] of html.matchAll(
            /class="reference-(?:previous|next)" href="([^"]+)"/gu
          )) {
            expect(destination).toStartWith(topic)
            expect(pages.has(destination)).toBe(true)
          }
          expect(html).toContain(`class="reference-topic-link" href="${topic}"`)
          expect(body).toContain(`href="${topic}"`)
          expect(pages.get(topic)).toContain(`href="${route}"`)
        }
        const entry = {
          route,
          kind: item.kind === "module" ? ("module" as const) : ("api" as const),
        }
        const count = assertAuthoredLibraryTitles(html, titles, entry)
        expect(count).toBeGreaterThanOrEqual(4)
        titleLinks += count
        const mutated = html.replace(
          /(<p><a\b[^>]*href="[^"#]+">)([^<]+)(<\/a><span>：|<\/a><span>: )/u,
          "$1wrong title$3"
        )
        expect(mutated).not.toBe(html)
        expect(() =>
          assertAuthoredLibraryTitles(mutated, titles, entry)
        ).toThrow()
        expect(html).toContain(`href="${prefix ? "" : "/ja"}${item.route}"`)
        bodies.push({ route, html, body, sha256: hash(html) })
      }
    }
    expect(bodies).toHaveLength(32)
    expect(titleLinks).toBeGreaterThanOrEqual(128)
    for (const route of [
      "/docs/library/path/function/normalize/",
      "/docs/library/fs/effect-function/exists/",
    ])
      expect(pages.get(route)).not.toContain('id="typescript-comparison"')
    const output = process.env.FILESYSTEM_READER_RENDER_DIR
    if (output) {
      mkdirSync(output, { recursive: true })
      writeFileSync(
        join(output, "all-rendered.json"),
        JSON.stringify(result, null, 2)
      )
      writeFileSync(
        join(output, "selected-bodies.json"),
        JSON.stringify(bodies, null, 2)
      )
      for (const page of bodies) {
        const destination = join(output, "rendered", page.route.slice(1))
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
