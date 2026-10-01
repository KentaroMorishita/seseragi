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
import * as lists from "../../../runtime/ts/src/list"
import * as sums from "../../../runtime/ts/src/sum"
import { traverseValues } from "../../../runtime/ts/src/traversable"
import * as validation from "../../../runtime/ts/src/validation"
import {
  dataValidationReaderCases,
  dataValidationReaderExamples,
} from "../scripts/data-validation-readers"
import { compilerReferenceModules } from "../scripts/reference"
import {
  assertAuthoredLibraryTitles,
  authoredLibraryArticle,
} from "./library-titles"
import { pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = dataValidationReaderExamples("https://seseragi.vercel.app/")
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

test("fifteen exact native and TypeScript sources execute identical documented outputs", () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-data-validation-examples-")
  )
  try {
    for (const item of dataValidationReaderCases) {
      const sample = examples.find(
        (x) => x.id === `data-validation-reader-${item.slug}`
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
        (x) => x.id === `data-validation-reader-${item.slug}-ts`
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
      ...dataValidationReaderCases.map((item) =>
        join(
          root,
          `apps/site/examples/comparisons/api-data-validation/${item.slug}.ts`
        )
      ),
    ])
    expect(checked.status, checked.stdout + checked.stderr).toBe(0)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("the fifteen exact Playground seeds compile through committed WASM and execute", async () => {
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
  for (const item of dataValidationReaderCases) {
    const sample = examples.find(
      (x) => x.id === `data-validation-reader-${item.slug}`
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

test("runtime instrumentation confirms callback order, branch selection and complete ordered errors", () => {
  const visited: number[] = []
  const missing = traverseValues(
    [0, 2, 3],
    (n: number) => {
      visited.push(n)
      return n === 0 ? sums.Nothing : sums.Just(n)
    },
    sums.maybeApplicative,
    (x: readonly number[]) => x
  )
  expect(missing).toEqual(sums.Nothing)
  expect(visited).toEqual([0, 2, 3])
  visited.length = 0
  const failed = traverseValues(
    [0, -1, 3],
    (n: number) => {
      visited.push(n)
      return n <= 0 ? sums.Left(`bad ${n}`) : sums.Right(n)
    },
    sums.eitherApplicative,
    (x: readonly number[]) => x
  )
  expect(failed).toEqual(sums.Left("bad 0"))
  expect(visited).toEqual([0, -1, 3])
  const selected: string[] = []
  const left = (value: string) => {
    selected.push("left")
    return value
  }
  const right = (value: number) => {
    selected.push("right")
    return String(value)
  }
  expect(sums.fold(left, right, sums.Right(7))).toBe("7")
  expect(sums.fold(left, right, sums.Left("bad"))).toBe("bad")
  expect(selected).toEqual(["right", "left"])
  selected.length = 0
  expect(sums.mapLeft(left, sums.Right(7))).toEqual(sums.Right(7))
  expect(selected).toEqual([])
  const two = validation.validationApplicative.apply(
    validation.invalid("name")
  )(validation.invalid("seats"))
  expect(validation.toEither(two)).toEqual(
    sums.Left(lists.consNonEmpty("name", lists.fromArray(["seats"])))
  )
  expect(validation.fromEither(sums.Left(["first", "second"]))).toEqual(
    validation.invalid(["first", "second"])
  )
  expect(validation.toEither(validation.valid(7))).toEqual(sums.Right(7))
})

test("native boundary probes preserve eager fallbacks, traversal, skipped callbacks and rejection diagnostics", () => {
  const probes = [
    {
      name: "withdefault-eager",
      source:
        'import * as maybe from "std/maybe"\npub effect fn main = println (show (maybe.withDefault (1 / 0) (Just 7)))\n',
      status: 70,
      output: "",
      diagnostic: "runtime defect",
    },
    {
      name: "orelse-eager",
      source:
        'import * as maybe from "std/maybe"\npub effect fn main = println (show (maybe.orElse (Just (1 / 0)) (Just 7)))\n',
      status: 70,
      output: "",
      diagnostic: "runtime defect",
    },
    {
      name: "coalesce-lazy",
      source: "pub effect fn main = println (show ((Just 7) ?? (1 / 0)))\n",
      status: 0,
      output: "7\n",
      diagnostic: "",
    },
    {
      name: "maybe-traverse-eager",
      source:
        'import * as maybe from "std/maybe"\nfn check n: Int -> Maybe<Int> = if n == 0 then Nothing else Just (1 / (n - 2))\npub effect fn main = println (show (maybe.traverse check [0, 2]))\n',
      status: 70,
      output: "",
      diagnostic: "runtime defect",
    },
    {
      name: "either-traverse-eager",
      source:
        'import * as either from "std/either"\nfn check n: Int -> Either<String, Int> = if n == 0 then Left "first" else Right (1 / (n - 2))\npub effect fn main = println (show (either.traverse check [0, 2]))\n',
      status: 70,
      output: "",
      diagnostic: "runtime defect",
    },
    {
      name: "fold-selected-only",
      source:
        'import * as either from "std/either"\nfn broken n: Int -> Int = n / 0\npub effect fn main = do {\n let right: Either<Int, Int> = Right 7\n let left: Either<Int, Int> = Left 8\n println (show (either.fold broken (\\n: Int -> n) right))\n println (show (either.fold (\\n: Int -> n) broken left))\n}\n',
      status: 0,
      output: "7\n8\n",
      diagnostic: "",
    },
    {
      name: "mapleft-skips-right",
      source:
        'import * as either from "std/either"\nfn broken n: Int -> Int = n / 0\npub effect fn main = do {\n let right: Either<Int, Int> = Right 7\n println (show (either.mapLeft broken right))\n}\n',
      status: 0,
      output: "Right 7\n",
      diagnostic: "",
    },
    {
      name: "flatmap-skips-left-and-nothing",
      source:
        'fn broken n: Int -> Either<String, Int> = Right (n / 0)\npub effect fn main = do {\n let left: Either<String, Int> = Left "bad"\n let absent: Maybe<Int> = Nothing\n println (show (flatMap broken left))\n println (show (flatMap (\\n: Int -> Just (n / 0)) absent))\n println (show (flatMap (\\n: Int -> [n, n + 10]) [1, 2]))\n}\n',
      status: 0,
      output: "Left bad\nNothing\n[1, 11, 2, 12]\n",
      diagnostic: "",
    },
    {
      name: "invalidmany-rejects-empty-list",
      source:
        'import * as validation from "std/validation"\nfn rejected -> validation.Validation<String, Int> = {\n let errors: List<String> = `[]\n validation.invalidMany errors\n}\npub effect fn main = println (show (rejected ()))\n',
      status: 2,
      output: "",
      diagnostic: "SES-T0101",
    },
    {
      name: "invalidmany-rejects-nonempty-list",
      source:
        'import * as validation from "std/validation"\nfn rejected -> validation.Validation<String, Int> = validation.invalidMany `["bad"]\npub effect fn main = println (show (rejected ()))\n',
      status: 2,
      output: "",
      diagnostic: "SES-T0101",
    },
    {
      name: "validation-no-flatmap",
      source:
        'import * as validation from "std/validation"\nfn rejected value: validation.Validation<String, Int> -> validation.Validation<String, Int> = flatMap (\\n: Int -> validation.valid (n + 1)) value\npub effect fn main = println (show (rejected (validation.valid 1)))\n',
      status: 2,
      output: "",
      diagnostic: "SES-T0201",
    },
  ]
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-data-validation-boundaries-")
  )
  try {
    for (const probe of probes) {
      writeFileSync(join(directory, "main.ssrg"), probe.source)
      const result = run(cli, ["run", "main.ssrg"], directory)
      expect(result.status, `${probe.name}: ${result.stderr}`).toBe(
        probe.status
      )
      expect(result.stdout, probe.name).toBe(probe.output)
      if (probe.diagnostic)
        expect(result.stderr, probe.name).toContain(probe.diagnostic)
      else expect(result.stderr, probe.name).toBe("")
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("exact guarded dispatch covers twelve functions and preserves unrelated same-name controls", () => {
  const all = compilerReferenceModules()
  expect(all).toHaveLength(63)
  expect(all.flatMap((x) => x.items)).toHaveLength(1812)
  const selected = dataValidationReaderCases
    .filter((x) => x.name)
    .map((item) => {
      const found = all
        .flatMap((x) => x.items)
        .filter(
          (x) =>
            x.identity === item.identity &&
            x.module === item.module &&
            x.namespace === "value" &&
            x.itemKind === "function"
        )
      expect(found).toHaveLength(1)
      return found[0]
    })
  expect(selected).toHaveLength(12)
  const probes = selected.flatMap((symbol) => [
    symbol,
    { ...symbol, module: "std/array" },
    { ...symbol, namespace: "type" },
    { ...symbol, itemKind: "effect-function" },
    { ...symbol, identity: `other::${symbol.name}` },
  ])
  const state = all
    .flatMap((x) => x.items)
    .find((x) => x.identity === "std/transformer/state::get")!
  expect(state).toBeDefined()
  probes.push(state)
  const input = {
    schema: 1,
    origin: "",
    playgroundUrl: "",
    tourUrl: "",
    grammar: "",
    examples: [],
    referenceModules: [{ ...all[0], items: probes }],
  }
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-data-validation-dispatch-")
  )
  try {
    const output = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/data-validation-catalog",
      ],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor } from "./reference/editorial/catalog"
import { dataValidationEditorialFor } from "./reference/editorial/data-validation-catalog"
fn present value: Maybe<Editorial> -> Bool = match value {
 Just _ -> True
 Nothing -> False
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
 Left _ -> println "[]"
 Right input -> println (json.encodeString (arrays.concat [[(present (dataValidationEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(output).toHaveLength(61)
    for (let i = 0; i < 60; i++)
      expect(output[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    expect(output[60]).toEqual([false, true])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all thirty production bodies retain exact copy, H1 link titles, declarations, sources and locale identities", () => {
  const all = compilerReferenceModules()
  const selected = new Set<string>(
    dataValidationReaderCases.filter((x) => x.name).map((x) => x.identity)
  )
  const controls = [
    "std/maybe::Show",
    "std/either::Show",
    "std/validation::Show",
    "std/collection::reduceUntil",
  ]
  // These three former controls became authored in the result-foundation batch.
  // Preserve their count with unchanged Show instance metadata; this is not a
  // claim that the instance pages received independent reader review.
  const expectedControls = [
    ["std/maybe::Show", "instance<A> Show<Maybe<A>> where Show<A>"],
    [
      "std/either::Show",
      "instance<A, B> Show<Either<A, B>> where Show<A>, Show<B>",
    ],
    [
      "std/validation::Show",
      "instance<A, B> Show<Validation<A, B>> where Show<A>, Show<B>",
    ],
  ] as const
  for (const [identity, signature] of expectedControls) {
    const symbol = all
      .flatMap((x) => x.items)
      .find((x) => x.identity === identity)!
    expect(symbol.namespace).toBe("instance")
    expect(symbol.itemKind).toBe("instance")
    expect(symbol.signature).toBe(signature)
    expect(symbol.reading.en).toContain(
      "an instance is not itself a callable function"
    )
    expect(symbol.reading.ja).toContain(
      "instance自体を関数として呼び出すことはできません"
    )
  }
  const modules = all
    .filter((x) =>
      [
        "std/maybe",
        "std/either",
        "std/validation",
        "std/prelude",
        "std/collection",
      ].includes(x.specifier)
    )
    .map((module) => ({
      ...module,
      items: module.items.filter(
        (symbol) =>
          selected.has(symbol.identity) || controls.includes(symbol.identity)
      ),
    }))
  expect(modules).toHaveLength(5)
  expect(modules.flatMap((x) => x.items)).toHaveLength(16)
  const input = {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "",
    grammar: "",
    examples,
    referenceModules: modules,
  }
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-data-validation-render-")
  )
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
    for (const item of dataValidationReaderCases)
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
            (x) => x.id === `data-validation-reader-${item.slug}${suffix}`
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
              `apps/site/src/reference/editorial/data-validation/${item.slug}/${language}.ssrg`
            ),
            "utf8"
          )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (m) => JSON.parse(m[1]) as string
          )
        const own = copy(locale),
          other = copy(locale === "en" ? "ja" : "en")
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
          if (item.slug === "flatmap")
            expect(bodyText).not.toContain(
              locale === "en" ? symbol.descriptionEn : symbol.descriptionJa
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
    expect(titleChecks).toBeGreaterThanOrEqual(30)
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
    if (process.env.DATA_VALIDATION_READER_RENDER_DIR)
      for (const page of output) {
        const destination = join(
          process.env.DATA_VALIDATION_READER_RENDER_DIR,
          page.route.slice(1)
        )
        mkdirSync(destination, { recursive: true })
        writeFileSync(join(destination, "index.html"), page.html)
      }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
