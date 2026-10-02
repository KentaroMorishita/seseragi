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
  bytesInspectionCases,
  bytesInspectionExamples,
  bytesInspectionRoutes,
} from "../scripts/bytes-inspection"
import { bytesInspectionReading } from "../scripts/bytes-inspection-reading"
import { bytesReaderExamples } from "../scripts/bytes-reader"
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
const playground = "https://seseragi.vercel.app/"
const examples = bytesInspectionExamples(playground)
const evidence = resolve(
  process.env.BYTES_INSPECTION_EVIDENCE_DIR ??
    join(root, "target/bytes-inspection-evidence")
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
    playgroundUrl: playground,
    tourUrl: "",
    grammar: "",
    examples: [...examples, ...bytesReaderExamples(playground)],
    referenceModules,
  }
}

test("Bytes inspection11 guards exact owner, namespace, kind and identity and retains same-owner controls", () => {
  const modules = compilerReferenceModules().filter((m) =>
    ["std/bytes"].includes(m.specifier)
  )
  const symbols = bytesInspectionRoutes.map(
    (route) =>
      modules
        .flatMap((m) => m.items)
        .find(
          (symbol) =>
            symbol.module === route.module &&
            symbol.identity === route.identity &&
            symbol.namespace === route.namespace &&
            symbol.itemKind === route.kind
        )!
  )
  expect(symbols).toHaveLength(11)
  const probes = symbols.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/http" },
    {
      ...symbol,
      module: "std/text",
    },
    { ...symbol, namespace: "wrong" },
    { ...symbol, itemKind: "wrong" },
    { ...symbol, identity: `${symbol.identity}-wrong` },
  ])
  const controls = modules
    .flatMap((m) => m.items)
    .filter(
      (symbol) =>
        !symbols.some((selected) => selected.identity === symbol.identity)
    )
  const data = input([{ ...modules[0], items: [...probes, ...controls] }])
  const directory = mkdtempSync(
    join(evidence, "seseragi-bytes-inspection11-guards-")
  )
  try {
    const result = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/bytes-inspection/catalog",
      ],
      timeoutMs: 240_000,
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor } from "./reference/editorial/catalog"
import { bytesInspectionEditorialFor } from "./reference/editorial/bytes-inspection/catalog"
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(data))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [[(present (bytesInspectionEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(result).toHaveLength(probes.length + controls.length)
    for (let i = 0; i < probes.length; i++)
      expect(result[i]).toEqual(i % 6 === 0 ? [true, true] : [false, false])
    for (let i = 0; i < controls.length; i++)
      expect(result[probes.length + i][0]).toBe(false)
    for (const identity of ["std/bytes::ByteError", "std/bytes::fromInts"]) {
      const index = controls.findIndex((symbol) => symbol.identity === identity)
      expect(index).toBeGreaterThanOrEqual(0)
      expect(result[probes.length + index][1]).toBe(true)
    }
    writeFileSync(
      join(evidence, "identity-guards.json"),
      JSON.stringify(
        {
          probes: probes.map((p) => ({
            identity: p.identity,
            module: p.module,
            namespace: p.namespace,
            itemKind: p.itemKind,
          })),
          controls: controls.map((p) => p.identity),
          result,
        },
        null,
        2
      )
    )
  } finally {
    if (!process.env.BYTES_INSPECTION_EVIDENCE_DIR)
      rmSync(directory, { recursive: true, force: true })
  }
})

test("all twenty-two bilingual bodies keep the task, exact runtime-labelled sources, outputs and navigation", () => {
  const modules = compilerReferenceModules().filter((m) =>
    ["std/bytes"].includes(m.specifier)
  )
  const data = input(modules)
  const directory = mkdtempSync(
    join(evidence, "seseragi-bytes-inspection11-render-")
  )
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
    const earlyOutput = process.env.BYTES_INSPECTION_RENDER_DIR
    if (earlyOutput) {
      mkdirSync(earlyOutput, { recursive: true })
      writeFileSync(
        join(earlyOutput, "all-rendered.json"),
        JSON.stringify(result, null, 2)
      )
      const selected = result
        .filter((page) =>
          bytesInspectionRoutes.some(
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
    for (const item of bytesInspectionRoutes)
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
              `apps/site/src/reference/editorial/bytes-inspection/${item.slug}/${lang}.ssrg`
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
          if (field)
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
          (e) => e.id === `bytes-inspection-${item.example}`
        )!
        const ts = examples.find(
          (e) => e.id === `bytes-inspection-${item.example}-ts`
        )!
        expect(panels[0]).toBe(ts.source)
        expect(panels[1]).toBe(native.source)
        expect(panels).toHaveLength(3)
        expect(text).toContain("Node 24.19.0")
        expect(text).toContain("Bun 1.3.9")
        const sample = bytesInspectionCases.find(
          (c) => c.slug === item.example
        )!
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
        {
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
          kind: "api" as const,
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
    expect(bodies).toHaveLength(22)
    expect(titleLinks).toBeGreaterThanOrEqual(66)
    const preservation = JSON.parse(
      readFileSync(
        join(
          root,
          "apps/site/tests/fixtures/bytes-inspection/unselected-controls.json"
        ),
        "utf8"
      )
    ) as Array<{ route: string; kind: "api" | "module"; articleSha256: string }>
    expect(preservation).toHaveLength(30)
    const checkedControls = preservation.map((control) => {
      expect(
        bytesInspectionRoutes.some(
          (item) =>
            control.route === item.route || control.route === `/ja${item.route}`
        )
      ).toBe(false)
      const html = pages.get(control.route)
      expect(html, control.route).toBeDefined()
      const article = authoredLibraryArticle(html!, {
        route: control.route,
        kind: control.kind,
      })
      expect(article, control.route).toBeDefined()
      const articleSha256 = createHash("sha256").update(article!).digest("hex")
      expect(articleSha256, control.route).toBe(control.articleSha256)
      return { ...control, currentArticleSha256: articleSha256 }
    })
    writeFileSync(
      join(evidence, "unselected-article-preservation.json"),
      JSON.stringify(checkedControls, null, 2)
    )
    const output = process.env.BYTES_INSPECTION_RENDER_DIR
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
    if (!process.env.BYTES_INSPECTION_EVIDENCE_DIR)
      rmSync(directory, { recursive: true, force: true })
  }
})

test("Bytes inspection11 retains canonical signatures and targets and narrows only the BytesSliceError reading", () => {
  const modules = compilerReferenceModules()
  const canonical = JSON.parse(
    readFileSync(
      join(
        root,
        "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
      ),
      "utf8"
    )
  )
  for (const route of bytesInspectionRoutes) {
    const module = modules.find((m) => m.specifier === route.module)!
    const symbol = module.items.find(
      (item) => item.identity === route.identity
    )!
    const original = canonical.modules
      .find((m: { specifier: string }) => m.specifier === route.module)
      .items.find(
        (item: { identity: string }) => item.identity === route.identity
      )
    expect(module.targets).toEqual(["process", "browser"])
    expect(symbol.signature).toBe(original.signature)
    expect(symbol.namespace).toBe(original.namespace)
    expect(symbol.itemKind).toBe(original.kind)
  }
})

test("only the exact BytesSliceError identity/owner/type/kind receives the new public-pattern reading", () => {
  const modules = compilerReferenceModules()
  const all = modules.flatMap((module) => module.items)
  const item = all.find(
    (symbol) => symbol.identity === "std/bytes::BytesSliceError"
  )!
  expect(item).toBeDefined()
  expect(item.itemKind).toBe("opaque-type")
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
  const updated = bytesInspectionReading(key, fallback)
  expect(updated).not.toEqual(fallback)
  expect(item.reading).toEqual(updated)
  for (const token of ["InvalidByteRange", "start", "end", "length"]) {
    expect(updated.en).toContain(token)
    expect(updated.ja).toContain(token)
  }
  for (const bad of [
    { ...key, module: "std/text" },
    { ...key, namespace: "value" },
    { ...key, kind: "constructor" },
    { ...key, identity: `${key.identity}-wrong` },
  ])
    expect(bytesInspectionReading(bad, fallback)).toBe(fallback)
  for (const symbol of all.filter(
    (symbol) => symbol.identity !== item.identity
  )) {
    const unchanged = { en: "unchanged", ja: "変更なし" }
    expect(
      bytesInspectionReading(
        {
          identity: symbol.identity,
          module: symbol.module,
          namespace: symbol.namespace,
          kind: symbol.itemKind,
        },
        unchanged
      )
    ).toBe(unchanged)
  }
})

test("all pre-existing declaration readings except the approved BytesSliceError explanation remain byte-identical", () => {
  const baseline = JSON.parse(
    readFileSync(
      join(
        root,
        "apps/site/tests/fixtures/bytes-inspection/unchanged-reading-hashes.json"
      ),
      "utf8"
    )
  ) as Array<{
    identity: string
    module: string
    namespace: string
    kind: string
    readingSha256: string
  }>
  const all = compilerReferenceModules()
    .flatMap((module) => module.items)
    .filter((item) => item.identity !== "std/bytes::BytesSliceError")
  expect(baseline).toHaveLength(all.length)
  for (const item of all) {
    const expected = baseline.find(
      (old) =>
        old.identity === item.identity &&
        old.module === item.module &&
        old.namespace === item.namespace &&
        old.kind === item.itemKind
    )
    expect(expected).toBeDefined()
    expect(
      createHash("sha256").update(JSON.stringify(item.reading)).digest("hex")
    ).toBe(expected!.readingSha256)
  }
})
