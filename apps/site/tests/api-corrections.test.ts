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
import { join, resolve } from "node:path"
import {
  type BrowserStorageHost,
  createBrowserStorageProvider,
} from "../../../runtime/ts/src/browser/provider-storage"
import * as effects from "../../../runtime/ts/src/effect"
import { createProviderStorage } from "../../../runtime/ts/src/provider-storage"
import * as storage from "../../../runtime/ts/src/storage"
import {
  QueueClosed,
  takeText,
} from "../examples/comparisons/api-corrections/queue-take"
import { readName } from "../examples/comparisons/api-corrections/storage-get"
import {
  apiCorrectionCases,
  apiCorrectionExamples,
} from "../scripts/api-corrections"
import { compilerReferenceModules } from "../scripts/reference"
import { renderPageClosure } from "./render-page-closure"

setDefaultTimeout(180_000)
const root = resolve(import.meta.dir, "../../..")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
const playground = "https://seseragi.vercel.app/"
const examples = apiCorrectionExamples(playground)
const nativeRoot = join(root, "apps/site/examples/src/api-corrections")
const typescriptRoot = join(
  root,
  "apps/site/examples/comparisons/api-corrections"
)
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

test("eight complete native snippets and scoped queue cleanup execute with exact results", () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-api-corrections-native-")
  )
  try {
    for (const item of apiCorrectionCases) {
      const sample = examples.find(
        (x) => x.id === `api-correction-${item.slug}`
      )!
      expect(sample.sha256).toBe(
        createHash("sha256").update(sample.source).digest("hex")
      )
      if (!item.standalone) {
        expect(sample.playgroundUrl).toBe("")
        expect(sample.source).not.toContain("pub effect fn main")
        continue
      }
      writeFileSync(join(directory, "main.ssrg"), sample.source)
      const result = run(cli, ["run", "main.ssrg"], directory)
      expect(result.status, `${item.slug}: ${result.stderr}`).toBe(0)
      expect(result.stderr).toBe("")
      expect(result.stdout).toBe(`${item.expected}\n`)
      expect(new URL(sample.playgroundUrl).searchParams.get("source")).toBe(
        sample.source
      )
    }
    writeFileSync(
      join(directory, "main.ssrg"),
      readFileSync(join(nativeRoot, "queue-cleanup.ssrg"))
    )
    const cleanup = run(cli, ["run", "main.ssrg"], directory)
    expect(cleanup.status, cleanup.stderr).toBe(0)
    expect(cleanup.stderr).toBe("")
    expect(cleanup.stdout).toBe("true\n42\n")
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("all nine TypeScript bridges typecheck; seven complete programs and two consumers preserve outcomes", async () => {
  const result = run("bun", [
    "x",
    "tsc",
    "--noEmit",
    "--strict",
    "--target",
    "ES2022",
    "--module",
    "ESNext",
    "--moduleResolution",
    "bundler",
    "--skipLibCheck",
    ...apiCorrectionCases.map((x) => join(typescriptRoot, `${x.slug}.ts`)),
  ])
  expect(result.status, result.stdout + result.stderr).toBe(0)
  for (const item of apiCorrectionCases) {
    const sample = examples.find(
      (x) => x.id === `api-correction-${item.slug}-ts`
    )!
    expect(sample.playgroundUrl).toBe("")
    if (item.typescriptStandalone) {
      const result = run("bun", [join(typescriptRoot, `${item.slug}.ts`)])
      expect(result.status, result.stderr).toBe(0)
      expect(result.stdout).toBe(`${item.typescriptExpected}\n`)
    }
  }
  const values = [10, 20]
  const queue = {
    async take() {
      const value = values.shift()
      if (value === undefined) throw new QueueClosed()
      return value
    },
  }
  expect([
    await takeText(queue),
    await takeText(queue),
    await takeText(queue),
  ]).toEqual(["10", "20", "closed"])
  const defect = new Error("unexpected")
  await expect(
    takeText({
      async take() {
        throw defect
      },
    })
  ).rejects.toBe(defect)
  for (const [value, expected] of [
    ["Aki", "found: [Aki]"],
    ["", "found: []"],
    [null, "missing"],
  ] as const)
    expect(
      readName({
        getItem(key) {
          expect(key).toBe("name")
          return value
        },
      })
    ).toBe(expected)
  expect(
    readName({
      getItem() {
        throw new Error("denied by mock")
      },
    })
  ).toBe("denied by mock")
  expect(
    readName({
      getItem() {
        throw "other thrown value"
      },
    })
  ).toBe("other thrown value")
})

test("web integration compiles with a caller and real storage adapter preserves mock absence and failures", async () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-api-storage-"))
  try {
    const source = readFileSync(join(nativeRoot, "storage-get.ssrg"), "utf8")
    writeFileSync(
      join(directory, "main.ssrg"),
      `${source}\npub effect fn main = do {\n  _ <- readName ()\n  effects.succeed ()\n}\n`
    )
    const result = run(
      cli,
      ["build", "main.ssrg", "--target", "web", "--out-dir", "web"],
      directory
    )
    expect(result.status, result.stderr).toBe(0)
    expect(
      JSON.parse(
        readFileSync(join(directory, "web/.seseragi-build.json"), "utf8")
      ).target
    ).toBe("web")
    // This verifies the browser-target artifact, not a browser session or visible application.
    const requests: string[] = []
    const host: BrowserStorageHost = {
      get(area, key) {
        requests.push(`${area}:${key}`)
        if (key === "blocked") {
          const error = new Error("denied by mock")
          error.name = "SecurityError"
          throw error
        }
        return (
          ({ name: "Aki", empty: "" } as Record<string, string>)[key] ?? null
        )
      },
      set() {
        throw new Error("must not write")
      },
      remove() {
        throw new Error("must not remove")
      },
      clear() {
        throw new Error("must not clear")
      },
      keys() {
        return []
      },
    }
    const service = createProviderStorage({
      provider: "seseragi/runtime-browser#storage",
      service: "std/web/storage::Storage",
      entry: createBrowserStorageProvider(host),
    })
    const outputs = []
    for (const key of ["name", "empty", "absent", "blocked"])
      outputs.push(
        await effects.run(storage.get(storage.Local, key), { storage: service })
      )
    const session = await effects.run(storage.get(storage.Session, "name"), {
      storage: service,
    })
    expect(outputs[0]).toEqual({
      kind: "success",
      value: storage.storageJust("Aki"),
    })
    expect(outputs[1]).toEqual({
      kind: "success",
      value: storage.storageJust(""),
    })
    expect(outputs[2]).toEqual({
      kind: "success",
      value: storage.storageNothing,
    })
    expect(outputs[3]).toMatchObject({
      kind: "failure",
      error: { tag: "StorageSecurityFailure" },
    })
    if (outputs[3].kind === "failure")
      expect(storage.errorMessage(outputs[3].error)).toBe("denied by mock")
    expect(session).toEqual(outputs[0])
    expect(requests).toEqual([
      "local:name",
      "local:empty",
      "local:absent",
      "local:blocked",
      "session:name",
    ])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("runtime probe separately verifies already-waiting queue cancellation, draining, and FIFO", () => {
  const result = run("bun", [
    join(root, "runtime/ts/probes/effect-concurrency.ts"),
  ])
  expect(result.status, result.stdout + result.stderr).toBe(0)
})

test("wrong argument shapes are rejected, while each published source is the verified repair", () => {
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-api-corrections-invalid-")
  )
  const mutations: Record<string, [string, string]> = {
    "map-get": ['maps.get "name" names', "maps.get 0 names"],
    "state-get": ["state.get ()", "state.get 7"],
    "queue-take": ["queues.take queue", "queues.take 42"],
    "storage-get": [
      'storage.get storage.Local "name"',
      "storage.get storage.Local 0",
    ],
    "nonempty-head": ["nonempty.head single", "nonempty.head `[]"],
    "nonempty-tail": ["nonempty.tail single", "nonempty.tail `[]"],
    "nonempty-tolist": ["nonempty.toList single", "nonempty.toList [10]"],
    "set-toarray": ["sets.toArray values", "sets.toArray `[2, 1]"],
    "set-tolist": ["sets.toList values", "sets.toList [2, 1]"],
  }
  try {
    for (const item of apiCorrectionCases) {
      const source = readFileSync(join(nativeRoot, `${item.slug}.ssrg`), "utf8")
      const [from, to] = mutations[item.slug]
      expect(source, item.slug).toContain(from)
      writeFileSync(join(directory, "main.ssrg"), source.replace(from, to))
      const result = run(cli, ["lint", "main.ssrg"], directory)
      expect(result.status, item.slug).not.toBe(0)
      expect(result.stderr, `${item.slug}: ${result.stderr}`).toContain(
        [
          "queue-take",
          "storage-get",
          "nonempty-tolist",
          "set-toarray",
          "set-tolist",
        ].includes(item.slug)
          ? "SES-T0201"
          : "SES-T0101"
      )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("exact identity, module, namespace and kind choose only the nine corrections", () => {
  const all = compilerReferenceModules()
  const items = apiCorrectionCases.map((item) => {
    const matches = all
      .flatMap((module) => module.items)
      .filter(
        (symbol) =>
          symbol.identity === item.identity &&
          symbol.namespace === "value" &&
          symbol.itemKind === item.kind
      )
    expect(matches).toHaveLength(1)
    return matches[0]
  })
  const probes = items.flatMap((item) => [
    item,
    { ...item, namespace: "type" },
    {
      ...item,
      itemKind: item.itemKind === "function" ? "effect-function" : "function",
    },
    { ...item, module: "std/array" },
    { ...item, identity: `std/array::${item.name}` },
  ])
  const controls = all
    .flatMap((module) => module.items)
    .filter((x) =>
      [
        "std/array::get",
        "std/list::take",
        "std/array::toList",
        "std/ref::get",
      ].includes(x.identity)
    )
  expect(controls).toHaveLength(4)
  const oldOverlay = all
    .flatMap((module) => module.items)
    .find((x) => x.identity === "std/array::chunksOf")!
  const input = {
    schema: 1,
    origin: "",
    playgroundUrl: "",
    tourUrl: "",
    grammar: "",
    examples: [],
    referenceModules: [
      {
        ...all[0],
        items: [
          ...probes,
          ...controls,
          oldOverlay,
          { ...oldOverlay, itemKind: "effect-function" },
        ],
      },
    ],
  }
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-api-corrections-identity-")
  )
  try {
    const result = renderPageClosure<Array<[boolean, boolean]>>({
      directory,
      modules: [
        "reference/editorial/catalog",
        "reference/editorial/correction-catalog",
      ],
      entry: `import * as json from "std/json"
import * as arrays from "std/array"
import { ReferenceSymbol, decodeBuildInput } from "./model/build"
import { Editorial } from "./reference/editorial/model"
import { editorialFor } from "./reference/editorial/catalog"
import { correctionEditorialFor } from "./reference/editorial/correction-catalog"
fn present value: Maybe<Editorial> -> Bool = match value {
  Just _ -> True
  Nothing -> False
}
pub effect fn main = match decodeBuildInput ${JSON.stringify(JSON.stringify(input))} {
  Left _ -> println "[]"
  Right input -> println (json.encodeString (arrays.concat [[(present (correctionEditorialFor symbol), present (editorialFor symbol)) | symbol <- module.items] | module <- input.referenceModules]))
}`,
    })
    expect(result).toHaveLength(51)
    for (let i = 0; i < 45; i++)
      expect(result[i]).toEqual(i % 5 === 0 ? [true, true] : [false, false])
    for (let i = 45; i < 49; i++) expect(result[i]).toEqual([false, false])
    expect(result[49]).toEqual([false, true])
    expect(result[50]).toEqual([false, false])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("eighteen current production bodies correct summaries, preserve signatures, and show exact canonical panels", () => {
  const all = compilerReferenceModules()
  const wanted = new Set<string>(apiCorrectionCases.map((x) => x.identity))
  const controls = new Set([
    "std/map::filter",
    "std/web/storage::keys",
    "std/transformer/state::put",
    "std/queue::offer",
    // The NonEmptyList type now has authored copy too. Its Hash instance
    // remains a separate, unreviewed no-overlay control in the same module.
    "std/non-empty-list::Hash",
    "std/set::filter",
  ])
  const modules = all
    .map((module) => ({
      ...module,
      items: module.items.filter(
        (item) => wanted.has(item.identity) || controls.has(item.identity)
      ),
    }))
    .filter((module) => module.items.length > 0)
  expect(modules).toHaveLength(6)
  expect(modules.flatMap((module) => module.items)).toHaveLength(15)
  const input = {
    schema: 1,
    origin: "https://seseragi.example",
    playgroundUrl: playground,
    tourUrl: "",
    grammar: "",
    examples,
    referenceModules: modules,
  }
  const directory = mkdtempSync(
    join(tmpdir(), "seseragi-api-corrections-render-")
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
    let home = PageDefinition { id: "library", path: "/docs/library/", kind: Reference,
      title: localized "Library" "ライブラリ", summary: localized "Library" "ライブラリ", blocks: [] }
    let site = SiteCatalog { home, pages, areas: [NavigationArea { id: "library", title: home.title, description: home.summary, landing: home, groups }] }
    println (json.encodeString [Output { route: localizedRoute locale page.path, html: renderDocument input site locale page } | locale <- [En, Ja], page <- pages])
  }
}`,
    })
    expect(output).toHaveLength(42)
    const pages = new Map(output.map((item) => [item.route, item.html]))
    for (const item of apiCorrectionCases) {
      const symbol = modules
        .flatMap((m) => m.items)
        .find((x) => x.identity === item.identity)!
      const relative = `${symbol.module.replace("std/", "")}/${symbol.itemKind}/${symbol.name.toLowerCase()}`
      for (const prefix of ["", "/ja"]) {
        const route = `${prefix}/docs/library/${relative}/`
        const html = pages.get(route)!
        const body = html.match(/<article[\s\S]*?<\/article>/u)?.[0] ?? html
        const text = plain(body)
        expect(text, route).toContain(symbol.signature)
        expect(text).not.toContain(
          prefix ? symbol.descriptionJa : symbol.descriptionEn
        )
        expect(text).toContain(item.expected)
        const copy = readFileSync(
          join(
            root,
            "apps/site/src/reference/editorial",
            item.path,
            `${prefix ? "ja" : "en"}.ssrg`
          ),
          "utf8"
        )
        const otherCopy = readFileSync(
          join(
            root,
            "apps/site/src/reference/editorial",
            item.path,
            `${prefix ? "en" : "ja"}.ssrg`
          ),
          "utf8"
        )
        const paragraphs = (source: string) =>
          [...source.matchAll(/^ {4}("(?:[^"\\]|\\.)*")/gmu)].map(
            (match) => JSON.parse(match[1]) as string
          )
        const expectedParagraphs = paragraphs(copy)
        expect(expectedParagraphs.length, route).toBeGreaterThanOrEqual(4)
        for (const paragraph of expectedParagraphs)
          expect(text, route).toContain(paragraph)
        for (const paragraph of paragraphs(otherCopy))
          expect(text, route).not.toContain(paragraph)

        expect(html).toContain('id="using-this-operation"')
        expect(html).toContain('id="typescript-comparison"')
        expect(html).toContain('id="operation-rules"')
        expect(html).toContain('id="canonical-declaration"')
        expect(html).not.toContain("Missing canonical example")
        const panels = [
          ...body.matchAll(
            /<code class="seseragi-highlight">([\s\S]*?)<\/code>/gu
          ),
        ].map((x) => plain(x[1]))
        for (const suffix of ["", "-ts"]) {
          const source = examples.find(
            (x) => x.id === `api-correction-${item.slug}${suffix}`
          )!
          expect(
            panels.filter((panel) => panel === source.source),
            `${route}:${suffix}`
          ).toHaveLength(1)
        }
        const moduleHtml = pages.get(
          `${prefix}/docs/library/${symbol.module.replace("std/", "")}/`
        )!
        expect(moduleHtml).toContain(`href="${route}"`)
        if (!item.standalone)
          expect(body).not.toContain("https://seseragi.vercel.app/?source=")
      }
      const localeSources = ["en", "ja"].map((locale) =>
        readFileSync(
          join(
            root,
            "apps/site/src/reference/editorial",
            item.path,
            `${locale}.ssrg`
          ),
          "utf8"
        )
      )
      const fields = (source: string) =>
        [...source.matchAll(/^ {2}(\w+): /gmu)].map((x) => x[1]).sort()
      expect(fields(localeSources[0])).toEqual(fields(localeSources[1]))
      for (const name of ["after", "details"]) {
        const count = (source: string) =>
          [
            ...(
              source.match(
                new RegExp(`  ${name}: \\[([\\s\\S]*?)\\n  \\]`, "u")
              )?.[1] ?? ""
            ).matchAll(/^ {4}"/gmu),
          ].length
        expect(count(localeSources[0]), `${item.slug}:${name}`).toBeGreaterThan(
          0
        )
        expect(count(localeSources[0]), `${item.slug}:${name}`).toBe(
          count(localeSources[1])
        )
      }
    }
    for (const module of modules)
      for (const symbol of module.items.filter((x) =>
        controls.has(x.identity)
      )) {
        for (const prefix of ["", "/ja"]) {
          const html = pages.get(
            `${prefix}/docs/library/${symbol.module.replace("std/", "")}/${symbol.itemKind}/${symbol.name.toLowerCase().replaceAll("<", "-").replaceAll(">", "").replaceAll(", ", "-")}/`
          )!
          expect(html).not.toContain('id="typescript-comparison"')
          if (symbol.itemKind === "instance") {
            expect(symbol.namespace).toBe("instance")
            expect(plain(html)).toContain(symbol.signature)
            expect(plain(html)).toContain(
              prefix ? symbol.reading.ja : symbol.reading.en
            )
            expect(html).not.toContain('id="using-this-type"')
          } else {
            expect(plain(html)).toContain(
              prefix ? symbol.descriptionJa : symbol.descriptionEn
            )
          }
        }
      }
    if (process.env.API_CORRECTIONS_RENDER_DIR) {
      mkdirSync(process.env.API_CORRECTIONS_RENDER_DIR, { recursive: true })
      for (const page of output)
        writeFileSync(
          join(
            process.env.API_CORRECTIONS_RENDER_DIR,
            `${page.route.replaceAll("/", "_")}.html`
          ),
          page.html
        )
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
