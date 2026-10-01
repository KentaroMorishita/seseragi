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
import {
  modelReaderCases,
  modelReaderExamples,
  modelReaderSupplement,
  newModelReaderExamples,
} from "../scripts/model-reader"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(240_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const playgroundUrl = "https://seseragi.vercel.app/"
const examples = modelReaderExamples(playgroundUrl)
const byId = (id: string) => {
  const example = examples.find((entry) => entry.id === id)
  if (!example) throw new Error(`Missing example ${id}`)
  return example
}
const text = (value: string) =>
  value
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
const quote = JSON.stringify

test("model reader sources, hashes, highlights, and runnable Playground seeds agree", () => {
  expect(modelReaderCases).toHaveLength(6)
  expect(examples).toHaveLength(11)
  expect(newModelReaderExamples(playgroundUrl)).toHaveLength(7)
  expect(new Set(examples.map((entry) => entry.id)).size).toBe(11)
  for (const example of examples) {
    const source = readFileSync(resolve(root, example.sourcePath), "utf8")
    expect(example.source).toBe(source)
    expect(example.sha256).toBe(
      createHash("sha256").update(source).digest("hex")
    )
    expect(example.highlighted.map((part) => part.text).join("")).toBe(source)
    if (example.id.endsWith("-invalid")) expect(example.playgroundUrl).toBe("")
    else expect(sourceFromPlaygroundUrl(example.playgroundUrl)).toBe(source)
  }
  expect(byId("model-reader-density").source).toContain(
    "let result = 3\n  // Double the input.\n  |> double\n  // Add one to the doubled value.\n  |> add 1"
  )
  expect(byId("model-reader-backend").source).toContain("-7.0 / 2.0")
})

test("model reader related links retain exact current bilingual destination titles", () => {
  let checked = 0
  for (const entry of modelReaderCases) {
    const source = readFileSync(
      resolve(root, "apps/site/src", `${entry.module}.ssrg`),
      "utf8"
    )
    const links = [
      ...source.matchAll(
        /InternalLink\s*\(\s*localized\s+("(?:\\.|[^"\\])*")\s+("(?:\\.|[^"\\])*")\s*,\s*"([^"#]+)"\s*\)/gu
      ),
    ]
    expect(links, entry.id).toHaveLength(3)
    for (const [, english, japanese, route] of links) {
      for (const [locale, label] of [
        ["en", english],
        ["ja", japanese],
      ]) {
        const destination = readFileSync(
          resolve(
            root,
            "apps/site/src/pages",
            route.replace(/^\/docs\//u, ""),
            `${locale}.ssrg`
          ),
          "utf8"
        )
        const title =
          destination.match(/ {2}title: ("(?:\\.|[^"\\])*"),/u)?.[1] ??
          destination.match(
            /pub fn title -> String = ("(?:\\.|[^"\\])*")/u
          )?.[1]
        expect(title, route).toBeDefined()
        expect(JSON.parse(label), `${entry.id} -> ${route} ${locale}`).toBe(
          JSON.parse(title!)
        )
        checked++
      }
    }
  }
  expect(checked).toBe(36)
})

function native(source: string, command: string) {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-model-reader-"))
  try {
    writeFileSync(join(temporary, "main.ssrg"), source)
    return spawnSync(cli, [command, "main.ssrg"], {
      cwd: temporary,
      encoding: "utf8",
      timeout: 30_000,
    })
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
}

for (const entry of modelReaderCases) {
  test(`model reader native ${entry.key}: output and meaningful rejection`, () => {
    for (const command of ["lint", "run"]) {
      const result = native(byId(entry.exampleId).source, command)
      expect(result.status, result.stderr || result.error?.message).toBe(0)
      expect(result.stderr).toBe("")
      expect(result.stdout).toBe(command === "run" ? entry.expectedOutput : "")
    }
    if (entry.expectedDiagnostic) {
      const result = native(
        byId(`model-reader-${entry.key}-invalid`).source,
        "lint"
      )
      expect(result.status, result.stderr).toBe(2)
      expect(result.stderr).toContain(`[${entry.expectedDiagnostic}]`)
      expect(result.stdout).toBe("")
    }
  })
}

test("model reader repairs and changed-order/unchanged-source claims execute", () => {
  const variants: Array<[string, string]> = [
    [
      byId("model-reader-expression-invalid").source.replace(
        "else 0",
        'else "retry"'
      ),
      "pass\n",
    ],
    [
      byId("model-reader-absence-invalid").source.replace(
        "\n}\n",
        '\n  Nothing -> "not found"\n}\n'
      ),
      "not found\n",
    ],
    [
      byId("model-reader-backend-invalid").source.replace(
        "-7 / 2.0",
        "-7.0 / 2.0"
      ),
      "-3.5\n",
    ],
    [
      byId("model-reader-backend-invalid").source.replace("-7 / 2.0", "-7 / 2"),
      "-3\n",
    ],
    [
      byId("model-reader-diagnostics-invalid").source.replace('= "3"', "= 3"),
      "3\n",
    ],
    [
      byId("principle-visible-costs").source.replace(
        "println (show doubled)",
        "println (show values)"
      ),
      "[1, 2, 3]\n",
    ],
    [
      byId("model-reader-density").source.replace(
        "|> double\n  // Add one to the doubled value.\n  |> add 1",
        "|> add 1\n  // Double the incremented value.\n  |> double"
      ),
      "8\n",
    ],
    [
      byId("model-reader-density").source.replace(
        "let result = 3\n  // Double the input.\n  |> double\n  // Add one to the doubled value.\n  |> add 1",
        "let doubled = double 3\nlet result = add 1 doubled"
      ),
      "7\n",
    ],
    [
      byId(modelReaderSupplement.id).source,
      modelReaderSupplement.expectedOutput,
    ],
  ]
  for (const [source, expected] of variants) {
    const result = native(source, "run")
    expect(result.status, result.stderr).toBe(0)
    expect(result.stdout).toBe(expected)
  }
})

test("model reader exact Playground seeds compile and run through WASM and browser runtime", async () => {
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
  const cases = [
    ...modelReaderCases.map((entry) => ({
      id: entry.exampleId,
      output: entry.expectedOutput,
    })),
    {
      id: modelReaderSupplement.id,
      output: modelReaderSupplement.expectedOutput,
    },
  ]
  for (const entry of cases) {
    const source = sourceFromPlaygroundUrl(byId(entry.id).playgroundUrl)
    expect(source).toBe(byId(entry.id).source)
    const result = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", source)
    ) as CompileResponse
    expect(result.status, JSON.stringify(result)).toBe("success")
    if (result.status !== "success" || !result.entry)
      throw new Error(`Missing runnable entry ${entry.id}`)
    expect(
      (
        await runtime.executeGeneratedModule(
          result.generated.typescript,
          result.entry
        )
      ).stdout
    ).toBe(entry.output.trimEnd())
  }
})

const titles = {
  expression: ["Expression-oriented", "式指向"],
  absence: ["No hidden danger", "危険を暗黙に隠さない"],
  backend: ["Backend-independent semantics", "生成先に依存しない意味"],
  diagnostics: ["Diagnosable behavior", "説明可能な振る舞い"],
  costs: ["Visible costs", "コストを見えるようにする"],
  density: ["Readable density", "読みやすい密度"],
}

test("model reader renders all twelve real article bodies with purpose before examples", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-model-reader-render-"))
  try {
    const encoded = examples
      .map(
        (entry) =>
          `ExampleSource { id: ${quote(entry.id)}, sourcePath: ${quote(entry.sourcePath)}, source: ${quote(entry.source)}, sha256: ${quote(entry.sha256)}, playgroundUrl: ${quote(entry.playgroundUrl)}, highlighted: [${entry.highlighted.map((part) => `HighlightPart { text: ${quote(part.text)}, className: ${quote(part.className)} }`).join(",")}] }`
      )
      .join(",")
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
      cli,
      timeoutMs: 210_000,
      modules: [
        ...modelReaderCases.map((entry) => entry.module),
        "render/document",
      ],
      entry: `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localeCode, localizedRoute, translate } from "./model/locale"
import { SiteCatalog } from "./model/page"
${modelReaderCases.map((entry, index) => `import { page as page${index} } from "./${entry.module}"`).join("\n")}
import { renderDocument } from "./render/document"
struct Output deriving JsonEncode { id: String, locale: String, route: String, title: String, html: String }
pub effect fn main = {
  let pages = [${modelReaderCases.map((_, index) => `page${index} ()`).join(",")}]
  let input = BuildInput { schema: 1, origin: "https://seseragi.example", playgroundUrl: ${quote(playgroundUrl)}, tourUrl: "https://seseragi.vercel.app/tour/", grammar: "", examples: [${encoded}], referenceModules: [] }
  let site = SiteCatalog { home: page0 (), areas: [], pages }
  println (json.encodeString [Output { id: page.id, locale: localeCode locale, route: localizedRoute locale page.path, title: translate locale page.title, html: renderDocument input site locale page } | locale <- [En, Ja], page <- pages])
}`,
    })
    expect(rendered).toHaveLength(12)
    for (const page of rendered) {
      const entry = modelReaderCases.find((entry) => entry.id === page.id)!
      const expectedTitle =
        titles[entry.key as keyof typeof titles][page.locale === "ja" ? 1 : 0]
      const prefix = page.locale === "ja" ? "/ja" : ""
      expect(page.route).toBe(`${prefix}${entry.route}`)
      expect(page.title).toBe(expectedTitle)
      expect(page.html).not.toContain("site-build-error")
      expect(page.html).toContain(`<html lang="${page.locale}">`)
      expect(page.html.match(/<h1>/gu)).toHaveLength(1)
      expect(text(page.html.match(/<h1>([\s\S]*?)<\/h1>/u)?.[1] ?? "")).toBe(
        expectedTitle
      )
      expect(page.html).toContain(
        `href="${page.locale === "ja" ? "" : "/ja"}${entry.route}"`
      )
      const headings = [
        ...page.html.matchAll(/<h2\b[^>]*\bid="([^"]+)"/gu),
      ].map((match) => match[1])
      expect(new Set(headings).size).toBe(headings.length)
      for (const id of [
        "understand-this",
        "meaning",
        "why",
        "limits",
        "mistakes",
        "related-rules",
      ])
        expect(headings).toContain(id)
      const panels = [
        ...page.html.matchAll(
          /<section class="code-panel">[\s\S]*?<\/section>/gu
        ),
      ].map((match) => match[0])
      expect(page.html.indexOf('<h2 id="understand-this"')).toBeLessThan(
        page.html.indexOf(panels[0])
      )
      for (const panel of panels) {
        const source = text(
          panel.match(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
          )?.[1] ?? ""
        )
        const canonical = examples.find((entry) => entry.source === source)
        expect(canonical, `${page.route}: ${source}`).toBeDefined()
        if (!canonical) continue
        const href = panel.match(
          /<a\b(?=[^>]*\bclass="playground-link")[^>]*\bhref="([^"]+)"/u
        )?.[1]
        if (canonical.playgroundUrl)
          expect(sourceFromPlaygroundUrl(text(href ?? ""))).toBe(source)
        else expect(href).toBeUndefined()
      }
      expect(
        panels.map((panel) =>
          text(
            panel.match(
              /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
            )?.[1] ?? ""
          )
        )
      ).toContain(byId(entry.exampleId).source)
      const terminals = [
        ...page.html.matchAll(
          /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
        ),
      ].map((match) => text(match[1]).trimEnd())
      expect(terminals).toContain(entry.expectedOutput.trimEnd())
      if (entry.expectedDiagnostic)
        expect(terminals.join("\n")).toContain(entry.expectedDiagnostic)
      if (entry.key === "diagnostics") {
        expect(
          text(
            panels[0].match(
              /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
            )?.[1] ?? ""
          )
        ).toBe(byId("model-reader-diagnostics-invalid").source)
        expect(terminals[0]).toContain("expected: Int")
      }
      if (process.env.SESERAGI_MODEL_READER_OUTPUT) {
        const destination = join(
          resolve(process.env.SESERAGI_MODEL_READER_OUTPUT),
          page.route.slice(1),
          "index.html"
        )
        mkdirSync(dirname(destination), { recursive: true })
        writeFileSync(destination, page.html)
      }
    }
    // Empty navigation catalog: this asserts article bodies and locale identity,
    // not global route closure, section navigation, CSS, browser, or reader acceptance.
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})
