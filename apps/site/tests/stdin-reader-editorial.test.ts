import { expect, setDefaultTimeout, test } from "bun:test"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { compilerReferenceModules } from "../scripts/reference"
import {
  stdinReaderCases,
  stdinReaderExamples,
  stdinReaderRoutes,
} from "../scripts/stdin-reader"
import { stdinReaderReading } from "../scripts/stdin-reader-reading"
import { terminalReaderExamples } from "../scripts/terminal-reader"
import {
  assertAuthoredLibraryTitles,
  authoredLibraryArticle,
} from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(600_000)
const root = resolve(import.meta.dir, "../../..")
const playground = "https://seseragi.vercel.app/"
const examples = stdinReaderExamples(playground)
const evidence = resolve(
  process.env.STDIN_READER_EVIDENCE_DIR ??
    join(root, "target/stdin-reader-evidence")
)
mkdirSync(evidence, { recursive: true })
const sha = (source: string) =>
  createHash("sha256").update(source).digest("hex")
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
    playgroundUrl: playground,
    tourUrl: "",
    grammar: "",
    examples: [...examples, ...terminalReaderExamples(playground)],
    referenceModules,
  }
}

test("Stdin12 exact dispatch rejects owner, namespace, kind and Prelude/stdin lookalikes", () => {
  const modules = compilerReferenceModules().filter((m) =>
    ["std/prelude", "std/stdin", "std/console"].includes(m.specifier)
  )
  const selected = stdinReaderRoutes
    .filter((r) => r.kind !== "module")
    .map(
      (route) =>
        modules
          .flatMap((m) => m.items)
          .find(
            (symbol) =>
              symbol.identity === route.identity &&
              symbol.module === route.module &&
              symbol.namespace === route.namespace &&
              symbol.itemKind === route.kind
          )!
    )
  expect(selected).toHaveLength(11)
  expect(selected.every(Boolean)).toBe(true)
  const probes = selected.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/http" },
    {
      ...symbol,
      module: symbol.module === "std/prelude" ? "std/stdin" : "std/prelude",
    },
    { ...symbol, namespace: "wrong" },
    { ...symbol, itemKind: "wrong" },
    { ...symbol, identity: `${symbol.identity}-wrong` },
  ])
  const controls = modules
    .flatMap((m) => m.items)
    .filter(
      (symbol) =>
        !selected.some(
          (s) => s.identity === symbol.identity && s.module === symbol.module
        )
    )
  const data = input([{ ...modules[0], items: [...probes, ...controls] }])
  const directory = mkdtempSync(join(evidence, "identity-"))
  const result = renderPageClosure<{ symbols: boolean[]; modules: boolean[] }>({
    directory,
    modules: ["reference/editorial/stdin-reader/catalog"],
    timeoutMs: 240_000,
    entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { stdinReaderEditorialFor, stdinReaderModuleBlocks } from "./reference/editorial/stdin-reader/catalog"
struct Output deriving JsonEncode { symbols: Array<Bool>, modules: Array<Bool> }
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "null"
  Right input -> println (json.encodeString (Output { symbols: arrays.concat [[present (stdinReaderEditorialFor symbol) | symbol <- module.items] | module <- input.referenceModules], modules: [arrays.length (stdinReaderModuleBlocks name) > 0 | name <- ["std/stdin", "std/console", "std/prelude", "std/stdin/wrong", "stdin"]] }))
}`,
  })
  expect(result.symbols).toHaveLength(probes.length + controls.length)
  for (let i = 0; i < probes.length; i++)
    expect(result.symbols[i]).toBe(i % 6 === 0)
  for (let i = 0; i < controls.length; i++)
    expect(result.symbols[probes.length + i]).toBe(false)
  expect(result.modules).toEqual([true, false, false, false, false])
  writeFileSync(
    join(evidence, "identity-guards.json"),
    JSON.stringify(
      {
        probes,
        controls: controls.map((s) => ({
          identity: s.identity,
          module: s.module,
          namespace: s.namespace,
          kind: s.itemKind,
        })),
        result,
      },
      null,
      2
    )
  )
})

test("Stdin12 renders twenty-four complete bodies and suppresses qualified browser launches", () => {
  const modules = compilerReferenceModules().filter((m) =>
    ["std/prelude", "std/stdin", "std/process"].includes(m.specifier)
  )
  const data = input(modules)
  const renderRoutes = [
    ...stdinReaderRoutes.map((r) => r.route),
    "/docs/library/prelude/",
    "/docs/library/process/",
    "/docs/library/process/opaque-type/process/",
    "/docs/library/prelude/function/println/",
  ]
  const directory = mkdtempSync(join(evidence, "render-"))
  const result = renderPageClosure<{
    pages: Array<{ route: string; html: string }>
    routes: string[]
  }>({
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
struct Page deriving JsonEncode { route: String, html: String }
struct Output deriving JsonEncode { pages: Array<Page>, routes: Array<String> }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "null"
  Right input -> {
    let groups = referenceGroups input.referenceModules
    let sections = arrays.concat [group.sections | group <- groups]
    let pages = arrays.concat [arrays.concat [[section.overview], section.pages] | section <- sections]
    let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference, title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
    let site = SiteCatalog { home, pages, areas: [NavigationArea { id: "library", title: home.title, description: home.summary, landing: home, groups }] }
    let selected = arrays.filter (\\page -> ${renderRoutes.map((route) => `page.path == ${JSON.stringify(route)}`).join(" || ")}) pages
    println (json.encodeString (Output { pages: [Page {route: localizedRoute locale page.path, html: renderDocument input site locale page} | locale <- [En, Ja], page <- selected], routes: [localizedRoute locale page.path | locale <- [En, Ja], page <- pages] }))
  }
}`,
  })
  const renderDir = resolve(
    process.env.STDIN_READER_RENDER_DIR ?? join(evidence, "rendered-bodies")
  )
  mkdirSync(renderDir, { recursive: true })
  writeFileSync(
    join(renderDir, "all-rendered.json"),
    JSON.stringify(result.pages, null, 2)
  )
  const pages = new Map(result.pages.map((p) => [p.route, p.html]))
  const titles = new Map(result.pages.map((p) => [p.route, pageTitle(p.html)]))
  const bodies: Array<{
    route: string
    html: string
    body: string
    sha256: string
    bodySha256: string
  }> = []
  let titleLinks = 0
  for (const item of stdinReaderRoutes)
    for (const locale of ["en", "ja"] as const) {
      const prefix = locale === "en" ? "" : "/ja"
      const route = prefix + item.route
      const html = pages.get(route)!
      expect(html, route).toBeDefined()
      const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0]
      const text = plain(body)
      for (const id of [
        "stdin-task",
        "typescript-comparison",
        "seseragi-example",
        "reading-the-example",
        "run-this-program",
        "operation-rules",
        "choose-next",
      ])
        expect(body).toContain(`id="${id}"`)
      expect(body.indexOf('id="typescript-comparison"')).toBeLessThan(
        body.indexOf('id="seseragi-example"')
      )
      for (const forbidden of [
        "Missing canonical example",
        "/workspace/",
        "agent_notes",
        "stdin12-plan",
        "Stdin12",
      ])
        expect(body).not.toContain(forbidden)
      const own = readFileSync(
        join(
          root,
          `apps/site/src/reference/editorial/stdin-reader/${item.slug}/${locale}.ssrg`
        ),
        "utf8"
      )
      const other = readFileSync(
        join(
          root,
          `apps/site/src/reference/editorial/stdin-reader/${item.slug}/${locale === "en" ? "ja" : "en"}.ssrg`
        ),
        "utf8"
      )
      const arrayStrings = (source: string) =>
        [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
          (m) => JSON.parse(m[1]) as string
        )
      expect(arrayStrings(own).length).toBe(arrayStrings(other).length)
      for (const p of arrayStrings(own))
        expect(text, `${route}: own paragraph`).toContain(p)
      for (const p of arrayStrings(other))
        expect(text, `${route}: locale leakage`).not.toContain(p)
      for (const [, field, value] of own.matchAll(
        /^ {2}(\w+): ("(?:[^"\\]|\\.)*")/gmu
      ))
        if (field && !(item.kind === "module" && field === "summary"))
          expect(plain(html)).toContain(JSON.parse(value))
      const panels = [
        ...body.matchAll(
          /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
        ),
      ].map((m) => plain(m[1]))
      const native = examples.find(
        (e) => e.id === `stdin-reader-${item.example}`
      )!
      const ts = examples.find(
        (e) => e.id === `stdin-reader-${item.example}-ts`
      )!
      expect(panels[0]).toBe(ts.source)
      expect(panels[1]).toBe(native.source)
      expect(panels).toHaveLength(item.kind === "module" ? 2 : 3)
      const sample = stdinReaderCases.find((c) => c.slug === item.example)!
      const terminals = [
        ...body.matchAll(
          /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
        ),
      ].map((m) => plain(m[1]))
      if (item.example === "limit-config") {
        expect(text).toContain(
          locale === "en"
            ? "Validate the setting in Seseragi"
            : "Seseragiで設定を検証する"
        )
        expect(text).not.toContain(
          locale === "en"
            ? "Supply input and run the program"
            : "入力を渡して実行する"
        )
      }
      expect(terminals[0]).toBe(sample.output)
      expect(terminals[1]).toBe(sample.output)
      const byteRecipe = [
        "std/stdin::LineLimit",
        "std/stdin::readLineWith",
        "std/stdin::StdinLineTooLong",
      ].includes(item.identity)
      const invalidRecipe = item.identity === "std/stdin::InvalidStdinUtf8"
      expect(terminals).toHaveLength(
        item.example === "limit-config"
          ? 3
          : byteRecipe || invalidRecipe
            ? 6
            : 4
      )
      if (byteRecipe)
        expect(terminals.slice(4)).toEqual([
          "printf '😀\\néé\\nééa\\n' | node dist/entry.js",
          "Line: 😀\nLine: éé\nLine exceeds 4 bytes\n",
        ])
      if (invalidRecipe)
        expect(terminals.slice(4)).toEqual([
          "printf 'ok\\na\\300\\200\\nok\\n' | node dist/entry.js",
          "Line: ok\nInvalid UTF-8 at byte 4\nLine: ok\n",
        ])
      expect(terminals[2]).toContain(`bun ${sample.slug}.ts`)
      expect(terminals[2]).toContain(
        `seseragi build ${sample.slug}.ssrg --target process --profile release --out-dir dist`
      )
      expect(terminals[2]).toContain("node dist/entry.js")
      if (item.example !== "limit-config") {
        expect(terminals[2]).toContain("printf '")
        expect(terminals[3]).toBe(
          "printf '\\n' | node dist/entry.js\nprintf '' | node dist/entry.js"
        )
      }
      const seedUrls = [...body.matchAll(/href="([^"]+)"/gu)]
        .map((m) => new URL(plain(m[1]), data.origin))
        .filter(
          (u) =>
            u.origin === "https://seseragi.vercel.app" &&
            u.searchParams.has("source")
        )
      expect(seedUrls.map((u) => u.searchParams.get("source"))).toEqual(
        sample.portable ? [native.source] : []
      )
      if (!sample.portable) {
        expect(native.playgroundUrl).toBe("")
        expect(body).not.toContain('class="playground')
        expect(terminals[2]).toContain("entry.js")
        expect(terminals[2]).not.toContain("seseragi run")
        expect(text).toContain(
          locale === "en"
            ? "no browser Playground launch link"
            : "ブラウザーのPlaygroundで起動するリンクはありません"
        )
      } else {
        expect(text).toContain(
          locale === "en"
            ? "It does not fill Input"
            : "Input欄には自動で入力されません"
        )
        expect(text).toContain(locale === "en" ? "finite text" : "有限の文字列")
        expect(seedUrls[0]!.searchParams.has("input")).toBe(false)
      }
      if (item.kind !== "module") {
        const symbol = modules
          .flatMap((m) => m.items)
          .find(
            (s) => s.identity === item.identity && s.module === item.module
          )!
        expect(panels.at(-1)).toBe(symbol.signature)
        expect(text).toContain(
          locale === "en" ? symbol.reading.en : symbol.reading.ja
        )
        const topic = `${prefix}/docs/library/${item.module.replace("std/", "")}/`
        expect(html).toContain(`class="reference-topic-link" href="${topic}"`)
        expect(pages.get(topic)).toContain(`href="${route}"`)
        for (const [, destination] of html.matchAll(
          /class="reference-(?:previous|next)" href="([^"]+)"/gu
        ))
          expect(result.routes).toContain(destination)
      } else expect(body).toContain('id="public-api"')
      const entry = {
        route,
        kind: item.kind === "module" ? ("module" as const) : ("api" as const),
      }
      titleLinks += assertAuthoredLibraryTitles(html, titles, entry)
      const authored = authoredLibraryArticle(html, entry)
      for (const [, destination] of authored.matchAll(
        /<p><a\b[^>]*href="([^"]+)"/gu
      ))
        expect(result.routes, `${route} -> ${destination}`).toContain(
          destination
        )
      expect(html).toContain(`href="${prefix ? "" : "/ja"}${item.route}"`)
      bodies.push({
        route,
        html,
        body,
        sha256: sha(html),
        bodySha256: sha(body),
      })
    }
  expect(bodies).toHaveLength(24)
  expect(titleLinks).toBeGreaterThanOrEqual(48)
  writeFileSync(
    join(renderDir, "selected-bodies.json"),
    JSON.stringify(bodies, null, 2)
  )
  for (const page of bodies) {
    const dir = join(renderDir, "rendered", page.route.slice(1))
    mkdirSync(dir, { recursive: true })
    writeFileSync(join(dir, "index.html"), page.html)
  }
})

