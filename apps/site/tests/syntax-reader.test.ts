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
import type {
  CompileResponse,
  EntryContract,
} from "../../playground/src/compiler/types"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { canonicalExample } from "../scripts/canonical-example"
import { syntaxExampleCases, syntaxExamples } from "../scripts/syntax-examples"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(200_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const playgroundUrl = "https://seseragi.vercel.app/"
const examples = syntaxExamples(playgroundUrl)
const quote = JSON.stringify

function exampleById(id: string) {
  const example = examples.find((value) => value.id === id)
  if (!example) throw new Error(`Missing test example: ${id}`)
  return example
}

function text(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
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
  return text(
    panel.match(/<code class="seseragi-highlight">([\s\S]*?)<\/code>/u)?.[1] ??
      ""
  )
}

function terminals(html: string) {
  return [
    ...html.matchAll(
      /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
    ),
  ].map((match) => text(match[1]))
}

test("syntax example metadata preserves source bytes and runnable Playground seeds", () => {
  expect(syntaxExampleCases).toHaveLength(11)
  expect(examples).toHaveLength(22)
  expect(new Set(examples.map((example) => example.id)).size).toBe(22)
  for (const example of examples) {
    const source = readFileSync(resolve(root, example.sourcePath), "utf8")
    expect(example.source, example.id).toBe(source)
    expect(example.sha256, example.id).toBe(
      createHash("sha256").update(source).digest("hex")
    )
    expect(example.highlighted.map((part) => part.text).join("")).toBe(source)
    if (example.id.endsWith("-invalid")) {
      expect(example.playgroundUrl).toBe("")
    } else {
      expect(example.playgroundUrl).not.toBe(playgroundUrl)
      expect(sourceFromPlaygroundUrl(example.playgroundUrl)).toBe(source)
    }
  }
  for (const key of ["layout", "pipelines"]) {
    expect(exampleById(`syntax-reader-${key}`).source).toMatch(
      /\n[\t ]*3\n[\t ]*\|> double\n[\t ]*\|> addOne/u
    )
  }
  expect(exampleById("syntax-reader-layout").source).toContain(
    "fn addOne value: Int -> Int = {\n"
  )
  const escapes = exampleById("syntax-reader-escapes").source
  expect(escapes).toContain('let quote = "\\\"Seseragi\\\""')
  expect(escapes).toContain('let path = "docs\\\\guide"')
  expect(escapes).toContain("let lambda: Char = '\\u{03BB}'")
  expect(escapes).toContain('let lines = "first\\nsecond"')
  expect(
    syntaxExampleCases.find((example) => example.key === "escapes")
      ?.expectedOutput
  ).toBe('"Seseragi"\ndocs\\guide\nλ\nfirst\nsecond\n')
})

for (const example of syntaxExampleCases) {
  test(`syntax native ${example.key}: exact output and rejected diagnostic`, () => {
    const temporary = mkdtempSync(join(tmpdir(), "seseragi-syntax-example-"))
    try {
      // Keep each source outside apps/site so lint checks the standalone file.
      const source = exampleById(`syntax-reader-${example.key}`).source
      writeFileSync(join(temporary, "main.ssrg"), source)
      for (const command of ["lint", "run"]) {
        const result = spawnSync(cli, [command, "main.ssrg"], {
          cwd: temporary,
          encoding: "utf8",
          timeout: 30_000,
        })
        expect(
          result.status,
          `${example.key} ${command}: ${result.stderr || result.error?.message}`
        ).toBe(0)
        expect(result.stderr).toBe("")
        expect(result.stdout).toBe(
          command === "run" ? example.expectedOutput : ""
        )
      }
      writeFileSync(
        join(temporary, "main.ssrg"),
        exampleById(`syntax-reader-${example.key}-invalid`).source
      )
      const rejected = spawnSync(cli, ["lint", "main.ssrg"], {
        cwd: temporary,
        encoding: "utf8",
        timeout: 30_000,
      })
      expect(rejected.error, example.key).toBeUndefined()
      expect(rejected.status, rejected.stderr).toBe(2)
      expect(rejected.stderr).toContain(`[${example.expectedDiagnostic}]`)
      expect(rejected.stdout).toBe("")
    } finally {
      rmSync(temporary, { recursive: true, force: true })
    }
  })
}

test("the documented syntax corrections compile and produce their stated results", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-syntax-repairs-"))
  try {
    const repairs: Record<string, (source: string) => [string, string]> = {
      "source-text": (source) => [
        `// greeting\n${source.replace("/* greeting */", "")}`,
        "Hello\n",
      ],
      literals: (source) => [source.replace("1__0", "1_0"), "10\n"],
      escapes: (source) => [
        source.replace(String.raw`\q`, String.raw`\\q`),
        "\\q\n",
      ],
      layout: (source) => [`${source}}\n`, "1\n"],
      names: (source) => [source.replaceAll("UserName", "userName"), "Aki\n"],
      "optional-fields": (source) => [
        source.replace(
          "= profile.nickname",
          "= profile.nickname ?? profile.name"
        ),
        "Aki\n",
      ],
      methods: (source) => [source.replaceAll("counter", "self"), "0\n"],
      pipelines: (source) => [
        source.replace("3 |> decorate", "show 3 |> decorate"),
        "(3)\n",
      ],
      precedence: (source) => [
        source.replace("low < middle < high", "low < middle && middle < high"),
        "true\n",
      ],
      "custom-operators": (source) => [
        source
          .replace(" ^ ", " <+> ")
          .replace("println 1", "println (1 <+> 2)"),
        "3\n",
      ],
      evaluation: (source) => [source.replace('"two"', "2"), "23\n"],
    }
    for (const example of syntaxExampleCases) {
      const [source, expected] = repairs[example.key](
        exampleById(`syntax-reader-${example.key}-invalid`).source
      )
      writeFileSync(join(temporary, "main.ssrg"), source)
      const result = spawnSync(cli, ["run", "main.ssrg"], {
        cwd: temporary,
        encoding: "utf8",
        timeout: 30_000,
      })
      expect(result.status, `${example.key}: ${result.stderr}`).toBe(0)
      expect(result.stdout, example.key).toBe(expected)
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test("syntax Playground seeds compile in WASM and execute with matching output", async () => {
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
  for (const example of syntaxExampleCases) {
    const canonical = exampleById(`syntax-reader-${example.key}`)
    const source = sourceFromPlaygroundUrl(canonical.playgroundUrl)
    expect(source).toBe(canonical.source)
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", source)
    ) as CompileResponse
    expect(compiled.status, `${example.key}: ${JSON.stringify(compiled)}`).toBe(
      "success"
    )
    if (compiled.status !== "success") continue
    expect(compiled.entry, example.key).toBeDefined()
    if (!compiled.entry) continue
    const result = await runtime.executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    // The Playground's capture API trims its final newline; native checks above
    // assert the exact stdout, including that newline.
    expect(result.stdout, example.key).toBe(example.expectedOutput.trimEnd())
  }
})

// Retained supplemental examples use the same paths and default standalone=true
// flags as build.ts. Production syntaxExamples must not register these twice.
const supplementalExamples = [
  ["syntax-invalid-numeric", "invalid-numeric"],
  ["syntax-char-invalid", "invalid-character-literals"],
  ["syntax-invalid-escape", "invalid-escape"],
  ["syntax-invalid-record-fields", "invalid-record-fields"],
  ["syntax-invalid-methods", "invalid-methods"],
  ["syntax-invalid-custom-operator", "invalid-custom-operator"],
].map(([id, file]) =>
  canonicalExample(
    id,
    `apps/site/examples/invalid/src/language/${file}.ssrg`,
    playgroundUrl
  )
)

const titles: Record<string, { en: string; ja: string }> = {
  "source-text": {
    en: "Source text and comments",
    ja: "ソーステキストとコメント",
  },
  literals: { en: "Literals", ja: "リテラル" },
  escapes: {
    en: "Character and string escapes",
    ja: "文字と文字列のエスケープ",
  },
  layout: { en: "Layout and line continuation", ja: "レイアウトと行継続" },
  names: { en: "Reserved words and naming", ja: "予約語と名前の規則" },
  "optional-fields": {
    en: "Optional record field syntax",
    ja: "省略可能なレコードフィールドの構文",
  },
  methods: { en: "Method calls", ja: "メソッド呼び出し" },
  pipelines: {
    en: "Pipelines and low-precedence application",
    ja: "パイプラインと低優先順位の適用",
  },
  precedence: { en: "Operator precedence", ja: "演算子の優先順位" },
  "custom-operators": {
    en: "Custom operators and fixity",
    ja: "カスタム演算子と結合規則",
  },
  evaluation: { en: "Evaluation order", ja: "評価順序" },
}

test("syntax render: all eleven real page modules preserve bilingual identities and executable panels", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-syntax-render-"))
  const allExamples = [...examples, ...supplementalExamples]
  try {
    const encodedExamples = allExamples
      .map(
        (example) => `ExampleSource {
  id: ${quote(example.id)}, sourcePath: ${quote(example.sourcePath)},
  source: ${quote(example.source)}, sha256: ${quote(example.sha256)},
  playgroundUrl: ${quote(example.playgroundUrl)},
  highlighted: [${example.highlighted
    .map(
      (part) => `HighlightPart {
    text: ${quote(part.text)}, className: ${quote(part.className)}
  }`
    )
    .join(",")}]
}`
      )
      .join(",")
    const pageNames = syntaxExampleCases.map((_, index) => `page${index}`)
    const rendered = renderPageClosure<
      Array<{
        id: string
        locale: "en" | "ja"
        route: string
        title: string
        html: string
      }>
    >({
      directory: temporary,
      timeoutMs: 180_000,
      modules: [
        ...syntaxExampleCases.map((example) => example.module),
        "render/document",
      ],
      cli,
      entry: `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localeCode, localizedRoute, translate } from "./model/locale"
import { SiteCatalog } from "./model/page"
${syntaxExampleCases
  .map(
    (example, index) =>
      `import { page as ${pageNames[index]} } from "./${example.module}"`
  )
  .join("\n")}
import { renderDocument } from "./render/document"
struct Output deriving JsonEncode {
  id: String, locale: String, route: String, title: String, html: String
}
pub effect fn main = {
  let pages = [${pageNames.map((name) => `${name} ()`).join(",")}]
  let input = BuildInput {
    schema: 1, origin: "https://seseragi.example",
    playgroundUrl: ${quote(playgroundUrl)},
    tourUrl: "https://seseragi.vercel.app/tour/", grammar: "",
    examples: [${encodedExamples}], referenceModules: []
  }
  let site = SiteCatalog { home: page0 (), areas: [], pages }
  println (json.encodeString [Output {
    id: page.id, locale: localeCode locale,
    route: localizedRoute locale page.path,
    title: translate locale page.title,
    html: renderDocument input site locale page
  } | locale <- [En, Ja], page <- pages])
}`,
    })
    if (process.env.SESERAGI_SYNTAX_OUTPUT) {
      for (const page of rendered) {
        const destination = join(
          resolve(process.env.SESERAGI_SYNTAX_OUTPUT),
          page.route.slice(1),
          "index.html"
        )
        mkdirSync(dirname(destination), { recursive: true })
        writeFileSync(destination, page.html)
      }
    }
    expect(rendered).toHaveLength(22)
    expect(new Set(rendered.map((page) => page.route)).size).toBe(22)
    for (const page of rendered) {
      const example = syntaxExampleCases.find((value) => value.id === page.id)
      expect(example, page.id).toBeDefined()
      if (!example) continue
      const prefix = page.locale === "ja" ? "/ja" : ""
      const otherPrefix = page.locale === "ja" ? "" : "/ja"
      const expectedTitle = titles[example.key][page.locale]
      expect(page.route).toBe(`${prefix}${example.route}`)
      expect(page.title).toBe(expectedTitle)
      expect(page.html).not.toContain("site-build-error")
      expect(page.html).toContain(`<html lang="${page.locale}">`)
      expect(text(page.html.match(/<h1>([\s\S]*?)<\/h1>/u)?.[1] ?? "")).toBe(
        expectedTitle
      )
      expect(page.html.match(/<h1>/gu)).toHaveLength(1)
      expect(
        text(page.html.match(/<title>([\s\S]*?)<\/title>/u)?.[1] ?? "")
      ).toBe(`${expectedTitle} · Seseragi`)
      expect(page.html).toContain(`href="${otherPrefix}${example.route}"`)
      expect(page.html).toContain(
        `href="https://seseragi.example${page.route}"`
      )
      const headings = [
        ...page.html.matchAll(/<h2\b[^>]*\bid="([^"]+)"/gu),
      ].map((match) => match[1])
      expect(new Set(headings).size, page.route).toBe(headings.length)
      for (const id of [
        "purpose",
        "reading-the-example",
        "rules",
        "mistakes",
        "related-rules",
      ]) {
        expect(headings, page.route).toContain(id)
        expect(page.html).toContain(`href="#${id}"`)
      }
      const codePanels = panels(page.html)
      const primary = exampleById(`syntax-reader-${example.key}`)
      const invalid = exampleById(`syntax-reader-${example.key}-invalid`)
      expect(code(codePanels[0]), page.route).toBe(primary.source)
      expect(codePanels.map(code), page.route).toContain(invalid.source)
      expect(terminals(page.html)[0]?.trimEnd(), page.route).toBe(
        example.expectedOutput.trimEnd()
      )
      for (const panel of codePanels) {
        const source = code(panel)
        const metadata = allExamples.find((value) => value.source === source)
        expect(metadata, `${page.route}: ${source}`).toBeDefined()
        if (!metadata) continue
        const href = panel.match(
          /<a\b(?=[^>]*\bclass="playground-link")[^>]*\bhref="([^"]+)"/u
        )?.[1]
        if (metadata.playgroundUrl === "") {
          expect(href).toBeUndefined()
        } else {
          expect(href).toBeDefined()
          expect(text(href ?? "")).toBe(metadata.playgroundUrl)
          expect(sourceFromPlaygroundUrl(text(href ?? ""))).toBe(source)
        }
      }
      if (example.key === "precedence") {
        for (const level of [9, 8, 7, 6, 5, 4, 3, 2, 1, 0, -1, -2, -3]) {
          expect(page.html).toContain(`<li><code>${level}</code>`)
        }
        expect(text(page.html)).toContain("infixr 4")
        expect(text(page.html)).not.toContain("7..4")
      }
      if (example.key === "names") {
        expect(text(page.html)).toContain("次の値")
        expect(text(page.html)).toContain(
          page.locale === "ja" ? "大文字・小文字の区別がない" : "Uncased"
        )
      }
      if (example.key === "layout" || example.key === "pipelines") {
        expect(code(codePanels[0])).toMatch(
          /\n[\t ]*3\n[\t ]*\|> double\n[\t ]*\|> addOne/u
        )
      }
      if (example.key === "escapes") {
        expect(code(codePanels[0])).toContain('"first\\nsecond"')
        expect(terminals(page.html)[0]?.trimEnd()).toBe(
          '"Seseragi"\ndocs\\guide\nλ\nfirst\nsecond'
        )
        expect(terminals(page.html)[0]).not.toContain("\\u{03BB}")
        expect(terminals(page.html)[0]).not.toContain("\\n")
      }
    }
    // This import-closure render intentionally has no navigation catalog areas.
    // It checks real article modules and locale switches, not sidebar/sequence,
    // complete route closure, CSS/mobile layout, or browser execution of the site.
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})
