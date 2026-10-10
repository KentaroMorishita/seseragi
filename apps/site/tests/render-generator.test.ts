import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, relative, resolve } from "node:path"
import { validateInternalLinks } from "../scripts/build"
import {
  collectRenderedPages,
  RENDER_BATCH_SIZE,
  type RenderSelection,
  renderGenerator,
} from "../scripts/render-generator"

const root = resolve(import.meta.dir, "../../..")
const sourceRoot = join(root, "apps/site/src")
const cli = resolve(root, process.env.SESERAGI_BIN ?? "target/debug/seseragi")
setDefaultTimeout(120_000)
const input = {
  schema: 1,
  origin: "https://seseragi.example",
  playgroundUrl: "https://seseragi.vercel.app/",
  tourUrl: "https://seseragi.vercel.app/tour/",
  grammar: "",
  examples: [],
  referenceModules: [],
}
const routes = ["/", "/docs/", "/ja/", "/ja/docs/"]

test("site validation ignores IDs in escaped snapshots and non-element text", () => {
  const snapshot =
    '&lt;button id="toggle-details"&gt;Show details&lt;/button&gt;'
  expect(() =>
    validateInternalLinks([
      {
        route: "/",
        html: `<main id="content"><pre><code>${snapshot.repeat(3)}</code></pre>
          <!-- <div id="content"></div> -->
          <script>const markup = '<div id="content"></div>'</script>
          <style>p::after { content: '<div id="content"></div>'; }</style>
          <textarea><div id="content"></div></textarea>
          <div data-example='<div id="content"></div>'></div></main>`,
      },
    ])
  ).not.toThrow()
})

test("site validation still rejects duplicate real element IDs", () => {
  for (const html of [
    '<h2 id="same">First</h2><h3 id="same">Second</h3>',
    "<h2 id='same'>First</h2><h3 ID=same>Second</h3>",
    '<h2 id="a>b">First</h2><h3 id="a>b">Second</h3>',
  ])
    expect(() => validateInternalLinks([{ route: "/", html }])).toThrow(
      "Duplicate HTML id in /"
    )
})

test("site validation follows only real anchors and real fragment targets", () => {
  expect(() =>
    validateInternalLinks([
      {
        route: "/",
        html: `<pre>&lt;a href="/absent/"&gt;example&lt;/a&gt;</pre>
          <!-- <a href="/absent/">comment</a> -->
          <script>const example = '<a href="/absent/">script</a>'</script>
          <a href='/docs/#a&amp;b'>Read the actual section</a>`,
      },
      { route: "/docs/", html: '<h2 id="a&amp;b">Section</h2>' },
    ])
  ).not.toThrow()
  for (const html of [
    '<pre>&lt;h2 id="example"&gt;Text&lt;/h2&gt;</pre>',
    '<!-- <h2 id="example">Text</h2> -->',
    `<script>const example = '<h2 id="example">Text</h2>'</script>`,
  ])
    expect(() =>
      validateInternalLinks([
        { route: "/", html: '<a href="/docs/#example">Missing section</a>' },
        { route: "/docs/", html },
      ])
    ).toThrow("Unresolved fragment from /: /docs/#example")
  expect(() =>
    validateInternalLinks([
      { route: "/", html: "<a href='/absent/'>Missing page</a>" },
    ])
  ).toThrow("Unresolved internal link from /: /absent/")
})

function fake(selection: RenderSelection) {
  return {
    schema: 1,
    mode: selection.mode,
    offset: selection.offset,
    total: routes.length,
    routes: selection.mode === "plan" ? routes : [],
    pages:
      selection.mode === "plan"
        ? []
        : routes
            .slice(selection.offset, selection.offset + selection.count)
            .map((route) => ({ route, html: `<h1>${route}</h1>` })),
  }
}

test("batch collection follows the complete inventory without changing order", () => {
  const calls: RenderSelection[] = []
  const pages = collectRenderedPages((selection) => {
    calls.push(selection)
    return fake(selection)
  }, 3)
  expect(pages.map(({ route }) => route)).toEqual(routes)
  expect(calls.map(({ mode, offset, count }) => [mode, offset, count])).toEqual(
    [
      ["plan", 0, 0],
      ["render", 0, 3],
      ["render", 3, 1],
    ]
  )
  for (const invalid of [0, -1, RENDER_BATCH_SIZE + 1, 1.5, Number.NaN])
    expect(() => collectRenderedPages(fake, invalid)).toThrow("batch size")
})

