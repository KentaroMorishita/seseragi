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
import {
  modelConceptExamples,
  modelConceptPages,
  modelConceptPrograms,
} from "../scripts/model-concepts"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const playground = "https://seseragi.vercel.app/"
const examples = [
  ...modelConceptExamples(playground),
  canonicalExample(
    "language-program-entry",
    "apps/site/examples/src/language/program-entry.ssrg",
    playground
  ),
]
const grammar = readFileSync(join(root, "docs/spec/grammar.md"), "utf8")
  .split("```ebnf\n")[1]
  .split("```")[0]
  .trimEnd()
const quote = JSON.stringify
function source(id: string) {
  const example = examples.find((example) => example.id === id)
  if (!example) throw new Error(`Missing example ${id}`)
  return example
}
function text(value: string) {
  return value
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}
function terminals(html: string) {
  return [
    ...html.matchAll(
      /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
    ),
  ].map((match) => text(match[1]))
}
function call(command: string, cwd: string) {
  return spawnSync(cli, [command, "main.ssrg"], {
    cwd,
    encoding: "utf8",
    timeout: 30_000,
  })
}

test("model concept source metadata preserves canonical bytes and runnable URL seeds", () => {
  expect(examples).toHaveLength(6)
  for (const example of examples) {
    const bytes = readFileSync(join(root, example.sourcePath), "utf8")
    expect(example.source).toBe(bytes)
    expect(example.sha256).toBe(
      createHash("sha256").update(bytes).digest("hex")
    )
    expect(example.highlighted.map((part) => part.text).join("")).toBe(bytes)
    if (example.id.endsWith("rejected") || example.id.endsWith("invalid"))
      expect(example.playgroundUrl).toBe("")
    else expect(sourceFromPlaygroundUrl(example.playgroundUrl)).toBe(bytes)
  }
})

for (const item of modelConceptPrograms) {
  test(`model concept native ${item.id}: lint and exact output`, () => {
    const directory = mkdtempSync(join(tmpdir(), "seseragi-concept-native-"))
    try {
      writeFileSync(join(directory, "main.ssrg"), source(item.id).source)
      for (const command of ["lint", "run"]) {
        const result = call(command, directory)
        expect(result.status, result.stderr).toBe(0)
        expect(result.stderr).toBe("")
        expect(result.stdout).toBe(command === "run" ? item.output : "")
      }
    } finally {
      rmSync(directory, { recursive: true, force: true })
    }
  })
}

