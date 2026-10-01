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
import * as sums from "../../../runtime/ts/src/sum"
import { collectionTypeExamples } from "../scripts/collection-type-readers"
import {
  dataValidationReaderCases,
  dataValidationReaderExamples,
} from "../scripts/data-validation-readers"
import { nonemptyIteratorReaderExamples } from "../scripts/nonempty-iterator-readers"
import { compilerReferenceModules } from "../scripts/reference"
import {
  resultFoundationCases,
  resultFoundationExamples,
} from "../scripts/result-foundation-readers"
import { resultFoundationReading } from "../scripts/result-foundation-reading"
import { signatureReading } from "../scripts/signature-reading"
import {
  assertAuthoredLibraryTitles,
  authoredLibraryArticle,
} from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = resultFoundationExamples("https://seseragi.vercel.app/")
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

test("fourteen exact native and TypeScript sources execute identical documented outputs", () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-result-foundation-examples-")
  )
  try {
    for (const item of resultFoundationCases) {
      const sample = examples.find(
        (x) => x.id === `result-foundation-${item.slug}`
      )!
      writeFileSync(join(directory, "main.ssrg"), sample.source)
      const lint = run(cli, ["lint", "main.ssrg"], directory)
      expect(lint.status, `${item.slug}: ${lint.stderr}`).toBe(0)
      const native = run(cli, ["run", "main.ssrg"], directory)
      expect(native.status, `${item.slug}: ${native.stderr}`).toBe(0)
      expect(native.stderr).toBe("")
      expect(native.stdout).toBe(item.output)
      expect(new URL(sample.playgroundUrl).searchParams.get("source")).toBe(
        sample.source
      )
      const comparison = examples.find(
        (x) => x.id === `result-foundation-${item.slug}-ts`
      )!
      expect(comparison.playgroundUrl).toBe("")
      const ts = run("bun", [join(root, comparison.sourcePath)])
      expect(ts.status, ts.stderr).toBe(0)
      expect(ts.stdout).toBe(item.output)
    }
    const checked = run("bun", [
      join(root, "node_modules/typescript/bin/tsc"),
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
      ...resultFoundationCases.map((item) =>
        join(
          root,
          `apps/site/examples/comparisons/result-foundations/${item.slug}.ts`
        )
      ),
    ])
    expect(checked.status, checked.stdout + checked.stderr).toBe(0)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("the fourteen exact Playground seeds compile through committed WASM and execute", async () => {
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
  for (const item of resultFoundationCases) {
    const sample = examples.find(
      (x) => x.id === `result-foundation-${item.slug}`
    )!
    const seed = new URL(sample.playgroundUrl).searchParams.get("source")!
    const result = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    )
    expect(result.status, `${item.slug}: ${JSON.stringify(result)}`).toBe(
      "success"
    )
    expect(result.entry).toBeDefined()
    if (result.status !== "success" || !result.entry) continue
    const executed = await runtime.executeGeneratedModule(
      result.generated.typescript,
      result.entry
    )
    expect(executed.stdout).toBe(item.output.trimEnd())
  }
})