test("Stdin12 reading fixes are exact and leave every other fallback untouched", () => {
  const fallback = { en: "unchanged English", ja: "変更しない日本語" }
  const identities = [
    "std/prelude::Stdin",
    "std/prelude::StdinError",
    "std/stdin::StdinConfigError",
  ]
  for (const identity of identities) {
    const item = {
      identity,
      module: "std/stdin",
      namespace: "type",
      kind: "opaque-type",
    }
    const reading = stdinReaderReading(item, fallback)
    expect(reading).not.toBe(fallback)
    expect(reading.en).not.toContain("owning module's constructors")
    expect(reading.ja).not.toContain("値を作ったり取り出したり")
    for (const wrong of [
      { ...item, module: "std/prelude" },
      { ...item, namespace: "value" },
      { ...item, kind: "struct" },
      { ...item, identity: `${identity}-wrong` },
    ])
      expect(stdinReaderReading(wrong, fallback)).toBe(fallback)
  }
  const modules = compilerReferenceModules()
  let changed = 0
  let preserved = 0
  for (const module of modules)
    for (const symbol of module.items) {
      const adapted = stdinReaderReading(
        { ...symbol, kind: symbol.itemKind },
        symbol.reading
      )
      if (
        identities.includes(symbol.identity) &&
        symbol.module === "std/stdin" &&
        symbol.namespace === "type" &&
        symbol.itemKind === "opaque-type"
      ) {
        changed++
        expect(adapted.en).toContain(symbol.name)
      } else {
        preserved++
        expect(adapted).toBe(symbol.reading)
      }
    }
  expect(changed).toBe(3)
  expect(preserved).toBe(1809)
})
