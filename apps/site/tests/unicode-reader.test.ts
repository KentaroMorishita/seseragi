import { expect, setDefaultTimeout, test } from "bun:test"
import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import type {
  CompileResponse,
  EntryContract,
} from "../../playground/src/compiler/types"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { compilerReferenceModules } from "../scripts/reference"
import { textEditorialExamples } from "../scripts/text-editorial"
import {
  unicodeReaderCases,
  unicodeReaderExamples,
  unicodeTypeScriptCases,
} from "../scripts/unicode-reader"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = unicodeReaderExamples("https://seseragi.vercel.app/")
const source = (slug: string) =>
  examples.find((example) => example.id === `unicode-${slug}`)!
function plain(html: string) {
  return html
    .replace(/<[^>]*>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function panels(html: string) {
  return [
    ...html.matchAll(/<section class="code-panel">[\s\S]*?<\/section>/gu),
  ].map((match) => match[0])
}
function code(panel: string) {
  return plain(
    panel.match(/<code class="seseragi-highlight">([\s\S]*?)<\/code>/u)?.[1] ??
      ""
  )
}
function assertAuthoredPageTitles(
  html: string,
  titles: ReadonlyMap<string, string>,
  route: string
) {
  const article = html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/u)?.[1]
  assert.ok(article, `${route}: missing article`)
  // Stop only at the explicit generated declaration/API-index boundary.
  const authored = article.split(
    /<h2 id="(?:canonical-declaration|unicode-api-index|grapheme-api-index)"/u
  )[0]
  let checked = 0
  for (const link of authored.matchAll(
    /<a\b[^>]*\bhref="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gu
  )) {
    const destination = plain(link[1])
    // Fragment links describe sections; external links include Playground actions.
    if (
      !/^\/(?:ja\/)?docs\/library\//u.test(destination) ||
      destination.includes("#")
    )
      continue
    assert.ok(
      titles.has(destination),
      `${route}: missing title for ${destination}`
    )
    assert.equal(
      plain(link[2]).trim(),
      titles.get(destination),
      `${route}: authored page-name link to ${destination} must match its heading`
    )
    checked++
  }
  return checked
}

function native(command: string, directory: string) {
  return spawnSync(cli, [command, "main.ssrg"], {
    cwd: directory,
    encoding: "utf8",
    timeout: 30_000,
  })
}

test("Unicode example descriptors preserve exact source bytes, hashes, highlights and runnable seeds", () => {
  expect(unicodeReaderCases).toHaveLength(12)
  expect(examples).toHaveLength(14)
  for (const example of examples) {
    const bytes = readFileSync(join(root, example.sourcePath), "utf8")
    expect(example.source).toBe(bytes)
    expect(example.sha256).toBe(
      createHash("sha256").update(bytes).digest("hex")
    )
    expect(example.highlighted.map((part) => part.text).join("")).toBe(bytes)
    if (example.sourcePath.endsWith(".ts"))
      expect(example.playgroundUrl).toBe("")
    else expect(sourceFromPlaygroundUrl(example.playgroundUrl)).toBe(bytes)
  }
})

for (const item of unicodeReaderCases)
  test(`Unicode native ${item.identity}: lint and exact output`, () => {
    const directory = mkdtempSync(join(tmpdir(), "seseragi-unicode-native-"))
    try {
      writeFileSync(join(directory, "main.ssrg"), source(item.slug).source)
      for (const command of ["lint", "run"]) {
        const result = native(command, directory)
        expect(result.status, result.stderr).toBe(0)
        expect(result.stderr).toBe("")
        expect(result.stdout).toBe(command === "run" ? item.output : "")
      }
    } finally {
      rmSync(directory, { recursive: true, force: true })
    }
  })

test("The documented invalid range repairs to the actual preview, and Unicode version follows the manifest", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-unicode-repair-"))
  try {
    const item = unicodeReaderCases.find((item) => item.slug === "slice")!
    const repaired = source("slice").source.replace(
      "preview 0 4 label",
      "preview 0 2 label"
    )
    expect(repaired).not.toBe(source("slice").source)
    writeFileSync(join(directory, "main.ssrg"), repaired)
    const result = native("run", directory)
    expect(result.status, result.stderr).toBe(0)
    expect(result.stdout).toBe(
      item.output.replace(
        "invalid: InvalidGraphemeRange { start: 0, end: 4, length: 3 }",
        "preview: [A👍🏽]"
      )
    )
    const manifest = JSON.parse(
      readFileSync(join(root, "runtime/unicode/manifest.json"), "utf8")
    )
    expect(
      String(unicodeReaderCases.find((item) => item.slug === "version")!.output)
    ).toBe(`Unicode: ${manifest.version}\n`)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("The same twelve Playground programs compile through WASM and execute their boundary results", async () => {
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
  const runtime: {
    executeGeneratedModule(
      typescript: string,
      entry: EntryContract
    ): Promise<{ stdout: string }>
  } = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  for (const item of unicodeReaderCases) {
    const seed = sourceFromPlaygroundUrl(source(item.slug).playgroundUrl)
    expect(seed).toBe(source(item.slug).source)
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    ) as CompileResponse
    expect(compiled.status, JSON.stringify(compiled)).toBe("success")
    if (compiled.status !== "success" || !compiled.entry)
      throw new Error(`Missing compiled entry for ${item.identity}`)
    expect(
      (
        await runtime.executeGeneratedModule(
          compiled.generated.typescript,
          compiled.entry
        )
      ).stdout
    ).toBe(item.output.trimEnd())
  }
})

test("TypeScript counterparts typecheck, run exact displayed results, and retain the full-fold distinction", () => {
  const paths = unicodeTypeScriptCases.map((item) =>
    join(root, `apps/site/examples/src/api-unicode/${item.slug}.ts`)
  )
  const checked = spawnSync(
    join(root, "node_modules/.bin/tsc"),
    [
      "--noEmit",
      "--skipLibCheck",
      "--target",
      "ES2022",
      "--module",
      "ESNext",
      "--moduleResolution",
      "Bundler",
      "--lib",
      "ES2022,ES2022.Intl,DOM",
      ...paths,
    ],
    { cwd: root, encoding: "utf8", timeout: 30_000 }
  )
  expect(checked.status, checked.stderr || checked.stdout).toBe(0)
  for (const item of unicodeTypeScriptCases) {
    const result = spawnSync(
      process.execPath,
      [join(root, `apps/site/examples/src/api-unicode/${item.slug}.ts`)],
      { cwd: root, encoding: "utf8", timeout: 30_000 }
    )
    expect(result.status, result.stderr).toBe(0)
    expect(result.stdout).toBe(item.output)
  }
  expect(
    unicodeTypeScriptCases.find((item) => item.slug === "normalization")!.output
  ).toContain("lowercase: straße\n")
  expect(
    unicodeReaderCases.find((item) => item.slug === "casefold")!.output
  ).toContain("folded text: strasse\n")
})

function selectedModules() {
  const identities = new Set<string>(
    unicodeReaderCases.map((item) => item.identity)
  )
  const controls = new Set(["std/text::concat", "std/text::isEmpty"])
  return compilerReferenceModules()
    .filter((module) =>
      ["std/text", "std/text/grapheme", "std/text/unicode"].includes(
        module.specifier
      )
    )
    .map((module) => ({
      ...module,
      items: module.items.filter(
        (item) => identities.has(item.identity) || controls.has(item.identity)
      ),
    }))
}

test("Fourteen Unicode/grapheme identities render paired production pages with unchanged declaration and Text controls", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-unicode-render-"))
  try {
    const modules = selectedModules()
    const allExamples = [
      ...examples,
      ...textEditorialExamples("https://seseragi.vercel.app/"),
    ]
    const input = {
      schema: 1,
      origin: "https://seseragi.example",
      playgroundUrl: "https://seseragi.vercel.app/",
      tourUrl: "https://seseragi.vercel.app/tour/",
      grammar: "",
      examples: allExamples,
      referenceModules: modules,
    }
    const output = renderPageClosure<Array<{ route: string; html: string }>>({
      directory,
      timeoutMs: 150_000,
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
  let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference, title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
  let site = SiteCatalog { home, pages, areas: [NavigationArea { id: "library", title: home.title, description: home.summary, landing: home, groups }] }
  println (json.encodeString [Output {route: localizedRoute locale page.path, html: renderDocument input site locale page} | locale <- [En, Ja], page <- pages])
 }
}`,
    })
    expect(output).toHaveLength(34) // Fourteen edited identities plus three unchanged controls, in both locales.
    const pages = new Map(output.map((page) => [page.route, page.html]))
    const titles = new Map(
      output.map((page) => [page.route, pageTitle(page.html)])
    )
    const authoredRoutes = new Map<string, number>([
      ...unicodeReaderCases.map(
        (item) => [item.route, item.slug === "casefold" ? 2 : 1] as const
      ),
      ["/docs/library/text/grapheme/", 7],
      ["/docs/library/text/unicode/", 8],
    ])
    let authoredLinks = 0
    for (const prefix of ["", "/ja"]) {
      for (const [path, expectedLinks] of authoredRoutes) {
        const route = prefix + path
        const checked = assertAuthoredPageTitles(
          pages.get(route)!,
          titles,
          route
        )
        expect(checked, route).toBe(expectedLinks)
        authoredLinks += checked
      }
      const route = `${prefix}/docs/library/text/unicode/`
      const html = pages.get(route)!
      // A decorated anchor must fail even if its destination and purpose remain valid.
      const decorated = html.replaceAll(
        ">normalize</a>",
        ">normalize: choose among four forms</a>"
      )
      expect(decorated).not.toBe(html)
      expect(() => assertAuthoredPageTitles(decorated, titles, route)).toThrow(
        "must match its heading"
      )
      expect(html).toContain(
        prefix === "/ja"
          ? ">normalize</a><span>：四つの形式を選ぶ</span>"
          : ">normalize</a><span>: choose among four forms</span>"
      )
    }
    expect(authoredLinks).toBe(56)
    for (const page of output) {
      if (process.env.SESERAGI_UNICODE_READER_OUTPUT) {
        const path = join(
          resolve(process.env.SESERAGI_UNICODE_READER_OUTPUT),
          page.route.slice(1),
          "index.html"
        )
        mkdirSync(dirname(path), { recursive: true })
        writeFileSync(path, page.html)
      }
      const isJapanese = page.route.startsWith("/ja/")
      expect(page.html).toContain(`<html lang="${isJapanese ? "ja" : "en"}">`)
      expect(page.html).toContain(
        `href="${isJapanese ? page.route.slice(3) : `/ja${page.route}`}"`
      )
      expect(page.html).not.toContain("site-build-error")
      expect(page.html).not.toContain("Missing canonical example")
      expect(page.html.match(/<h1>/gu)).toHaveLength(1)
      const ids = [...page.html.matchAll(/\bid="([^"]+)"/gu)].map(
        (match) => match[1]
      )
      expect(new Set(ids).size, page.route).toBe(ids.length)
      for (const panel of panels(page.html)) {
        const bytes = code(panel)
        const canonical = allExamples.find(
          (example) => example.source === bytes
        )
        expect(canonical, page.route).toBeDefined()
        if (!canonical) continue
        const url = panel.match(
          /<a\b(?=[^>]*\bclass="playground-link")[^>]*\bhref="([^"]+)"/u
        )?.[1]
        if (canonical.playgroundUrl)
          expect(sourceFromPlaygroundUrl(plain(url ?? ""))).toBe(bytes)
        else expect(url).toBeUndefined()
      }
    }
    for (const prefix of ["", "/ja"]) {
      for (const item of unicodeReaderCases) {
        const html = pages.get(prefix + item.route)!
        const text = plain(html)
        const declared = modules
          .flatMap((module) => module.items)
          .find((symbol) => symbol.identity === item.identity)!
        expect(html).toContain(`<h1>${item.name}</h1>`)
        expect(text).toContain(declared.signature)
        expect(text).toContain(item.output.trimEnd())
        for (const id of [
          "using-this-operation",
          "reading-the-example",
          "operation-rules",
          "operation-mistakes",
          "canonical-declaration",
          "related",
        ])
          expect(html).toContain(`id="${id}"`)
        expect(panels(html).map(code)).toEqual([source(item.slug).source])
        expect(html).toContain(
          `href="${prefix}/docs/library/${item.module.slice(4)}/"`
        )
        expect(text).not.toContain("A public standard-library item.")
        expect(text).not.toContain("標準ライブラリの公開項目です。")
        if (item.slug === "version") {
          expect(text).toContain(
            prefix === "/ja"
              ? "追加の情報を持たないUnitの値"
              : "is a Unit value"
          )
          expect(text).not.toContain(
            prefix === "/ja" ? "引数を渡さず" : "without a value argument"
          )
        }
        if (item.slug === "at")
          expect(text).toContain(
            prefix === "/ja" ? "Unicodeの番号の単位" : "Unicode code values"
          )
        expect(text).toContain("process")
        expect(text).toContain("browser")
        const copies = ["en", "ja"].map((locale) =>
          readFileSync(
            join(
              root,
              `apps/site/src/reference/editorial/unicode-reader/${item.slug}/${locale}.ssrg`
            ),
            "utf8"
          )
        )
        expect(
          [...copies[0].matchAll(/^ {2}(\w+): /gm)].map((match) => match[1])
        ).toEqual(
          [...copies[1].matchAll(/^ {2}(\w+): /gm)].map((match) => match[1])
        )
      }
      const grapheme = pages.get(`${prefix}/docs/library/text/grapheme/`)!
      expect(panels(grapheme).map(code)).toEqual([
        source("length").source,
        source("graphemes-typescript").source,
        source("slice").source,
      ])
      for (const name of [
        "working-with-graphemes",
        "grapheme-comparison",
        "choose-grapheme-operation",
        "checked-grapheme-ranges",
        "grapheme-boundaries",
        "grapheme-api-index",
        "using-this-module",
        "public-api",
      ])
        expect(grapheme).toContain(`id="${name}"`)
      const unicode = pages.get(`${prefix}/docs/library/text/unicode/`)!
      for (const slug of [
        "normalize",
        "fullcasefold",
        "normalization-typescript",
      ])
        expect(panels(unicode).map(code)).toContain(source(slug).source)
      expect(unicode).toContain('id="unicode-api-index"')
      expect(plain(unicode)).toContain(
        unicodeTypeScriptCases[1].output.trimEnd()
      )
      expect(
        pages.get(`${prefix}/docs/library/text/function/casefold/`)!
      ).toContain(
        `href="${prefix}/docs/library/text/unicode/function/fullcasefold/"`
      )
      // Existing Text module and concat overlay are exercised, not edited.
      const previous = pages.get(`${prefix}/docs/library/text/`)!
      expect(previous).toContain('id="working-with-text"')
      expect(panels(previous).map(code)).toEqual(
        [
          "text-units",
          "text-units-typescript",
          "text-scalarat",
          "text-slicescalars",
        ].map((id) => allExamples.find((example) => example.id === id)!.source)
      )
      const concat = pages.get(`${prefix}/docs/library/text/function/concat/`)!
      expect(panels(concat).map(code)).toEqual([
        allExamples.find((example) => example.id === "text-concat")!.source,
      ])
      const control = pages.get(
        `${prefix}/docs/library/text/function/isempty/`
      )!
      expect(control).not.toContain('id="using-this-operation"')
      expect(plain(control)).toContain(
        modules
          .flatMap((module) => module.items)
          .find((item) => item.identity === "std/text::isEmpty")!.signature
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("Unicode dispatcher uses exact identities, namespaces, kinds and owning modules", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-unicode-identity-"))
  try {
    const modules = selectedModules()
    const items = unicodeReaderCases.map(
      (item) =>
        modules
          .flatMap((module) => module.items)
          .find((symbol) => symbol.identity === item.identity)!
    )
    expect(items).toHaveLength(12)
    const first = items[0]
    const probes = [
      ...items,
      { ...first, identity: "std/text/grapheme::absent" },
      { ...first, namespace: "type" },
      { ...first, itemKind: "constructor" },
      { ...first, module: "std/array" },
    ]
    const input = {
      schema: 1,
      origin: "",
      playgroundUrl: "",
      tourUrl: "",
      grammar: "",
      examples: [],
      referenceModules: [{ ...modules[0], items: probes }],
    }
    const output = renderPageClosure<boolean[]>({
      directory,
      modules: ["reference/editorial/unicode-reader/catalog"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { ReferenceSymbol, decodeBuildInput } from "./model/build"
import { unicodeReaderEditorialFor } from "./reference/editorial/unicode-reader/catalog"
fn covered symbol: ReferenceSymbol -> Bool = match unicodeReaderEditorialFor symbol { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} { Left _ -> println "[]"; Right input -> println (json.encodeString (arrays.concat [[covered symbol | symbol <- module.items] | module <- input.referenceModules])) }`,
    })
    expect(output).toEqual([...Array(12).fill(true), ...Array(4).fill(false)])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
