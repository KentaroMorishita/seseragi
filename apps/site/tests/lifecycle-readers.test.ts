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
import { generatorInput } from "../scripts/build"
import {
  lifecycleReaderExamples,
  lifecycleReaders,
} from "../scripts/lifecycle-readers"

const lifecycleCases = lifecycleReaders.map((entry) => ({
  ...entry,
  path: `effects/${entry.path}`,
  id: entry.id,
  stage: "lint" as const,
  source: `lifecycle-reader-${entry.example}`,
  example: `lifecycle-reader-${entry.example}`,
  invalid: `lifecycle-reader-${entry.example}`,
  invalidId: `lifecycle-reader-${entry.example}-invalid`,
}))

import { renderPageClosure } from "./render-page-closure"

const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const validRoot = "apps/site/examples/src/language"
const invalidRoot = "apps/site/examples/invalid/src/language"
const playground = "https://seseragi.vercel.app/"
setDefaultTimeout(240_000)

function run(command: string, args: string[], cwd: string) {
  return spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    timeout: 15_000,
    maxBuffer: 16 * 1024 * 1024,
  })
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

test("eleven lifecycle questions keep their existing page identities and local bilingual explanations", () => {
  expect(lifecycleCases).toHaveLength(11)
  for (const item of lifecycleCases) {
    const directory = join(root, "apps/site/src/pages/language", item.path)
    const source = readFileSync(join(directory, "page.ssrg"), "utf8")
    expect(source).toContain(`"${item.id}"`)
    expect(source).toContain(`"/docs/language/${item.path}/"`)
    expect(source).toContain("let page = detailed ()")
    expect(source).toContain("page.blocks")
    for (const locale of ["en", "ja"]) {
      const copy = readFileSync(join(directory, `${locale}.ssrg`), "utf8")
      for (const field of ["question", "summary", "reading", "result"])
        expect(copy).toMatch(new RegExp(`  ${field}: "[^\"]`))
      for (const field of [
        "readerBridgeOne",
        "readerBridgeTwo",
        "readerMistake",
      ])
        expect(copy).toContain(`pub fn ${field} -> String`)
    }
  }
})