test("the finite batch cap keeps a complete final partial batch", () => {
  const inventory = Array.from(
    { length: RENDER_BATCH_SIZE + 1 },
    (_, index) => `/page-${index}/`
  )
  const calls: RenderSelection[] = []
  const pages = collectRenderedPages((selection) => {
    calls.push(selection)
    return {
      schema: 1,
      mode: selection.mode,
      offset: selection.offset,
      total: inventory.length,
      routes: selection.mode === "plan" ? inventory : [],
      pages:
        selection.mode === "plan"
          ? []
          : inventory
              .slice(selection.offset, selection.offset + selection.count)
              .map((route) => ({ route, html: route })),
    }
  })
  expect(pages.map(({ route }) => route)).toEqual(inventory)
  expect(calls.slice(1).map(({ offset, count }) => [offset, count])).toEqual([
    [0, RENDER_BATCH_SIZE],
    [RENDER_BATCH_SIZE, 1],
  ])
})

test("batch collection rejects malformed plans and incomplete or reordered output", () => {
  for (const mutate of [
    (value: ReturnType<typeof fake>) => ({ ...value, schema: 2 }),
    (value: ReturnType<typeof fake>) => ({ ...value, offset: 9 }),
    (value: ReturnType<typeof fake>) => ({ ...value, total: 3 }),
    (value: ReturnType<typeof fake>) => ({
      ...value,
      routes: ["/", "/", "/ja/", "/ja/docs/"],
    }),
    (value: ReturnType<typeof fake>) => ({
      ...value,
      routes: ["/../escape/", ...routes.slice(1)],
    }),
    (value: ReturnType<typeof fake>) => ({
      ...value,
      pages: [{ route: "/", html: "unexpected" }],
    }),
  ])
    expect(() =>
      collectRenderedPages((selection) => mutate(fake(selection)), 2)
    ).toThrow()
  for (const mutate of [
    (value: ReturnType<typeof fake>) => ({ ...value, total: 5 }),
    (value: ReturnType<typeof fake>) => ({ ...value, routes: ["/extra/"] }),
    (value: ReturnType<typeof fake>) => ({
      ...value,
      pages: value.pages.slice(1),
    }),
    (value: ReturnType<typeof fake>) => ({
      ...value,
      pages: [...value.pages].reverse(),
    }),
    (value: ReturnType<typeof fake>) => ({
      ...value,
      pages: [...value.pages, value.pages[0]],
    }),
    (value: ReturnType<typeof fake>) => ({
      ...value,
      pages: value.pages.map((page) => ({ ...page, route: "/unknown/" })),
    }),
    (value: ReturnType<typeof fake>) => ({
      ...value,
      pages: [{ route: "/", html: 42 }],
    }),
  ])
    expect(() =>
      collectRenderedPages(
        (selection) =>
          selection.mode === "plan" ? fake(selection) : mutate(fake(selection)),
        2
      )
    ).toThrow()
  for (const value of [null, [], {}, "not an object"])
    expect(() => collectRenderedPages(() => value)).toThrow()
})

function checked(args: string[], cwd: string) {
  const started = performance.now()
  const result = spawnSync(cli, args, {
    cwd,
    encoding: "utf8",
    timeout: 90_000,
    maxBuffer: 4 * 1024 * 1024,
  })
  const elapsed = Math.round(performance.now() - started)
  expect(
    result.status,
    `${args[0]} after ${elapsed}ms: ${result.stderr || result.error?.message || result.status}`
  ).toBe(0)
  console.info(`Render protocol ${args[0]} completed in ${elapsed}ms`)
  return result.stdout
}

