import { expect, setDefaultTimeout, test } from "bun:test"
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
import { join, resolve } from "node:path"
import {
  numericReaderCases,
  numericReaderExamples,
} from "../scripts/numeric-readers"
import { compilerReferenceModules } from "../scripts/reference"
import { assertAuthoredLibraryTitles } from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = numericReaderExamples("https://seseragi.vercel.app/")
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

test("thirteen canonical native and strict TypeScript programs execute the documented outputs", () => {
  expect(numericReaderCases).toHaveLength(13)
  expect(examples).toHaveLength(26)
  const directory = mkdtempSync(join(tmpdir(), "seseragi-numeric-examples-"))
  try {
    for (const item of numericReaderCases) {
      const sample = examples.find(
        (x) => x.id === `numeric-reader-${item.slug}`
      )!
      writeFileSync(join(directory, "main.ssrg"), sample.source)
      const lint = run(cli, ["lint", "main.ssrg"], directory)
      expect(lint.status, `${item.slug}: ${lint.stderr}`).toBe(0)
      const result = run(cli, ["run", "main.ssrg"], directory)
      expect(result.status, `${item.slug}: ${result.stderr}`).toBe(0)
      expect(result.stderr).toBe("")
      expect(result.stdout, item.slug).toBe(item.output)
      expect(sample.sha256).toBe(
        createHash("sha256").update(sample.source).digest("hex")
      )
      expect(new URL(sample.playgroundUrl).searchParams.get("source")).toBe(
        sample.source
      )
      const comparison = examples.find(
        (x) => x.id === `numeric-reader-${item.slug}-ts`
      )!
      expect(comparison.playgroundUrl).toBe("")
      const ts = run("bun", [join(root, comparison.sourcePath)])
      expect(ts.status, ts.stderr).toBe(0)
      expect(ts.stderr).toBe("")
      expect(ts.stdout, item.slug).toBe(item.typescriptOutput)
    }
    writeFileSync(
      join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          noEmit: true,
          strict: true,
          skipLibCheck: true,
          target: "ES2022",
          lib: ["ESNext", "DOM"],
          module: "ESNext",
          moduleResolution: "Bundler",
          types: [],
        },
        files: examples
          .filter((x) => x.id.endsWith("-ts"))
          .map((x) => join(root, x.sourcePath)),
      })
    )
    const checked = run("bun", [
      join(root, "node_modules/typescript/bin/tsc"),
      "--project",
      join(directory, "tsconfig.json"),
    ])
    expect(checked.status, checked.stdout + checked.stderr).toBe(0)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("the exact Playground seeds compile through committed WASM and execute with the documented outputs", async () => {
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
  for (const item of numericReaderCases) {
    const sample = examples.find((x) => x.id === `numeric-reader-${item.slug}`)!
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
    expect(executed.stdout, item.slug).toBe(item.output.trimEnd())
  }
})

