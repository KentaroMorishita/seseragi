import { expect, setDefaultTimeout, test } from "bun:test"
import { createHash } from "node:crypto"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { join, resolve } from "node:path"
import {
  jsonReaderCases,
  jsonReaderExamples,
  jsonReaderRoutes,
} from "../scripts/json-reader"
import { jsonReaderReading } from "../scripts/json-reader-reading"
import { compilerReferenceModules } from "../scripts/reference"
import { signatureReading } from "../scripts/signature-reading"
import {
  assertAuthoredLibraryTitles,
  authoredLibraryArticle,
} from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(600_000)
const root = resolve(import.meta.dir, "../../..")
const examples = jsonReaderExamples("https://seseragi.vercel.app/")
const evidence = resolve(
  process.env.JSON_READER_EVIDENCE_DIR ??
    join(root, "target/json-reader-evidence")
)
mkdirSync(evidence, { recursive: true })
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

test("JSON20 guards exact identity, owner, namespace and kind without changing other JSON pages", () => {
  const all = compilerReferenceModules()
  const modules = all.filter(
    (m) => m.specifier === "std/json" || m.specifier === "std/prelude"
  )
  const module = modules.find((m) => m.specifier === "std/json")!
  const leaves = jsonReaderRoutes.filter((x) => x.kind !== "module")
  expect(leaves).toHaveLength(19)
  const symbols = leaves.map((item) => {
    const matches = modules
      .flatMap((m) => m.items)
      .filter(
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
    { ...symbol, module: "std/http" },
    { ...symbol, namespace: "wrong" },
    { ...symbol, itemKind: "wrong" },
    { ...symbol, identity: `${symbol.identity}-wrong` },
  ])
  const controls = module.items.filter(
    (s) => !symbols.some((x) => x.identity === s.identity)
  )
  const data = input([{ ...module, items: [...probes, ...controls] }])
  const directory = mkdtempSync(join(evidence, "seseragi-json20-guards-"))
  try {
    const result = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/json-reader/catalog",
      ],
      timeoutMs: 240_000,
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor, moduleEditorial } from "./reference/editorial/catalog"
import { jsonReaderEditorialFor, jsonReaderModuleBlocks } from "./reference/editorial/json-reader/catalog"
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [
    arrays.concat [[(present (jsonReaderEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules],
    [(arrays.length (jsonReaderModuleBlocks name) > 0, arrays.length (moduleEditorial name) > 0) | name <- ["std/json", "std/json-extra", "std/json::Json"]]
  ]))
}`,
    })
    expect(result).toHaveLength(probes.length + controls.length + 3)
    for (let i = 0; i < probes.length; i++)
      expect(result[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    for (let i = probes.length; i < probes.length + controls.length; i++)
      expect(result[i]).toEqual([false, false])
    expect(result.slice(-3)).toEqual([
      [true, true],
      [false, false],
      [false, false],
    ])
  } finally {
    if (!process.env.JSON_READER_EVIDENCE_DIR)
      rmSync(directory, { recursive: true, force: true })
  }
})

test("all forty bilingual bodies keep the task, both exact sources, outputs, declarations and navigation", () => {
  const modules = compilerReferenceModules()
    .filter((m) => m.specifier === "std/json" || m.specifier === "std/prelude")
    .map((m) =>
      m.specifier === "std/prelude"
        ? {
            ...m,
            items: m.items.filter((s) =>
              jsonReaderRoutes.some((r) => r.identity === s.identity)
            ),
          }
        : m
    )
  const data = input(modules)
  const directory = mkdtempSync(join(evidence, "seseragi-json20-render-"))
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
    const earlyOutput = process.env.JSON_READER_RENDER_DIR
    if (earlyOutput) {
      mkdirSync(earlyOutput, { recursive: true })
      writeFileSync(
        join(earlyOutput, "all-rendered.json"),
        JSON.stringify(result, null, 2)
      )
      const selected = result
        .filter((page) =>
          jsonReaderRoutes.some(
            (item) =>
              page.route === item.route || page.route === `/ja${item.route}`
          )
        )
        .map((page) => ({
          ...page,
          body: page.html.match(/<article\b[\s\S]*?<\/article>/u)?.[0] ?? "",
          sha256: createHash("sha256").update(page.html).digest("hex"),
        }))
      writeFileSync(
        join(earlyOutput, "selected-bodies.json"),
        JSON.stringify(selected, null, 2)
      )
    }
    const pages = new Map(result.map((p) => [p.route, p.html]))
    const titles = new Map(result.map((p) => [p.route, pageTitle(p.html)]))
    const bodies: Array<{
      route: string
      html: string
      body: string
      sha256: string
    }> = []
    let titleLinks = 0
    for (const item of jsonReaderRoutes)
      for (const locale of ["en", "ja"] as const) {
        const prefix = locale === "en" ? "" : "/ja"
        const route = prefix + item.route
        const html = pages.get(route)!
        expect(html, route).toBeDefined()
        const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0]
        const text = plain(body)
        for (const id of [
          "using-this-operation",
          "typescript-comparison",
          "seseragi-example",
          "reading-the-example",
          "operation-rules",
          "choose-next",
        ])
          expect(body).toContain(`id="${id}"`)
        expect(body.indexOf('id="typescript-comparison"')).toBeLessThan(
          body.indexOf('id="seseragi-example"')
        )
        expect(body.indexOf('id="seseragi-example"')).toBeLessThan(
          body.indexOf('id="reading-the-example"')
        )
        for (const forbidden of [
          "Missing canonical example",
          "/docs/applications/",
          "/tmp/",
          "/workspace/",
          "agent_notes",
        ])
          expect(body).not.toContain(forbidden)
        const copy = (lang: string) =>
          readFileSync(
            join(
              root,
              `apps/site/src/reference/editorial/json-reader/${item.slug}/${lang}.ssrg`
            ),
            "utf8"
          )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const own = copy(locale),
          other = copy(locale === "en" ? "ja" : "en")
        expect(paragraphs(own)).toHaveLength(paragraphs(other).length)
        for (const p of paragraphs(own))
          expect(text, `${route}: own paragraph`).toContain(p)
        for (const p of paragraphs(other))
          expect(text, `${route}: locale leakage`).not.toContain(p)
        for (const [, field, value] of own.matchAll(
          /^ {2}(\w+): ("(?:[^"\\]|\\.)*")/gmu
        )) {
          if (field !== "summary" || item.kind !== "module")
            expect(plain(html), `${route}: ${field}`).toContain(
              JSON.parse(value)
            )
        }
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((m) => plain(m[1]))
        const native = examples.find(
          (e) => e.id === `json-reader-${item.example}`
        )!
        const ts = examples.find(
          (e) => e.id === `json-reader-${item.example}-ts`
        )!
        expect(panels[0]).toBe(ts.source)
        expect(panels[1]).toBe(native.source)
        expect(panels).toHaveLength(2 + (item.kind === "module" ? 0 : 1))
        const sample = jsonReaderCases.find((c) => c.slug === item.example)!
        const terminals = [
          ...body.matchAll(
            /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
          ),
        ].map((m) => plain(m[1]))
        expect(terminals).toEqual([
          sample.output,
          sample.output,
          `seseragi run ${item.example}.ssrg --target process`,
        ])
        const seeds = [...body.matchAll(/href="([^"]+)"/gu)]
          .map((m) => new URL(plain(m[1]), data.origin))
          .filter(
            (u) =>
              u.origin === "https://seseragi.vercel.app" &&
              u.searchParams.has("source")
          )
          .map((u) => u.searchParams.get("source"))
        expect(seeds).toEqual([native.source])
        if (item.kind !== "module") {
          const symbol = modules
            .flatMap((m) => m.items)
            .find((s) => s.identity === item.identity)!
          expect(panels.at(-1)).toBe(symbol.signature)
          expect(text).toContain(
            locale === "en" ? symbol.reading.en : symbol.reading.ja
          )
          expect(body).toContain('id="canonical-declaration"')
          const topic = `${prefix}/docs/library/${item.module.replace("std/", "")}/`
          for (const [, destination] of html.matchAll(
            /class="reference-(?:previous|next)" href="([^"]+)"/gu
          )) {
            expect(destination).toStartWith(topic)
            expect(pages.has(destination)).toBe(true)
          }
          expect(html).toContain(`class="reference-topic-link" href="${topic}"`)
          expect(pages.get(topic)).toContain(`href="${route}"`)
        }
        if (item.slug === "array") {
          expect(text).not.toContain(
            "Decodes every JSON array element in source order."
          )
          expect(text).not.toContain("JSON配列の全要素を順に読み取ります。")
        }
        if (item.slug === "module" || item.slug === "decodeerror") {
          if (locale === "ja") {
            expect(text).toContain("直接の構築が非公開のstruct")
            expect(text).not.toContain("フィールドが非公開のstruct")
          } else expect(text).toContain("Opaque struct")
        }
        if (item.slug === "record") {
          expect(text).not.toContain(
            "An argument containing -> is a function value"
          )
          expect(text).not.toContain("->を含む引数の型は関数です")
        }
        const entry = {
          route,
          kind: item.kind === "module" ? ("module" as const) : ("api" as const),
        }
        titleLinks += assertAuthoredLibraryTitles(html, titles, entry)
        const authored = authoredLibraryArticle(html, entry)
        const mutated = html.replace(
          /(<p><a\b[^>]*href="[^"#]+">)([^<]+)(<\/a><span>：|<\/a><span>: )/u,
          "$1wrong title$3"
        )
        expect(mutated).not.toBe(html)
        expect(() =>
          assertAuthoredLibraryTitles(mutated, titles, entry)
        ).toThrow()
        for (const [, destination] of authored.matchAll(
          /<p><a\b[^>]*href="([^"]+)"/gu
        ))
          expect(pages.has(destination), `${route} -> ${destination}`).toBe(
            true
          )
        expect(html).toContain(`href="${prefix ? "" : "/ja"}${item.route}"`)
        bodies.push({
          route,
          html,
          body,
          sha256: createHash("sha256").update(html).digest("hex"),
        })
      }
    expect(bodies).toHaveLength(40)
    expect(titleLinks).toBeGreaterThanOrEqual(120)
    for (const prefix of ["", "/ja"])
      expect(
        pages.get(`${prefix}/docs/library/json/function/oneof/`)
      ).not.toContain('id="typescript-comparison"')
    const output = process.env.JSON_READER_RENDER_DIR
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
        const directory = join(output, "rendered", page.route.slice(1))
        mkdirSync(directory, { recursive: true })
        writeFileSync(join(directory, "index.html"), page.html)
      }
    }
  } finally {
    if (!process.env.JSON_READER_EVIDENCE_DIR)
      rmSync(directory, { recursive: true, force: true })
  }
})

test("only four exact JSON type readings clarify public access without changing signatures", () => {
  const all = compilerReferenceModules().flatMap((m) => m.items)
  for (const name of [
    "Json",
    "JsonParseError",
    "JsonReadError",
    "DecodeError",
  ]) {
    const item = all.find(
      (s) => s.identity === `std/json::${name}` && s.namespace === "type"
    )!
    const key = {
      identity: item.identity,
      module: item.module,
      namespace: item.namespace,
      kind: item.itemKind,
    }
    const baseline = signatureReading(
      item.signature,
      item.itemKind,
      item.typeParameters,
      item.constraints
    )
    const corrected = jsonReaderReading(key, baseline)
    expect(corrected.ja).not.toBe(baseline.ja)
    if (name !== "DecodeError") expect(corrected.en).toBe(baseline.en)
    else expect(corrected.en).not.toBe(baseline.en)
    expect(item.signature).toBe(name)
    expect(item.reading).toEqual(corrected)
    for (const wrong of [
      { ...key, identity: `${key.identity}-extra` },
      { ...key, module: "std/other" },
      { ...key, namespace: "value" },
      { ...key, kind: "type" },
    ])
      expect(jsonReaderReading(wrong, baseline)).toBe(baseline)
    expect(() =>
      jsonReaderReading(key, { en: baseline.en, ja: "changed baseline" })
    ).toThrow()
  }
})

test("JSON reading overlay leaves every other catalog identity unchanged", () => {
  const all = compilerReferenceModules().flatMap((module) => module.items)
  let english = 0,
    japanese = 0
  for (const item of all) {
    const baseline = signatureReading(
      item.signature,
      item.itemKind,
      item.typeParameters,
      item.constraints
    )
    const key = {
      identity: item.identity,
      module: item.module,
      namespace: item.namespace,
      kind: item.itemKind,
    }
    const adapted = jsonReaderReading(key, baseline)
    if (adapted.en !== baseline.en) english++
    if (adapted.ja !== baseline.ja) japanese++
    if (
      ![
        "std/json::Json",
        "std/json::JsonParseError",
        "std/json::JsonReadError",
        "std/json::DecodeError",
        "std/json::record",
      ].includes(item.identity)
    )
      expect(adapted).toBe(baseline)
  }
  expect(english).toBe(2)
  expect(japanese).toBe(5)
  const item = all.find((item) => item.identity === "std/json::DecodeError")!
  const baseline = signatureReading(
    item.signature,
    item.itemKind,
    item.typeParameters,
    item.constraints
  )
  expect(() =>
    jsonReaderReading(
      {
        identity: item.identity,
        module: item.module,
        namespace: item.namespace,
        kind: item.itemKind,
      },
      { en: "changed baseline", ja: baseline.ja }
    )
  ).toThrow()
  writeFileSync(
    join(evidence, "reading-overlay-scope.json"),
    JSON.stringify(
      {
        checked: all.length,
        changedEnglish: english,
        changedJapanese: japanese,
      },
      null,
      2
    )
  )
})

test("record callback reading corrects only its exact pair-array argument", () => {
  const item = compilerReferenceModules()
    .flatMap((m) => m.items)
    .find((x) => x.identity === "std/json::record")!
  const key = {
    identity: item.identity,
    module: item.module,
    namespace: item.namespace,
    kind: item.itemKind,
  }
  const fallback = signatureReading(
    item.signature,
    item.itemKind,
    item.typeParameters,
    item.constraints
  )
  const result = jsonReaderReading(key, fallback)
  expect(result.en).toContain("array of field-name/checker pairs")
  expect(result.ja).toContain("各組の2番目")
  expect(item.reading).toEqual(result)
  for (const wrong of [
    { ...key, identity: "std/json::oneOf" },
    { ...key, module: "std/other" },
    { ...key, namespace: "type" },
    { ...key, kind: "constructor" },
  ])
    expect(jsonReaderReading(wrong, fallback)).toBe(fallback)
  expect(() =>
    jsonReaderReading(key, { en: "changed", ja: fallback.ja })
  ).toThrow()
  expect(() =>
    jsonReaderReading(key, { en: fallback.en, ja: "changed" })
  ).toThrow()
})

test("DecodeError label preserves English and every unrelated kind label", () => {
  const all = compilerReferenceModules().flatMap((m) => m.items)
  const item = all.find((x) => x.identity === "std/json::DecodeError")!
  const symbols = [
    item,
    { ...item, identity: `${item.identity}-wrong` },
    { ...item, module: "std/other" },
    { ...item, namespace: "value" },
    { ...item, itemKind: "struct" },
    ...all.filter(
      (x) => x.itemKind === "opaque-struct" && x.identity !== item.identity
    ),
  ]
  const module = compilerReferenceModules().find(
    (m) => m.specifier === "std/json"
  )!
  const data = input([{ ...module, items: symbols }])
  const directory = mkdtempSync(join(evidence, "json20-labels-"))
  const result = renderPageClosure<Array<[string, string, string, string]>>({
    directory,
    modules: ["reference/labels"],
    timeoutMs: 120000,
    entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { itemKindLabel, referenceKindLabel } from "./reference/labels"
import { En, Ja, translate } from "./model/locale"
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [[{
    let original = itemKindLabel symbol.itemKind
    let corrected = referenceKindLabel symbol
    (translate En original, translate Ja original, translate En corrected, translate Ja corrected)
  } | symbol <- module.items] | module <- input.referenceModules]))
}`,
  })
  expect(result).toHaveLength(symbols.length)
  expect(result[0]).toEqual([
    "Opaque struct",
    "フィールドが非公開のstruct",
    "Opaque struct",
    "直接の構築が非公開のstruct",
  ])
  for (const [index, row] of result.entries()) {
    expect(row[0]).toBe(row[2])
    if (index > 0) expect(row[1]).toBe(row[3])
  }
  writeFileSync(
    join(evidence, "label-guard-results.json"),
    JSON.stringify({ checked: result.length, rows: result }, null, 2)
  )
})
