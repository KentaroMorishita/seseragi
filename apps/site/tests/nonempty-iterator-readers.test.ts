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
import * as iterator from "../../../runtime/ts/src/iterator"
import * as lists from "../../../runtime/ts/src/list"
import { Just, Nothing } from "../../../runtime/ts/src/sum"
import { apiCorrectionExamples } from "../scripts/api-corrections"
import {
  nonemptyIteratorReaderCases,
  nonemptyIteratorReaderExamples,
} from "../scripts/nonempty-iterator-readers"
import { compilerReferenceModules } from "../scripts/reference"
import { assertAuthoredLibraryTitles } from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = nonemptyIteratorReaderExamples("https://seseragi.vercel.app/")
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
test("eight canonical native and TypeScript programs execute their documented normal and boundary cases", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-sequence-examples-"))
  try {
    for (const item of nonemptyIteratorReaderCases) {
      const sample = examples.find(
        (x) => x.id === `sequence-reader-${item.slug}`
      )!
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
        (x) => x.id === `sequence-reader-${item.slug}-ts`
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
      ...nonemptyIteratorReaderCases.map((item) =>
        join(
          root,
          `apps/site/examples/comparisons/api-nonempty-iterator/${item.slug}.ts`
        )
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
  for (const item of nonemptyIteratorReaderCases) {
    const sample = examples.find(
      (x) => x.id === `sequence-reader-${item.slug}`
    )!
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

test("persistent pulls, lazy execution, end results and left reduction follow the documented contracts", () => {
  const calls: number[] = []
  const values = iterator.unfold((state: number) => {
    calls.push(state)
    return state > 0 ? Just([state, state - 1] as const) : Nothing
  }, 2)
  expect(calls).toEqual([])
  const first = iterator.next(values)
  const repeated = iterator.next(values)
  expect(calls).toEqual([2, 2])
  expect(first.tag).toBe("Just")
  expect(repeated.tag).toBe("Just")
  if (first.tag !== "Just" || repeated.tag !== "Just")
    throw Error("missing first")
  expect(first.value[0]).toBe(2)
  expect(repeated.value[0]).toBe(2)
  const second = iterator.next(first.value[1])
  expect(second.tag).toBe("Just")
  if (second.tag !== "Just") throw Error("missing second")
  expect(second.value[0]).toBe(1)
  expect(iterator.next(second.value[1])).toBe(Nothing)
  expect(iterator.next(second.value[1])).toBe(Nothing)
  expect(calls).toEqual([2, 2, 1, 0, 0])
  // A constant state remains endless without eventually overflowing Int.
  let repeatedMaximum = iterator.unfold(
    (value: number) => Just([value, value] as const),
    Number.MAX_SAFE_INTEGER
  )
  for (let index = 0; index < 4; index++) {
    const result = iterator.next(repeatedMaximum)
    expect(result.tag).toBe("Just")
    if (result.tag !== "Just") throw Error("constant iterator ended")
    expect(result.value[0]).toBe(Number.MAX_SAFE_INTEGER)
    repeatedMaximum = result.value[1]
  }
  const amounts = lists.consNonEmpty(20, lists.fromArray([3, 2]))
  const visits: number[][] = []
  const subtract = (a: number) => (b: number) => {
    visits.push([a, b])
    return a - b
  }
  expect(lists.reduce1NonEmpty(subtract, amounts)).toBe(15)
  expect(visits).toEqual([
    [20, 3],
    [17, 2],
  ])
  visits.length = 0
  expect(lists.reduce1NonEmpty(subtract, lists.singleton(20))).toBe(20)
  expect(visits).toEqual([])
  expect(lists.fromListNonEmpty(lists.Empty)).toBe(Nothing)
  const zero = lists.fromListNonEmpty(lists.fromArray([0, 20]))
  expect(zero.tag).toBe("Just")
  if (zero.tag === "Just") expect(lists.headNonEmpty(zero.value)).toBe(0)
  expect(lists.toArray(lists.toListNonEmpty(amounts))).toEqual([20, 3, 2])
})

test("List arguments, nonempty inputs and pure iterator steps reject plausible mistakes", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-sequence-invalid-"))
  try {
    const cases = [
      'import * as n from "std/non-empty-list"\npub fn bad -> NonEmptyList<Int> = n.cons 1 [2, 3]\n',
      'import * as n from "std/non-empty-list"\npub fn bad -> Int = n.reduce1 (\\a b -> a + b) `[]\n',
      'import * as i from "std/iterator"\npub fn bad -> Iterator<Unit> = i.unfold (\\state: Int -> println (show state)) 0\n',
    ]
    for (const source of cases) {
      writeFileSync(join(directory, "main.ssrg"), source)
      const result = run(cli, ["lint", "main.ssrg"], directory)
      expect(result.status, source).not.toBe(0)
      expect(result.stderr).toContain("SES-T0101")
    }
    writeFileSync(
      join(directory, "invalid.ts"),
      "const values: readonly [number, ...number[]] = [];\n"
    )
    const ts = run("bun", [
      "x",
      "tsc",
      "--noEmit",
      "--strict",
      "--skipLibCheck",
      join(directory, "invalid.ts"),
    ])
    expect(ts.status).not.toBe(0)
    expect(ts.stdout).toContain("TS2322")
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("six exact function identities and two exact module introductions receive editorial; prior corrections survive", () => {
  const all = compilerReferenceModules()
  const selected = nonemptyIteratorReaderCases
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
  expect(selected).toHaveLength(6)
  const probes = selected.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/list" },
    { ...symbol, namespace: "type" },
    { ...symbol, itemKind: "effect-function" },
    { ...symbol, identity: `std/list::${symbol.name}` },
  ])
  const controls = [
    ...all
      .find((m) => m.specifier === "std/non-empty-list")!
      .items.filter(
        (x) =>
          ["head", "tail", "toList"].includes(x.name) &&
          x.itemKind === "function"
      ),
    all
      .find((m) => m.specifier === "std/list")!
      .items.find((x) => x.name === "singleton" && x.itemKind === "function")!,
  ]
  expect(controls).toHaveLength(4)
  const input = {
    schema: 1,
    origin: "",
    playgroundUrl: "",
    tourUrl: "",
    grammar: "",
    examples: [],
    referenceModules: [{ ...all[0], items: [...probes, ...controls] }],
  }
  const directory = mkdtempSync(join(tmpdir(), "seseragi-sequence-identities-"))
  try {
    const output = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/nonempty-iterator-catalog",
      ],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor, moduleEditorial } from "./reference/editorial/catalog"
import { nonemptyIteratorEditorialFor, nonemptyIteratorModuleBlocks } from "./reference/editorial/nonempty-iterator-catalog"
fn present value: Maybe<Editorial> -> Bool = match value { Just _ -> True; Nothing -> False }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [
    arrays.concat [[(present (nonemptyIteratorEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules],
    [(arrays.length (nonemptyIteratorModuleBlocks name) > 0, arrays.length (moduleEditorial name) > 0) | name <- ["std/non-empty-list", "std/iterator", "std/list", "std/non-empty-list-extra", "std/iterator::next"]]
  ]))
}`,
    })
    expect(output).toHaveLength(39)
    for (let i = 0; i < 30; i++)
      expect(output[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    for (let i = 30; i < 34; i++) expect(output[i]).toEqual([false, true])
    expect(output.slice(34)).toEqual([
      [true, true],
      [true, true],
      [false, true],
      [false, false],
      [false, false],
    ])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
test("all sixteen production NonEmptyList and Iterator bodies preserve exact declarations, localized copy, outputs and source panels", () => {
  const all = compilerReferenceModules()
  const modules = all
    .filter((x) => ["std/non-empty-list", "std/iterator"].includes(x.specifier))
    .map((module) => ({
      ...module,
      items: module.items.filter(
        // Both opaque types are now authored by the type/constructor batch.
        // Keep one independent instance control per module instead.
        (x) =>
          x.itemKind === "function" ||
          ["std/non-empty-list::Hash", "std/iterator::Iterable"].includes(
            x.identity
          )
      ),
    }))
  expect(modules).toHaveLength(2)
  const allExamples = [
    ...examples,
    ...apiCorrectionExamples("https://seseragi.vercel.app/").filter((x) =>
      x.id.includes("nonempty-")
    ),
  ]
  const input = {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "",
    grammar: "",
    examples: allExamples,
    referenceModules: modules,
  }
  const directory = mkdtempSync(join(tmpdir(), "seseragi-sequence-render-"))
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
    expect(output).toHaveLength(26)
    const pages = new Map(output.map((p) => [p.route, p.html]))
    for (const item of nonemptyIteratorReaderCases)
      for (const locale of ["en", "ja"]) {
        const prefix = locale === "en" ? "" : "/ja"
        const owner = item.module.slice(4)
        const route = `${prefix}/docs/library/${owner}/${item.name ? `function/${item.slug}/` : ""}`
        const html = pages.get(route)!
        const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0]
        const text = plain(body)
        for (const anchor of [
          "using-this-operation",
          "typescript-comparison",
          "operation-rules",
        ])
          expect(body).toContain(`id="${anchor}"`)
        const terminals = [
          ...body.matchAll(
            /<section class="code-panel terminal-panel"><div class="code-panel-header"><span class="code-panel-title">([^<]*)<\/span><\/div><pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
          ),
        ].map((match) => [plain(match[1]), plain(match[2])])
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
        expect(body).not.toContain("Missing canonical example")
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((x) => plain(x[1]))
        for (const suffix of ["", "-ts"]) {
          const sample = examples.find(
            (x) => x.id === `sequence-reader-${item.slug}${suffix}`
          )!
          expect(
            panels.filter((x) => x === sample.source),
            `${route}:${suffix}`
          ).toHaveLength(1)
        }
        const playgroundSources = [...body.matchAll(/href="([^"]+)"/gu)]
          .map((match) => new URL(plain(match[1]), input.origin))
          .filter((url) => url.origin === "https://seseragi.vercel.app")
          .filter((url) => url.searchParams.has("source"))
          .map((url) => url.searchParams.get("source"))
        expect(playgroundSources, route).toEqual([
          examples.find((x) => x.id === `sequence-reader-${item.slug}`)!.source,
        ])
        const own = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/${owner}/${item.name ? item.slug : "module"}/${locale}.ssrg`
          ),
          "utf8"
        )
        const other = readFileSync(
          join(
            root,
            `apps/site/src/reference/editorial/${owner}/${item.name ? item.slug : "module"}/${locale === "en" ? "ja" : "en"}.ssrg`
          ),
          "utf8"
        )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const expected = paragraphs(own)
        expect(expected.length).toBeGreaterThanOrEqual(4)
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
          const symbol = modules
            .find((x) => x.specifier === item.module)!
            .items.find((x) => x.identity === item.identity)!
          expect(text).toContain(symbol.signature)
          expect(body).toContain('id="canonical-declaration"')
          expect(body).toContain(`href="${prefix}/docs/library/${owner}/"`)
          expect(pages.get(`${prefix}/docs/library/${owner}/`)).toContain(
            `href="${route}"`
          )
        }
      }
    const titles = new Map(output.map((p) => [p.route, pageTitle(p.html)]))
    let links = 0
    for (const item of nonemptyIteratorReaderCases)
      for (const prefix of ["", "/ja"]) {
        const route = `${prefix}/docs/library/${item.module.slice(4)}/${item.name ? `function/${item.slug}/` : ""}`
        const entry = {
          route,
          kind: item.name ? ("api" as const) : ("module" as const),
        }
        links += assertAuthoredLibraryTitles(pages.get(route)!, titles, entry)
        const mutated = pages
          .get(route)!
          .replace(
            /(<p><a\b[^>]*href="[^"#]+">)([^<]+)(<\/a><span>：|<\/a><span>: )/u,
            "$1wrong title$3"
          )
        expect(mutated).not.toBe(pages.get(route))
        expect(() =>
          assertAuthoredLibraryTitles(mutated, titles, entry)
        ).toThrow()
      }
    expect(links).toBe(30)
    for (const module of modules)
      for (const control of module.items.filter(
        (x) => x.namespace === "instance"
      ))
        for (const prefix of ["", "/ja"]) {
          const html = pages.get(
            `${prefix}/docs/library/${module.specifier.slice(4)}/${control.itemKind}/${control.name.toLowerCase().replaceAll("<", "-").replaceAll(">", "").replaceAll(", ", "-")}/`
          )!
          expect(html).toBeDefined()
          expect(html).not.toContain('id="typescript-comparison"')
          expect(control.namespace).toBe("instance")
          expect(plain(html)).toContain(control.signature)
          expect(plain(html)).toContain(
            prefix ? control.reading.ja : control.reading.en
          )
          expect(html).not.toContain('id="using-this-type"')
        }
    for (const slug of ["head", "tail", "tolist"])
      for (const prefix of ["", "/ja"]) {
        const html = pages.get(
          `${prefix}/docs/library/non-empty-list/function/${slug}/`
        )!
        expect(html).toContain('id="typescript-comparison"')
        expect(html).not.toContain("Missing canonical example")
        const source = allExamples.find(
          (x) => x.id === `api-correction-nonempty-${slug}`
        )!.source
        expect(plain(html)).toContain(source)
      }
    if (process.env.SEQUENCE_READER_RENDER_DIR)
      for (const page of output) {
        const destination = join(
          process.env.SEQUENCE_READER_RENDER_DIR,
          page.route.slice(1)
        )
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
