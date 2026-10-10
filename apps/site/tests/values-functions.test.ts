import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, relative, resolve } from "node:path"
import { canonicalExample } from "../scripts/canonical-example"
import { chapterExamples } from "../scripts/function-chapter-examples"

const root = resolve(import.meta.dir, "../../..")
const sourceRoot = join(root, "apps/site/src")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const exampleRoot = "apps/site/examples"
const pilot = [
  [
    "model/immutable-by-default",
    "language.model.immutable-by-default",
    "principle-immutable-by-default",
    "10 -> 11",
    "pilot-immutable",
    "SES-P0001",
  ],
  [
    "types/built-in-types",
    "language.types.built-in-types",
    "pilot-built-in-types",
    "Notebook: 3, 2.5, True",
    "pilot-built-in-types",
    "SES-T0101",
  ],
  [
    "types/annotations-and-inference",
    "language.types.annotations-and-inference",
    "pilot-annotations",
    "22",
    "pilot-annotations",
    "SES-T0101",
  ],
  [
    "syntax/function-application",
    "language.syntax.function-application",
    "pilot-function-application",
    "3",
    "pilot-function-application",
    "SES-T0101",
  ],
  [
    "expressions/blocks-and-local-declarations",
    "language.expressions.blocks-and-local-declarations",
    "pilot-blocks",
    "65",
    "pilot-blocks",
    "SES-N0001",
  ],
  [
    "types/function-types-and-currying",
    "language.types.function-types",
    "pilot-currying",
    "3, 3",
    "pilot-currying",
    "SES-T0101",
  ],
] as const
setDefaultTimeout(120_000)

