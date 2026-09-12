import assert from "node:assert/strict"
import { spawnSync } from "node:child_process"
import { createHash } from "node:crypto"
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, relative, resolve } from "node:path"
import { highlightSeseragi } from "../../playground/src/editor/seseragi-language"
import { playgroundUrlForSource } from "../../playground/src/workspace/source-link"
import { robots, sitemap, validatePublishedSite } from "./quality"

const app = resolve(import.meta.dir, "..")
const root = resolve(app, "../..")
const referenceArtifact = join(
  root,
  "examples/spec/artifacts/stdlib-schema-1/reference/module.json"
)
export const digest = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")
type HighlightPart = { text: string; className: string }
type PreparedBlock = {
  kind: "paragraph" | "heading" | "code" | "reference" | "link" | "terminal"
  text: string
  url: string
  identity: string
  name: string
  namespace: string
  itemKind: string
  typeParameters: string[]
  signature: HighlightPart[]
  description: string
  constraints: string[]
  anchorId: string
  copyText: string
}
const block = (
  values: Pick<PreparedBlock, "kind"> & Partial<Omit<PreparedBlock, "kind">>
): PreparedBlock => ({
  text: "",
  url: "",
  identity: "",
  name: "",
  namespace: "",
  itemKind: "",
  typeParameters: [],
  signature: [],
  description: "",
  constraints: [],
  anchorId: "",
  copyText: "",
  ...values,
})
export function route(value: string): string {
  assert.equal(typeof value, "string")
  assert.ok(
    /^\/(?:[a-z0-9]+(?:-[a-z0-9]+)*\/)*$/.test(value),
    `Unsafe route: ${value}`
  )
  return value
}
export function literal(value: string): string {
  // Seseragi uses scalar escapes, not JSON's UTF-16 \uXXXX escapes.
  return `"${Array.from(value)
    .map((c) => {
      const n = c.codePointAt(0)!
      assert.ok(n < 0xd800 || n > 0xdfff, "Invalid Unicode scalar")
      if (c === '"' || c === "\\") return `\\${c}`
      if (n < 32 || n === 127 || n === 0x2028 || n === 0x2029)
        return `\\u{${n.toString(16)}}`
      return c
    })
    .join("")}"`
}
function text(value: unknown): string {
  assert.equal(typeof value, "string")
  assert.ok((value as string).trim().length > 0, "Empty text")
  return value as string
}
export function prepare(
  input: unknown,
  base: string,
  playgroundOrigin = "https://seseragi.vercel.app/"
) {
  route(base)
  const playground = new URL(playgroundOrigin)
  assert.ok(
    playground.protocol === "https:" &&
      playground.origin + playground.pathname === playgroundOrigin &&
      !playground.username &&
      !playground.password,
    "Expected an HTTPS Playground URL without query or fragment"
  )
  assert.ok(Array.isArray(input) && input.length > 0, "Pages required")
  const ids = new Set<string>()
  const routes = new Set<string>()
  for (const page of input) {
    assert.ok(page && typeof page === "object")
    const id = text(page.id)
    assert.ok(
      /^[a-z0-9-]+$/.test(id) && !ids.has(id),
      "Duplicate/invalid page id"
    )
    ids.add(id)
    route(page.route)
    assert.ok(!routes.has(page.route), "Duplicate route")
    routes.add(page.route)
  }
  assert.ok(routes.has("/"), "Home page required")
  const sources: { path: string; sha256: string }[] = []
  const pages = input.map((page) => {
    assert.ok(Array.isArray(page.blocks), "Blocks required")
    const blocks = page.blocks.map((entry: Record<string, unknown>) => {
      assert.ok(entry && typeof entry === "object")
      switch (entry.kind) {
        case "paragraph":
          return block({ kind: "paragraph", text: text(entry.text) })
        case "heading":
          return block({ kind: "heading", text: text(entry.text) })
        case "terminal":
          return block({ kind: "terminal", text: text(entry.text) })
        case "sample": {
          const path = text(entry.source)
          assert.ok(
            /^examples\/spec\/[a-zA-Z0-9_./-]+\.ssrg$/.test(path) &&
              !path.split("/").includes(".."),
            "Unsafe sample path"
          )
          const absolute = realpathSync(resolve(root, path))
          assert.ok(
            !relative(join(root, "examples/spec"), absolute).startsWith(".."),
            "Sample escapes source root"
          )
          const source = readFileSync(absolute, "utf8")
          sources.push({ path, sha256: digest(source) })
          const signature = highlightSeseragi(source).map(
            ({ text: part, classes }) => ({ text: part, className: classes })
          )
          return block({
            kind: "code",
            url: playgroundUrlForSource(playgroundOrigin, source),
            signature,
            copyText: source,
          })
        }
        case "link": {
          assert.ok(
            (entry.route === undefined) !== (entry.url === undefined),
            "Choose route or URL"
          )
          let url: string
          if (entry.route !== undefined) {
            const target = route(text(entry.route))
            assert.ok(routes.has(target), `Unknown route: ${target}`)
            url = base + target.slice(1)
          } else {
            url = text(entry.url)
            const parsed = new URL(url)
            assert.ok(
              parsed.protocol === "https:" &&
                !parsed.username &&
                !parsed.password,
              "Unsafe URL"
            )
          }
          return block({ kind: "link", text: text(entry.text), url })
        }
        default:
          throw new Error(`Unknown block: ${entry.kind}`)
      }
    })
    return {
      route: base + page.route.slice(1),
      title: text(page.title),
      summary: text(page.summary),
      navigation: true,
      blocks,
    }
  })
  return { pages, sources }
}

