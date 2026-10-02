import { expect, setDefaultTimeout, test } from "bun:test"
import { createHash } from "node:crypto"
import { mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { highlightSeseragi } from "../../playground/src/editor/seseragi-language"
import { generatorInput } from "../scripts/build"
import { highlightTypeScript } from "../scripts/typescript-highlight"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(120_000)
const root = resolve(import.meta.dir, "../../..")
const input = generatorInput("https://seseragi.vercel.app/")
const typescript = input.examples.filter((entry) =>
  entry.sourcePath.endsWith(".ts")
)
const edgeSource = [
  "// 日本語 🐟",
  '/* const fake = "<tag>" */',
  "type Box<T> = { value: T };",
  "const active: boolean = true;",
  "const amount = 1_000n;",
  "const pattern = /a<[^>]+>/gu;",
  "const ratio = 8 / 2;",
  // biome-ignore lint/suspicious/noTemplateCurlyInString: literal TypeScript fixture
  'const value = `outer ${active ? `inner ${amount}` : "<&>"} tail`;',
  'const escaped = "\\\"quoted\\\" & \\n";',
  "",
].join("\r\n")

test("TypeScript classification preserves source and nested lexical contexts", () => {
  const parts = highlightTypeScript(edgeSource)
  expect(parts.map((part) => part.text).join("")).toBe(edgeSource)
  for (const [text, className] of [
    ["// 日本語 🐟", "tok-comment"],
    ['/* const fake = "<tag>" */', "tok-comment"],
    ["const", "tok-keyword"],
    ["T", "tok-typeName"],
    ["boolean", "tok-standardType"],
    ["true", "tok-bool"],
    ["1_000n", "tok-number"],
    ["/a<[^>]+>/gu", "tok-string"],
    ["/", "tok-operator"],
    ["`outer ${", "tok-string"],
    ["`inner ${", "tok-string"],
    ["amount", "tok-variableName"],
    ["} tail`", "tok-string"],
    ['"<&>"', "tok-string"],
  ])
    expect(parts).toContainEqual({ text, className })
  expect(highlightTypeScript("")).toEqual([])
  expect(highlightTypeScript("\t\r\n")).toEqual([
    { text: "\t\r\n", className: "" },
  ])
  const incomplete = 'const partial = `value ${active ? "yes" : '
  expect(
    highlightTypeScript(incomplete)
      .map((part) => part.text)
      .join("")
  ).toBe(incomplete)
})

test("keyword-free TypeScript calls classify identifiers, punctuation and strings without inventing keywords", () => {
  const source = 'console.log("Saved 3 files")\n'
  const parts = highlightTypeScript(source)
  expect(parts).toEqual([
    { text: "console", className: "tok-variableName" },
    { text: ".", className: "tok-punctuation" },
    { text: "log", className: "tok-variableName" },
    { text: "(", className: "tok-punctuation" },
    { text: '"Saved 3 files"', className: "tok-string" },
    { text: ")", className: "tok-punctuation" },
    { text: "\n", className: "" },
  ])
  expect(parts.map((part) => part.text).join("")).toBe(source)
  expect(parts.some((part) => part.className === "tok-keyword")).toBe(false)
})

test("every registered source uses its own grammar without changing bytes or hashes", () => {
  expect(typescript.length).toBeGreaterThan(0)
  for (const example of input.examples) {
    const source = readFileSync(join(root, example.sourcePath), "utf8")
    expect(example.source, example.id).toBe(source)
    expect(example.sha256, example.id).toBe(
      createHash("sha256").update(source).digest("hex")
    )
    expect(
      example.highlighted.map((part) => part.text).join(""),
      example.id
    ).toBe(source)
    if (example.sourcePath.endsWith(".ts")) {
      expect(example.highlighted, example.id).toEqual(
        highlightTypeScript(source)
      )
      expect(example.playgroundUrl, example.id).toBe("")
    } else if (example.sourcePath.endsWith(".ssrg")) {
      expect(example.highlighted, example.id).toEqual(
        highlightSeseragi(source).map(({ text, classes }) => ({
          text,
          className: classes,
        }))
      )
    } else {
      expect(example.sourcePath.endsWith(".toml"), example.id).toBe(true)
      expect(example.highlighted, example.id).toEqual([
        { text: source, className: "" },
      ])
    }
  }
})

function escapeHtml(text: string) {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
}

test("the production renderer emits every TypeScript token in both locales and leaves terminal output plain", () => {
  const sourceExamples = [
    ...typescript,
    input.examples.find((entry) => entry.sourcePath.endsWith(".ssrg"))!,
    input.examples.find((entry) => entry.sourcePath.endsWith(".toml"))!,
    {
      id: "typescript-escaping",
      sourcePath: "test.ts",
      source: edgeSource,
      sha256: createHash("sha256").update(edgeSource).digest("hex"),
      playgroundUrl: "",
      highlighted: highlightTypeScript(edgeSource),
    },
  ]
  const reference = input.referenceModules[0].items[0]
  const directory = mkdtempSync(join(tmpdir(), "seseragi-highlight-render-"))
  try {
    const output = renderPageClosure<
      Array<{ id: string; locale: string; html: string }>
    >({
      directory,
      modules: ["components/article", "i18n/messages"],
      stdin: `${JSON.stringify({ ...input, examples: sourceExamples, referenceModules: [{ ...input.referenceModules[0], items: [reference] }] })}\n`,
      entry: `import * as effects from "std/effect"
import * as inputs from "std/stdin"
import * as json from "std/json"
import * as arrays from "std/array"
import * as html from "std/web/html"
import { decodeBuildInput } from "./model/build"
import { En, Ja, localized, localeCode } from "./model/locale"
import { CodeExample, Terminal, ApiReference } from "./model/page"
import { messages } from "./i18n/messages"
import { renderBlock } from "./components/article"
type Failure deriving Show = | InputFailure | DecodeFailure | ConsoleFailure ConsoleError
struct Output deriving JsonEncode { id: String, locale: String, html: String }
fn requireInput value: Maybe<String> -> Either<Failure, String> = match value {
  Nothing -> Left InputFailure
  Just encoded -> Right encoded
}
pub effect fn main -> Unit with Console, Stdin fails Failure = do {
  limit <- inputs.lineLimit 67108864 |> effects.fromEither |> effects.mapError (\\_ -> InputFailure)
  line <- inputs.readLineWith limit |> effects.mapError (\\_ -> InputFailure)
  encoded <- requireInput line |> effects.fromEither
  input <- decodeBuildInput encoded |> effects.fromEither |> effects.mapError (\\_ -> DecodeFailure)
  let sources = [Output { id: example.id, locale: localeCode locale, html: html.renderToString (renderBlock input (messages locale) locale (CodeExample (example.id, localized "Source" "ソース"))) } | locale <- [En, Ja], example <- input.examples]
  let terminals = [Output { id: "terminal", locale: localeCode locale, html: html.renderToString (renderBlock input (messages locale) locale (Terminal (localized "Output" "出力", "const true = 123 <output> & text\\n"))) } | locale <- [En, Ja]]
  let apis = [Output { id: "api", locale: localeCode locale, html: html.renderToString (renderBlock input (messages locale) locale (ApiReference (${JSON.stringify(reference.identity)}, ${JSON.stringify(reference.namespace)}, ${JSON.stringify(reference.itemKind)}))) } | locale <- [En, Ja]]
  println (json.encodeString (arrays.concat [sources, terminals, apis])) |> effects.mapError ConsoleFailure
}`,
    })
    expect(output).toHaveLength(2 * (sourceExamples.length + 2))
    const byId = new Map(sourceExamples.map((entry) => [entry.id, entry]))
    const css = readFileSync(join(root, "apps/site/styles/code.css"), "utf8")
    for (const page of output) {
      expect(page.html, page.id).not.toContain("site-build-error")
      if (page.id === "terminal") {
        expect(page.html).toContain(
          "<pre><code>const true = 123 &lt;output&gt; &amp; text\n</code></pre>"
        )
        expect(page.html).not.toContain("tok-")
        continue
      }
      const example = page.id === "api" ? reference : byId.get(page.id)!
      const markup = example.highlighted
        .map(
          ({ text, className }) =>
            `<span class="${className}">${escapeHtml(text)}</span>`
        )
        .join("")
      expect(page.html, `${page.locale}/${page.id}`).toContain(
        `<code class="seseragi-highlight">${markup}</code>`
      )
      for (const { className } of example.highlighted)
        if (className) expect(css, className).toContain(`.${className}`)
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
