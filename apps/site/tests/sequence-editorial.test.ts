import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { arrayEditorialExamples } from "../scripts/array-editorial"
import { compilerReferenceModules } from "../scripts/reference"
import {
  sequenceEditorialCases,
  sequenceEditorialExamples,
} from "../scripts/sequence-editorial"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(120_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = [
  ...sequenceEditorialExamples("https://seseragi.vercel.app/"),
  ...arrayEditorialExamples("https://seseragi.vercel.app/"),
]
function plain(html: string) {
  return html
    .replace(/<[^>]*>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}

test("twelve canonical collection snippets execute normal and empty cases", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-sequence-examples-"))
  try {
    for (const sample of sequenceEditorialCases) {
      const canonical = examples.find(
        (x) => x.id === `sequence-${sample.module}-${sample.slug}`
      )!
      const path = join(directory, `${sample.module}-${sample.slug}.ssrg`)
      writeFileSync(path, canonical.source)
      const result = spawnSync(cli, ["run", path], {
        cwd: root,
        encoding: "utf8",
        timeout: 30_000,
      })
      expect(result.status, `${sample.identity}: ${result.stderr}`).toBe(0)
      expect(result.stdout.trim(), sample.identity).toBe(sample.expected)
      expect(new URL(canonical.playgroundUrl).searchParams.get("source")).toBe(
        canonical.source
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("fourteen editorial pages render exact canonical declarations and samples in both locales", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-sequence-render-"))
  try {
    const identities = new Set<string>(
      sequenceEditorialCases.map((x) => x.identity)
    )
    const modules = compilerReferenceModules()
      .filter((x) => ["std/array", "std/list"].includes(x.specifier))
      .map((module) => ({
        ...module,
        items: module.items.filter(
          (x) =>
            identities.has(x.identity) ||
            x.identity === `${module.specifier}::length`
        ),
      }))
    expect(modules).toHaveLength(2)
    for (const module of modules) expect(module.items).toHaveLength(7)
    const input = {
      schema: 1,
      origin: "https://seseragi.example",
      playgroundUrl: "https://seseragi.vercel.app/",
      tourUrl: "https://seseragi.vercel.app/tour/",
      grammar: "",
      examples,
      referenceModules: modules,
    }
    const output = renderPageClosure<Array<{ route: string; html: string }>>({
      directory,
      modules: ["reference/catalog", "render/document"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { En, Ja, localized, localizedRoute } from "./model/locale"
import { Reference, SiteCatalog, PageDefinition, NavigationArea } from "./model/page"
import { referenceGroups } from "./reference/catalog"
import { renderDocument } from "./render/document"
struct Output deriving JsonEncode { route: String, html: String }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> {
    let groups = referenceGroups input.referenceModules
    let sections = arrays.concat [group.sections | group <- groups]
    let pages = arrays.concat [arrays.concat [[section.overview], section.pages] | section <- sections]
    let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference,
      title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
    let site = SiteCatalog { home, pages, areas: [NavigationArea {
      id: "library", title: home.title, description: home.summary, landing: home, groups
    }] }
    println (json.encodeString [Output { route: localizedRoute locale page.path,
      html: renderDocument input site locale page } | locale <- [En, Ja], page <- pages])
  }
}`,
    })
    expect(output).toHaveLength(32)
    const pages = new Map(output.map((x) => [x.route, x.html]))
    for (const prefix of ["", "/ja"]) {
      for (const module of ["array", "list"]) {
        const html = pages.get(`${prefix}/docs/library/${module}/`)!
        expect(html).toContain('id="public-api"')
        expect(html).toContain(
          module === "array"
            ? 'id="construct-and-pair"'
            : 'id="working-with-lists"'
        )
        if (module === "list") {
          expect(plain(html)).toContain(
            sequenceEditorialCases.find((x) => x.identity === "std/list::zip")!
              .expected
          )
          expect(html).toContain(
            `href="${prefix}/docs/library/array/#construct-and-pair"`
          )
        } else expect(html).toContain(`href="${prefix}/docs/library/list/"`)
        const control = pages.get(
          `${prefix}/docs/library/${module}/function/length/`
        )!
        const item = modules
          .find((x) => x.specifier === `std/${module}`)!
          .items.find((x) => x.identity === `std/${module}::length`)!
        expect(control).not.toContain('id="using-this-operation"')
        expect(plain(control)).toContain(
          prefix === "/ja" ? item.descriptionJa : item.descriptionEn
        )
        expect(plain(control)).toContain(item.signature)
      }
      for (const sample of sequenceEditorialCases) {
        const route = `${prefix}/docs/library/${sample.module}/function/${sample.slug}/`
        const html = pages.get(route)!
        const text = plain(html)
        const item = modules
          .find((x) => x.specifier === `std/${sample.module}`)!
          .items.find((x) => x.identity === sample.identity)!
        for (const anchor of [
          "using-this-operation",
          "reading-the-example",
          "operation-rules",
          "operation-mistakes",
          "canonical-declaration",
        ])
          expect(html).toContain(`id="${anchor}"`)
        expect(text).toContain(item.signature)
        expect(text).toContain(sample.expected)
        expect(text).not.toContain("A public standard-library item.")
        expect(text).not.toContain("標準ライブラリの公開項目です。")
        const canonical = examples.find(
          (x) => x.id === `sequence-${sample.module}-${sample.slug}`
        )!
        const code = html.match(
          /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
        )?.[1]
        expect(plain(code ?? ""), route).toBe(canonical.source)
        expect(html).toContain(
          `href="${prefix}/docs/library/${sample.module}/"`
        )
        expect(
          pages.get(`${prefix}/docs/library/${sample.module}/`)!
        ).toContain(`href="${route}"`)
        expect(html).not.toContain("Missing canonical example")
        const localeFields = ["ja", "en"].map((locale) =>
          [
            ...readFileSync(
              join(
                root,
                `apps/site/src/reference/editorial/${sample.module}/${sample.slug}/${locale}.ssrg`
              ),
              "utf8"
            ).matchAll(/^ {2}(\w+): /gm),
          ].map((x) => x[1])
        )
        expect(localeFields[0]).toEqual(localeFields[1])
      }
    }
    if (process.env.SEQUENCE_EDITORIAL_RENDER_DIR) {
      mkdirSync(process.env.SEQUENCE_EDITORIAL_RENDER_DIR, { recursive: true })
      for (const page of output)
        writeFileSync(
          join(
            process.env.SEQUENCE_EDITORIAL_RENDER_DIR,
            `${page.route.replaceAll("/", "_")}.html`
          ),
          page.html
        )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("each selected identity exists once and wrong module, namespace, kind and symbol remain untouched", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-sequence-identities-"))
  try {
    const all = compilerReferenceModules()
    const modules = ["array", "list"].map((name) => {
      const module = all.find((x) => x.specifier === `std/${name}`)!
      const items = sequenceEditorialCases
        .filter((x) => x.module === name)
        .map((sample) => {
          const matches = module.items.filter(
            (x) =>
              x.identity === sample.identity &&
              x.namespace === "value" &&
              x.itemKind === "function"
          )
          expect(matches, sample.identity).toHaveLength(1)
          return matches[0]!
        })
      const first = items[0]!
      return {
        ...module,
        items: [
          ...items,
          { ...first, identity: `std/${name}::length` },
          { ...first, namespace: "type" },
          { ...first, itemKind: "constructor" },
          { ...first, module: "std/text" },
          { ...first, identity: "std/text::empty" },
        ],
      }
    })
    const input = {
      schema: 1,
      origin: "",
      playgroundUrl: "",
      tourUrl: "",
      grammar: "",
      examples: [],
      referenceModules: modules,
    }
    const result = renderPageClosure<boolean[]>({
      directory,
      modules: ["reference/editorial/catalog"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { ReferenceSymbol, decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor } from "./reference/editorial/catalog"
fn hasEditorial symbol: ReferenceSymbol -> Bool = match editorialFor symbol {
  Just _ -> True
  Nothing -> False
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [[hasEditorial symbol | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(result).toEqual([
      ...Array(6).fill(true),
      ...Array(5).fill(false),
      ...Array(6).fill(true),
      ...Array(5).fill(false),
    ])
    const dispatch = readFileSync(
      join(root, "apps/site/src/reference/editorial/catalog.ssrg"),
      "utf8"
    )
    const mapped = [
      ...dispatch.matchAll(/"(std\/(?:array|list)::[^"]+)" -> Just/g),
    ].map((x) => x[1])
    expect(new Set(mapped).size).toBe(mapped.length)
    for (const sample of sequenceEditorialCases)
      expect(mapped.filter((x) => x === sample.identity)).toHaveLength(1)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