export function prepareReference(
  input: unknown,
  base: string,
  provenance = {
    source: "inline",
    sha256: digest(JSON.stringify(input)),
  }
) {
  assert.ok(input && typeof input === "object")
  const surface = input as Record<string, unknown>
  assert.equal(surface.schema, 1)
  assert.equal(surface.kind, "standard-reference")
  const languageVersion = text(surface.languageVersion)
  assert.ok(Array.isArray(surface.modules) && surface.modules.length > 0)
  const routes = new Set<string>()
  let itemCount = 0
  const modulePages = surface.modules.map((rawModule) => {
    assert.ok(rawModule && typeof rawModule === "object")
    const module = rawModule as Record<string, unknown>
    const specifier = text(module.specifier)
    assert.ok(/^std\/[a-z0-9]+(?:[/-][a-z0-9]+)*$/.test(specifier))
    const moduleRoute = route(`/reference/${specifier}/`)
    assert.ok(
      !routes.has(moduleRoute),
      `Duplicate Reference route: ${moduleRoute}`
    )
    routes.add(moduleRoute)
    assert.ok(
      module.availability === "implicit" || module.availability === "available"
    )
    assert.ok(Array.isArray(module.targets) && module.targets.length > 0)
    const targets = module.targets.map(text)
    assert.ok(
      targets.every((target) => ["process", "browser"].includes(target))
    )
    assert.equal(new Set(targets).size, targets.length, "Duplicate target")
    assert.ok(Array.isArray(module.items) && module.items.length > 0)
    const identities = new Set<string>()
    const blocks = [
      block({ kind: "paragraph", text: `Targets: ${targets.join(", ")}` }),
      ...module.items.map((rawItem, itemIndex) => {
        assert.ok(rawItem && typeof rawItem === "object")
        const item = rawItem as Record<string, unknown>
        const identity = text(item.identity)
        const kind = text(item.kind)
        assert.equal(text(item.module), specifier)
        text(item.category)
        assert.ok(
          [
            "alias",
            "constructor",
            "effect-function",
            "function",
            "opaque-struct",
            "opaque-type",
            "operator",
            "struct",
            "trait",
            "type",
            "value",
          ].includes(kind)
        )
        const namespace = text(item.namespace)
        assert.ok(["value", "type", "trait", "operator"].includes(namespace))
        const key = `${identity}\0${kind}`
        assert.ok(!identities.has(key), `Duplicate Reference item: ${identity}`)
        identities.add(key)
        const signature = text(item.signature)
        const description = text(item.description)
        assert.ok(Array.isArray(item.typeParameters))
        const typeParameters = item.typeParameters.map(text)
        assert.ok(Array.isArray(item.constraints))
        const constraints = item.constraints.map(text)
        itemCount++
        const highlighted = highlightSeseragi(signature).map(
          ({ text: part, classes }) => ({ text: part, className: classes })
        )
        return block({
          kind: "reference",
          identity,
          name: text(item.name),
          namespace,
          itemKind: kind,
          typeParameters,
          signature: highlighted,
          description,
          constraints,
          anchorId: `reference-${itemIndex + 1}`,
          copyText: signature,
        })
      }),
    ]
    return {
      route: base + moduleRoute.slice(1),
      title: specifier,
      summary: `${specifier} の compiler-owned API Reference。`,
      navigation: false,
      blocks,
    }
  })
  const landing = {
    route: `${base}reference/`,
    title: "API Reference",
    summary: `Seseragi ${languageVersion} の公開標準ライブラリ。`,
    navigation: true,
    blocks: [
      block({
        kind: "paragraph",
        text: "署名と公開範囲は compiler-owned module metadata から生成されています。",
      }),
      ...modulePages.map((page) =>
        block({ kind: "link", text: page.title, url: page.route })
      ),
    ],
  }
  return {
    pages: [landing, ...modulePages],
    manifest: {
      languageVersion,
      modules: modulePages.length,
      items: itemCount,
      ...provenance,
    },
  }
}