test("complete bounded lifecycle examples execute independently and rejected cases report the observed diagnostic", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-lifecycle-examples-"))
  try {
    for (const item of lifecycleCases) {
      const source = readFileSync(
        join(root, validRoot, `${item.source}.ssrg`),
        "utf8"
      )
      writeFileSync(join(temporary, "main.ssrg"), source)
      const valid = run(cli, ["run", "main.ssrg"], temporary)
      expect(valid.status, `${item.path}: ${valid.stderr}`).toBe(0)
      expect(valid.stderr).toBe("")
      expect(valid.stdout, item.path).toBe(`${item.output}\n`)
      const invalidSource = readFileSync(
        join(root, invalidRoot, `${item.invalid}.ssrg`),
        "utf8"
      )
      writeFileSync(join(temporary, "main.ssrg"), invalidSource)

      const invalid = run(cli, [item.stage, "main.ssrg"], temporary)
      expect(invalid.status, item.path).not.toBe(0)
      expect(invalid.stderr, item.path).toContain(item.diagnostic)
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test("twenty-two lifecycle locale pages render canonical first examples and preserve detailed sections", () => {
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-lifecycle-pages-"))
  try {
    const published = new Map(
      generatorInput(playground).examples.map((example) => [
        example.id,
        example,
      ])
    )
    const sources = new Map(
      lifecycleReaderExamples(playground).map((example) => {
        const registered = published.get(example.id)
        expect(registered, example.id).toEqual(example)
        return [example.id, registered ?? example]
      })
    )
    for (const item of lifecycleCases) {
      const pageSource = readFileSync(
        join(root, "apps/site/src/pages/language", item.path, "page.ssrg"),
        "utf8"
      )
      const ids = [
        item.example,
        item.invalidId,
        ...[...pageSource.matchAll(/CodeExample\s*\(\s*"([^"]+)"/gu)].map(
          (match) => match[1]
        ),
      ]
      for (const id of ids) {
        if (sources.has(id)) continue
        const example = published.get(id)
        expect(example, id).toBeDefined()
        if (example) sources.set(id, example)
      }
    }
    const input = {
      schema: 1,
      origin: "https://seseragi.example",
      playgroundUrl: playground,
      tourUrl: "https://seseragi.vercel.app/tour/",
      grammar: "",
      referenceModules: [],
      examples: [...sources.values()],
    }
    const pages = renderPageClosure<Array<{ route: string; html: string }>>({
      directory: temporary,
      timeoutMs: 180_000,
      modules: [
        ...lifecycleCases.map((item) => `pages/language/${item.path}/page`),
        "render/document",
      ],
      entry: `import * as json from "std/json"
import { decodeBuildInput } from "./model/build"
import { En, Ja, localizedRoute } from "./model/locale"
import { SiteCatalog } from "./model/page"
import { renderDocument } from "./render/document"
${lifecycleCases.map((item, i) => `import { page as page${i} } from "./pages/language/${item.path}/page"`).join("\n")}
struct Output deriving JsonEncode { route: String, html: String }
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> {
  let pages = [${lifecycleCases.map((_, i) => `page${i} ()`).join(", ")}]
  let site = SiteCatalog { home: page0 (), areas: [], pages }
  println (json.encodeString [Output { route: localizedRoute locale page.path, html: renderDocument input site locale page }
    | locale <- [En, Ja], page <- site.pages])
  }
}`,
    })
    expect(pages).toHaveLength(22)
    for (const item of lifecycleCases) {
      for (const prefix of ["", "/ja"]) {
        const route = `${prefix}/docs/language/${item.path}/`
        const html = pages.find((page) => page.route === route)?.html ?? ""
        expect(html, route).not.toContain("site-build-error")
        const firstCode =
          html.match(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
          )?.[1] ?? ""
        const panels = [
          ...html.matchAll(/<section class="code-panel">[\s\S]*?<\/section>/gu),
        ].map((match) => match[0])
        expect(panels[0], route).toContain('class="playground-link"')
        const fragmentIds = [
          "effects-effectful-for",
          "effects-task",
          "effects-execution-order",
          "effects-cancellation-resources",
          "effects-scheduler-fairness",
          "effects-fiber-supervision",
          "effects-signal-transactions",
          "effects-derived-signals",
          "effects-signal-operators",
          "effects-signal-subscription",
          "effects-foreign-failure",
        ]
        for (const fragmentId of fragmentIds) {
          const fragment = published.get(fragmentId)
          if (!fragment) continue
          const panel = panels.find(
            (candidate) =>
              text(
                candidate.match(
                  /<code class="seseragi-highlight">([\s\S]*?)<\/code>/u
                )?.[1] ?? ""
              ) === fragment.source
          )
          if (panel) {
            expect(fragment.playgroundUrl).toBe("")
            expect(panel).not.toContain('class="playground-link"')
          }
        }
        expect(text(firstCode), route).toBe(
          readFileSync(join(root, validRoot, `${item.source}.ssrg`), "utf8")
        )
        expect(text(html), route).toContain(item.output)
        expect(html).toContain('id="related-rules"')
        // Resolve every related-link title from its actual destination locale,
        // including pages outside this deliberately bounded render fixture.
        const related =
          html.split('id="related-rules"')[1]?.split("</ul>")[0] ?? ""
        const relatedLinks = [
          ...related.matchAll(
            /<li><a[^>]*href="([^"]+)"[^>]*>([\s\S]*?)<\/a><span>([\s\S]*?)<\/span><\/li>/gu
          ),
        ]
        expect(relatedLinks, route).toHaveLength(3)
        for (const link of relatedLinks) {
          const destination = link[1].replace(/^\/ja(?=\/)/u, "")
          expect(destination).toStartWith("/docs/language/")
          const copy = readFileSync(
            join(
              root,
              "apps/site/src/pages",
              destination.replace(/^\/docs\//u, ""),
              `${prefix ? "ja" : "en"}.ssrg`
            ),
            "utf8"
          )
          const titleLiteral = copy.match(
            /\btitle:\s*("(?:[^"\\]|\\.)*")/u
          )?.[1]
          expect(titleLiteral, link[1]).toBeDefined()
          const destinationTitle = JSON.parse(titleLiteral ?? '""') as string
          expect(text(link[2]), `${route} → ${link[1]}`).toBe(destinationTitle)
          expect(
            text(link[3]).replace(/^[:：]\s*/u, "").length
          ).toBeGreaterThan(0)
        }
        const oldSections = [
          "rule",
          "typing",
          "evaluation",
          "diagnostics",
          "declaration-example",
        ]
        for (const section of oldSections)
          expect(html, route).toContain(`id="${section}"`)
        const alternate = `${prefix ? "" : "/ja"}/docs/language/${item.path}/`
        expect(html).toContain(`href="${alternate}"`)
        if (process.env.SESERAGI_LIFECYCLE_OUTPUT) {
          const destination = join(
            resolve(process.env.SESERAGI_LIFECYCLE_OUTPUT),
            route,
            "index.html"
          )
          mkdirSync(dirname(destination), { recursive: true })
          writeFileSync(destination, html)
        }
      }
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true })
  }
})

