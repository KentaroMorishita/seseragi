import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"

const root = resolve(import.meta.dir, "../../..")
setDefaultTimeout(180_000)

type SiteManifest = {
  pages: string[]
  examples: Array<{ id: string; sourcePath: string }>
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
    expect(manifest.pages).toHaveLength(3886)
    for (const route of [
      "/",
      "/docs/",
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
      "/docs/language/expressions/evaluation/",
      "/docs/language/expressions/blocks-and-local-declarations/",
      "/docs/language/expressions/conditionals/",
      "/docs/language/expressions/ranges-and-comprehensions/",
      "/docs/language/expressions/lambdas/",
      "/docs/language/data/algebraic-data-types/",
      "/docs/language/data/structs/",
      "/docs/language/data/newtypes/",
      "/docs/language/data/records/",
      "/docs/language/data/tuples-arrays-and-lists/",
      "/docs/language/data/impls-and-methods/",
      "/docs/language/data/operator-overloads/",
      "/docs/language/patterns/binding-rules/",
      "/docs/language/patterns/irrefutable-patterns/",
      "/docs/language/patterns/match/",
      "/ja/docs/language/patterns/match/",
      "/docs/language/traits/model/",
      "/docs/language/traits/declarations/",
      "/docs/language/traits/instances/",
      "/docs/language/traits/constraints/",
      "/docs/language/traits/method-calls/",
      "/docs/language/traits/coherence/",
      "/docs/language/traits/standard-operators/",
      "/docs/language/traits/laws/",
      "/docs/language/traits/deriving/",
      "/docs/language/traits/methods-versus-traits/",
      "/docs/language/traits/do-notation/",
      "/docs/language/traits/do-desugaring/",
      "/docs/language/traits/do-block-typing/",
      "/ja/docs/language/traits/model/",
      "/ja/docs/language/traits/do-block-typing/",
      "/docs/library/",
      "/docs/library/array/",
      "/docs/library/array/function/get/",
      "/docs/library/array/instance/eq-array-a/",
      "/docs/library/prelude/type/product/",
      "/docs/library/prelude/constructor/product/",
      "/docs/library/prelude/operator/add/",
      "/docs/library/prelude/function/reducible-reduce/",
      "/ja/docs/library/array/function/get/",
      "/ja/docs/language/model/non-features/",
      "/ja/docs/language/syntax/literals/",
      "/ja/releases/",
    ]) {
      expect(manifest.pages).toContain(route)
    }
    for (const route of manifest.pages) {
      expect(route).not.toContain("/docs/get-started/")
    }
    expect(manifest.referenceModules).toHaveLength(63)
    const nonReferenceExamples = new Set(["hello-world", "web-starter-main"])
    const languageExamples = manifest.examples.filter(
      ({ id }) => !nonReferenceExamples.has(id)
    )
    expect(languageExamples.length).toBeGreaterThan(0)
    for (const example of languageExamples) {
      expect(example.sourcePath).toStartWith("apps/site/examples/")
      expect(example.sourcePath).not.toContain("/lessons/")
      expect(example.sourcePath).not.toContain("/fixtures/")
      expect(example.sourcePath).not.toContain("/artifacts/")
    }
    expect(
      manifest.referenceModules
        .flatMap(({ symbols }) => symbols)
        .filter(({ itemKind }) => itemKind === "instance")
    ).toHaveLength(357)
    const home = readFileSync(join(output, "index.html"), "utf8")
    expect(home).toContain('<html lang="en">')
    expect(home).toContain("THE SESERAGI PROGRAMMING LANGUAGE")
    expect(textContent(home)).toContain("pub effect fn main")
    expect(home).toContain('href="https://seseragi.vercel.app/tour/"')
    const documentation = readFileSync(join(output, "docs/index.html"), "utf8")
    expect(textContent(documentation)).toContain("Seseragi Reference")
    expect(textContent(documentation)).toContain("Language Reference")
    expect(textContent(documentation)).toContain("Standard Library")
    expect(textContent(documentation)).toContain("The interactive Tour")
    expect(textContent(documentation)).not.toContain("Get Started")
    expect(documentation).not.toContain("docs-sidebar")
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
    expect(japanese).toContain("詳しい日本語解説は準備中です")
    expect(textContent(japanese)).not.toMatch(/#[0-9]+/u)
    expect(japanese).toContain("Language Reference")
    expect(japanese).toContain('class="breadcrumbs"')
    expect(japanese).toContain('class="reference-sequence"')
    expect(japanese).toContain('href="/ja/docs/language/syntax/method-calls/"')
    const operators = readFileSync(
      join(output, "docs/language/syntax/operator-precedence/index.html"),
      "utf8"
    )
    expect(textContent(operators)).toContain("9: field/method ., index []")
    expect(textContent(operators)).toContain("a < b < c")
    expect(textContent(operators)).toContain("Functions and operators")
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
    expect(textContent(typeSystem)).toContain("TypesUnderstanding types")
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
})