export type SearchEntry = {
  title: string
  summary: string
  route: string
  terms: string
}

export function searchIndex(
  pages: ReturnType<typeof prepare>["pages"]
): SearchEntry[] {
  return pages.flatMap((page) => [
    {
      title: page.title,
      summary: page.summary,
      route: page.route,
      terms: `${page.title} ${page.summary}`,
    },
    ...page.blocks
      .filter((item) => item.kind === "reference")
      .map((item) => ({
        title: `${page.title} · ${item.name}`,
        summary: item.description,
        route: `${page.route}#${item.anchorId}`,
        terms: [
          page.title,
          item.name,
          item.identity,
          item.itemKind,
          item.namespace,
          item.description,
        ].join(" "),
      })),
  ])
}

function searchSource(entries: SearchEntry[]): string {
  const template = readFileSync(join(app, "src/search.ssrg"), "utf8")
  const marker = "__SEARCH_ENTRIES__"
  assert.equal(
    template.split(marker).length,
    2,
    "Search source marker mismatch"
  )
  const values = entries
    .map(
      (entry) =>
        `  SearchEntry { title: ${literal(entry.title)}, summary: ${literal(entry.summary)}, route: ${literal(entry.route)}, terms: ${literal(entry.terms)} }`
    )
    .join(",\n")
  return template.replace(marker, values)
}
function run(command: string[], cwd: string, input?: string): string {
  const [executable, ...args] = command
  assert.ok(executable, "Command required")
  const result = spawnSync(executable, args, {
    cwd,
    input,
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  })
  if (result.error) throw result.error
  assert.equal(result.status, 0, result.stderr)
  return result.stdout
}
export function build(options: {
  out: string
  origin: string
  base: string
  profile?: "development" | "release"
  content?: unknown
  reference?: unknown | false
  playgroundOrigin?: string
}) {
  const origin = new URL(options.origin)
  assert.ok(
    ["https:", "http:"].includes(origin.protocol) &&
      origin.origin === options.origin &&
      !origin.username &&
      !origin.password,
    "Expected origin without path"
  )
  const authored = prepare(
    options.content ??
      JSON.parse(readFileSync(join(app, "content/pages.json"), "utf8")),
    options.base,
    options.playgroundOrigin
  )
  const referenceRaw = readFileSync(referenceArtifact, "utf8")
  const reference =
    options.reference === false
      ? { pages: [], manifest: undefined }
      : prepareReference(
          options.reference ?? JSON.parse(referenceRaw),
          options.base,
          options.reference === undefined
            ? {
                source: relative(root, referenceArtifact),
                sha256: digest(referenceRaw),
              }
            : {
                source: "inline",
                sha256: digest(JSON.stringify(options.reference)),
              }
        )
  const prepared = {
    pages: [...authored.pages, ...reference.pages],
    sources: authored.sources,
  }
  const index = searchIndex(prepared.pages)
  const encodedIndex = `${JSON.stringify(index)}\n`
  assert.equal(
    new Set(prepared.pages.map((page) => page.route)).size,
    prepared.pages.length,
    "Authored and Reference routes overlap"
  )
  const destination = resolve(options.out)
  assert.ok(
    !existsSync(destination),
    `Output exists; select a fresh output directory: ${destination}`
  )
  const temporary = mkdtempSync(join(tmpdir(), "seseragi-docs-"))
  mkdirSync(dirname(destination), { recursive: true })
  const staging = mkdtempSync(join(dirname(destination), ".docs-stage-"))
  try {
    const source = join(temporary, "main.ssrg")
    const renderer = readFileSync(join(app, "src/render.ssrg"), "utf8")
    writeFileSync(
      source,
      `${renderer}\npub effect fn main -> Unit with Console, Stdin fails GeneratorError =\n  generate ${literal(options.origin)} ${literal(options.base)}\n`
    )
    const artifact = join(temporary, "generator")
    run(
      [
        process.env.SESERAGI_BIN ?? "seseragi",
        "build",
        source,
        "--target",
        "process",
        "--profile",
        options.profile ?? "release",
        "--source-map",
        "omit",
        "--out-dir",
        artifact,
      ],
      root
    )
    const manifest = JSON.parse(
      readFileSync(join(artifact, "artifact-manifest.json"), "utf8")
    )
    const searchProject = join(temporary, "search.ssrg")
    writeFileSync(searchProject, searchSource(index))
    const searchArtifact = join(temporary, "search")
    run(
      [
        process.env.SESERAGI_BIN ?? "seseragi",
        "build",
        searchProject,
        "--target",
        "web",
        "--profile",
        "release",
        "--source-map",
        "omit",
        "--out-dir",
        searchArtifact,
      ],
      root
    )
    const searchManifest = JSON.parse(
      readFileSync(join(searchArtifact, "artifact-manifest.json"), "utf8")
    )
    const records = JSON.parse(
      run(
        ["bun", manifest.entry],
        artifact,
        `${JSON.stringify(prepared.pages)}\n`
      )
    )
    assert.ok(
      Array.isArray(records) && records.length === prepared.pages.length,
      "Generator page count mismatch"
    )
    const files: { path: string; bytes: number; sha256: string }[] = []
    const write = (path: string, content: string) => {
      mkdirSync(dirname(join(staging, path)), { recursive: true })
      writeFileSync(join(staging, path), content)
      files.push({
        path,
        bytes: Buffer.byteLength(content),
        sha256: digest(content),
      })
    }
    for (let index = 0; index < records.length; index++) {
      const record = records[index]
      assert.equal(
        record.route,
        prepared.pages[index].route,
        "Generator route mismatch"
      )
      assert.ok(
        typeof record.html === "string" &&
          record.html.startsWith("<!doctype html>"),
        "Invalid document"
      )
      assert.ok(
        !record.html.includes('class="docs-build-error"'),
        `Renderer rejected URL or metadata: ${record.html}`
      )
      const scripts = `<script type="module" src="${options.base}assets/search.js"></script><script type="module" src="${options.base}assets/copy.js"></script>`
      assert.equal(
        record.html.split("</body>").length,
        2,
        "Document body mismatch"
      )
      write(
        `${record.route.slice(options.base.length)}index.html`,
        record.html.replace("</body>", `${scripts}</body>`)
      )
    }
    write("assets/docs.css", readFileSync(join(app, "public/docs.css"), "utf8"))
    write(
      "assets/search.js",
      readFileSync(join(searchArtifact, searchManifest.entry), "utf8")
    )
    write("assets/copy.js", readFileSync(join(app, "public/copy.js"), "utf8"))
    const logo = readFileSync(
      join(root, "assets/brand/public/brand/seseragi-icon.svg"),
      "utf8"
    )
    write("assets/seseragi-icon.svg", logo)
    write("sitemap.xml", sitemap(options.origin, prepared.pages))
    write("robots.txt", robots(options.origin, options.base))
    const clientJavascriptBytes = files
      .filter((file) => file.path.endsWith(".js"))
      .reduce((total, file) => total + file.bytes, 0)
    const quality = validatePublishedSite({
      root: staging,
      origin: options.origin,
      base: options.base,
      pages: prepared.pages,
      files,
      clientJavascriptBytes,
      searchEntries: index.length,
    })
    const siteManifest = {
      schema: 1,
      origin: options.origin,
      base: options.base,
      pages: prepared.pages.map(({ route, title }) => ({ route, title })),
      sources: prepared.sources,
      reference: reference.manifest,
      search: {
        entries: index.length,
        source: "validated authored pages + compiler-owned Reference pages",
        indexBytes: Buffer.byteLength(encodedIndex),
        indexSha256: digest(encodedIndex),
        artifact: searchManifest,
      },
      files,
      clientJavascriptBytes,
      quality,
      generator: manifest,
    }
    writeFileSync(
      join(staging, "site-manifest.json"),
      JSON.stringify(siteManifest, null, 2)
    )
    renameSync(staging, destination)
    return siteManifest
  } finally {
    rmSync(temporary, { recursive: true, force: true })
    rmSync(staging, { recursive: true, force: true })
  }
}
if (import.meta.main) {
  const args = process.argv.slice(2)
  assert.equal(
    args.length,
    3,
    "Usage: bun apps/docs/scripts/build.ts OUTPUT ORIGIN BASE"
  )
  console.log(
    JSON.stringify(
      build({ out: args[0], origin: args[1], base: args[2] }),
      null,
      2
    )
  )
}