test("existing resource and concurrency probes retain cancellation, LIFO, and failure selection contracts", () => {
  for (const name of ["effect-resource", "effect-concurrency"]) {
    const result = run(
      process.execPath,
      [join(root, `runtime/ts/probes/${name}.ts`)],
      root
    )
    expect(result.status, `${name}: ${result.stderr}`).toBe(0)
    expect(result.stdout).toContain(`${name.replaceAll("-", " ")} probe passed`)
  }
})

test("Signal plans stage in order, roll back a staging defect, and publish only complete values", async () => {
  const effects = await import("../../../runtime/ts/src/effect")
  const signals = await import("../../../runtime/ts/src/signal")
  async function success<A>(
    effect: import("../../../runtime/ts/src/effect").Effect<unknown, never, A>
  ): Promise<A> {
    const result = await effects.run(effect, {})
    expect(result.kind).toBe("success")
    if (result.kind !== "success")
      throw new Error("Expected successful runtime operation")
    return result.value
  }
  const left = await success(signals.make(1))
  const right = await success(signals.make(2))
  const total = signals.combine(
    (a: number) => (b: number) => a + b,
    left,
    right
  )
  const observed: number[] = []
  const subscription = await success(
    signals.subscribe(
      (value: number) =>
        effects.defer(() => {
          observed.push(value)
          return effects.succeed(undefined)
        }),
      total
    )
  )
  try {
    await success(
      signals.transaction([
        signals.planSet(10, left),
        signals.planUpdate((value: number) => value + 1, left),
        signals.planSet(20, right),
      ])
    )
    expect(await success(signals.read(left))).toBe(11)
    expect(await success(signals.read(total))).toBe(31)
    expect(observed).toEqual([3, 31])
    await expect(
      effects.run(
        signals.transaction([
          signals.planSet(100, left),
          signals.planUpdate(() => {
            throw new Error("staging defect")
          }, right),
        ]),
        {}
      )
    ).rejects.toThrow("staging defect")
    expect(await success(signals.read(left))).toBe(11)
    expect(await success(signals.read(right))).toBe(20)
    expect(observed).toEqual([3, 31])
    await success(signals.unsubscribe(subscription))
    await success(signals.unsubscribe(subscription))
    await success(signals.set(40, right))
    expect(observed).toEqual([3, 31])
  } finally {
    await success(signals.unsubscribe(subscription))
  }
})
