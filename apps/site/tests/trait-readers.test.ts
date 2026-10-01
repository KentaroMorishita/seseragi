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
import { dirname, join, resolve } from "node:path"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import {
  traitReaderExamples,
  traitReaderExtras,
  traitReaders,
} from "../scripts/trait-readers"
import { assertReferenceLinkTitles, pageTitle } from "./reference-titles"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = traitReaderExamples("https://seseragi.vercel.app/")
const quote = JSON.stringify
function example(id: string) {
  const found = examples.find((item) => item.id === id)
  if (!found) throw new Error(`Missing canonical example: ${id}`)
  return found
}
function run(command: string, args: string[], cwd: string) {
  return spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    timeout: 45_000,
    maxBuffer: 16 * 1024 * 1024,
  })
}
function instance(source: string) {
  const found = source.match(/instance [\s\S]*?\n\}/u)?.[0]
  expect(found).toBeDefined()
  return found ?? ""
}
function replace(source: string, before: string, after: string) {
  expect(source).toContain(before)
  return source.replace(before, after)
}
function repair(key: string, source: string, valid: string): [string, string] {
  switch (key) {
    case "model":
      return [`${instance(valid)}\n${source}`, "ticket-42\n"]
    case "declarations":
      return [replace(source, "MissingType", "A"), "ticket-42\n#42\n"]
    case "instances":
      return [
        replace(
          source,
          "-> Int = value.number",
          '-> String = "ticket-" + show value.number'
        ),
        "ready\n",
      ]
    case "constraints":
      return [`${instance(valid)}\n${source}`, "value: []\n"]
    case "methods-versus-traits":
      return [replace(source, "user.show", "show user"), "Aki\nUser(Aki)\n"]
    case "method-calls":
      return [
        replace(source, "label 42", "label ticket"),
        "ticket-42\nlog: ticket-42\n",
      ]
    case "deriving":
      return [replace(source, "deriving Label", "deriving Eq"), "ready\n"]
    case "standard-operators":
      return [replace(source, "22.0", "22"), "42\n"]
    case "coherence": {
      const duplicate = source.match(/instance Label<Ticket> \{[\s\S]*?\n\}/gu)
      expect(duplicate).toHaveLength(2)
      return [replace(source, duplicate?.[1] ?? "", ""), "ticket-42\n"]
    }
    case "laws":
      return [replace(source, '= "True"', "= value == value"), "True\nTrue\n"]
    case "do-notation":
      return [
        replace(
          replace(source, '"1 River Road" <- street', "streetName <- street"),
          '"1 River Road, " + cityName',
          'streetName + ", " + cityName'
        ),
        "1 River Road, Tokyo\naddress incomplete\n",
      ]
    case "do-block-typing":
      return [
        replace(source, "Right (customer", "pure (customer"),
        "Just Aki: 2\nNothing\n",
      ]
    case "do-desugaring":
      return [
        replace(source, "streetName: Maybe<String>", "streetName: String"),
        "Just 1 River Road, Tokyo\n",
      ]
    default:
      throw new Error(`Unverified repair ${key}`)
  }
}
function visible(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function code(panel: string) {
  return visible(
    panel.match(/<code class="seseragi-highlight">([\s\S]*?)<\/code>/u)?.[1] ??
      ""
  )
}

test("thirteen trait articles retain identities and canonical complete programs", () => {
  expect(traitReaders).toHaveLength(13)
  expect(examples).toHaveLength(30)
  expect(new Set(examples.map((item) => item.id)).size).toBe(examples.length)
  for (const item of examples) {
    const source = readFileSync(join(root, item.sourcePath), "utf8")
    expect(item.source).toBe(source)
    expect(item.highlighted.map((part) => part.text).join("")).toBe(source)
    expect(item.sha256).toBe(createHash("sha256").update(source).digest("hex"))
    if (item.id.endsWith("-invalid") || item.id.endsWith("-typescript"))
      expect(item.playgroundUrl).toBe("")
    else {
      expect(source).toContain("pub effect fn main")
      expect(sourceFromPlaygroundUrl(item.playgroundUrl)).toBe(source)
    }
  }
  for (const { key } of traitReaders) {
    const directory = join(root, "apps/site/src/pages/language/traits", key)
    const page = readFileSync(join(directory, "page.ssrg"), "utf8")
    expect(page).toContain(`"language.traits.${key}"`)
    expect(page).toContain(`"/docs/language/traits/${key}/"`)
    const guide = readFileSync(join(directory, "guide.ssrg"), "utf8")
    expect(guide).toContain(`ArticleExample "trait-reader-${key}"`)
  }
})

for (const item of traitReaders)
  test(`trait ${item.key}: native output, observed rejection and literal repair`, () => {
    const directory = mkdtempSync(join(tmpdir(), "seseragi-trait-example-"))
    try {
      const accepted = example(`trait-reader-${item.key}`).source
      writeFileSync(join(directory, "main.ssrg"), accepted)
      for (const command of ["lint", "run"]) {
        const result = run(cli, [command, "main.ssrg"], directory)
        expect(result.status, result.stderr || result.error?.message).toBe(0)
        expect(result.stderr).toBe("")
        expect(result.stdout).toBe(command === "run" ? item.output : "")
      }
      const invalid = example(`trait-reader-${item.key}-invalid`).source
      writeFileSync(join(directory, "main.ssrg"), invalid)
      const rejected = run(cli, ["lint", "main.ssrg"], directory)
      expect(rejected.status, rejected.stderr).toBe(2)
      expect(rejected.stderr).toContain(`[${item.diagnostic}]`)
      expect(rejected.stderr).toContain(item.message)
      const [fixed, output] = repair(item.key, invalid, accepted)
      writeFileSync(join(directory, "main.ssrg"), fixed)
      const corrected = run(cli, ["run", "main.ssrg"], directory)
      expect(corrected.status, corrected.stderr).toBe(0)
      expect(corrected.stderr).toBe("")
      expect(corrected.stdout).toBe(output)
    } finally {
      rmSync(directory, { recursive: true, force: true })
    }
  })

test("laws and Effect/generic Monad witnesses distinguish meaning, typechecking and supported sequencing", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-trait-boundaries-"))
  try {
    for (const item of traitReaderExtras) {
      writeFileSync(
        join(directory, "main.ssrg"),
        example(`trait-reader-${item.key}`).source
      )
      const result = run(cli, ["run", "main.ssrg"], directory)
      expect(result.status, result.stderr).toBe(0)
      expect(result.stdout).toBe(item.output)
    }
    const broken = example("trait-reader-laws-broken").source
    const corrected = replace(
      broken,
      "-> Bool = False",
      "-> Bool = left.number == right.number"
    )
    writeFileSync(join(directory, "main.ssrg"), corrected)
    const lawRepair = run(cli, ["run", "main.ssrg"], directory)
    expect(lawRepair.status, lawRepair.stderr).toBe(0)
    expect(lawRepair.stdout).toBe("True\nTrue\n")
    const invalid = example("trait-reader-generic-monad-invalid").source
    writeFileSync(join(directory, "main.ssrg"), invalid)
    const rejected = run(cli, ["lint", "main.ssrg"], directory)
    expect(rejected.status).toBe(2)
    expect(rejected.stderr).toContain("SES-T0101")
    expect(rejected.stderr).toContain("expected M<A>, received Int")
    const withoutInstance = `struct Box<A> { value: A }\n\n${invalid}`
      .replace("duplicate 42", "duplicate (Box { value: 42 })")
      .replace("println (show result)", 'println "ready"')
    writeFileSync(join(directory, "main.ssrg"), withoutInstance)
    const missingInstance = run(cli, ["lint", "main.ssrg"], directory)
    expect(missingInstance.status).toBe(2)
    expect(missingInstance.stderr).toContain("SES-T0201")
    expect(missingInstance.stderr).toContain("no Monad instance")
    writeFileSync(
      join(directory, "main.ssrg"),
      replace(invalid, "duplicate 42", "duplicate (Just 42)")
    )
    const repaired = run(cli, ["run", "main.ssrg"], directory)
    expect(repaired.status, repaired.stderr).toBe(0)
    expect(repaired.stdout).toBe("Just (42, 42)\n")
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("TypeScript interface comparison typechecks and produces the same labels", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-trait-typescript-"))
  try {
    const file = join(directory, "comparison.ts")
    const source = example("trait-reader-model-typescript").source
    writeFileSync(file, source)
    const args = [
      "--noEmit",
      "--skipLibCheck",
      "--strict",
      "--target",
      "ES2022",
      "--module",
      "Preserve",
      "--moduleResolution",
      "Bundler",
      file,
    ]
    const tsc = join(root, "node_modules/.bin/tsc")
    const checked = run(tsc, args, root)
    expect(checked.status, checked.stdout || checked.stderr).toBe(0)
    const result = run(process.execPath, [file], directory)
    expect(result.status, result.stderr).toBe(0)
    expect(result.stdout).toBe(
      traitReaders.find((item) => item.key === "model")?.output
    )
    writeFileSync(file, `${source}\nticketLabel.label({ number: "42" })\n`)
    const rejected = run(tsc, args, root)
    expect(rejected.status).not.toBe(0)
    expect(rejected.stdout).toContain("TS2322")
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all twenty-six trait locale articles render canonical sources, paired explanations and exact links", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-trait-reader-"))
  try {
    const encodedExamples = examples
      .map(
        (item) => `ExampleSource {
  id: ${quote(item.id)}, sourcePath: ${quote(item.sourcePath)}, source: ${quote(item.source)}, sha256: ${quote(item.sha256)}, playgroundUrl: ${quote(item.playgroundUrl)},
  highlighted: [${item.highlighted.map((part) => `HighlightPart { text: ${quote(part.text)}, className: ${quote(part.className)} }`).join(",")}]
}`
      )
      .join(",")
    const rendered = renderPageClosure<{
      pages: Array<{
        id: string
        locale: "en" | "ja"
        route: string
        title: string
        html: string
      }>
      counts: Array<{ id: string; english: number[]; japanese: number[] }>
    }>({
      directory,
      cli,
      timeoutMs: 180_000,
      modules: [
        ...traitReaders.map(({ key }) => `pages/language/traits/${key}/page`),
        "render/document",
      ],
      entry: `import * as arrays from "std/array"
import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localeCode, localizedRoute, translate } from "./model/locale"
import { SiteCatalog } from "./model/page"
import { ReaderCopy } from "./pages/language/reader-article"
import { renderDocument } from "./render/document"
${traitReaders
  .map(
    (
      { key },
      i
    ) => `import { page as page${i} } from "./pages/language/traits/${key}/page"
import * as en${i} from "./pages/language/traits/${key}/en"
import * as ja${i} from "./pages/language/traits/${key}/ja"`
  )
  .join("\n")}
struct Rendered deriving JsonEncode { id: String, locale: String, route: String, title: String, html: String }
struct Counts deriving JsonEncode { id: String, english: Array<Int>, japanese: Array<Int> }
struct Output deriving JsonEncode { pages: Array<Rendered>, counts: Array<Counts> }
fn sizes copy: ReaderCopy -> Array<Int> = [arrays.length copy.purpose, arrays.length copy.reading, arrays.length copy.rules, arrays.length copy.mistakes, arrays.length copy.related]
pub effect fn main = {
  let pages = [${traitReaders.map((_, i) => `page${i} ()`).join(", ")}]
  let input = BuildInput { schema: 1, origin: "https://seseragi.example", playgroundUrl: "https://seseragi.vercel.app/", tourUrl: "https://seseragi.vercel.app/tour/", grammar: "", referenceModules: [], examples: [${encodedExamples}] }
  let site = SiteCatalog { home: page0 (), areas: [], pages }
  println (json.encodeString (Output {
    pages: [Rendered { id: page.id, locale: localeCode locale, route: localizedRoute locale page.path, title: translate locale page.title, html: renderDocument input site locale page } | locale <- [En, Ja], page <- pages],
    counts: [${traitReaders.map(({ key }, i) => `Counts { id: "language.traits.${key}", english: sizes (en${i}.copy ()), japanese: sizes (ja${i}.copy ()) }`).join(", ")}]
  }))
}`,
    })
    expect(rendered.pages).toHaveLength(26)
    expect(rendered.counts).toHaveLength(13)
    const pages = new Map(rendered.pages.map((page) => [page.route, page.html]))
    const titles = new Map(
      [...pages].map(([route, html]) => [route, pageTitle(html)])
    )
    expect(
      [...pages].reduce(
        (count, [route, html]) =>
          count + assertReferenceLinkTitles(html, titles, route),
        0
      )
    ).toBe(78)
    for (const pair of rendered.counts) {
      expect(pair.english, pair.id).toEqual(pair.japanese)
      expect(pair.english[1]).toBeGreaterThan(0)
      expect(pair.english[2]).toBeGreaterThan(0)
      expect(pair.english[3]).toBeGreaterThan(0)
      expect(pair.english[4]).toBe(3)
    }
    for (const item of traitReaders)
      for (const locale of ["en", "ja"] as const) {
        const route = `${locale === "ja" ? "/ja" : ""}/docs/language/traits/${item.key}/`
        const page = rendered.pages.find(
          (candidate) => candidate.route === route
        )
        expect(page, route).toBeDefined()
        if (!page) continue
        expect(page.id).toBe(`language.traits.${item.key}`)
        expect(page.locale).toBe(locale)
        expect(pageTitle(page.html)).toBe(page.title)
        expect(page.html).not.toContain("site-build-error")
        for (const id of [
          "understand-this",
          "rules",
          "mistakes",
          "related-rules",
        ])
          expect(page.html).toContain(`id="${id}"`)
        const panels = [
          ...page.html.matchAll(
            /<section class="code-panel">[\s\S]*?<\/section>/gu
          ),
        ].map((match) => match[0])
        const primary = example(`trait-reader-${item.key}`)
        expect(code(panels[0] ?? "")).toBe(primary.source)
        expect(panels[0]).toContain('class="playground-link"')
        expect(
          panels.filter((panel) => code(panel) === primary.source)
        ).toHaveLength(1)
        const invalid = example(`trait-reader-${item.key}-invalid`)
        const rejectedPanel = panels.find(
          (panel) => code(panel) === invalid.source
        )
        expect(rejectedPanel).toBeDefined()
        expect(rejectedPanel).not.toContain('class="playground-link"')
        const body = visible(page.html)
        expect(body).toContain(item.output.trimEnd())
        expect(body).toContain(item.diagnostic)
        expect(body).toContain("seseragi run main.ssrg")
        if (item.key === "model")
          expect(
            panels.some(
              (panel) =>
                code(panel) === example("trait-reader-model-typescript").source
            )
          ).toBe(true)
        if (item.key === "laws") {
          expect(
            panels.some(
              (panel) =>
                code(panel) === example("trait-reader-laws-broken").source
            )
          ).toBe(true)
          expect(body).toContain("False\nTrue")
        }
        if (["method-calls", "coherence", "do-block-typing"].includes(item.key))
          expect(body).toContain(locale === "ja" ? "検証した" : "tested")
        const alternate = `${locale === "ja" ? "" : "/ja"}/docs/language/traits/${item.key}/`
        expect(page.html).toContain(`href="${alternate}"`)
        if (process.env.SESERAGI_TRAITS_OUTPUT) {
          const destination = join(
            resolve(process.env.SESERAGI_TRAITS_OUTPUT),
            route,
            "index.html"
          )
          mkdirSync(dirname(destination), { recursive: true })
          writeFileSync(destination, page.html)
        }
      }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