function fixture(directory: string) {
  const copied = new Set<string>()
  function copyModule(path: string) {
    if (
      copied.has(path) ||
      path === join(sourceRoot, "navigation/catalog.ssrg")
    )
      return
    expect(path.startsWith(`${sourceRoot}/`)).toBe(true)
    copied.add(path)
    const source = readFileSync(path, "utf8")
    const destination = join(directory, "src", relative(sourceRoot, path))
    mkdirSync(dirname(destination), { recursive: true })
    copyFileSync(path, destination)
    for (const [, specifier] of source.matchAll(/from\s+"(\.[^"]+)"/gu))
      copyModule(resolve(dirname(path), `${specifier}.ssrg`))
  }
  copyModule(join(sourceRoot, "main.ssrg"))
  mkdirSync(join(directory, "src/navigation"), { recursive: true })
  writeFileSync(
    join(directory, "seseragi.toml"),
    '[package]\nname = "render-protocol-verification"\nversion = "0.0.0"\nlanguage = ">=0.1.0 <0.2.0"\n\n[run]\nentry = "main"\ntarget = "process"\n'
  )
  writeFileSync(
    join(directory, "src/navigation/catalog.ssrg"),
    `import { BuildInput } from "../model/build"
import { localized } from "../model/locale"
import { Article, FlatSection, InternalLink, NavigationArea, NavigationGroup, NavigationSection, PageDefinition, Paragraph, SiteCatalog, Words } from "../model/page"
fn page path: String -> title: String -> PageDefinition = PageDefinition {
  id: path, path, kind: Article, title: localized title title,
  summary: localized "Quotes \\\" and Unicode 日本語" "引用符\\\"と日本語",
  blocks: [Paragraph ([Words (localized "line one\\nline two" "一行目\\n二行目")]), Paragraph ([InternalLink (localized "Other" "別の記事", "/docs/third/")])]
}
pub fn catalog input: BuildInput -> SiteCatalog = {
  let first = page "/" "First"
  let second = page "/docs/" "Second"
  let third = page "/docs/third/" "Third"
  let section = NavigationSection { id: "one", title: localized "Section" "節", kind: FlatSection, overview: first, pages: [second, third] }
  let group = NavigationGroup { id: "one", title: localized "Group" "分類", sections: [section] }
  let area = NavigationArea { id: "one", title: localized "Area" "領域", description: localized "All pages" "全記事", landing: first, groups: [group] }
  SiteCatalog { home: first, areas: [area], pages: [first, second, third] }
}
`
  )
  // Independent direct render of the same small catalog is a byte-level oracle.
  writeFileSync(
    join(directory, "src/direct.ssrg"),
    `import * as arrays from "std/array"
import * as json from "std/json"
import { BuildInput } from "./model/build"
import { En, Ja, localizedRoute } from "./model/locale"
import { RenderedPage } from "./model/render-request"
import { PageDefinition, SiteCatalog } from "./model/page"
import { catalog } from "./navigation/catalog"
import { renderDocument } from "./render/document"
pub effect fn main = {
  let input = BuildInput { schema: 1, origin: "https://seseragi.example", playgroundUrl: "https://seseragi.vercel.app/", tourUrl: "https://seseragi.vercel.app/tour/", grammar: "", examples: [], referenceModules: [] }
  let site = catalog input
  println (json.encodeString [RenderedPage { route: localizedRoute locale page.path, html: renderDocument input site locale page } | locale <- [En, Ja], page <- site.pages])
}
`
  )
  checked(["lock", "update", directory], directory)
  checked(
    [
      "build",
      directory,
      "--profile",
      "release",
      "--out-dir",
      join(directory, "dist"),
    ],
    directory
  )
  return join(directory, "dist/entry.js")
}

test("typed protocol preserves full navigation and exact HTML across batch sizes", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-render-protocol-"))
  try {
    const entry = fixture(directory)
    const transportDirectory = dirname(entry)
    writeFileSync(
      join(transportDirectory, "render-input.jsonl"),
      "keep existing input"
    )
    writeFileSync(
      join(transportDirectory, "rendered-batch.json"),
      "keep existing output"
    )
    const one = renderGenerator(entry, input, 1)
    const four = renderGenerator(entry, input, 4)
    expect(one).toEqual(four)
    expect(
      readFileSync(join(transportDirectory, "render-input.jsonl"), "utf8")
    ).toBe("keep existing input")
    expect(
      readFileSync(join(transportDirectory, "rendered-batch.json"), "utf8")
    ).toBe("keep existing output")
    expect(
      readdirSync(transportDirectory).filter((name) =>
        name.startsWith(".render-")
      )
    ).toEqual([])
    expect(one.map(({ route }) => route)).toEqual([
      "/",
      "/docs/",
      "/docs/third/",
      "/ja/",
      "/ja/docs/",
      "/ja/docs/third/",
    ])
    for (const page of one) {
      const prefix = page.route.startsWith("/ja/") ? "/ja" : ""
      expect(page.html).toContain(`href="${prefix}/docs/third/"`)
      expect(page.html).toContain('class="reference-sequence"')
      expect(page.html).toContain("日本語")
      expect(page.html).not.toContain("site-build-error")
    }
    // Select a different entry in the temporary manifest, leaving production untouched.
    const manifest = join(directory, "seseragi.toml")
    writeFileSync(
      manifest,
      readFileSync(manifest, "utf8").replace(
        'entry = "main"',
        'entry = "direct"'
      )
    )
    checked(["lock", "update", directory], directory)
    expect(
      JSON.parse(checked(["run", directory, "--profile", "release"], directory))
    ).toEqual(one)
    for (const selection of [
      { schema: 2, mode: "plan", offset: 0, count: 0 },
      { schema: 1, mode: "unknown", offset: 0, count: 0 },
      { schema: 1, mode: "plan", offset: 0, count: 1 },
      { schema: 1, mode: "render", offset: -1, count: 1 },
      { schema: 1, mode: "render", offset: 0, count: 0 },
      { schema: 1, mode: "render", offset: 0, count: RENDER_BATCH_SIZE + 1 },
      { schema: 1, mode: "render", offset: 5, count: 2 },
    ]) {
      const result = spawnSync("bun", [entry], {
        cwd: dirname(entry),
        encoding: "utf8",
        input: `${JSON.stringify({ ...selection, input })}\n`,
        timeout: 10_000,
      })
      expect(result.status).not.toBe(0)
      expect(result.stdout).toBe("")
      expect(result.stderr).toContain("RequestFailure")
    }
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
  // Four independently capped CLI steps (lock/build/lock/run) took 186s in
  // current-source Cloud verification. Cover their 4 * 90s budgets plus the
  // transport assertions; preserve each CLI cap and the full-site 420s cap.
}, 390_000)

