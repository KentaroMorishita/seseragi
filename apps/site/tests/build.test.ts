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
    expect(manifest.pages).toHaveLength(3852)
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
      "/docs/language/syntax/source-text/",
      "/docs/language/syntax/literals/",
      "/docs/language/syntax/character-string-escapes/",
      "/docs/language/syntax/layout-and-line-continuation/",
      "/docs/language/syntax/function-application/",
      "/docs/language/syntax/method-calls/",
      "/docs/language/syntax/pipelines-and-low-precedence-application/",
      "/docs/language/syntax/operator-precedence/",
      "/docs/language/syntax/custom-operators/",
      "/docs/language/syntax/reserved-words-and-names/",
      "/docs/language/syntax/optional-record-fields/",
      "/ja/docs/language/syntax/operator-precedence/",
      "/docs/language/types/type-system/",
      "/docs/language/types/built-in-types/",
      "/docs/language/types/type-constructors/",
      "/docs/language/types/annotations-and-inference/",
      "/docs/language/types/polymorphism/",
      "/ja/docs/language/types/type-system/",
      "/docs/language/types/nominal-and-structural-types/",
      "/docs/language/types/optional-record-fields/",
      "/docs/language/types/closed-records/",
      "/docs/language/types/requirement-merge/",
      "/docs/language/types/function-types-and-currying/",
      "/docs/language/types/type-identity-and-coercion/",
      "/docs/language/types/recursive-declarations/",
      "/ja/docs/language/types/requirement-merge/",
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
      "/ja/docs/language/syntax/literals/",
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
    const literals = readFileSync(
      join(output, "docs/language/syntax/literals/index.html"),
      "utf8"
    )
    expect(textContent(literals)).toContain("6.022e23")
    expect(textContent(literals)).toContain("let broken = 1__0")
    expect(literals).toContain("SES-P0203")
    const language = readFileSync(
      join(output, "docs/language/syntax/function-application/index.html"),
      "utf8"
    )
    expect(language).toContain("Function application")
    expect(textContent(language)).toContain("fn add left")
    expect(textContent(language)).toContain("add(1, 2)")
    expect(textContent(language)).toContain("Effect constructs the cold Effect")
    const japanese = readFileSync(
      join(output, "ja/docs/language/syntax/function-application/index.html"),
      "utf8"
    )
    expect(japanese).toContain('<html lang="ja">')
    expect(japanese).toContain("日本語本文は#630で整備中です")
    const operators = readFileSync(
      join(output, "docs/language/syntax/operator-precedence/index.html"),
      "utf8"
    )
    expect(textContent(operators)).toContain("9: field/method ., index []")
    expect(textContent(operators)).toContain("a < b < c")
    expect(textContent(operators)).toContain("Operators and names")
    const customOperators = readFileSync(
      join(output, "docs/language/syntax/custom-operators/index.html"),
      "utf8"
    )
    expect(textContent(customOperators)).toContain("operator infixr 4 <+>")
    expect(textContent(customOperators)).toContain("operator infixl 4 ^")
    const optionalFields = readFileSync(
      join(output, "docs/language/syntax/optional-record-fields/index.html"),
      "utf8"
    )
    expect(textContent(optionalFields)).toContain("id?: String")
    expect(textContent(optionalFields)).toContain(
      "required field of type Maybe"
    )
    const typeSystem = readFileSync(
      join(output, "docs/language/types/type-system/index.html"),
      "utf8"
    )
    expect(textContent(typeSystem)).toContain("implicit Any or Unknown")
    expect(textContent(typeSystem)).toContain("Type systemFoundations")
    const builtInTypes = readFileSync(
      join(output, "docs/language/types/built-in-types/index.html"),
      "utf8"
    )
    expect(textContent(builtInTypes)).toContain("-9007199254740991")
    expect(textContent(builtInTypes)).toContain("Unit / Never")
    const constructors = readFileSync(
      join(output, "docs/language/types/type-constructors/index.html"),
      "utf8"
    )
    expect(textContent(constructors)).toContain("Array<Array<Int>>")
    expect(textContent(constructors)).toContain("Maybe<Int, String>")
    const polymorphism = readFileSync(
      join(output, "docs/language/types/polymorphism/index.html"),
      "utf8"
    )
    expect(textContent(polymorphism)).toContain("forall A. A -> A")
    expect(textContent(polymorphism)).toContain("identity True")
    const recordTypes = readFileSync(
      join(
        output,
        "docs/language/types/nominal-and-structural-types/index.html"
      ),
      "utf8"
    )
    expect(textContent(recordTypes)).toContain("width subtyping")
    expect(textContent(recordTypes)).toContain("Player struct")
    const requirementMerge = readFileSync(
      join(output, "docs/language/types/requirement-merge/index.html"),
      "utf8"
    )
    expect(textContent(requirementMerge)).toContain("SES-E0001")
    expect(textContent(requirementMerge)).toContain("SES-T0501")
    const recursion = readFileSync(
      join(output, "docs/language/types/recursive-declarations/index.html"),
      "utf8"
    )
    expect(textContent(recursion)).toContain("monomorphic")
    expect(textContent(recursion)).toContain("forward reference outside rec")
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