test("whole input, safe integer boundaries, operand order and Float special values follow the bounded contracts", async () => {
  const ints = await import(
    new URL("../../../runtime/ts/src/int.ts", import.meta.url).href
  )
  const floats = await import(
    new URL("../../../runtime/ts/src/float.ts", import.meta.url).href
  )
  for (const [text, expected] of [
    ["+12", 12],
    ["-0", 0],
    ["9007199254740991", ints.MAX_INT],
    ["-9007199254740991", ints.MIN_INT],
  ] as const)
    expect(ints.parse(text)).toEqual({ tag: "Right", value: expected })
  for (const text of [
    "",
    " 12",
    "12 ",
    "12\n",
    "12\r\n",
    "012",
    "12px",
    "12.0",
    "1_000",
    "0x10",
    "１２",
  ])
    expect(ints.parse(text).tag, text).toBe("Left")
  expect(ints.parse("")).toEqual({ tag: "Left", value: { tag: "EmptyInt" } })
  expect(ints.parse("12é")).toEqual({
    tag: "Left",
    value: { tag: "InvalidIntDigit", value: { offset: 2, radix: 10 } },
  })
  for (const text of [
    "9007199254740992",
    "-9007199254740992",
    "9007199254740992x",
  ])
    expect(ints.parse(text)).toEqual({
      tag: "Left",
      value: { tag: "IntOutsideRange" },
    })
  expect(ints.checkedAdd(5, 12)).toEqual({ tag: "Just", value: 17 })
  expect(ints.checkedAdd(1, ints.MAX_INT)).toEqual({ tag: "Nothing" })
  expect(ints.checkedSubtract(5, 12)).toEqual({ tag: "Just", value: 7 })
  expect(ints.checkedSubtract(12, 5)).toEqual({ tag: "Just", value: -7 })
  expect(ints.checkedSubtract(1, ints.MIN_INT)).toEqual({ tag: "Nothing" })
  expect(ints.checkedMultiply(5, 12)).toEqual({ tag: "Just", value: 60 })
  expect(ints.checkedMultiply(2, ints.MAX_INT)).toEqual({ tag: "Nothing" })
  for (const [divisor, dividend, quotient, remainder] of [
    [3, 10, 3, 1],
    [3, -10, -3, -1],
    [-3, 10, -3, 1],
    [3, -9, -3, 0],
    [3, ints.MAX_INT, 3002399751580330, 1],
    [10, 3, 0, 3],
  ] as const) {
    expect(ints.checkedDivide(divisor, dividend)).toEqual({
      tag: "Right",
      value: quotient,
    })
    expect(ints.checkedRemainder(divisor, dividend)).toEqual({
      tag: "Right",
      value: remainder,
    })
  }
  for (const result of [
    ints.checkedDivide(0, 10),
    ints.checkedRemainder(0, 10),
  ])
    expect(result).toEqual({ tag: "Left", value: { tag: "IntDivisionByZero" } })
  const zero = ints.checkedRemainder(3, -9)
  expect(zero.tag === "Right" && Object.is(zero.value, -0)).toBe(false)
  for (const [text, expected] of [
    [".5", 0.5],
    ["1.", 1],
    ["001.5", 1.5],
    ["1e3", 1000],
    ["Infinity", Infinity],
    ["-Infinity", -Infinity],
    ["NaN", NaN],
    ["-1e-999", -0],
  ] as const)
    expect(floats.parse(text)).toEqual({ tag: "Right", value: expected })
  for (const text of [
    " 12.5",
    "12.5 ",
    "12.5\n",
    "1e",
    "12px",
    "+Infinity",
    "infinity",
    "1_000.0",
    "0x10",
  ])
    expect(floats.parse(text).tag, text).toBe("Left")
  expect(floats.parse("")).toEqual({
    tag: "Left",
    value: { tag: "EmptyFloat" },
  })
  expect(floats.parse("1e309")).toEqual({
    tag: "Left",
    value: { tag: "FloatParseOverflow" },
  })
  expect(floats.format(12)).toBe("12.0")
  expect(floats.format(-0)).toBe("-0.0")
  expect(floats.format(1e21)).toBe("1e21")
  expect(floats.format(NaN)).toBe("NaN")
  for (const value of [ints.MIN_INT, ints.MAX_INT, 0, 5])
    expect(floats.fromInt(value)).toBe(value)
  for (const value of [0, -0, 12.5, -12.5, 1e30])
    expect(floats.isFinite(value)).toBe(true)
  for (const value of [NaN, Infinity, -Infinity])
    expect(floats.isFinite(value)).toBe(false)
})

