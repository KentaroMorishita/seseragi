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
import { dirname, join, resolve } from "node:path"
import type {
  CompileResponse,
  EntryContract,
} from "../../playground/src/compiler/types"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { canonicalExample } from "../scripts/canonical-example"
import { entranceComparison } from "../scripts/comparisons"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(120_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const examples = [
  {
    example: canonicalExample(
      "comparison-shipping-total-seseragi",
      entranceComparison.seseragi,
      "https://seseragi.vercel.app/"
    ),
    output: readFileSync(
      resolve(root, entranceComparison.expectedOutput),
      "utf8"
    ),
  },
  {
    example: canonicalExample(
      "reader-records",
      "apps/site/examples/src/language/reader-records.ssrg",
      "https://seseragi.vercel.app/"
    ),
    output: "10\n42\nAki\n",
  },
  {
    example: canonicalExample(
      "reader-pipelines",
      "apps/site/examples/src/language/reader-pipelines.ssrg",
      "https://seseragi.vercel.app/"
    ),
    output: "7\n7\n7\n",
  },
]

function text(html: string) {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&")
}

test("curated examples typecheck and print the documented output as independent files", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-curated-examples-"))
  try {
    for (const { example, output } of examples) {
      const directory = join(temporary, example.id)
      mkdirSync(directory)
      writeFileSync(join(directory, "main.ssrg"), example.source)
      for (const command of ["lint", "run"]) {
        const result = spawnSync(cli, [command, "main.ssrg"], {
          cwd: directory,
          encoding: "utf8",
          timeout: 30_000,
        })
        expect(result.status, `${example.id}: ${result.stderr}`).toBe(0)
        expect(result.stderr).toBe("")
        expect(result.stdout).toBe(command === "run" ? output : "")
      }
      const [changed, changedOutput] =
        example.id === "comparison-shipping-total-seseragi"
          ? [
              example.source.replace(
                "standardTotal 3200",
                "standardTotal 4999"
              ),
              "5499, 5000\n",
            ]
          : example.id === "reader-records"
            ? [
                example.source.replace("score: 42", "score: 99"),
                "10\n99\nAki\n",
              ]
            : [
                example.source
                  .replace("3 |> double", "4 |> double")
                  .replaceAll("double 3", "double 4"),
                "9\n9\n9\n",
              ]
      writeFileSync(join(directory, "main.ssrg"), changed)
      const edited = spawnSync(cli, ["run", "main.ssrg"], {
        cwd: directory,
        encoding: "utf8",
        timeout: 30_000,
      })
      expect(edited.status, edited.stderr).toBe(0)
      expect(edited.stdout).toBe(changedOutput)
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test("each curated Playground link carries the exact executable source", async () => {
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
  for (const { example, output } of examples) {
    const source = sourceFromPlaygroundUrl(example.playgroundUrl)
    expect(source).toBe(example.source)
    const compiled = JSON.parse(
      bindings.compile_single_file("main.ssrg", "playground/main", source)
    ) as CompileResponse
    expect(compiled.status, example.id).toBe("success")
    if (compiled.status !== "success") continue
    expect(compiled.entry).toBeDefined()
    if (!compiled.entry) continue
    const result = await runtime.executeGeneratedModule(
      compiled.generated.typescript,
      compiled.entry
    )
    expect(result.stdout).toBe(output.trimEnd())
  }
})

test("Examples and Releases render actionable bilingual content and pinned official links", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-examples-releases-"))
  try {
    const encodedExamples = examples
      .map(
        ({ example }) => `ExampleSource {
      id: ${JSON.stringify(example.id)},
      sourcePath: ${JSON.stringify(example.sourcePath)},
      source: ${JSON.stringify(example.source)},
      sha256: ${JSON.stringify(example.sha256)},
      playgroundUrl: ${JSON.stringify(example.playgroundUrl)},
      highlighted: [${example.highlighted
        .map(
          (part) => `HighlightPart {
        text: ${JSON.stringify(part.text)}, className: ${JSON.stringify(part.className)}
      }`
        )
        .join(",")}]
    }`
      )
      .join(",")
    const rendered = renderPageClosure<Array<{ route: string; html: string }>>({
      directory: temporary,
      modules: [
        "pages/examples/page",
        "pages/releases/page",
        "render/document",
      ],
      cli,
      entry: `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { En, Ja, localizedRoute } from "./model/locale"
import { SiteCatalog } from "./model/page"
import { page as examplesPage } from "./pages/examples/page"
import { page as releasesPage } from "./pages/releases/page"
import { renderDocument } from "./render/document"
struct Output deriving JsonEncode { route: String, html: String }
pub effect fn main = {
  let examples = examplesPage ()
  let releases = releasesPage ()
  let input = BuildInput {
    schema: 1, origin: "https://seseragi.example",
    playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "https://seseragi.vercel.app/tour/", grammar: "",
    examples: [${encodedExamples}], referenceModules: []
  }
  let site = SiteCatalog { home: examples, areas: [], pages: [examples, releases] }
  println (json.encodeString [Output {
    route: localizedRoute locale page.path,
    html: renderDocument input site locale page
  } | locale <- [En, Ja], page <- site.pages])
}`,
    })
    expect(rendered.map(({ route }) => route)).toEqual([
      "/examples/",
      "/releases/",
      "/ja/examples/",
      "/ja/releases/",
    ])
    const expectedAssets = [
      "darwin-arm64",
      "darwin-x64",
      "linux-x64",
      "win32-x64",
    ]
      .flatMap((target) => {
        const archive = `seseragi-v0.61.19-${target}.${target === "win32-x64" ? "zip" : "tar.gz"}`
        return [
          archive,
          `${archive}.sha256`,
          `seseragi-v0.61.19-vscode-${target}.vsix`,
        ]
      })
      .map(
        (file) =>
          `https://github.com/KentaroMorishita/seseragi/releases/download/v0.61.19/${file}`
      )
      .sort()
    for (const { route, html } of rendered) {
      expect(html, route).not.toContain("site-build-error")
      expect(html, route).not.toContain("docs-sidebar")
      const prefix = route.startsWith("/ja/") ? "/ja" : ""
      expect(html).toContain(`href="${prefix}/docs/first-run/"`)
      if (route.endsWith("/examples/")) {
        const panels = [
          ...html.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((match) => text(match[1]))
        expect(panels).toEqual(examples.map(({ example }) => example.source))
        const playgroundLinks = [
          ...html.matchAll(
            /<a\b(?=[^>]*\bclass="playground-link")[^>]*\bhref="([^"]+)"/gu
          ),
        ].map((match) => sourceFromPlaygroundUrl(text(match[1])))
        expect(playgroundLinks).toEqual(panels)
        const terminals = [
          ...html.matchAll(
            /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code><\/pre><\/section>/gu
          ),
        ].map((match) => text(match[1]))
        expect(terminals).toEqual([
          "seseragi lint main.ssrg\nseseragi run main.ssrg",
          ...examples.map(({ output }) => output.trimEnd()),
        ])
        expect(html).toContain(`href="${prefix}/docs/language/data/records/"`)
        expect(text(html)).toContain("4999")
      } else {
        const assets = [
          ...html.matchAll(
            /href="(https:\/\/github.com\/KentaroMorishita\/seseragi\/releases\/download\/v0.61.19\/[^"]+)"/gu
          ),
        ]
          .map((match) => match[1])
          .sort()
        expect(assets).toEqual(expectedAssets)
        expect(html).toContain(
          'href="https://github.com/KentaroMorishita/seseragi/releases/tag/v0.61.19"'
        )
        expect(text(html)).toContain("a8641b5a81a4")
        expect(text(html)).toContain("glibc 2.34")
        expect(text(html)).toContain("development")
        expect(text(html)).toContain("SES-K0102")
      }
      if (process.env.SITE_EXAMPLES_RELEASES_OUTPUT) {
        const output = resolve(
          root,
          process.env.SITE_EXAMPLES_RELEASES_OUTPUT,
          route.slice(1),
          "index.html"
        )
        mkdirSync(dirname(output), { recursive: true })
        writeFileSync(output, html)
      }
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})
