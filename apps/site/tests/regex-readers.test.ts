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
import { entries } from "../../../runtime/ts/src/map"
import * as regex from "../../../runtime/ts/src/regex"
import { compilerReferenceModules } from "../scripts/reference"
import { regexReaderCases, regexReaderExamples } from "../scripts/regex-readers"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = regexReaderExamples("https://seseragi.vercel.app/")
function plain(html: string) {
  return html
    .replace(/<[^>]*>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function run(command: string, args: string[], cwd = root) {
  return spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 16 * 1024 * 1024,
  })
}
function compiled(source: string) {
  const result = regex.compile(source)
  if (result.tag === "Left") throw new Error(JSON.stringify(result.value))
  return result.value
}

test("eleven canonical native and TypeScript programs execute their documented normal and boundary cases", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-regex-examples-"))
  try {
    for (const item of regexReaderCases) {
      const sample = examples.find((x) => x.id === `regex-reader-${item.slug}`)!
      writeFileSync(join(directory, "main.ssrg"), sample.source)
      const lint = run(cli, ["lint", "main.ssrg"], directory)
      expect(lint.status, `${item.slug}: ${lint.stderr}`).toBe(0)
      const result = run(cli, ["run", "main.ssrg"], directory)
      expect(result.status, `${item.slug}: ${result.stderr}`).toBe(0)
      expect(result.stderr).toBe("")
      expect(result.stdout).toBe(item.output)
      expect(new URL(sample.playgroundUrl).searchParams.get("source")).toBe(
        sample.source
      )
      const comparison = examples.find(
        (x) => x.id === `regex-reader-${item.slug}-ts`
      )!
      expect(comparison.playgroundUrl).toBe("")
      const ts = run("bun", [join(root, comparison.sourcePath)])
      expect(ts.status, ts.stderr).toBe(0)
      expect(ts.stdout).toBe(item.typescriptOutput)
    }
    const checked = run("bun", [
      "x",
      "tsc",
      "--noEmit",
      "--strict",
      "--skipLibCheck",
      "--target",
      "ES2022",
      "--lib",
      "ESNext,DOM",
      "--module",
      "ESNext",
      "--moduleResolution",
      "bundler",
      ...regexReaderCases.map((item) =>
        join(root, `apps/site/examples/comparisons/api-regex/${item.slug}.ts`)
      ),
    ])
    expect(checked.status, checked.stdout + checked.stderr).toBe(0)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("the exact Playground seeds compile through committed WASM and execute in the runtime harness", async () => {
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
  const runtime = await import(
    new URL(
      "../../playground/src/runtime/browser-execution.ts",
      import.meta.url
    ).href
  )
  for (const item of regexReaderCases) {
    const sample = examples.find((x) => x.id === `regex-reader-${item.slug}`)!
    const seed = new URL(sample.playgroundUrl).searchParams.get("source")!
    const result = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    )
    expect(result.status, JSON.stringify(result)).toBe("success")
    expect(result.entry).toBeDefined()
    if (result.status !== "success" || !result.entry) continue
    const executed = await runtime.executeGeneratedModule(
      result.generated.typescript,
      result.entry
    )
    expect(executed.stdout).toBe(item.output.trimEnd())
  }
})

test("portable syntax, capture absence, scalar progress, and literal callback boundaries retain their contracts", () => {
  const bad = regex.compile("é[")
  expect(bad).toEqual({
    tag: "Left",
    value: { kind: { tag: "UnexpectedRegexEnd" }, offset: 3 },
  })
  for (const source of [
    "(a)\\1",
    "(?=a)",
    "(?<=a)",
    "(?i:a)",
    "a+?",
    "a++",
    "(?>a)",
    "(?(a)b)",
    "(?R)",
    "\\b",
    "\\B",
  ]) {
    const rejected = regex.compile(source)
    expect(rejected.tag, source).toBe("Left")
    if (rejected.tag === "Left")
      expect(rejected.value.kind.tag, source).toBe("UnsupportedRegexFeature")
  }
  expect(regex.find(compiled("a|aa"), "xaa")).toMatchObject({
    tag: "Just",
    value: { text: "a", span: { start: 1, end: 2 } },
  })
  const capture = regex.find(compiled("(?<empty>a?)(?<absent>b)?"), "")
  expect(capture.tag).toBe("Just")
  if (capture.tag === "Just") {
    expect(capture.value.captures).toEqual([
      { tag: "Just", value: { text: "", span: { start: 0, end: 0 } } },
      { tag: "Nothing" },
    ])
    expect(
      entries(capture.value.named).map(([key, value]) => [key, value.tag])
    ).toEqual([
      ["empty", "Just"],
      ["absent", "Nothing"],
    ])
  }
  expect(regex.findAll(compiled(""), "").map((m) => m.span)).toEqual([
    { start: 0, end: 0 },
  ])
  expect(regex.findAll(compiled("."), "👍🏽").map((m) => m.span)).toEqual([
    { start: 0, end: 4 },
    { start: 4, end: 8 },
  ])
  expect(regex.split(compiled("(,)"), ",a,")).toEqual(["", "a", ""])
  expect(regex.split(compiled(""), "a👍")).toEqual(["", "a", "👍", ""])
  expect(regex.replaceAll(compiled("a"), "$1$&\\", "a")).toBe("$1$&\\")
  const visited: string[] = []
  expect(
    regex.replaceAllWith(
      compiled("[0-9]+"),
      (match) => {
        visited.push(match.text)
        return `[${match.text}]`
      },
      "a12b3"
    )
  ).toBe("a[12]b[3]")
  expect(visited).toEqual(["12", "3"])
  expect(regex.replaceAll(compiled("a"), "aa", "a")).toBe("aa")
  const pattern = compiled("a")
  expect([regex.isMatch(pattern, "a"), regex.isMatch(pattern, "a")]).toEqual([
    true,
    true,
  ])
})

test("Regex arguments and pure replacement callbacks are checked in genuine function contexts", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-regex-invalid-"))
  try {
    const cases = [
      [
        'import * as regex from "std/regex"\npub fn check -> Bool = regex.isMatch "a" "a"\n',
        "SES-T0101",
      ],
      [
        'import * as regex from "std/regex"\npub fn replace pattern: regex.Regex -> String = regex.replaceAllWith pattern (\\found: regex.RegexMatch -> println found.text) "a"\n',
        "SES-T0101",
      ],
    ]
    for (const [source, diagnostic] of cases) {
      writeFileSync(join(directory, "main.ssrg"), source)
      const result = run(cli, ["lint", "main.ssrg"], directory)
      expect(result.status).not.toBe(0)
      expect(result.stderr).toContain(diagnostic)
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("Regex editorial dispatch preserves ten exact identities and ignores unrelated symbols", () => {
  const all = compilerReferenceModules()
  const module = all.find((x) => x.specifier === "std/regex")!
  const selected = regexReaderCases
    .filter((x) => x.name)
    .map((item) => {
      const matches = module.items.filter(
        (symbol) =>
          symbol.identity === item.identity &&
          symbol.itemKind === "function" &&
          symbol.namespace === "value"
      )
      expect(matches).toHaveLength(1)
      return matches[0]
    })
  expect(selected).toHaveLength(10)
  const probes = selected.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/array" },
    { ...symbol, namespace: "type" },
    { ...symbol, itemKind: "effect-function" },
    { ...symbol, identity: `std/array::${symbol.name}` },
  ])
  const input = {
    schema: 1,
    origin: "",
    playgroundUrl: "",
    tourUrl: "",
    grammar: "",
    examples: [],
    referenceModules: [{ ...module, items: probes }],
  }
  const directory = mkdtempSync(join(tmpdir(), "seseragi-regex-identities-"))
  try {
    const output = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/regex-catalog",
      ],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor } from "./reference/editorial/catalog"
import { regexEditorialFor } from "./reference/editorial/regex-catalog"
fn present value: Maybe<Editorial> -> Bool = match value {
  Just _ -> True
  Nothing -> False
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [[(present (regexEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(output).toHaveLength(50)
    for (let i = 0; i < 50; i++)
      expect(output[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all twenty-two production Regex bodies preserve exact declarations, localized copy, outputs and source panels", () => {
  const all = compilerReferenceModules()
  const original = all.find((x) => x.specifier === "std/regex")!
  const identities = new Set<string>(
    regexReaderCases.filter((x) => x.name).map((x) => x.identity)
  )
  const control = original.items.find(
    (x) => x.name === "RegexSpan" && x.namespace === "type"
  )!
  expect(control).toBeDefined()
  const module = {
    ...original,
    items: original.items.filter(
      (x) => identities.has(x.identity) || x === control
    ),
  }
  expect(module.items).toHaveLength(11)
  const input = {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "",
    grammar: "",
    examples,
    referenceModules: [module],
  }
  const directory = mkdtempSync(join(tmpdir(), "seseragi-regex-render-"))
  try {
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
    let site = SiteCatalog { home, pages, areas: [NavigationArea { id: "library", title: home.title, description: home.summary, landing: home, groups }] }
    println (json.encodeString [Output {route: localizedRoute locale page.path, html: renderDocument input site locale page} | locale <- [En, Ja], page <- pages])
  }
}`,
    })
    expect(output).toHaveLength(24)
    const pages = new Map(output.map((p) => [p.route, p.html]))
    for (const item of regexReaderCases)
      for (const locale of ["en", "ja"]) {
        const prefix = locale === "en" ? "" : "/ja"
        const route = `${prefix}/docs/library/regex/${item.name ? `function/${item.slug}/` : ""}`
        const html = pages.get(route)!
        const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0]
        const text = plain(body)
        for (const anchor of [
          "using-this-operation",
          "typescript-comparison",
          "operation-rules",
        ])
          expect(body).toContain(`id="${anchor}"`)
        expect(text).toContain(item.output.trimEnd())
        expect(text).toContain(item.typescriptOutput.trimEnd())
        expect(body).not.toContain("Missing canonical example")
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((x) => plain(x[1]))
        for (const suffix of ["", "-ts"]) {
          const sample = examples.find(
            (x) => x.id === `regex-reader-${item.slug}${suffix}`
          )!
          expect(
            panels.filter((x) => x === sample.source),
            `${route}:${suffix}`
          ).toHaveLength(1)
        }
        const own = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/regex/${item.slug}/${locale}.ssrg`
          ),
          "utf8"
        )
        const other = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/regex/${item.slug}/${locale === "en" ? "ja" : "en"}.ssrg`
          ),
          "utf8"
        )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const expected = paragraphs(own)
        expect(expected.length).toBeGreaterThanOrEqual(5)
        expect(expected.length).toBe(paragraphs(other).length)
        for (const paragraph of expected)
          expect(text, route).toContain(paragraph)
        for (const paragraph of paragraphs(other))
          expect(text, route).not.toContain(paragraph)
        // Check scalar copy fields too, especially escaped regex notation in prose.
        for (const [, field, literal] of own.matchAll(
          /^ {2}(\w+): ("(?:[^"\\]|\\.)*")/gmu
        )) {
          if (field === "summary" && !item.name) continue
          expect(plain(html), `${route}:${field}`).toContain(
            JSON.parse(literal)
          )
        }
        if (item.name) {
          const symbol = module.items.find((x) => x.identity === item.identity)!
          expect(text).toContain(symbol.signature)
          expect(body).toContain('id="canonical-declaration"')
          expect(body).toContain(`href="${prefix}/docs/library/regex/"`)
          expect(pages.get(`${prefix}/docs/library/regex/`)).toContain(
            `href="${route}"`
          )
          if (item.name === "find")
            expect(text).not.toContain(
              locale === "en" ? symbol.descriptionEn : symbol.descriptionJa
            )
        }
      }
    for (const prefix of ["", "/ja"]) {
      const html = pages.get(
        `${prefix}/docs/library/regex/${control.itemKind}/regexspan/`
      )!
      expect(html).not.toContain('id="typescript-comparison"')
      expect(plain(html)).toContain(
        prefix ? control.descriptionJa : control.descriptionEn
      )
    }
    if (process.env.REGEX_READER_RENDER_DIR)
      for (const page of output) {
        const destination = join(
          process.env.REGEX_READER_RENDER_DIR,
          page.route.slice(1)
        )
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