test("native boundary fixture and plausible type mistakes retain their exact observations", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-numeric-boundaries-"))
  try {
    const source = readFileSync(
      join(root, "apps/site/tests/fixtures/numeric-boundaries.ssrg"),
      "utf8"
    )
    writeFileSync(join(directory, "main.ssrg"), source)
    const result = run(cli, ["run", "main.ssrg"], directory)
    expect(result.status, result.stderr).toBe(0)
    expect(result.stderr).toBe("")
    expect(result.stdout).toBe(
      readFileSync(
        join(root, "apps/site/tests/fixtures/numeric-boundaries.stdout"),
        "utf8"
      )
    )
    for (const source of [
      'import * as ints from "std/int"\npub fn bad -> Maybe<Int> = ints.checkedAdd 1.5 2\n',
      "pub fn measurement count: Int -> Float = count\n",
    ]) {
      writeFileSync(join(directory, "main.ssrg"), source)
      const rejected = run(cli, ["lint", "main.ssrg"], directory)
      expect(rejected.status).not.toBe(0)
      expect(rejected.stderr).toContain("SES-T0101")
    }
    // This uses each displayed TypeScript parser, not a separately maintained oracle.
    for (const slug of ["int-module", "int-parse"]) {
      const source = readFileSync(
        join(
          root,
          `apps/site/examples/comparisons/api-numeric-reader/${slug}.ts`
        ),
        "utf8"
      )
      writeFileSync(
        join(directory, "probe.ts"),
        source +
          '\nfor (const value of ["12\\n", "12\\r\\n", " 12", "12 ", "012", "12px", "9007199254740992"]) { if (readCount(value) !== undefined) throw new Error(JSON.stringify(value)) }\n'
      )
      const checked = run("bun", ["probe.ts"], directory)
      expect(checked.status, checked.stderr).toBe(0)
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("only eleven exact value/function identities and two exact module introductions receive numeric editorial", () => {
  const all = compilerReferenceModules()
  const selected = numericReaderCases
    .filter((x) => x.name)
    .map((item) => {
      const matches = all
        .find((m) => m.specifier === item.module)!
        .items.filter(
          (x) =>
            x.identity === item.identity &&
            x.namespace === "value" &&
            x.itemKind === "function"
        )
      expect(matches).toHaveLength(1)
      return matches[0]
    })
  expect(selected).toHaveLength(11)
  const probes = selected.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/math" },
    { ...symbol, namespace: "type" },
    { ...symbol, itemKind: "effect-function" },
    { ...symbol, identity: `std/math::${symbol.name}` },
  ])
  const controls = all
    .filter((m) => ["std/int", "std/float", "std/math"].includes(m.specifier))
    .flatMap((m) =>
      m.items.filter(
        (x) =>
          x.namespace === "value" &&
          x.itemKind === "function" &&
          !selected.some((y) => y.identity === x.identity)
      )
    )
  expect(controls).toHaveLength(47)
  const input = {
    schema: 1,
    origin: "",
    playgroundUrl: "",
    tourUrl: "",
    grammar: "",
    examples: [],
    referenceModules: [{ ...all[0], items: [...probes, ...controls] }],
  }
  const directory = mkdtempSync(join(tmpdir(), "seseragi-numeric-identities-"))
  try {
    const output = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      timeoutMs: 180_000,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/numeric-reader/catalog",
      ],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor, moduleEditorial } from "./reference/editorial/catalog"
import { numericReaderEditorialFor, numericReaderModuleBlocks } from "./reference/editorial/numeric-reader/catalog"
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [
    arrays.concat [[(present (numericReaderEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules],
    [(arrays.length (numericReaderModuleBlocks name) > 0, arrays.length (moduleEditorial name) > 0) | name <- ["std/int", "std/float", "std/math", "std/int-extra", "std/float::parse"]]
  ]))
}`,
    })
    expect(output).toHaveLength(107)
    for (let i = 0; i < 55; i++)
      expect(output[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    for (let i = 55; i < 102; i++) expect(output[i]).toEqual([false, false])
    expect(output.slice(102)).toEqual([
      [true, true],
      [true, true],
      [false, false],
      [false, false],
      [false, false],
    ])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all twenty-six production bodies preserve exact locale, declarations, sources, outputs and purpose-labelled title links", () => {
  const all = compilerReferenceModules()
  const selected = new Set<string>(numericReaderCases.map((x) => x.identity))
  const modules = all
    .filter((x) => ["std/int", "std/float"].includes(x.specifier))
    .map((m) => ({
      ...m,
      items: m.items.filter(
        (x) =>
          selected.has(x.identity) ||
          [
            "std/float::toInt",
            "std/float::roundIntegral",
            "std/int::maxValue",
            "std/int::IntParseError",
            "std/float::FloatParseError",
          ].includes(x.identity)
      ),
    }))
  const input = {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "",
    grammar: "",
    examples,
    referenceModules: modules,
  }
  const directory = mkdtempSync(join(tmpdir(), "seseragi-numeric-render-"))
  try {
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
    let site = SiteCatalog { home, pages, areas: [NavigationArea { id: "library", title: home.title, description: home.summary, landing: home, groups }] }
    println (json.encodeString [Output {route: localizedRoute locale page.path, html: renderDocument input site locale page} | locale <- [En, Ja], page <- pages])
  }
}`,
    })
    if (process.env.NUMERIC_READER_RENDER_DIR)
      for (const page of output) {
        const destination = join(
          process.env.NUMERIC_READER_RENDER_DIR,
          page.route.slice(1)
        )
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
    const pages = new Map(output.map((x) => [x.route, x.html]))
    const titles = new Map(output.map((x) => [x.route, pageTitle(x.html)]))
    const summaries = new Set<string>()
    let links = 0
    for (const item of numericReaderCases)
      for (const locale of ["en", "ja"]) {
        const prefix = locale === "en" ? "" : "/ja",
          route = prefix + item.route
        const html = pages.get(route)!
        expect(html, route).toBeDefined()
        const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0],
          text = plain(body)
        for (const anchor of [
          "using-this-operation",
          "typescript-comparison",
          "operation-rules",
        ])
          expect(body).toContain(`id="${anchor}"`)
        expect(body).not.toContain("Missing canonical example")
        const terminals = [
          ...body.matchAll(
            /<section class="code-panel terminal-panel"><div class="code-panel-header"><span class="code-panel-title">([^<]*)<\/span><\/div><pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
          ),
        ].map((m) => [plain(m[1]), plain(m[2])])
        expect(terminals, route).toEqual([
          [
            locale === "en" ? "Seseragi output" : "Seseragiの実行結果",
            item.output.trimEnd(),
          ],
          [
            locale === "en" ? "TypeScript output" : "TypeScriptの実行結果",
            item.typescriptOutput.trimEnd(),
          ],
        ])
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((m) => plain(m[1]))
        for (const suffix of ["", "-ts"]) {
          const sample = examples.find(
            (x) => x.id === `numeric-reader-${item.slug}${suffix}`
          )!
          expect(
            panels.filter((x) => x === sample.source),
            route + suffix
          ).toHaveLength(1)
        }
        const playgroundSources = [...body.matchAll(/href="([^"]+)"/gu)]
          .map((m) => new URL(plain(m[1]), input.origin))
          .filter(
            (x) =>
              x.origin === "https://seseragi.vercel.app" &&
              x.searchParams.has("source")
          )
          .map((x) => x.searchParams.get("source"))
        expect(playgroundSources).toEqual([
          examples.find((x) => x.id === `numeric-reader-${item.slug}`)!.source,
        ])
        const owner = item.module.slice(4),
          slug = item.name ? item.name.toLowerCase() : "module"
        const copy = (language: string) =>
          readFileSync(
            join(
              root,
              `apps/site/src/reference/editorial/numeric-reader/${owner}/${slug}/${language}.ssrg`
            ),
            "utf8"
          )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const own = copy(locale),
          other = copy(locale === "en" ? "ja" : "en")
        expect(paragraphs(own).length).toBeGreaterThanOrEqual(4)
        expect(paragraphs(own)).toHaveLength(paragraphs(other).length)
        for (const paragraph of paragraphs(own))
          expect(text, route).toContain(paragraph)
        for (const paragraph of paragraphs(other))
          expect(text, route).not.toContain(paragraph)
        for (const [, field, literal] of own.matchAll(
          /^ {2}(\w+): ("(?:[^"\\]|\\.)*")/gmu
        )) {
          if (field === "summary") {
            summaries.add(JSON.parse(literal))
            if (!item.name) continue
          }
          expect(plain(html), route + field).toContain(JSON.parse(literal))
        }
        if (item.name) {
          const symbol = modules
            .find((x) => x.specifier === item.module)!
            .items.find((x) => x.identity === item.identity)!
          expect(text).toContain(symbol.signature)
          expect(body).toContain('id="canonical-declaration"')
          expect(body).toContain(`href="${prefix}/docs/library/${owner}/"`)
          expect(pages.get(`${prefix}/docs/library/${owner}/`)).toContain(
            `href="${route}"`
          )
          expect(text).toContain(
            locale === "en" ? symbol.reading.en : symbol.reading.ja
          )
        }
        const entry = {
          route,
          kind: item.name ? ("api" as const) : ("module" as const),
        }
        const count = assertAuthoredLibraryTitles(html, titles, entry)
        expect(count, route).toBeGreaterThanOrEqual(2)
        links += count
        const mutated = html.replace(
          /(<p><a\b[^>]*href="[^"#]+">)([^<]+)(<\/a><span>：|<\/a><span>: )/u,
          "$1wrong title$3"
        )
        expect(mutated).not.toBe(html)
        expect(() =>
          assertAuthoredLibraryTitles(mutated, titles, entry)
        ).toThrow()
        expect(html).toContain(`href="${prefix ? "" : "/ja"}${item.route}"`)
      }
    expect(summaries.size).toBe(26)
    expect(links).toBe(74)
    for (const slug of ["toint", "roundintegral"])
      for (const prefix of ["", "/ja"]) {
        const html = pages.get(
          `${prefix}/docs/library/float/function/${slug}/`
        )!
        expect(html).toBeDefined()
        expect(html).not.toContain('id="typescript-comparison"')
        const symbol = modules
          .find((x) => x.specifier === "std/float")!
          .items.find((x) => x.name.toLowerCase() === slug)!
        expect(plain(html)).toContain(symbol.signature)
      }
    const max = pages.get("/docs/library/int/function/maxvalue/")!
    expect(max).not.toContain('id="typescript-comparison"')
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