test("transport removes request/output files after malformed output or a child failure", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-render-transport-"))
  const entry = join(directory, "entry.ts")
  try {
    for (const source of [
      'console.log("not JSON")\n',
      'throw new Error("deliberate generator failure")\n',
      "console.log(JSON.stringify({ schema: 2 }))\n",
    ]) {
      writeFileSync(entry, source)
      expect(() => renderGenerator(entry, input)).toThrow()
      for (const filename of ["render-input.jsonl", "rendered-batch.json"])
        expect(() => readFileSync(join(directory, filename))).toThrow()
      expect(
        readdirSync(directory).filter((name) => name.startsWith(".render-"))
      ).toEqual([])
    }
    const existingOutput = join(directory, "rendered-batch.json")
    const existingInput = join(directory, "render-input.jsonl")
    writeFileSync(existingInput, "preserve an existing input")
    writeFileSync(
      existingOutput,
      "preserve an output not opened by this request"
    )
    expect(() => renderGenerator(entry, input)).toThrow()
    expect(readFileSync(existingOutput, "utf8")).toBe(
      "preserve an output not opened by this request"
    )
    expect(readFileSync(existingInput, "utf8")).toBe(
      "preserve an existing input"
    )
    expect(
      readdirSync(directory).filter((name) => name.startsWith(".render-"))
    ).toEqual([])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})

test("concurrent callers sharing a generator entry use isolated transport files", async () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-render-concurrent-"))
  try {
    const entry = join(directory, "entry.ts")
    writeFileSync(
      entry,
      `const request = JSON.parse(await Bun.stdin.text())
await Bun.sleep(30)
console.log(JSON.stringify({schema: 1, mode: request.mode, offset: request.offset, total: 1,
 routes: request.mode === "plan" ? ["/"] : [],
 pages: request.mode === "plan" ? [] : [{route: "/", html: request.input.token}]}))
`
    )
    const worker = join(directory, "worker.ts")
    writeFileSync(
      worker,
      `import { renderGenerator } from ${JSON.stringify(resolve(import.meta.dir, "../scripts/render-generator.ts"))}
console.log(JSON.stringify(renderGenerator(process.argv[2], { token: process.argv[3] })))
`
    )
    const children = ["first", "second"].map((token) =>
      Bun.spawn(["bun", worker, entry, token], {
        cwd: directory,
        stdout: "pipe",
        stderr: "pipe",
      })
    )
    const results = await Promise.all(
      children.map(async (child) => ({
        code: await child.exited,
        output: await new Response(child.stdout).text(),
        error: await new Response(child.stderr).text(),
      }))
    )
    for (const [index, result] of results.entries()) {
      expect(result.code, result.error).toBe(0)
      expect(JSON.parse(result.output)).toEqual([
        { route: "/", html: ["first", "second"][index] },
      ])
    }
    expect(
      readdirSync(directory).filter((name) => name.startsWith(".render-"))
    ).toEqual([])
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
