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
import { apiCorrectionExamples } from "../scripts/api-corrections"
import {
  bytesReaderCases,
  bytesReaderExamples,
  bytesReaderRoutes,
} from "../scripts/bytes-reader"
import { bytesReaderReading } from "../scripts/bytes-reader-reading"
import { charTextExamples } from "../scripts/char-text-reader"
import { compilerReferenceModules } from "../scripts/reference"
import { signatureReading } from "../scripts/signature-reading"
import { textEditorialExamples } from "../scripts/text-editorial"
import { unicodeReaderExamples } from "../scripts/unicode-reader"
import {
  assertAuthoredLibraryTitles,
  authoredLibraryArticle,
} from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(600_000)
const root = resolve(import.meta.dir, "../../..")
const examples = bytesReaderExamples("https://seseragi.vercel.app/")
const evidence = resolve(
  process.env.BYTES_READER_EVIDENCE_DIR ??
    join(root, "target/bytes-reader-evidence")
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
    examples: [
      ...examples,
      ...textEditorialExamples("https://seseragi.vercel.app/"),
      ...charTextExamples("https://seseragi.vercel.app/"),
      ...apiCorrectionExamples("https://seseragi.vercel.app/"),
      ...unicodeReaderExamples("https://seseragi.vercel.app/"),
    ],
    referenceModules,
  }
}

