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
import { charTextCases, charTextExamples } from "../scripts/char-text-reader"
import { compilerReferenceModules } from "../scripts/reference"
import { textEditorialExamples } from "../scripts/text-editorial"
import { unicodeReaderExamples } from "../scripts/unicode-reader"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = charTextExamples("https://seseragi.vercel.app/")
const source = (slug: string, typescript = false) =>
  examples.find(
    (item) => item.id === `char-text-${slug}${typescript ? "-typescript" : ""}`
  )!
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
function authoredPageLinks(
  html: string,
  titles: ReadonlyMap<string, string>,
  route: string
) {
  const article = html.match(/<article\b[^>]*>([\s\S]*?)<\/article>/u)?.[1]
  assert.ok(article, `${route}: missing article`)
  const authored = article.split(
    /<h2 id="(?:canonical-declaration|char-api-index)"/u
  )[0]
  let count = 0
  for (const link of authored.matchAll(
    /<a\b[^>]*\bhref="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gu
  )) {
    const destination = plain(link[1])
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
      `${route}: page-name anchor must match its heading`
    )
    count++
  }
  return count
}
function native(command: string, directory: string) {
  return spawnSync(cli, [command, "main.ssrg"], {
    cwd: directory,
    encoding: "utf8",
    timeout: 30_000,
  })
}

