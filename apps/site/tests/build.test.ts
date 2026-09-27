import { expect, test } from "bun:test"
import { mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { buildSite } from "../scripts/build"

function textContent(html: string): string {
  return html
    .replace(/<[^>]+>/gu, "")
    .replaceAll("&gt;", ">")
    .replaceAll("&lt;", "<")
    .replaceAll("&amp;", "&")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
}

test("Seseragi SSG renders the bilingual vertical slice", () => {
  const directory = mkdtempSync(join(tmpdir(), "seseragi-site-test-"))
  const output = join(directory, "site")
  const repeatedOutput = join(directory, "site-repeated")
  try {
    const manifest = buildSite({
      output,
      origin: "https://seseragi.example",
    })
    expect(manifest.pages).toEqual([
      "/",
      "/docs/",
      "/docs/language/",
      "/docs/language/syntax/function-application/",
      "/docs/library/",
      "/docs/library/array/get/",
      "/examples/",
      "/ja/",
      "/ja/docs/",
      "/ja/docs/language/",
      "/ja/docs/language/syntax/function-application/",
      "/ja/docs/library/",
      "/ja/docs/library/array/get/",
      "/ja/examples/",
      "/ja/releases/",
      "/releases/",
    ])
    const home = readFileSync(join(output, "index.html"), "utf8")
    expect(home).toContain('<html lang="en">')
    expect(home).toContain("THE SESERAGI PROGRAMMING LANGUAGE")
    expect(textContent(home)).toContain("pub effect fn main")
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
      join(output, "docs/library/array/get/index.html"),
      "utf8"
    )
    expect(reference).toContain("std/array::get")
    expect(textContent(reference)).toContain("Maybe<A>")
    const repeatedManifest = buildSite({
      output: repeatedOutput,
      origin: "https://seseragi.example",
    })
    expect(repeatedManifest).toEqual(manifest)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
}, 30_000)