test("Bytes20 guards exact identity, owner, namespace and kind while preserving prior text content", () => {
  const all = compilerReferenceModules()
  const owners = ["std/bytes", "std/text", "std/bytes/hex", "std/bytes/base64"]
  const modules = all.filter((m) => owners.includes(m.specifier))
  const leaves = bytesReaderRoutes.filter((x) => x.kind !== "module")
  expect(leaves).toHaveLength(17)
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
  const controls = modules
    .flatMap((m) => m.items)
    .filter((s) => !symbols.some((x) => x.identity === s.identity))
  const data = input([{ ...modules[0], items: [...probes, ...controls] }])
  const directory = mkdtempSync(join(evidence, "seseragi-bytes20-guards-"))
  try {
    const result = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/bytes-reader/catalog",
      ],
      timeoutMs: 240_000,
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor, moduleEditorial } from "./reference/editorial/catalog"
import { bytesReaderEditorialFor, bytesReaderModuleBlocks } from "./reference/editorial/bytes-reader/catalog"
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [
    arrays.concat [[(present (bytesReaderEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules],
    [(arrays.length (bytesReaderModuleBlocks name) > 0, arrays.length (moduleEditorial name) > 0) | name <- ["std/bytes", "std/bytes/hex", "std/bytes/base64", "std/text", "std/bytes-extra", "std/bytes::Bytes"]]
  ]))
}`,
    })
    expect(result).toHaveLength(probes.length + controls.length + 6)
    for (let i = 0; i < probes.length; i++)
      expect(result[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    for (let i = 0; i < controls.length; i++) {
      const row = result[probes.length + i]
      expect(row[0]).toBe(false)
      if (controls[i].module !== "std/text") expect(row[1]).toBe(false)
      if (controls[i].identity === "std/text::contains")
        expect(row[1]).toBe(true)
    }
    expect(result.slice(-6)).toEqual([
      [true, true],
      [true, true],
      [true, true],
      [false, true],
      [false, false],
      [false, false],
    ])
  } finally {
    if (!process.env.BYTES_READER_EVIDENCE_DIR)
      rmSync(directory, { recursive: true, force: true })
  }
})

test("all forty bilingual bodies keep the task, exact runtime-labelled sources, outputs and navigation", () => {
  const modules = compilerReferenceModules().filter((m) =>
    ["std/bytes", "std/bytes/hex", "std/bytes/base64", "std/text"].includes(
      m.specifier
    )
  )
  const data = input(modules)
  const directory = mkdtempSync(join(evidence, "seseragi-bytes20-render-"))
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
    const earlyOutput = process.env.BYTES_READER_RENDER_DIR
    if (earlyOutput) {
      mkdirSync(earlyOutput, { recursive: true })
      writeFileSync(
        join(earlyOutput, "all-rendered.json"),
        JSON.stringify(result, null, 2)
      )
      const selected = result
        .filter((page) =>
          bytesReaderRoutes.some(
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
    for (const item of bytesReaderRoutes)
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
              `apps/site/src/reference/editorial/bytes-reader/${item.slug}/${lang}.ssrg`
            ),
            "utf8"
          )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const own = copy(locale),
          other = copy(locale === "en" ? "ja" : "en")
        // The narrow-layout regression is a prose run, not an inline code
        // chip. Preserve the complete authored note in paragraph markup.
        const nodeNote = JSON.parse(
          own.match(/^ {2}nodeNote: ("(?:[^"\\]|\\.)*")/mu)![1]
        ) as string
        const nodeParagraph = body.match(
          /<h2 id="node-alternative">[^<]*<\/h2><p><span>([^<]*)<\/span><\/p>/u
        )
        expect(nodeParagraph, `${route}: Node prose structure`).not.toBeNull()
        expect(plain(nodeParagraph![1])).toBe(nodeNote)
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
          (e) => e.id === `bytes-reader-${item.example}`
        )!
        const ts = examples.find(
          (e) => e.id === `bytes-reader-${item.example}-ts`
        )!
        expect(panels[0]).toBe(ts.source)
        expect(panels[1]).toBe(native.source)
        const hasNode = ![
          "validate-byte-input",
          "inspect-binary-payload",
          "explain-utf8-failure",
          "preview-damaged-text",
        ].includes(item.example)
        if (hasNode) {
          const node = examples.find(
            (e) => e.id === `bytes-reader-${item.example}-node-ts`
          )!
          expect(panels[2]).toBe(node.source)
          expect(text).toContain("Node 24.19.0")
        }
        expect(panels).toHaveLength(
          2 + (hasNode ? 1 : 0) + (item.kind === "module" ? 0 : 1)
        )
        expect(text).toContain("Bun 1.3.9")
        const sample = bytesReaderCases.find((c) => c.slug === item.example)!
        const terminals = [
          ...body.matchAll(
            /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
          ),
        ].map((m) => plain(m[1]))
        expect(terminals).toEqual([
          sample.typescriptOutput,
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
        pages.get(`${prefix}/docs/library/bytes/function/slice/`)
      ).not.toContain('id="typescript-comparison"')
    const output = process.env.BYTES_READER_RENDER_DIR
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
    if (!process.env.BYTES_READER_EVIDENCE_DIR)
      rmSync(directory, { recursive: true, force: true })
  }
})

test("Bytes20 preserves canonical signatures and limits declaration reading to four exact error types", () => {
  const modules = compilerReferenceModules()
  const artifact = JSON.parse(
    readFileSync(
      join(
        root,
        "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
      ),
      "utf8"
    )
  )
  for (const route of bytesReaderRoutes) {
    const module = modules.find((m) => m.specifier === route.module)!
    expect(module.targets).toEqual(["process", "browser"])
    if (route.kind === "module") continue
    const canonical = artifact.modules
      .find((m: { specifier: string }) => m.specifier === route.module)
      .items.find((x: { identity: string }) => x.identity === route.identity)
    const symbol = module.items.find((x) => x.identity === route.identity)!
    expect(symbol.signature).toBe(canonical.signature)
    expect(symbol.namespace).toBe(canonical.namespace)
    expect(symbol.itemKind).toBe(canonical.kind)
  }
  const selected = [
    "std/bytes::ByteError",
    "std/text::Utf8DecodeError",
    "std/bytes/hex::HexDecodeError",
    "std/bytes/base64::Base64DecodeError",
  ]
  for (const identity of selected) {
    const item = modules
      .flatMap((m) => m.items)
      .find((x) => x.identity === identity)!
    const key = {
      identity,
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
    const updated = bytesReaderReading(key, fallback)
    expect(updated).not.toEqual(fallback)
    expect(item.reading).toEqual(updated)
    for (const bad of [
      { ...key, module: "std/http" },
      { ...key, namespace: "value" },
      { ...key, kind: "function" },
      { ...key, identity: identity + "-wrong" },
    ])
      expect(bytesReaderReading(bad, fallback)).toBe(fallback)
  }
  for (const item of modules
    .flatMap((m) => m.items)
    .filter((x) => !selected.includes(x.identity))) {
    const fallback = { en: "unchanged", ja: "変更なし" }
    expect(
      bytesReaderReading(
        {
          identity: item.identity,
          module: item.module,
          namespace: item.namespace,
          kind: item.itemKind,
        },
        fallback
      )
    ).toBe(fallback)
  }
})