test("native constructor, sequence, transformation and rejection boundaries retain their contracts", () => {
  const cases = [
    {
      name: "mapright-skips-left",
      status: 0,
      output: "Left bad\n",
      diagnostic: "",
    },
    {
      name: "bimap-selected-only",
      status: 0,
      output: "Left 7\nRight 8\n",
      diagnostic: "",
    },
    {
      name: "mapright-does-not-flatten",
      status: 0,
      output: "Right Right 40\nRight 40\n",
      diagnostic: "",
    },
    {
      name: "swap-roundtrip",
      status: 0,
      output: "Right bad\nLeft 8\nLeft bad\nRight 8\n",
      diagnostic: "",
    },
    {
      name: "sequence-list-shape",
      status: 0,
      output: "Just `[2, 1]\nJust `[]\nLeft first\n",
      diagnostic: "",
    },
    {
      name: "maybe-sequence-eager-input",
      status: 70,
      output: "",
      diagnostic: "runtime defect",
    },
    {
      name: "either-sequence-eager-input",
      status: 70,
      output: "",
      diagnostic: "runtime defect",
    },
    {
      name: "just-eager-payload",
      status: 70,
      output: "",
      diagnostic: "runtime defect",
    },
    {
      name: "valid-eager-payload",
      status: 70,
      output: "",
      diagnostic: "runtime defect",
    },
    {
      name: "nothing-is-not-function",
      status: 2,
      output: "",
      diagnostic: "SES-T0101",
    },
    {
      name: "invalid-rejects-string",
      status: 2,
      output: "",
      diagnostic: "SES-T0101",
    },
    {
      name: "invalid-rejects-list",
      status: 2,
      output: "",
      diagnostic: "SES-T0101",
    },
    {
      name: "invalid-rejects-empty-list",
      status: 2,
      output: "",
      diagnostic: "SES-T0101",
    },
    {
      name: "maybe-exhaustive-match",
      status: 2,
      output: "",
      diagnostic: "SES-T0301",
    },
    {
      name: "right-is-not-unwrapped-int",
      status: 2,
      output: "",
      diagnostic: "SES-T0101",
    },
  ]
  const directory = mkdtempSync(join(tmpdir(), "seseragi-result-boundaries-"))
  try {
    for (const item of cases) {
      writeFileSync(
        join(directory, "main.ssrg"),
        readFileSync(
          join(
            root,
            `apps/site/tests/fixtures/result-foundations/${item.name}.ssrg`
          )
        )
      )
      const result = run(cli, ["run", "main.ssrg"], directory)
      expect(result.status, `${item.name}: ${result.stderr}`).toBe(item.status)
      expect(result.stdout, item.name).toBe(item.output)
      if (item.diagnostic)
        expect(result.stderr, item.name).toContain(item.diagnostic)
      else expect(result.stderr, item.name).toBe("")
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("mapRight and bimap call only the selected body exactly once", () => {
  const calls: string[] = []
  const failure = (value: string) => {
    calls.push(`error:${value}`)
    return `form: ${value}`
  }
  const success = (value: number) => {
    calls.push(`success:${value}`)
    return `seats=${value}`
  }
  expect(sums.mapRight(success, sums.Left("missing"))).toEqual(
    sums.Left("missing")
  )
  expect(sums.mapRight(success, sums.Right(2))).toEqual(sums.Right("seats=2"))
  expect(sums.bimap(failure, success, sums.Left("bad"))).toEqual(
    sums.Left("form: bad")
  )
  expect(sums.bimap(failure, success, sums.Right(3))).toEqual(
    sums.Right("seats=3")
  )
  expect(calls).toEqual(["success:2", "error:bad", "success:3"])
})

test("only the exact swap identity receives a branch-accurate declaration reading", () => {
  const all = compilerReferenceModules().flatMap((x) => x.items)
  const swap = all.find((x) => x.identity === "std/either::swap")!
  const key = {
    identity: swap.identity,
    module: swap.module,
    namespace: swap.namespace,
    kind: swap.itemKind,
  }
  const base = signatureReading(
    swap.signature,
    swap.itemKind,
    swap.typeParameters,
    swap.constraints
  )
  const changed = resultFoundationReading(key, base)
  expect(changed).toEqual(swap.reading)
  expect(changed.en).toContain(
    "Left contains the original Right value and Right contains the original Left value"
  )
  expect(changed.ja).toContain("Leftには元のRightの値、Rightには元のLeftの値")
  expect(changed.en).not.toContain("Left for failure and Right for success")
  expect(changed.ja).not.toContain("失敗はLeft、成功はRight")
  expect(
    changed.en.replace(
      "After swapping, Left contains the original Right value and Right contains the original Left value. The payloads are unchanged.",
      "Use match to handle Left for failure and Right for success."
    )
  ).toBe(base.en)
  expect(
    changed.ja.replace(
      "入れ替えた後のLeftには元のRightの値、Rightには元のLeftの値が入ります。中の値そのものは変わりません。",
      "失敗はLeft、成功はRightで表します。matchでどちらの場合も扱ってください。"
    )
  ).toBe(base.ja)
  for (const bad of [
    { ...key, identity: "std/other::swap" },
    { ...key, module: "std/prelude" },
    { ...key, namespace: "type" },
    { ...key, kind: "constructor" },
  ])
    expect(resultFoundationReading(bad, base)).toBe(base)
  const mapRight = all.find((x) => x.identity === "std/either::mapRight")!
  expect(mapRight.reading).toEqual(
    signatureReading(
      mapRight.signature,
      mapRight.itemKind,
      mapRight.typeParameters,
      mapRight.constraints
    )
  )
  expect(mapRight.reading.en).toContain(
    "Left for failure and Right for success"
  )
  expect(mapRight.reading.ja).toContain("失敗はLeft、成功はRight")
  expect(() =>
    resultFoundationReading(key, { en: "changed", ja: "changed" })
  ).toThrow("baseline changed")
})

test("only the exact Validation type clarifies its public constructors in Japanese", () => {
  const all = compilerReferenceModules().flatMap((x) => x.items)
  const symbol = all.find((x) => x.identity === "std/validation::Validation")!
  const key = {
    identity: symbol.identity,
    module: symbol.module,
    namespace: symbol.namespace,
    kind: symbol.itemKind,
  }
  const base = signatureReading(
    symbol.signature,
    symbol.itemKind,
    symbol.typeParameters,
    symbol.constraints
  )
  const fixed = resultFoundationReading(key, base)
  expect(fixed).toEqual(symbol.reading)
  expect(fixed.en).toBe(base.en)
  expect(fixed.ja).toContain(
    "公開されたValidとInvalidで値を作り、matchで中身を読めます"
  )
  expect(
    fixed.ja.replace(
      "この型の内部の表現は非公開です。公開されたValidとInvalidで値を作り、matchで中身を読めます。",
      "この型の内部の表現は非公開です。値を作ったり取り出したりするには、同じモジュールが提供する関数を使ってください。"
    )
  ).toBe(base.ja)
  for (const bad of [
    { ...key, identity: "std/validation::Other" },
    { ...key, module: "std/prelude" },
    { ...key, namespace: "value" },
    { ...key, kind: "type" },
  ])
    expect(resultFoundationReading(bad, base)).toBe(base)
  const map = all.find((x) => x.identity === "std/map::Map")!
  expect(map.reading).toEqual(
    signatureReading(
      map.signature,
      map.itemKind,
      map.typeParameters,
      map.constraints
    )
  )
  expect(map.reading.ja).toContain(
    "同じモジュールが提供する関数を使ってください"
  )
  expect(() =>
    resultFoundationReading(key, { en: "same", ja: "changed" })
  ).toThrow("baseline changed")
})

test("all fourteen exact type, constructor and function identities dispatch without broadening prior overlays", () => {
  const all = compilerReferenceModules()
  expect(all).toHaveLength(63)
  expect(all.flatMap((x) => x.items)).toHaveLength(1812)
  const selected = resultFoundationCases.map((item) => {
    const matches = all
      .flatMap((x) => x.items)
      .filter(
        (x) =>
          x.identity === item.identity &&
          x.module === item.module &&
          x.namespace === item.namespace &&
          x.itemKind === item.itemKind
      )
    expect(matches).toHaveLength(1)
    expect(matches[0].signature).toBe(item.signature)
    return matches[0]
  })
  const probes = selected.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/array" },
    { ...symbol, namespace: "other" },
    { ...symbol, itemKind: "effect-function" },
    { ...symbol, identity: `other::${symbol.name}` },
  ])
  const prior = all
    .flatMap((x) => x.items)
    .filter(
      (x) =>
        dataValidationReaderCases.some(
          (p) => p.name && p.identity === x.identity
        ) || x.identity === "std/transformer/state::get"
    )
  expect(prior).toHaveLength(13)
  probes.push(...prior)
  const untouchedInstances = all
    .flatMap((x) => x.items)
    .filter((x) =>
      ["std/maybe::Show", "std/either::Show", "std/validation::Show"].includes(
        x.identity
      )
    )
  expect(untouchedInstances).toHaveLength(3)
  probes.push(...untouchedInstances)
  const input = {
    schema: 1,
    origin: "",
    playgroundUrl: "",
    tourUrl: "",
    grammar: "",
    examples: [],
    referenceModules: [{ ...all[0], items: probes }],
  }
  const directory = mkdtempSync(join(tmpdir(), "seseragi-result-dispatch-"))
  try {
    const output = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      timeoutMs: 180_000,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/result-foundations/catalog",
      ],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor } from "./reference/editorial/catalog"
import { resultFoundationEditorialFor } from "./reference/editorial/result-foundations/catalog"
fn present value: Maybe<Editorial> -> Bool = match value {
 Just _ -> True
 Nothing -> False
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
 Left _ -> println "[]"
 Right input -> println (json.encodeString (arrays.concat [[(present (resultFoundationEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(output).toHaveLength(86)
    for (let i = 0; i < 70; i++)
      expect(output[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    for (let i = 70; i < 83; i++) expect(output[i]).toEqual([false, true])
    for (let i = 83; i < 86; i++) expect(output[i]).toEqual([false, false])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all twenty-eight production bodies retain exact copy, H1 link titles, declarations, sources and locale identities", () => {
  const all = compilerReferenceModules()
  const selected = new Set<string>([
    ...resultFoundationCases.map((x) => x.identity),
    ...dataValidationReaderCases.filter((x) => x.name).map((x) => x.identity),
    "std/non-empty-list::NonEmptyList",
  ])
  const controls = [
    "std/maybe::Show",
    "std/either::Show",
    "std/validation::Show",
  ]
  const modules = all
    .filter((x) =>
      [
        "std/maybe",
        "std/either",
        "std/validation",
        "std/prelude",
        "std/non-empty-list",
      ].includes(x.specifier)
    )
    .map((module) => ({
      ...module,
      items: module.items.filter(
        (symbol) =>
          selected.has(symbol.identity) || controls.includes(symbol.identity)
      ),
    }))
  const renderExamples = [
    ...examples,
    ...dataValidationReaderExamples("https://seseragi.vercel.app/"),
    ...collectionTypeExamples("https://seseragi.vercel.app/"),
    ...nonemptyIteratorReaderExamples("https://seseragi.vercel.app/"),
  ]
  const input = {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "",
    grammar: "",
    examples: renderExamples,
    referenceModules: modules,
  }
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-data-validation-render-")
  )
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
  let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference, title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
  let site = SiteCatalog { home, pages, areas: [NavigationArea { id: "library", title: home.title, description: home.summary, landing: home, groups }] }
  println (json.encodeString [Output {route: localizedRoute locale page.path, html: renderDocument input site locale page} | locale <- [En, Ja], page <- pages])
 }
}`,
    })
    const pages = new Map(output.map((page) => [page.route, page.html]))
    const titles = new Map(
      output.map((page) => [page.route, pageTitle(page.html)])
    )
    let titleChecks = 0
    for (const item of resultFoundationCases)
      for (const locale of ["en", "ja"] as const) {
        const prefix = locale === "en" ? "" : "/ja"
        const route = prefix + item.route
        const html = pages.get(route)!
        expect(html, route).toBeDefined()
        expect(pageTitle(html)).toBe(item.name || item.module)
        const body = html.match(/<article\b[\s\S]*?<\/article>/u)![0]
        const bodyText = plain(body)
        for (const anchor of [
          "using-this-operation",
          "typescript-comparison",
          "operation-rules",
        ])
          expect(body).toContain(`id="${anchor}"`)
        expect(body).not.toContain("Missing canonical example")
        expect(bodyText).toContain(item.output.trimEnd())
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((x) => plain(x[1]))
        for (const suffix of ["", "-ts"]) {
          const sample = examples.find(
            (x) => x.id === `result-foundation-${item.slug}${suffix}`
          )!
          expect(
            panels.filter((x) => x === sample.source),
            `${route}:${suffix}`
          ).toHaveLength(1)
          if (!suffix)
            expect(body).toContain(
              sample.playgroundUrl.replaceAll("&", "&amp;")
            )
          else expect(sample.playgroundUrl).toBe("")
        }
        const copy = (language: string) =>
          readFileSync(
            join(
              root,
              `apps/site/src/reference/editorial/result-foundations/${item.slug}/${language}.ssrg`
            ),
            "utf8"
          )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const own = copy(locale),
          other = copy(locale === "en" ? "ja" : "en")
        const wrapper = paragraphs(own).find((text) =>
          text.startsWith("pub effect fn main")
        )!
        const nativeSource = examples.find(
          (x) => x.id === `result-foundation-${item.slug}`
        )!.source
        expect(wrapper.includes("show"), route).toBe(
          /\bshow\b/u.test(nativeSource)
        )
        if (item.slug === "valid" || item.slug === "invalid") {
          for (const alias of [
            "validation.valid",
            "validation.invalid",
            "validation.invalidMany",
          ])
            expect(bodyText).not.toContain(alias)
        }
        if (item.slug === "nothing" && locale === "ja")
          expect(bodyText).not.toContain("後で文字列")
        expect(paragraphs(own).length).toBe(paragraphs(other).length)
        expect(paragraphs(own).length).toBeGreaterThanOrEqual(5)
        for (const paragraph of paragraphs(own))
          expect(bodyText, route).toContain(paragraph)
        for (const paragraph of paragraphs(other))
          expect(bodyText, route).not.toContain(paragraph)
        for (const [, field, literal] of own.matchAll(
          /^ {2}(\w+): ("(?:[^"\\]|\\.)*")/gmu
        )) {
          if (field === "summary" && !item.name) continue
          expect(plain(html), `${route}:${field}`).toContain(
            JSON.parse(literal)
          )
        }
        if (item.name) {
          const symbol = all
            .flatMap((x) => x.items)
            .find((x) => x.identity === item.identity)!
          expect(bodyText).toContain(symbol.signature)
          expect(body).toContain('id="canonical-declaration"')
          expect(bodyText).toContain(
            locale === "en" ? symbol.reading.en : symbol.reading.ja
          )
          if (item.slug === "swap")
            expect(bodyText).not.toContain(
              locale === "en"
                ? "Use match to handle Left for failure and Right for success."
                : "失敗はLeft、成功はRightで表します。"
            )
        }
        const entry = {
          route,
          kind: item.name ? ("api" as const) : ("module" as const),
        }
        titleChecks += assertAuthoredLibraryTitles(html, titles, entry)
        const authored = authoredLibraryArticle(html, entry)
        const choices = [
          ...authored.matchAll(
            /<p\b[^>]*>\s*<a\b[^>]*href="([^"]+)"[^>]*>[^<]+<\/a>\s*<span>([\s\S]*?)<\/span>\s*<\/p>/gu
          ),
        ]
        expect(choices.length, route).toBeGreaterThan(0)
        for (const choice of choices) {
          expect(pages.has(choice[1]), `${route} -> ${choice[1]}`).toBe(true)
          expect(plain(choice[2]).length).toBeGreaterThan(5)
        }
        const alternate = (locale === "en" ? "/ja" : "") + item.route
        expect(html).toContain(`href="${alternate}"`)
      }
    expect(titleChecks).toBeGreaterThanOrEqual(28)
    for (const prefix of ["", "/ja"])
      for (const control of [
        "/docs/library/maybe/instance/show-maybe-a/",
        "/docs/library/either/instance/show-either-a-b/",
        "/docs/library/validation/instance/show-validation-a-b/",
      ]) {
        const controlHtml = pages.get(prefix + control)!
        expect(controlHtml).not.toContain('id="typescript-comparison"')
        const owner = control.split("/")[3]
        const symbol = all
          .flatMap((x) => x.items)
          .find((x) => x.identity === `std/${owner}::Show`)!
        expect(plain(controlHtml)).toContain(symbol.signature)
        expect(plain(controlHtml)).toContain(
          prefix ? symbol.reading.ja : symbol.reading.en
        )
      }
    if (process.env.RESULT_FOUNDATION_READER_RENDER_DIR)
      for (const page of output) {
        const destination = join(
          process.env.RESULT_FOUNDATION_READER_RENDER_DIR,
          page.route.slice(1)
        )
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