test("Char/Text canonical descriptors preserve all 24 exact sources, hashes, highlights and seeds", () => {
  expect(charTextCases).toHaveLength(12)
  expect(examples).toHaveLength(24)
  for (const item of examples) {
    const bytes = readFileSync(join(root, item.sourcePath), "utf8")
    expect(item.source).toBe(bytes)
    expect(item.sha256).toBe(createHash("sha256").update(bytes).digest("hex"))
    expect(item.highlighted.map((part) => part.text).join("")).toBe(bytes)
    if (item.sourcePath.endsWith(".ts")) expect(item.playgroundUrl).toBe("")
    else expect(sourceFromPlaygroundUrl(item.playgroundUrl)).toBe(bytes)
  }
})
for (const item of charTextCases)
  test(`Char/Text native ${item.identity}: lint and exact output`, () => {
    const directory = mkdtempSync(join(tmpdir(), "seseragi-char-text-native-"))
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

test("Char literal and argument mistakes are rejected and the documented repairs execute", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-char-text-repair-"))
  const cases = [
    {
      source:
        'import * as char from "std/char"\nfn inspect value: String -> Int = char.codePoint value\npub effect fn main = println (inspect "A")\n',
      diagnostic: "SES-T0101",
      repair:
        "import * as char from \"std/char\"\nfn inspect value: Char -> Int = char.codePoint value\npub effect fn main = println (inspect 'A')\n",
      output: "65\n",
    },
    {
      source:
        "import * as char from \"std/char\"\npub effect fn main = println (char.codePoint 'e\\u{0301}')\n",
      diagnostic: "SES-P0202",
      repair:
        'import * as text from "std/text"\nlet value = "e\\u{0301}"\npub effect fn main = println (text.lengthScalars value)\n',
      output: "2\n",
    },
    {
      source:
        "import * as char from \"std/char\"\npub effect fn main = println (char.codePoint '\\u{D800}')\n",
      diagnostic: "SES-P0201",
      repair:
        'import * as char from "std/char"\npub effect fn main = println (show (char.fromCodePoint 55296))\n',
      output: "Nothing\n",
    },
    {
      source:
        'import * as unicode from "std/text/unicode"\npub effect fn main = println (unicode.isAlphabetic "A")\n',
      diagnostic: "SES-T0101",
      repair:
        "import * as unicode from \"std/text/unicode\"\npub effect fn main = println (show (unicode.isAlphabetic 'A'))\n",
      output: "True\n",
    },
    {
      source:
        "import * as unicode from \"std/text/unicode\"\npub effect fn main = println (unicode.isMark 'e\\u{0301}')\n",
      diagnostic: "SES-P0202",
      repair:
        "import * as unicode from \"std/text/unicode\"\npub effect fn main = println (show (unicode.isMark '\\u{0301}'))\n",
      output: "True\n",
    },
    {
      source:
        'import * as unicode from "std/text/unicode"\npub effect fn main = println (unicode.simpleCaseFold "Σς")\n',
      diagnostic: "SES-T0101",
      repair:
        'import * as unicode from "std/text/unicode"\npub effect fn main = println (unicode.fullCaseFold "Σς")\n',
      output: "σσ\n",
    },
  ]
  try {
    for (const item of cases) {
      writeFileSync(join(directory, "main.ssrg"), item.source)
      const rejected = native("lint", directory)
      expect(rejected.status).toBe(2)
      expect(rejected.stderr).toContain(item.diagnostic)
      writeFileSync(join(directory, "main.ssrg"), item.repair)
      expect(native("lint", directory).status).toBe(0)
      const repaired = native("run", directory)
      expect(repaired.status, repaired.stderr).toBe(0)
      expect(repaired.stdout).toBe(item.output)
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("All twelve canonical programs compile through WASM and execute the same results", async () => {
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
  for (const item of charTextCases) {
    const seed = sourceFromPlaygroundUrl(source(item.slug).playgroundUrl)
    expect(seed).toBe(source(item.slug).source)
    const result = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    ) as CompileResponse
    expect(result.status, JSON.stringify(result)).toBe("success")
    if (result.status !== "success" || !result.entry)
      throw new Error(`Missing entry: ${item.identity}`)
    expect(
      (
        await runtime.executeGeneratedModule(
          result.generated.typescript,
          result.entry
        )
      ).stdout
    ).toBe(item.output.trimEnd())
  }
})

test("Every displayed TS counterpart typechecks and executes its stated comparison boundaries", () => {
  const paths = charTextCases.map((item) =>
    join(root, source(item.slug, true).sourcePath)
  )
  const checked = spawnSync(
    join(root, "node_modules/.bin/tsc"),
    [
      "--noEmit",
      "--strict",
      "--skipLibCheck",
      "--target",
      "ES2022",
      "--module",
      "ESNext",
      "--moduleResolution",
      "Bundler",
      ...paths,
    ],
    { cwd: root, encoding: "utf8", timeout: 30_000 }
  )
  expect(checked.status, checked.stderr + checked.stdout).toBe(0)
  for (const item of charTextCases) {
    const result = spawnSync(
      "bun",
      [join(root, source(item.slug, true).sourcePath)],
      { cwd: root, encoding: "utf8", timeout: 30_000 }
    )
    expect(result.status, result.stderr).toBe(0)
    expect(result.stderr).toBe("")
    expect(result.stdout).toBe(item.typescriptOutput)
  }
  expect(
    charTextCases.find((item) => item.slug === "fromcodepoint")!
      .typescriptOutput
  ).toContain("JS accepts surrogate: 55296")
  for (const slug of ["trimstart", "trimend"]) {
    const result = charTextCases.find((item) => item.slug === slug)!
    expect(result.output).toContain("NEL removed: True")
    expect(result.typescriptOutput).toContain("JS removes NEL: false")
    expect(result.typescriptOutput).toContain("Unicode removes NEL: true")
    expect(result.typescriptOutput).toContain("JS preserves FEFF: false")
    expect(result.typescriptOutput).toContain("Unicode preserves FEFF: true")
  }
})

function selectedModules() {
  const identities = new Set<string>(charTextCases.map((item) => item.identity))
  const controls = new Set([
    "std/text::concat",
    "std/text::isEmpty",
    "std/text/unicode::fullCaseFold",
  ])
  return compilerReferenceModules()
    .filter((module) =>
      [
        "std/char",
        "std/text",
        "std/text/unicode",
        "std/text/grapheme",
      ].includes(module.specifier)
    )
    .map((module) => ({
      ...module,
      items: module.items.filter(
        (item) => identities.has(item.identity) || controls.has(item.identity)
      ),
    }))
}

test("Thirteen Char/Text identities render paired production bodies and preserve older Text/Unicode controls", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-char-text-render-"))
  try {
    const modules = selectedModules()
    const allExamples = [
      ...examples,
      ...textEditorialExamples("https://seseragi.vercel.app/"),
      ...unicodeReaderExamples("https://seseragi.vercel.app/"),
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
    expect(output).toHaveLength(38) // 26 edited bodies and twelve unchanged control bodies.
    const pages = new Map(output.map((page) => [page.route, page.html]))
    const titles = new Map(
      output.map((page) => [page.route, pageTitle(page.html)])
    )
    for (const page of output) {
      if (process.env.CHAR_TEXT_RENDER_DIR) {
        const path = join(
          resolve(process.env.CHAR_TEXT_RENDER_DIR),
          page.route.slice(1),
          "index.html"
        )
        mkdirSync(dirname(path), { recursive: true })
        writeFileSync(path, page.html)
      }
      expect(page.html).not.toContain("Missing canonical example")
      expect(page.html).not.toContain("site-build-error")
      const ja = page.route.startsWith("/ja/")
      expect(page.html).toContain(`<html lang="${ja ? "ja" : "en"}">`)
      expect(page.html).toContain(
        `href="${ja ? page.route.slice(3) : `/ja${page.route}`}"`
      )
      const ids = [...page.html.matchAll(/\bid="([^"]+)"/gu)].map(
        (match) => match[1]
      )
      expect(new Set(ids).size, page.route).toBe(ids.length)
      for (const panel of panels(page.html)) {
        const canonical = allExamples.find(
          (item) => item.source === code(panel)
        )
        expect(canonical, page.route).toBeDefined()
        const url = panel.match(
          /<a\b(?=[^>]*\bclass="playground-link")[^>]*\bhref="([^"]+)"/u
        )?.[1]
        if (canonical?.playgroundUrl)
          expect(sourceFromPlaygroundUrl(plain(url ?? ""))).toBe(
            canonical.source
          )
        else expect(url).toBeUndefined()
      }
    }
    let checkedLinks = 0
    for (const prefix of ["", "/ja"]) {
      const moduleRoute = `${prefix}/docs/library/char/`
      const moduleHtml = pages.get(moduleRoute)!
      const count = authoredPageLinks(moduleHtml, titles, moduleRoute)
      expect(count).toBe(6)
      checkedLinks += count
      expect(panels(moduleHtml).map(code)).toEqual([
        source("codepoint").source,
        source("fromcodepoint").source,
        source("fromcodepoint", true).source,
      ])
      expect(moduleHtml).toContain('id="char-api-index"')
      expect(plain(moduleHtml)).toContain("SES-P0202")
      expect(plain(moduleHtml)).toContain("SES-P0201")
      const broken = moduleHtml.replaceAll(
        ">codePoint</a>",
        ">codePoint: read a number</a>"
      )
      expect(broken).not.toBe(moduleHtml)
      expect(() => authoredPageLinks(broken, titles, moduleRoute)).toThrow(
        "must match its heading"
      )
      for (const item of charTextCases) {
        const route = prefix + item.route
        const html = pages.get(route)!
        const text = plain(html)
        expect(pageTitle(html)).toBe(item.name)
        expect(text).toContain(
          modules
            .flatMap((module) => module.items)
            .find((symbol) => symbol.identity === item.identity)!.signature
        )
        expect(text).toContain(item.output.trimEnd())
        expect(text).toContain(item.typescriptOutput.trimEnd())
        expect(panels(html).map(code)).toEqual([
          source(item.slug).source,
          source(item.slug, true).source,
        ])
        for (const id of [
          "using-this-operation",
          "reading-the-example",
          "operation-rules",
          "typescript-comparison",
          "operation-mistakes",
          "canonical-declaration",
        ])
          expect(html).toContain(`id="${id}"`)
        expect(text).toContain("process")
        expect(text).toContain("browser")
        expect(text).not.toContain("A public standard-library item.")
        expect(text).not.toContain("標準ライブラリの公開項目です。")
        const count = authoredPageLinks(html, titles, route)
        expect(count, route).toBe(
          ["trimstart", "trimend", "words", "simplecasefold"].includes(
            item.slug
          )
            ? 2
            : 1
        )
        checkedLinks += count
        const copies = ["en", "ja"].map((locale) =>
          readFileSync(
            join(
              root,
              `apps/site/src/reference/editorial/char-text-reader/${item.slug}/${locale}.ssrg`
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
      for (const [path, id] of [
        ["text/", "working-with-text"],
        ["text/unicode/", "choose-unicode-operation"],
        ["text/grapheme/", "working-with-graphemes"],
      ])
        expect(pages.get(`${prefix}/docs/library/${path}`)!).toContain(
          `id="${id}"`
        )
      expect(
        panels(pages.get(`${prefix}/docs/library/text/function/concat/`)!).map(
          code
        )
      ).toEqual([allExamples.find((item) => item.id === "text-concat")!.source])
      expect(
        panels(
          pages.get(
            `${prefix}/docs/library/text/unicode/function/fullcasefold/`
          )!
        ).map(code)
      ).toEqual([
        allExamples.find((item) => item.id === "unicode-fullcasefold")!.source,
      ])
      expect(
        pages.get(`${prefix}/docs/library/text/function/isempty/`)!
      ).not.toContain('id="using-this-operation"')
    }
    expect(checkedLinks).toBe(44)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("Char/Text dispatch rejects similar identities and wrong owner, namespace and kind", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-char-text-identity-"))
  try {
    const modules = selectedModules()
    const items = charTextCases.map(
      (item) =>
        modules
          .flatMap((module) => module.items)
          .find((symbol) => symbol.identity === item.identity)!
    )
    const first = items[0]
    const probes = [
      ...items,
      { ...first, identity: "std/char::missing" },
      { ...first, module: "std/array" },
      { ...first, namespace: "type" },
      { ...first, itemKind: "constructor" },
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
      modules: ["reference/editorial/char-text-reader/catalog"],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { ReferenceSymbol, decodeBuildInput } from "./model/build"
import { charTextEditorialFor } from "./reference/editorial/char-text-reader/catalog"
fn covered symbol: ReferenceSymbol -> Bool = match charTextEditorialFor symbol { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} { Left _ -> println "[]"; Right input -> println (json.encodeString (arrays.concat [[covered symbol | symbol <- module.items] | module <- input.referenceModules])) }`,
    })
    expect(output).toEqual([...Array(12).fill(true), ...Array(4).fill(false)])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
