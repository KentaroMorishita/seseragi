import assert from "node:assert/strict"
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

const app = resolve(import.meta.dir, "..")
const root = resolve(app, "../..")
export const digest = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")
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
    const blocks = page.blocks.map((block: Record<string, unknown>) => {
      assert.ok(block && typeof block === "object")
      switch (block.kind) {
        case "paragraph":
          return `Paragraph ${literal(text(block.text))}`
        case "heading":
          return `Heading ${literal(text(block.text))}`
        case "sample": {
          const path = text(block.source)
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
          const parts = highlightSeseragi(source).map(
            ({ text: part, classes }) =>
              `HighlightPart { text: ${literal(part)}, className: ${literal(classes)} }`
          )
          return `Code ([${parts.join(",")}], ${literal(playgroundUrlForSource(playgroundOrigin, source))})`
        }
        case "link": {
          assert.ok(
            (block.route === undefined) !== (block.url === undefined),
            "Choose route or URL"
          )
          let url: string
          if (block.route !== undefined) {
            const target = route(text(block.route))
            assert.ok(routes.has(target), `Unknown route: ${target}`)
            url = base + target.slice(1)
          } else {
            url = text(block.url)
            const parsed = new URL(url)
            assert.ok(
              parsed.protocol === "https:" &&
                !parsed.username &&
                !parsed.password,
              "Unsafe URL"
            )
          }
          return `Link (${literal(text(block.text))}, ${literal(url)})`
        }
        default:
          throw new Error(`Unknown block: ${block.kind}`)
      }
    })
    return {
      route: base + page.route.slice(1),
      title: text(page.title),
      summary: text(page.summary),
      blocks,
    }
  })
  return { pages, sources }
}
function run(command: string[], cwd: string): string {
  const result = Bun.spawnSync(command, { cwd, stdout: "pipe", stderr: "pipe" })
  assert.equal(result.exitCode, 0, result.stderr.toString())
  return result.stdout.toString()
}
export function build(options: {
  out: string
  origin: string
  base: string
  profile?: "development" | "release"
  content?: unknown
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
  const prepared = prepare(
    options.content ??
      JSON.parse(readFileSync(join(app, "content/pages.json"), "utf8")),
    options.base,
    options.playgroundOrigin
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
    const values = prepared.pages.map(
      (p) =>
        `Page { route: ${literal(p.route)}, title: ${literal(p.title)}, summary: ${literal(p.summary)}, blocks: [${p.blocks.join(",")}] }`
    )
    writeFileSync(
      source,
      `${renderer}\nlet pages: Array<Page> = [${values.join(",")} ]\npub effect fn main -> Unit with Console fails ConsoleError =\n  println $ json.encodeString (output pages ${literal(options.origin)} ${literal(options.base)})\n`
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
    const records = JSON.parse(run(["bun", manifest.entry], artifact))
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
      write(`${record.route.slice(options.base.length)}index.html`, record.html)
    }
    write("assets/docs.css", readFileSync(join(app, "public/docs.css"), "utf8"))
    const logo = readFileSync(
      join(root, "assets/brand/public/brand/seseragi-icon.svg"),
      "utf8"
    )
    write("assets/seseragi-icon.svg", logo)
    const siteManifest = {
      schema: 1,
      origin: options.origin,
      base: options.base,
      pages: prepared.pages.map(({ route, title }) => ({ route, title })),
      sources: prepared.sources,
      files,
      clientJavascriptBytes: 0,
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
