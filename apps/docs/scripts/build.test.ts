import { expect, test } from "bun:test"
import { mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { sourceFromPlaygroundUrl } from "../../playground/src/workspace/source-link"
import { build, prepare, route } from "./build"

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
        blocks: [{ kind: "link", text: "home", route: "/" }],
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
    expect(html).not.toContain("<script")
    expect(html).toContain('src="/guide/assets/seseragi-icon.svg"')
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
      })
    ).toThrow("Output exists")
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
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
  expect(prepared.pages[0].blocks[0]).toContain("tok-keyword")
  const encoded = prepared.pages[0].blocks[0].match(/https[^"]+/)?.[0]
  expect(encoded).toBeDefined()
  expect(sourceFromPlaygroundUrl(encoded!)).toBe(
    readFileSync(
      resolve(
        import.meta.dir,
        "../../../examples/spec/lessons/02-values-and-functions.ssrg"
      ),
      "utf8"
    )
  )
})