function checked(args: string[], cwd: string) {
  const result = spawnSync(cli, args, {
    cwd,
    encoding: "utf8",
    timeout: 90_000,
    maxBuffer: 16 * 1024 * 1024,
  })
  expect(result.status, result.stderr || result.error?.message).toBe(0)
  return result.stdout
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

function readArray(source: string, field: string): string[] {
  const marker = `  ${field}: [`
  const start = source.indexOf(marker) + marker.length - 1
  expect(start >= marker.length - 1, field).toBe(true)
  let quoted = false
  let escaped = false
  for (let end = start + 1; end < source.length; end++) {
    const character = source[end]
    if (escaped) {
      escaped = false
      continue
    }
    if (quoted && character === "\\") {
      escaped = true
      continue
    }
    if (character === '"') quoted = !quoted
    if (!quoted && character === "]")
      return JSON.parse(source.slice(start, end + 1)) as string[]
  }
  throw new Error(`Unterminated ${field} array`)
}

function renderPilot(
  directory: string
): Array<{ id: string; route: string; html: string }> {
  const copied = new Set<string>()
  function copyModule(path: string) {
    if (copied.has(path)) return
    expect(path.startsWith(`${sourceRoot}/`)).toBe(true)
    copied.add(path)
    const code = readFileSync(path, "utf8")
    const destination = join(directory, "src", relative(sourceRoot, path))
    mkdirSync(dirname(destination), { recursive: true })
    copyFileSync(path, destination)
    for (const [, specifier] of code.matchAll(/from\s+"(\.[^"]+)"/gu))
      copyModule(resolve(dirname(path), `${specifier}.ssrg`))
  }
  for (const [path] of pilot)
    copyModule(join(sourceRoot, `pages/language/${path}/page.ssrg`))
  copyModule(join(sourceRoot, "render/document.ssrg"))
  writeFileSync(
    join(directory, "seseragi.toml"),
    '[package]\nname = "values-functions-verification"\nversion = "0.0.0"\nlanguage = ">=0.1.0 <0.2.0"\n\n[run]\nentry = "verify"\ntarget = "process"\n'
  )
  const sources = [
    ...chapterExamples("https://seseragi.vercel.app/"),
    ...pilot.flatMap(([, , example, , invalid]) => [
      canonicalExample(
        example,
        `${exampleRoot}/src/language/${example}.ssrg`,
        "https://seseragi.vercel.app/"
      ),
      canonicalExample(
        `${invalid}-invalid`,
        `${exampleRoot}/invalid/src/language/${invalid}.ssrg`,
        "https://seseragi.vercel.app/",
        false
      ),
    ]),
  ]
  writeFileSync(
    join(directory, "src/verify.ssrg"),
    `import * as json from "std/json"
import { BuildInput, ExampleSource, HighlightPart } from "./model/build"
import { articlePage, ArticleCopy, prose } from "./model/article"
import { En, Ja, localizedRoute } from "./model/locale"
import { SiteCatalog } from "./model/page"
import { renderDocument } from "./render/document"
${pilot.map(([path], i) => `import { page as page${i} } from "./pages/language/${path}/page"`).join("\n")}
struct Output deriving JsonEncode { id: String, route: String, html: String }
pub effect fn main = {
  let short = articlePage "composition-probe" "/composition-probe/" (ArticleCopy { title: "Usage", summary: "A short article" }) (ArticleCopy { title: "使い方", summary: "短い記事" }) [prose "Just the usage." "使い方だけ。"]
  let pages = [${pilot.map((_, i) => `page${i} ()`).join(", ")}, short]
  let input = BuildInput {
    schema: 1, origin: "https://seseragi.example", playgroundUrl: "https://seseragi.vercel.app/",
    tourUrl: "https://seseragi.vercel.app/tour/", grammar: "", referenceModules: [],
    examples: [${sources
      .map(
        ({
          id,
          sourcePath,
          source,
          sha256,
          playgroundUrl,
          highlighted,
        }) => `ExampleSource {
      id: ${JSON.stringify(id)}, sourcePath: ${JSON.stringify(sourcePath)}, source: ${JSON.stringify(source)},
      sha256: ${JSON.stringify(sha256)}, playgroundUrl: ${JSON.stringify(playgroundUrl)},
      highlighted: [${highlighted.map(({ text, className }) => `HighlightPart { text: ${JSON.stringify(text)}, className: ${JSON.stringify(className)} }`).join(",")}]
    }`
      )
      .join(",")}]
  }
  let site = SiteCatalog { home: page0 (), areas: [], pages }
  println (json.encodeString [Output {
    id: page.id,
    route: localizedRoute locale page.path,
    html: renderDocument input site locale page
  } | locale <- [En, Ja], page <- site.pages])
}
`
  )
  checked(["lock", "update", directory], directory)
  return JSON.parse(checked(["run", directory], directory))
}

test("values/functions pilot preserves identities, paired copy, and exact destination titles", () => {
  for (const [path, id, example] of pilot) {
    const directory = join(sourceRoot, "pages/language", path)
    const source = readFileSync(join(directory, "page.ssrg"), "utf8")
    expect(source).toContain(`"${id}"`)
    expect(source).toContain(`"/docs/language/${path}/"`)
    const en = readFileSync(join(directory, "en.ssrg"), "utf8")
    const ja = readFileSync(join(directory, "ja.ssrg"), "utf8")
    if (
      ![
        "syntax/function-application",
        "types/function-types-and-currying",
      ].includes(path)
    ) {
      for (const field of [
        "purpose",
        "reading",
        "rules",
        "mistakes",
        "related",
      ])
        expect(readArray(en, field).length, `${path}: ${field}`).toBe(
          readArray(ja, field).length
        )
      expect(readFileSync(join(directory, "guide.ssrg"), "utf8")).toContain(
        `ArticleExample "${example}"`
      )
    }
    for (const [, english, japanese, destination] of source.matchAll(
      /localized\s+"([^"]+)"\s+"([^"]+)"\s*,\s*"\/docs\/language\/([^"]+)\/"/gu
    )) {
      for (const [locale, expected] of [
        ["en", english],
        ["ja", japanese],
      ]) {
        const copy = readFileSync(
          join(sourceRoot, `pages/language/${destination}/${locale}.ssrg`),
          "utf8"
        )
        expect(
          JSON.parse(copy.match(/ {2}title: ("[^"\n]+")/u)?.[1] ?? '""'),
          destination
        ).toBe(expected)
      }
    }
  }
})

test("pilot programs run as copied and rejected snippets produce their stated diagnostics", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-pilot-examples-"))
  try {
    for (const [, , example, expected, invalid, diagnostic] of pilot) {
      const path = join(directory, "main.ssrg")
      copyFileSync(
        join(root, `${exampleRoot}/src/language/${example}.ssrg`),
        path
      )
      checked(["lint", path], directory)
      expect(checked(["run", path], directory), example).toBe(`${expected}\n`)
      copyFileSync(
        join(root, `${exampleRoot}/invalid/src/language/${invalid}.ssrg`),
        path
      )
      const result = spawnSync(cli, ["lint", path], {
        cwd: directory,
        encoding: "utf8",
        timeout: 30_000,
      })
      expect(result.status, invalid).toBe(2)
      expect(result.stderr, invalid).toContain(diagnostic)
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

// Render only the six production page import closures. This verifies typed
// authoring and HTML parity; it does not claim full catalog, browser layout,
// navigation round-trip, or first-time-reader acceptance.
test("pilot pages render canonical examples, exact output, and same-identity locale links", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-pilot-render-"))
  try {
    const pages = renderPilot(directory)
    expect(pages).toHaveLength(14)
    for (const [route, paragraph] of [
      ["/composition-probe/", "Just the usage."],
      ["/ja/composition-probe/", "使い方だけ。"],
    ]) {
      const html = pages.find((page) => page.route === route)?.html ?? ""
      const body = html.match(
        /<article class="article-content">([\s\S]*?)<\/article>/u
      )?.[1]
      expect(text(body ?? "")).toBe(paragraph)
      expect(body?.match(/<p>/gu)).toHaveLength(1)
      expect(body).not.toContain("<h2")
    }
    for (const [path, id] of pilot)
      for (const prefix of ["", "/ja"])
        expect(
          pages.find(
            (page) => page.route === `${prefix}/docs/language/${path}/`
          )?.id
        ).toBe(id)
    const rendered = new Map(pages.map(({ route, html }) => [route, html]))
    for (const [path, , example, output, invalid] of pilot) {
      const route = `/docs/language/${path}/`
      const source = readFileSync(
        join(root, `${exampleRoot}/src/language/${example}.ssrg`),
        "utf8"
      )
      const rejected = readFileSync(
        join(root, `${exampleRoot}/invalid/src/language/${invalid}.ssrg`),
        "utf8"
      )
      for (const prefix of ["", "/ja"]) {
        const html = rendered.get(`${prefix}${route}`) ?? ""
        expect(html).not.toContain("site-build-error")
        expect(html).toContain(`href="${prefix ? "" : "/ja"}${route}"`)
        const code = [
          ...html.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((match) => text(match[1]))
        const chapter =
          path === "syntax/function-application"
            ? "chapter-functions"
            : path === "types/function-types-and-currying"
              ? "chapter-function-values"
              : undefined
        const secondary =
          path === "types/function-types-and-currying"
            ? "chapter-label-lengths"
            : undefined
        const secondaryCode = secondary
          ? [
              readFileSync(
                join(root, `${exampleRoot}/src/language/${secondary}.ssrg`),
                "utf8"
              ),
            ]
          : []
        const expectedCode = chapter
          ? [
              readFileSync(
                join(root, `${exampleRoot}/src/language/${chapter}.ssrg`),
                "utf8"
              ),
              ...secondaryCode,
              source,
              rejected,
            ]
          : [source, ...secondaryCode, rejected]
        expect(code, `${prefix}${route}`).toEqual(expectedCode)
        const terminals = [
          ...html.matchAll(
            /<section class="code-panel terminal-panel">[\s\S]*?<pre><code>([\s\S]*?)<\/code>/gu
          ),
        ].map((match) => text(match[1]))
        expect(terminals, route).toContain(output)
        if (path === "types/function-types-and-currying") {
          expect(terminals, route).toContain("[ Build , bundle]\n[bundle]")
          expect(terminals, route).toContain("[5, 6, 5, 3]\n19")
          expect(text(html), route).toContain(
            prefix
              ? "mainのdoは、この二つの出力Effectを順につなぐ"
              : "The do block in main sequences these two output Effects"
          )
          expect(text(html), route).not.toContain("Search once")
        }
        const headings = [...html.matchAll(/<h2[^>]*\bid="([^"]+)"/gu)].map(
          (match) => match[1]
        )
        expect(new Set(headings).size, route).toBe(headings.length)
        if (path === "syntax/function-application") {
          for (const anchor of [
            "understand-this",
            "reading-the-example",
            "why",
            "rules",
            "mistakes",
            "related-rules",
          ])
            expect(headings, `${prefix}${route}: retained fragment`).toContain(
              anchor
            )
          const outputPosition = html.indexOf("terminal-panel")
          expect(outputPosition).toBeGreaterThan(-1)
          expect(html.indexOf('id="rules"')).toBeGreaterThan(outputPosition)
          expect(text(html), route).toContain("SES-T0101")
          expect(text(html), route).toContain("addOne 2")
          expect(text(html), route).toContain("Int -> (Int -> Int)")
          expect(text(html), route).toContain("identity<String>")
          expect(text(html), route).toContain("answer ()")
        }

        if (process.env.SESERAGI_PILOT_OUTPUT) {
          const target = join(
            resolve(root, process.env.SESERAGI_PILOT_OUTPUT),
            `${prefix}${route}`,
            "index.html"
          )
          mkdirSync(dirname(target), { recursive: true })
          writeFileSync(target, html)
        }
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("suggested repairs and later call forms execute with the stated types", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-pilot-repairs-"))
  try {
    const repairs = [
      [
        "pilot-immutable",
        "original.score = 11",
        "let updated = { ...original, score: 11 }",
        "",
        "11",
      ],
      ["pilot-built-in-types", "= 3", "= 3.0", "price", "3.0"],
      ["pilot-annotations", '"3"', "3", "count", "3"],
      ["pilot-function-application", "add(1, 2)", "add 1 2", "answer", "3"],
      [
        "pilot-blocks",
        "let answer = subtotal",
        "let answer = total 3",
        "answer",
        "65",
      ],
      ["pilot-currying", "= add 1", "= add 1 2", "answer", "3"],
    ]
    for (const [name, before, after, result, expected] of repairs) {
      let source = readFileSync(
        join(root, `${exampleRoot}/invalid/src/language/${name}.ssrg`),
        "utf8"
      )
      expect(source).toContain(before)
      source = source.replace(before, after)
      source = result
        ? `${source}\npub effect fn main = println (show ${result})\n`
        : source.replace("show original.score", "show updated.score")
      const path = join(directory, "main.ssrg")
      writeFileSync(path, source)
      checked(["lint", path], directory)
      expect(checked(["run", path], directory), name).toBe(`${expected}\n`)
    }
    for (const [source, expected] of [
      [
        'import { fromInt } from "std/float"\nlet price: Float = fromInt 3\npub effect fn main = println (show price)\n',
        "3.0",
      ],
      [
        "fn answer -> Int = 42\npub effect fn main = println (show (answer ()))\n",
        "42",
      ],
      [
        'fn identity<A> value: A -> A = value\npub effect fn main = println (identity<String> "ready")\n',
        "ready",
      ],
    ]) {
      const path = join(directory, "main.ssrg")
      writeFileSync(path, source)
      expect(checked(["run", path], directory)).toBe(`${expected}\n`)
    }
    const path = join(directory, "main.ssrg")
    writeFileSync(
      path,
      "fn total quantity: Int -> Int = { let subtotal = quantity * 20 }\npub effect fn main = println (show (total 3))\n"
    )
    const result = spawnSync(cli, ["lint", path], {
      cwd: directory,
      encoding: "utf8",
    })
    expect(result.status).toBe(2)
    expect(result.stderr).toContain("SES-T0101")
    expect(result.stderr).toContain("body produces Unit")
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
