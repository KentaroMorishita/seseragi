import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  apiCorrectionCases,
  apiCorrectionExamples,
} from "../scripts/api-corrections"
import { compilerReferenceModules } from "../scripts/reference"
import {
  setEditorialCases,
  setEditorialExamples,
} from "../scripts/set-editorial"
import { assertReferenceLinkTitles, pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = [
  ...setEditorialExamples("https://seseragi.vercel.app/"),
  ...apiCorrectionExamples("https://seseragi.vercel.app/"),
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

test("twelve published Set pairs execute normal, empty, duplicate and missing-key cases", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-set-examples-"))
  try {
    for (const sample of setEditorialCases) {
      const canonical = examples.find((x) => x.id === `set-${sample.slug}`)!
      const path = join(directory, `${sample.slug}.ssrg`)
      writeFileSync(path, canonical.source)
      const result = spawnSync(cli, ["run", path], {
        cwd: root,
        encoding: "utf8",
        timeout: 15_000,
      })
      expect(result.status, `${sample.name}: ${result.stderr}`).toBe(0)
      expect(result.stdout.trim(), sample.name).toBe(sample.expected)
      const comparison = examples.find((x) => x.id === `set-${sample.slug}-ts`)!
      const tsPath = join(directory, `${sample.slug}.ts`)
      writeFileSync(tsPath, comparison.source)
      const ts = spawnSync(process.execPath, [tsPath], {
        cwd: directory,
        encoding: "utf8",
        timeout: 15_000,
      })
      expect(ts.status, ts.stderr).toBe(0)
      expect(ts.stdout).toBe(result.stdout)
      expect(comparison.playgroundUrl).toBe("")
      expect(new URL(canonical.playgroundUrl).searchParams.get("source")).toBe(
        canonical.source
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("identity overlay renders thirteen Set pages in both locales and preserves declarations", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-set-render-"))
  try {
    const allModules = compilerReferenceModules()
    const sourceModule = allModules.find((x) => x.specifier === "std/set")!
    const identities = new Set<string>(setEditorialCases.map((x) => x.identity))
    const module = {
      ...sourceModule,
      items: sourceModule.items.filter(
        (x) =>
          identities.has(x.identity) ||
          ["std/set::toArray", "std/set::toList"].includes(x.identity)
      ),
    }
    expect(module.items).toHaveLength(14)
    for (const item of module.items) {
      expect(item.namespace).toBe("value")
      expect(item.itemKind).toBe("function")
    }
    // Every canonical function in this family is now authored. Keep the
    // global no-editorial check on real, still-unclaimed Stream functions.
    const stream = allModules.find((x) => x.specifier === "std/stream")!
    const fallbackModule = {
      ...stream,
      items: stream.items.filter((x) =>
        ["std/stream::filterMap", "std/stream::empty"].includes(x.identity)
      ),
    }
    expect(fallbackModule.items).toHaveLength(2)
    for (const item of fallbackModule.items) {
      expect(item.module).toBe("std/stream")
      expect(item.namespace).toBe("value")
      expect(item.itemKind).toBe("function")
    }
    const input = {
      schema: 1,
      origin: "https://seseragi.example",
      playgroundUrl: "https://seseragi.vercel.app/",
      tourUrl: "https://seseragi.vercel.app/tour/",
      grammar: "",
      examples,
      referenceModules: [module, fallbackModule],
    }
    const output = renderPageClosure<Array<{ route: string; html: string }>>({
      directory,
      timeoutMs: 180_000,
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
    expect(output).toHaveLength(36) // Set articles plus two generic Stream controls, each in EN/JA.
    const pages = new Map(output.map((x) => [x.route, x.html]))
    for (const prefix of ["", "/ja"]) {
      const moduleHtml = pages.get(`${prefix}/docs/library/set/`)!
      expect(moduleHtml).toContain('id="working-with-sets"')
      const titles = new Map(
        output.map((page) => [page.route, pageTitle(page.html)])
      )
      // These two published Set destinations are outside this legacy
      // rendering fixture; retain exact module-link checks against their
      // canonical compiler names rather than dropping title coverage.
      for (const symbol of sourceModule.items.filter((x) =>
        ["std/set::filter", "std/set::isEmpty"].includes(x.identity)
      )) {
        titles.set(
          `${prefix}/docs/library/set/function/${symbol.name.toLowerCase()}/`,
          symbol.name
        )
      }
      expect(
        assertReferenceLinkTitles(
          `${moduleHtml.split('id="set-api-index"')[0]}</article>`,
          titles,
          `${prefix}/docs/library/set/`
        )
      ).toBe(16)
      expect(moduleHtml).toContain('id="public-api"')
      for (const name of ["toArray", "toList"]) {
        const control = apiCorrectionCases.find(
          (x) => x.identity === `std/set::${name}`
        )!
        expect(
          plain(
            pages.get(
              `${prefix}/docs/library/set/function/${name.toLowerCase()}/`
            )!
          )
        ).toContain(control.expected)
      }
      for (const sample of setEditorialCases) {
        const route = `${prefix}/docs/library/set/function/${sample.slug}/`
        const html = pages.get(route)!
        const text = plain(html)
        // Check the complete authored leaf body, not only the module selector.
        // Generated declarations/index labels follow this explicit boundary.
        expect(
          assertReferenceLinkTitles(
            `${html.split('id="canonical-declaration"')[0]}</article>`,
            titles,
            route
          )
        ).toBe(1)
        const item = module.items.find((x) => x.identity === sample.identity)!
        expect(html).toContain('id="using-this-operation"')
        expect(html).toContain('id="operation-rules"')
        expect(html).toContain('id="typescript-comparison"')
        expect(text).toContain(item.signature)
        expect(text).not.toContain("A public standard-library item.")
        expect(text).not.toContain("標準ライブラリの公開項目です。")
        expect(text).toContain(sample.expected)
        const canonical = examples.find((x) => x.id === `set-${sample.slug}`)!
        const code = html.match(
          /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
        )?.[1]
        expect(plain(code ?? ""), route).toBe(canonical.source)
        expect(html).toContain(`href="${prefix}/docs/library/set/"`)
        expect(moduleHtml).toContain(`href="${route}"`)
        expect(html).not.toContain("Missing canonical example")
        const comparison = examples.find(
          (x) => x.id === `set-${sample.slug}-ts`
        )!
        // Syntax and output help must describe the displayed programs only.
        const notes: Array<[boolean, string, string]> = [
          [
            canonical.source.includes(": sets.Set<Int>"),
            "Set<Int> contains distinct integers.",
            "Set<Int>は異なる整数の集合です。",
          ],
          [
            canonical.source.includes("["),
            "Square brackets make an Array.",
            "角括弧でArrayを作ります。",
          ],
          [
            canonical.source.includes("`["),
            "A leading backtick makes `[3, 1, 3, 2] a List.",
            "先頭にバッククォートを付けた`[3, 1, 3, 2]はListです。",
          ],
          [
            canonical.source.includes("\nfn parity "),
            ": gives the parameter type, -> gives the result type",
            ":の後が引数の型、->の後が結果の型",
          ],
          [
            canonical.source.includes("json.encodeString"),
            "sets.toArray reads members in insertion order; json.encodeString",
            "sets.toArrayが要素を挿入順に読み、json.encodeString",
          ],
          [
            canonical.source.includes(": sets.Set<Int>"),
            "The Set<Int> annotation supplies the empty Set's element type.",
            "Set<Int>という型注釈が空のSetの要素型を指定します。",
          ],
          [
            canonical.source.includes("sets.empty<Int>"),
            "In sets.empty<Int> (), <Int> supplies the empty Set's element type.",
            "sets.empty<Int> ()の<Int>が空のSetの要素型を指定します。",
          ],
          [
            canonical.source.includes("sets.empty"),
            "() supplies Unit, meaning no additional input.",
            "()は追加の情報を持たないUnitの引数です。",
          ],
          [
            comparison.source.includes("[..."),
            "Spreading a Set inside [...] collects its members into an array.",
            "[...]の中でSetを展開すると、要素を配列に集められます。",
          ],
          [
            comparison.source.includes("JSON.stringify"),
            "JSON.stringify formats the array as JSON text.",
            "JSON.stringifyが配列をJSONの文字列にします。",
          ],
        ]
        for (const [shown, enNote, jaNote] of notes) {
          const note = prefix ? jaNote : enNote
          expect(text.includes(note), `${route}: ${note}`).toBe(shown)
        }
        const copiesBeforeMutation = prefix
          ? text.includes("で要素をコピーするため")
          : /cop(?:y|ies) members before (?:delete or )?add/u.test(text)
        expect(copiesBeforeMutation, route).toBe(
          /\.(?:add|delete)\(/u.test(comparison.source)
        )
        const panels = [
          ...html.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ]
        expect(plain(panels[1]?.[1] ?? ""), route).toBe(comparison.source)
        const ja = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/set/${sample.slug}/ja.ssrg`
          ),
          "utf8"
        )
        const en = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/set/${sample.slug}/en.ssrg`
          ),
          "utf8"
        )
        expect([...ja.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])).toEqual(
          [...en.matchAll(/^ {2}(\w+): /gm)].map((x) => x[1])
        )
      }
      for (const name of ["filterMap", "empty"]) {
        const control = fallbackModule.items.find(
          (x) => x.identity === `std/stream::${name}`
        )!
        const html = pages.get(
          `${prefix}/docs/library/stream/function/${name.toLowerCase()}/`
        )!
        expect(html).not.toContain('id="using-this-operation"')
        expect(html).not.toContain("Missing canonical example")
        expect(plain(html)).toContain(control.signature)
        expect(plain(html)).toContain(
          prefix ? control.descriptionJa : control.descriptionEn
        )
      }
    }
    if (process.env.SET_EDITORIAL_RENDER_DIR) {
      const { mkdirSync } = require("node:fs")
      mkdirSync(process.env.SET_EDITORIAL_RENDER_DIR, { recursive: true })
      for (const page of output) {
        const destination = join(
          process.env.SET_EDITORIAL_RENDER_DIR,
          page.route
        )
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("overlay identity inventory stays valid and ignores unrelated namespace, kind, module, and symbol", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-set-identities-"))
  try {
    const all = compilerReferenceModules().find(
      (x) => x.specifier === "std/set"
    )!
    const items = setEditorialCases.map((sample) => {
      const matches = all.items.filter(
        (x) =>
          x.identity === sample.identity &&
          x.namespace === "value" &&
          x.itemKind === "function"
      )
      expect(matches, sample.identity).toHaveLength(1)
      return matches[0]
    })
    const dispatch = readFileSync(
      join(root, "apps/site/src/reference/editorial/set-catalog.ssrg"),
      "utf8"
    )
    const mapped = [...dispatch.matchAll(/"(std\/set::[^"]+)" -> Just/g)].map(
      (x) => x[1]
    )
    expect(new Set(mapped).size).toBe(mapped.length)
    expect(mapped.sort()).toEqual(
      setEditorialCases.map((x) => x.identity).sort()
    )
    const first = items[0]
    const probes = [
      ...items,
      { ...first, identity: "std/set::filter" },
      { ...first, namespace: "type" },
      { ...first, itemKind: "constructor" },
      { ...first, module: "std/array" },
      { ...first, identity: "std/array::chunksOf" },
    ]
    const input = {
      schema: 1,
      origin: "",
      playgroundUrl: "",
      tourUrl: "",
      grammar: "",
      examples: [],
      referenceModules: [{ ...all, items: probes }],
    }
    const result = renderPageClosure<boolean[]>({
      directory,
      modules: ["reference/editorial/set-catalog"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { ReferenceSymbol, decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { setEditorialFor } from "./reference/editorial/set-catalog"
fn hasEditorial symbol: ReferenceSymbol -> Bool = match setEditorialFor symbol {
  Just _ -> True
  Nothing -> False
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [[hasEditorial symbol | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(result).toEqual([...Array(12).fill(true), ...Array(5).fill(false)])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("existing Set runtime contracts cover callback counts, collisions and preserved versions", () => {
  const result = spawnSync(
    process.execPath,
    ["test", "runtime/ts/tests/set.test.ts"],
    { cwd: root, encoding: "utf8", timeout: 15000 }
  )
  expect(result.status, result.stderr).toBe(0)
  expect(result.stderr).toContain("0 fail")
})
