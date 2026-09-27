import { expect, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")

type SiteManifest = {
  pages: string[]
  referenceModules: Array<{
    symbols: Array<{ itemKind: string }>
  }>
}

function build(output: string): SiteManifest {
  const result = spawnSync(
    "bun",
    ["apps/site/scripts/build.ts", output, "https://seseragi.example"],
    {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        SESERAGI_BIN: resolve(root, "target/debug/seseragi"),
      },
    }
  )
  expect(result.status, result.stderr || result.stdout).toBe(0)
  return JSON.parse(
    readFileSync(join(output, "site-manifest.json"), "utf8")
  ) as SiteManifest
}

function textContent(html: string): string {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&gt;", ">")
    .replaceAll("&lt;", "<")
    .replaceAll("&amp;", "&")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
}

test("Seseragi SSG renders the bilingual site and compiler Reference", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-site-test-"))
  const output = join(directory, "site")
  const repeatedOutput = join(directory, "site-repeated")
  try {
    const manifest = build(output)
    expect(manifest.pages).toHaveLength(3808)
    for (const route of [
      "/",
      "/docs/",
      "/docs/get-started/",
      "/docs/get-started/install/",
      "/docs/get-started/hello-seseragi/",
      "/docs/get-started/create-project/",
      "/docs/get-started/project-layout/",
      "/docs/get-started/run/",
      "/docs/get-started/format-and-check/",
      "/docs/get-started/test/",
      "/docs/get-started/production-build/",
      "/docs/get-started/web-application/",
      "/docs/get-started/next/",
      "/docs/language/model/what-is-seseragi/",
      "/docs/language/model/design-principles/",
      "/docs/language/model/expression-oriented/",
      "/docs/language/model/immutable-by-default/",
      "/docs/language/model/no-hidden-danger/",
      "/docs/language/model/backend-independent-semantics/",
      "/docs/language/model/diagnosable-behavior/",
      "/docs/language/model/visible-costs/",
      "/docs/language/model/readable-density/",
      "/docs/language/model/programs/",
      "/docs/language/model/non-features/",
      "/docs/language/syntax/function-application/",
      "/docs/library/",
      "/docs/library/array/",
      "/docs/library/array/function/get/",
      "/docs/library/array/instance/eq-array-a/",
      "/docs/library/prelude/type/product/",
      "/docs/library/prelude/constructor/product/",
      "/docs/library/prelude/operator/add/",
      "/docs/library/prelude/function/reducible-reduce/",
      "/ja/docs/library/array/function/get/",
      "/ja/docs/get-started/install/",
      "/ja/docs/language/model/non-features/",
      "/ja/releases/",
    ]) {
      expect(manifest.pages).toContain(route)
    }
    expect(manifest.referenceModules).toHaveLength(63)
    expect(
      manifest.referenceModules
        .flatMap(({ symbols }) => symbols)
        .filter(({ itemKind }) => itemKind === "instance")
    ).toHaveLength(357)
    const home = readFileSync(join(output, "index.html"), "utf8")
    expect(home).toContain('<html lang="en">')
    expect(home).toContain("THE SESERAGI PROGRAMMING LANGUAGE")
    expect(textContent(home)).toContain("pub effect fn main")
    const gettingStarted = readFileSync(
      join(output, "docs/get-started/hello-seseragi/index.html"),
      "utf8"
    )
    expect(gettingStarted).toContain("Before you begin")
    expect(textContent(gettingStarted)).toContain(
      'pub effect fn main = println "Hello, Seseragi!"'
    )
    expect(gettingStarted).toContain("guide-next")
    const japaneseGuide = readFileSync(
      join(output, "ja/docs/get-started/install/index.html"),
      "utf8"
    )
    expect(japaneseGuide).toContain("日本語本文は#630で整備中です")
    const languageModel = readFileSync(
      join(output, "docs/language/model/non-features/index.html"),
      "utf8"
    )
    expect(languageModel).toContain(
      "Not present does not mean not implemented yet"
    )
    expect(textContent(languageModel)).toContain("return, break, and continue")
    const language = readFileSync(
      join(output, "docs/language/syntax/function-application/index.html"),
      "utf8"
    )
    expect(language).toContain("Function application")
    expect(textContent(language)).toContain("fn add left")
    expect(language).toContain("Function declarations use fn")
    const japanese = readFileSync(
      join(output, "ja/docs/language/syntax/function-application/index.html"),
      "utf8"
    )
    expect(japanese).toContain('<html lang="ja">')
    expect(japanese).toContain("関数宣言にはfnを使う")
    const reference = readFileSync(
      join(output, "docs/library/array/function/get/index.html"),
      "utf8"
    )
    expect(reference).toContain("std/array::get")
    expect(textContent(reference)).toContain("Maybe<A>")
    expect(textContent(reference)).toContain("Type parametersA")
    expect(textContent(reference)).toContain("Related documentation")
    const instance = readFileSync(
      join(output, "docs/library/array/instance/eq-array-a/index.html"),
      "utf8"
    )
    expect(instance).toContain("std/array::Eq")
    expect(textContent(instance)).toContain(
      "instance<A> Eq<Array<A>> where Eq<A>"
    )
    const repeatedManifest = build(repeatedOutput)
    expect(repeatedManifest).toEqual(manifest)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
}, 120_000)
