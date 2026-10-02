import { expect, setDefaultTimeout, test } from "bun:test"
import { createHash } from "node:crypto"
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { filesystemReaderExamples } from "../scripts/filesystem-reader"
import { compilerReferenceModules } from "../scripts/reference"
import {
  terminalReaderCases,
  terminalReaderExamples,
  terminalReaderRoutes,
} from "../scripts/terminal-reader"
import { terminalReaderReading } from "../scripts/terminal-reader-reading"
import {
  assertAuthoredLibraryTitles,
  authoredLibraryArticle,
} from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(600_000)
const root = resolve(import.meta.dir, "../../..")
const playground = "https://seseragi.vercel.app/"
const examples = terminalReaderExamples(playground)
const evidence = resolve(
  process.env.TERMINAL_READER_EVIDENCE_DIR ??
    join(root, "target/terminal-reader-evidence")
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
    examples: [...examples, ...filesystemReaderExamples(playground)],
    referenceModules,
  }
}

test("Terminal9 exact dispatch rejects owner, namespace, kind and gated Console lookalikes", () => {
  const modules = compilerReferenceModules().filter((m) =>
    ["std/prelude", "std/process", "std/console"].includes(m.specifier)
  )
  const selected = terminalReaderRoutes
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
  expect(selected).toHaveLength(8)
  expect(selected.every(Boolean)).toBe(true)
  const probes = selected.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/http" },
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
    modules: ["reference/editorial/terminal-reader/catalog"],
    timeoutMs: 240_000,
    entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { terminalReaderEditorialFor, terminalReaderModuleBlocks } from "./reference/editorial/terminal-reader/catalog"
struct Output deriving JsonEncode { symbols: Array<Bool>, modules: Array<Bool> }
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "null"
  Right input -> println (json.encodeString (Output { symbols: arrays.concat [[present (terminalReaderEditorialFor symbol) | symbol <- module.items] | module <- input.referenceModules], modules: [arrays.length (terminalReaderModuleBlocks name) > 0 | name <- ["std/process", "std/console", "std/prelude", "std/process/wrong", "process"]] }))
}`,
  })
  expect(result.symbols).toHaveLength(probes.length + controls.length)
  for (let i = 0; i < probes.length; i++)
    expect(result.symbols[i]).toBe(i % 5 === 0)
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

test("Terminal9 renders eighteen complete bodies and suppresses every Process browser launch", () => {
  const modules = compilerReferenceModules().filter((m) =>
    ["std/prelude", "std/process", "std/path"].includes(m.specifier)
  )
  const data = input(modules)
  const renderRoutes = [
    ...terminalReaderRoutes.map((r) => r.route),
    "/docs/library/prelude/",
    "/docs/library/path/",
    "/docs/library/path/function/render/",
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
    process.env.TERMINAL_READER_RENDER_DIR ?? join(evidence, "rendered-bodies")
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
  for (const item of terminalReaderRoutes)
    for (const locale of ["en", "ja"] as const) {
      const prefix = locale === "en" ? "" : "/ja"
      const route = prefix + item.route
      const html = pages.get(route)!
      expect(html, route).toBeDefined()
      const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0]
      const text = plain(body)
      for (const id of [
        "terminal-task",
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
        "terminal17",
        "Terminal9",
      ])
        expect(body).not.toContain(forbidden)
      const own = readFileSync(
        join(
          root,
          `apps/site/src/reference/editorial/terminal-reader/${item.slug}/${locale}.ssrg`
        ),
        "utf8"
      )
      const other = readFileSync(
        join(
          root,
          `apps/site/src/reference/editorial/terminal-reader/${item.slug}/${locale === "en" ? "ja" : "en"}.ssrg`
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
        (e) => e.id === `terminal-reader-${item.example}`
      )!
      const ts = examples.find(
        (e) => e.id === `terminal-reader-${item.example}-ts`
      )!
      expect(panels[0]).toBe(ts.source)
      expect(panels[1]).toBe(native.source)
      expect(panels).toHaveLength(item.kind === "module" ? 2 : 3)
      const sample = terminalReaderCases.find((c) => c.slug === item.example)!
      const terminals = [
        ...body.matchAll(
          /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
        ),
      ].map((m) => plain(m[1]))
      expect(terminals[0]).toBe(sample.output)
      expect(terminals[1]).toBe(sample.output)
      expect(terminals).toHaveLength(
        ["std/process::environment", "std/process::arguments"].includes(
          item.identity
        )
          ? 5
          : 3
      )
      expect(terminals[2]).toContain(
        sample.portable
          ? `seseragi run ${sample.slug}.ssrg --target process`
          : `seseragi build ${sample.slug}.ssrg --target process --profile release --out-dir dist`
      )
      if (item.identity === "std/process::environment") {
        expect(terminals.slice(3)).toEqual([
          "(unset SESERAGI_DOCS_MODE; node dist/entry.js)\nSESERAGI_DOCS_MODE= node dist/entry.js",
          "Mode: preview (default)\nMode: []\n",
        ])
      }
      if (item.identity === "std/process::arguments") {
        expect(terminals.slice(3)).toEqual([
          "node dist/entry.js",
          "No arguments\n",
        ])
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
            ? "Process programs do not run in the browser Playground"
            : "Processを使うプログラムはブラウザーのPlaygroundでは動かない"
        )
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
  expect(bodies).toHaveLength(18)
  expect(titleLinks).toBeGreaterThanOrEqual(36)
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

test("Terminal9 reading fixes are exact and leave every other fallback untouched", () => {
  const fallback = { en: "unchanged English", ja: "変更しない日本語" }
  const identities = ["std/process::Process", "std/process::ProcessError"]
  for (const identity of identities) {
    const item = {
      identity,
      module: "std/process",
      namespace: "type",
      kind: "opaque-type",
    }
    const reading = terminalReaderReading(item, fallback)
    expect(reading).not.toBe(fallback)
    expect(reading.en).not.toContain("owning module's constructors")
    expect(reading.ja).not.toContain("値を作ったり取り出したり")
    for (const wrong of [
      { ...item, module: "std/prelude" },
      { ...item, namespace: "value" },
      { ...item, kind: "struct" },
      { ...item, identity: `${identity}-wrong` },
    ])
      expect(terminalReaderReading(wrong, fallback)).toBe(fallback)
  }
  const modules = compilerReferenceModules()
  let changed = 0
  let preserved = 0
  for (const module of modules)
    for (const symbol of module.items) {
      const adapted = terminalReaderReading(
        { ...symbol, kind: symbol.itemKind },
        symbol.reading
      )
      if (
        identities.includes(symbol.identity) &&
        symbol.module === "std/process" &&
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
  expect(changed).toBe(2)
  expect(preserved).toBeGreaterThan(1800)
})