test("entry validation differs from type checking, and both documented repairs run", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-concept-repair-"))
  try {
    const entry = source("model-reader-entry-rejected").source
    writeFileSync(join(directory, "main.ssrg"), entry)
    expect(call("lint", directory).status).toBe(0)
    const invalidEntry = call("run", directory)
    expect(invalidEntry.status).toBe(2)
    expect(invalidEntry.stderr).toContain(
      "invalid entry point: program must export `pub effect fn main`"
    )
    writeFileSync(
      join(directory, "main.ssrg"),
      entry.replace(
        'pub fn main -> String = greeting "Aki"',
        'pub effect fn main = println (greeting "Aki")'
      )
    )
    const repairedEntry = call("run", directory)
    expect(repairedEntry.status, repairedEntry.stderr).toBe(0)
    expect(repairedEntry.stdout).toBe("Hello, Aki!\n")
    const invalid = source("model-reader-grammar-invalid").source
    writeFileSync(join(directory, "main.ssrg"), invalid)
    const mismatch = call("lint", directory)
    expect(mismatch.status).toBe(2)
    expect(mismatch.stderr).toContain("SES-T0101")
    writeFileSync(join(directory, "main.ssrg"), invalid.replace('"three"', "3"))
    const repaired = call("run", directory)
    expect(repaired.status, repaired.stderr).toBe(0)
    expect(repaired.stdout).toBe("3\n")
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("model concept Playground sources compile and run through the existing WASM runtime", async () => {
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
  for (const item of modelConceptPrograms) {
    const canonical = source(item.id)
    const seed = sourceFromPlaygroundUrl(canonical.playgroundUrl)
    expect(seed).toBe(canonical.source)
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", seed)
    ) as CompileResponse
    expect(compiled.status, JSON.stringify(compiled)).toBe("success")
    if (compiled.status !== "success") continue
    expect(compiled.entry).toBeDefined()
    if (!compiled.entry) continue
    const result = await runtime.executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(result.stdout).toBe(item.output.trimEnd())
  }
})

function destinationTitle(route: string, locale: string) {
  const directory = route.startsWith("/docs/language/")
    ? `apps/site/src/pages/language/${route.slice("/docs/language/".length)}`
    : `apps/site/src/pages/docs/${route.slice("/docs/".length)}`
  const copy = readFileSync(join(root, directory, `${locale}.ssrg`), "utf8")
  const literal =
    copy.match(/\btitle:\s*("(?:[^"\\]|\\.)*")/u)?.[1] ??
    copy.match(/pub fn title -> String =\s*("(?:[^"\\]|\\.)*")/u)?.[1]
  if (!literal) throw new Error(`No literal title for ${route} ${locale}`)
  return JSON.parse(literal) as string
}

test("model overview and grammar render real bilingual articles without changing EBNF bytes", () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-model-concept-render-")
  )
  try {
    const encoded = examples
      .map(
        (example) =>
          `ExampleSource { id: ${quote(example.id)}, sourcePath: ${quote(example.sourcePath)}, source: ${quote(example.source)}, sha256: ${quote(example.sha256)}, playgroundUrl: ${quote(example.playgroundUrl)}, highlighted: [${example.highlighted.map((part) => `HighlightPart {text: ${quote(part.text)}, className: ${quote(part.className)}}`).join(",")}] }`
      )
      .join(",")
    const pages = renderPageClosure<
      Array<{
        id: string
        locale: string
        route: string
        title: string
        html: string
      }>
    >({
      directory,
      cli,
      timeoutMs: 150_000,
      modules: [
        ...modelConceptPages.map((page) => page.module),
        "render/document",
      ],
      entry: `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localeCode, localizedRoute, translate } from "./model/locale"
import { SiteCatalog } from "./model/page"
${modelConceptPages.map((page, index) => `import { page as page${index} } from "./${page.module}"`).join("\n")}
import { renderDocument } from "./render/document"
struct Output deriving JsonEncode { id: String, locale: String, route: String, title: String, html: String }
pub effect fn main = {
 let input = BuildInput { schema: 1, origin: "https://seseragi.example", playgroundUrl: ${quote(playground)}, tourUrl: "https://seseragi.vercel.app/tour/", grammar: ${quote(grammar)}, examples: [${encoded}], referenceModules: [] }
 let pages = [page0 (), page1 (), page2 (), page3 input]
 let site = SiteCatalog { home: page0 (), areas: [], pages }
 println (json.encodeString [Output { id: page.id, locale: localeCode locale, route: localizedRoute locale page.path, title: translate locale page.title, html: renderDocument input site locale page } | locale <- [En, Ja], page <- pages])
}`,
    })
    expect(pages).toHaveLength(8)
    for (const page of pages) {
      if (process.env.SESERAGI_MODEL_CONCEPT_OUTPUT) {
        const file = join(
          resolve(process.env.SESERAGI_MODEL_CONCEPT_OUTPUT),
          page.route.slice(1),
          "index.html"
        )
        mkdirSync(dirname(file), { recursive: true })
        writeFileSync(file, page.html)
      }
      const item = modelConceptPages.find((item) => item.id === page.id)
      expect(item).toBeDefined()
      if (!item) continue
      const prefix = page.locale === "ja" ? "/ja" : ""
      const other = page.locale === "ja" ? "" : "/ja"
      expect(page.route).toBe(prefix + item.route)
      expect(page.title).toBe(item.titles[page.locale === "ja" ? 1 : 0])
      expect(page.html).toContain(`<html lang="${page.locale}">`)
      expect(page.html).not.toContain("site-build-error")
      expect(page.html.match(/<h1>/gu)).toHaveLength(1)
      expect(page.html).toContain(`href="${other}${item.route}"`)
      const headings = [
        ...page.html.matchAll(/<h2\b[^>]*\bid="([^"]+)"/gu),
      ].map((match) => match[1])
      expect(new Set(headings).size).toBe(headings.length)
      for (const anchor of item.anchors) expect(headings).toContain(anchor)
      const links = [
        ...page.html.matchAll(
          /<a\b[^>]*\bhref="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gu
        ),
      ].map((match) => [text(match[1]), text(match[2])])
      for (const route of item.related)
        expect(links, page.route).toContainEqual([
          prefix + route,
          destinationTitle(route, page.locale),
        ])
      for (const match of page.html.matchAll(
        /<section class="code-panel">[\s\S]*?<\/section>/gu
      )) {
        const panel = match[0]
        const bytes = text(
          panel.match(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
          )?.[1] ?? ""
        )
        const canonical = examples.find((example) => example.source === bytes)
        expect(canonical, page.route).toBeDefined()
        if (!canonical) continue
        const url = panel.match(
          /<a\b(?=[^>]*\bclass="playground-link")[^>]*\bhref="([^"]+)"/u
        )?.[1]
        if (!canonical.playgroundUrl) expect(url).toBeUndefined()
        else expect(sourceFromPlaygroundUrl(text(url ?? ""))).toBe(bytes)
      }
      if (item.key === "grammar") {
        expect(terminals(page.html)).toContain(grammar)
        expect(terminals(page.html)).toContain("3, 4")
        expect(text(page.html)).toContain("次の値")
        expect(text(page.html)).toContain("SES-T0101")
      } else if (item.key === "programs") {
        expect(terminals(page.html)).toContain("Hello, Aki!")
        expect(terminals(page.html)).toContain("Hello, Seseragi!")
        expect(text(page.html)).toContain("invalid entry point")
      } else if (item.key === "non-features")
        expect(terminals(page.html)).toContain("79, 80, pass")
      else expect(page.html).not.toContain('class="code-panel"')
    }
    // Real article modules with an intentionally small catalog: no sidebar tree,
    // visual/keyboard/browser journey or full-route-closure claim.
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
