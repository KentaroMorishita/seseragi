import { expect, test } from "bun:test"
import { mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { build, prepare, prepareReference, route, searchIndex } from "./build"

const page = {
  id: "home",
  route: "/",
  title: '引用 " & < > 🌊',
  // biome-ignore lint/suspicious/noTemplateCurlyInString: verify literal source interpolation text
  summary: "複数行\n\t${literal}\\",
  blocks: [
    { kind: "paragraph", text: "改行\n<script>alert(1)</script> & 日本語" },
  ],
}

test("reject traversal, encoded/query routes, duplicate identity and broken links", () => {
  for (const input of [
    "//host/",
    "/../",
    "/%2e%2e/",
    "/a?b/",
    "/a#b/",
    "/a\\b/",
    "/a",
    "/assets/../",
  ])
    expect(() => route(input)).toThrow()
  expect(() => prepare([page, page], "/")).toThrow()
  expect(() => prepare([page, { ...page, id: "other" }], "/")).toThrow()
  expect(() =>
    prepare(
      [
        {
          ...page,
          blocks: [{ kind: "link", text: "missing", route: "/missing/" }],
        },
      ],
      "/"
    )
  ).toThrow()
  for (const url of [
    "javascript:alert(1)",
    "https://user:pass@example.com",
    "file:///tmp/a",
  ])
    expect(() =>
      prepare([{ ...page, blocks: [{ kind: "link", text: "bad", url }] }], "/")
    ).toThrow()
  expect(() =>
    prepare(
      [
        {
          ...page,
          blocks: [
            { kind: "sample", source: "examples/spec/../../secret.ssrg" },
          ],
        },
      ],
      "/"
    )
  ).toThrow()
})

test("canonical builds preserve escaped multiline content, base paths and reproducible site files", () => {
  const dir = mkdtempSync(join(tmpdir(), "docs-check-"))
  try {
    const content = [
      page,
      {
        ...page,
        id: "child",
        route: "/child/",
        blocks: [
          { kind: "terminal", text: "run <unsafe>&\nnext" },
          { kind: "link", text: "home", route: "/" },
        ],
      },
    ]
    const manifests = ["development", "release", "release"].map(
      (profile, index) =>
        build({
          out: join(dir, String(index)),
          origin: "https://docs.example.com",
          base: "/guide/",
          profile: profile as "development" | "release",
          content,
          reference: false,
        })
    )
    expect(manifests[0].files).toEqual(manifests[1].files)
    expect(manifests[1].files).toEqual(manifests[2].files)
    expect(manifests[1].generator.provenance.buildId).toEqual(
      manifests[2].generator.provenance.buildId
    )
    const html = readFileSync(join(dir, "0/index.html"), "utf8")
    expect(html).toContain('charset="utf-8"')
    expect(html).toContain('href="/guide/child/"')
    expect(html).toContain('href="https://docs.example.com/guide/"')
    expect(html).toContain('href="/guide/assets/docs.css"')
    expect(html).toContain(
      "改行\n&lt;script&gt;alert(1)&lt;/script&gt; &amp; 日本語"
    )
    expect(html).toContain(
      '<script type="module" src="/guide/assets/search.js"></script>'
    )
    expect(html).toContain(
      '<script type="module" src="/guide/assets/copy.js"></script>'
    )
    expect(manifests[0].clientJavascriptBytes).toBeGreaterThan(0)
    expect(manifests[0].search.entries).toBe(2)
    expect(html).toContain('src="/guide/assets/seseragi-icon.svg"')
    expect(readFileSync(join(dir, "0/child/index.html"), "utf8")).toContain(
      "run &lt;unsafe&gt;&amp;\nnext"
    )
    expect(readFileSync(join(dir, "0/assets/seseragi-icon.svg"), "utf8")).toBe(
      readFileSync(
        resolve(
          import.meta.dir,
          "../../../assets/brand/public/brand/seseragi-icon.svg"
        ),
        "utf8"
      )
    )
    expect(() =>
      build({
        out: join(dir, "0"),
        origin: "https://docs.example.com",
        base: "/",
        content,
        reference: false,
      })
    ).toThrow("Output exists")
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
})

test("compiler-owned Reference metadata generates every module and public item", () => {
  const artifact = JSON.parse(
    readFileSync(
      resolve(
        import.meta.dir,
        "../../../examples/spec/artifacts/stdlib-schema-1/reference/module.json"
      ),
      "utf8"
    )
  )
  const reference = prepareReference(artifact, "/docs/")
  expect(reference.manifest.modules).toBe(62)
  expect(reference.manifest.items).toBe(1387)
  expect(reference.pages).toHaveLength(63)
  expect(new Set(reference.pages.map(({ route }) => route)).size).toBe(63)
  expect(reference.pages.filter(({ navigation }) => navigation)).toHaveLength(1)
  const effect = reference.pages.find(
    ({ route }) => route === "/docs/reference/std/effect/"
  )
  expect(effect).toBeDefined()
  expect(
    effect!.blocks.filter((block) => block.identity === "std/effect::fail")
  ).toHaveLength(1)
  expect(
    effect!.blocks.find((block) => block.identity === "std/effect::fail")
      ?.itemKind
  ).toBe("effect-function")
  expect(
    effect!.blocks.find((block) => block.identity === "std/effect::fail")
      ?.namespace
  ).toBe("value")

  const dir = mkdtempSync(join(tmpdir(), "docs-reference-check-"))
  try {
    const manifest = build({
      out: join(dir, "site"),
      origin: "https://docs.example.com",
      base: "/docs/",
      content: [page],
    })
    expect(manifest.reference).toEqual({
      languageVersion: "0.1.0",
      modules: 62,
      items: 1387,
      source: "examples/spec/artifacts/stdlib-schema-1/reference/module.json",
      sha256: expect.any(String),
    })
    const html = readFileSync(
      join(dir, "site/reference/std/effect/index.html"),
      "utf8"
    )
    expect(html).toContain("std/effect::fail")
    expect(html).toContain("effect-function")
    expect(html).toContain("Creates an Effect that fails with a typed error.")
    expect(html).toContain("tok-keyword")
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
})

test("Reference rejects unsafe modules, unsupported targets and duplicate exports", () => {
  const item = {
    identity: "std/example::value",
    name: "value",
    module: "std/example",
    category: "Example",
    namespace: "value",
    kind: "function",
    typeParameters: [],
    signature: "fn value -> Int",
    description: "A value.",
    constraints: [],
  }
  const artifact = (overrides: Record<string, unknown> = {}) => ({
    schema: 1,
    kind: "standard-reference",
    languageVersion: "0.1.0",
    modules: [
      {
        specifier: "std/example",
        availability: "available",
        targets: ["process", "browser"],
        items: [item],
        ...overrides,
      },
    ],
  })
  expect(() =>
    prepareReference(artifact({ specifier: "std/../escape" }), "/docs/")
  ).toThrow()
  expect(() =>
    prepareReference(artifact({ targets: ["server"] }), "/docs/")
  ).toThrow()
  expect(() =>
    prepareReference(artifact({ targets: ["browser", "browser"] }), "/docs/")
  ).toThrow("Duplicate target")
  expect(() =>
    prepareReference(
      artifact({ items: [{ ...item, module: "std/other" }] }),
      "/docs/"
    )
  ).toThrow()
  expect(() =>
    prepareReference(
      artifact({ items: [{ ...item, namespace: "scope" }] }),
      "/docs/"
    )
  ).toThrow()
  expect(() =>
    prepareReference(artifact({ items: [item, item] }), "/docs/")
  ).toThrow("Duplicate Reference item")
  expect(() =>
    prepareReference(
      {
        ...artifact(),
        modules: [artifact().modules[0], artifact().modules[0]],
      },
      "/docs/"
    )
  ).toThrow("Duplicate Reference route")
})

test("sample blocks use Playground highlighting and preserve the source link", () => {
  const prepared = prepare(
    [
      {
        ...page,
        blocks: [
          {
            kind: "sample",
            source: "examples/spec/lessons/02-values-and-functions.ssrg",
          },
        ],
      },
    ],
    "/"
  )
  expect(
    prepared.pages[0].blocks[0].signature.some(({ className }) =>
      className.includes("tok-keyword")
    )
  ).toBe(true)
  expect(sourceFromPlaygroundUrl(prepared.pages[0].blocks[0].url)).toBe(
    readFileSync(
      resolve(
        import.meta.dir,
        "../../../examples/spec/lessons/02-values-and-functions.ssrg"
      ),
      "utf8"
    )
  )
  expect(prepared.pages[0].blocks[0].copyText).toBe(
    readFileSync(
      resolve(
        import.meta.dir,
        "../../../examples/spec/lessons/02-values-and-functions.ssrg"
      ),
      "utf8"
    )
  )
})

test("search index is derived from page and compiler-owned Reference data", () => {
  const artifact = JSON.parse(
    readFileSync(
      resolve(
        import.meta.dir,
        "../../../examples/spec/artifacts/stdlib-schema-1/reference/module.json"
      ),
      "utf8"
    )
  )
  const authored = prepare([page], "/docs/").pages
  const reference = prepareReference(artifact, "/docs/").pages
  const entries = searchIndex([...authored, ...reference])
  expect(entries).toHaveLength(1451)
  expect(
    entries.find(({ route }) =>
      route.startsWith("/docs/reference/std/effect/#reference-")
    )
  ).toEqual({
    title: expect.stringContaining("std/effect ·"),
    summary: expect.any(String),
    route: expect.stringContaining("/docs/reference/std/effect/#reference-"),
    terms: expect.stringContaining("std/effect::"),
  })
})

test("human-facing baseline keeps every executable block on canonical lessons", () => {
  const content = JSON.parse(
    readFileSync(resolve(import.meta.dir, "../content/pages.json"), "utf8")
  )
  const prepared = prepare(content, "/docs/")
  expect(prepared.pages).toHaveLength(15)
  expect(prepared.sources).toHaveLength(17)
  expect(
    prepared.sources.every(({ path }) =>
      path.startsWith("examples/spec/lessons/")
    )
  ).toBe(true)
  expect(new Set(prepared.sources.map(({ path }) => path)).size).toBe(17)
  expect(
    prepared.pages
      .map(({ route }) => route)
      .filter((route) =>
        [
          "/docs/getting-started/",
          "/docs/language/",
          "/docs/concepts/",
          "/docs/applications/",
        ].includes(route)
      )
  ).toHaveLength(4)
  expect(
    prepared.pages
      .flatMap(({ blocks }) => blocks)
      .filter(({ kind }) => kind === "terminal").length
  ).toBe(5)
})
