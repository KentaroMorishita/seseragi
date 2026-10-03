import { expect, setDefaultTimeout, test } from "bun:test"
import { spawnSync } from "node:child_process"
import { cpSync, existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import {
  assertAuthoredLibraryTitles,
  authoredLibraryPages,
} from "./library-titles"
import { assertReferenceLinkTitles, pageTitle } from "./reference-titles"

const root = resolve(import.meta.dir, "../../..")
// This test builds all 3976 routes twice to verify deterministic output.
// Release-profile complete-metadata batches measured about 1.6s each (125 batches).
// Include compilation and retain a finite deadline for each complete build.
const buildTimeout = 420_000
setDefaultTimeout(2 * buildTimeout + 60_000)

type SiteManifest = {
  pages: string[]
  referenceCoverage: Array<{
    area: string
    planned: number
    published: number
    missing: string[]
  }>
  examples: Array<{ id: string; sourcePath: string }>
  referenceModules: Array<{
    symbols: Array<{ itemKind: string }>
  }>
}

function build(output: string): SiteManifest {
  const started = performance.now()
  const result = spawnSync(
    "bun",
    ["apps/site/scripts/build.ts", output, "https://seseragi.example"],
    {
      cwd: root,
      encoding: "utf8",
      timeout: buildTimeout,
      env: {
        ...process.env,
        NODE_ENV: "production",
        SESERAGI_BIN: resolve(
          root,
          process.env.SESERAGI_BIN ?? "target/debug/seseragi"
        ),
      },
    }
  )
  const elapsed = Math.round(performance.now() - started)
  const detail = `SSG build (${output}) after ${elapsed}ms: ${
    result.error?.message ?? `status=${result.status}, signal=${result.signal}`
  }\n${result.stderr || result.stdout}`
  expect(result.error, detail).toBeUndefined()
  expect(result.signal, detail).toBeNull()
  expect(result.status, detail).toBe(0)
  console.info(`SSG build completed in ${elapsed}ms`)
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
    expect(manifest.pages).toHaveLength(3976)
    const retainedOutput = process.env.SESERAGI_SITE_TEST_RETAIN_OUTPUT
    if (retainedOutput) {
      const destination = resolve(retainedOutput)
      expect(
        existsSync(destination),
        "Retained output must use a fresh path"
      ).toBe(false)
      // Only the complete generated HTML/assets tree is copied. Default CI
      // cleanup and every assertion remain unchanged when this is unset.
      cpSync(output, destination, {
        recursive: true,
        force: false,
        errorOnExist: true,
      })
      expect(
        readFileSync(join(destination, "site-manifest.json"), "utf8")
      ).toBe(readFileSync(join(output, "site-manifest.json"), "utf8"))
      console.info(
        `Retained complete generated site: ${destination} (${manifest.pages.length} routes)`
      )
    }
    const languagePages = new Map(
      manifest.pages
        .filter((route) => /^(?:\/ja)?\/docs\/language\//u.test(route))
        .map((route) => [
          route,
          readFileSync(join(output, route.slice(1), "index.html"), "utf8"),
        ])
    )
    const titles = new Map(
      [...languagePages].map(([route, html]) => [route, pageTitle(html)])
    )
    let checkedReferenceLinks = 0
    for (const [route, html] of languagePages) {
      checkedReferenceLinks += assertReferenceLinkTitles(html, titles, route)
      expect(html, route).not.toContain('id="specification"')
      if (route.startsWith("/ja/"))
        expect(textContent(html), route).not.toContain("式中心")
    }
    expect(titles.get("/ja/docs/language/model/expression-oriented/")).toBe(
      "式指向"
    )
    expect(checkedReferenceLinks).toBeGreaterThan(500)
    let checkedSequences = 0
    for (const [route, html] of languagePages) {
      const topic = html.match(
        /<a\b(?=[^>]*\bclass="reference-topic-link")[^>]*\bhref="([^"]+)"/u
      )?.[1]
      if (!topic) continue
      const sequence = html.match(
        /<nav\b[^>]*class="reference-sequence"[^>]*>([\s\S]*?)<\/nav>/u
      )?.[1]
      expect(sequence, route).toBeDefined()
      for (const [, destination] of (sequence ?? "").matchAll(
        /\bhref="([^"]+)"/gu
      )) {
        const target = languagePages.get(destination)
        expect(target, `${route} -> ${destination}`).toBeDefined()
        const targetTopic = target?.match(
          /<a\b(?=[^>]*\bclass="reference-topic-link")[^>]*\bhref="([^"]+)"/u
        )?.[1]
        expect(targetTopic, `${route}: sequence crosses topics`).toBe(topic)
        checkedSequences++
      }
    }
    expect(checkedSequences).toBeGreaterThan(100)
    console.info(`Verified ${checkedReferenceLinks} bilingual page-name links`)
    const completeTitles = new Map(
      manifest.pages.map((route) => [
        route,
        pageTitle(
          readFileSync(join(output, route.slice(1), "index.html"), "utf8")
        ),
      ])
    )
    const authoredLibrary = authoredLibraryPages()
    let checkedLibraryBodies = 0
    let checkedLibraryLinks = 0
    for (const entry of authoredLibrary)
      for (const prefix of ["", "/ja"]) {
        const route = prefix + entry.route
        const html = readFileSync(
          join(output, route.slice(1), "index.html"),
          "utf8"
        )
        checkedLibraryLinks += assertAuthoredLibraryTitles(
          html,
          completeTitles,
          { ...entry, route }
        )
        checkedLibraryBodies++
      }
    expect(checkedLibraryBodies).toBeGreaterThanOrEqual(200)
    expect(checkedLibraryLinks).toBeGreaterThanOrEqual(283)
    console.info(
      `Verified ${checkedLibraryLinks} authored Library page-name links across ${checkedLibraryBodies} localized bodies`
    )

    const mobileArticle = readFileSync(
      join(output, "ja/docs/language/syntax/function-application/index.html"),
      "utf8"
    )
    expect(mobileArticle).toContain('id="mobile-reference-drawer"')
    expect(mobileArticle).toContain("目次を閉じる")
    expect(mobileArticle).toContain('class="language-trigger"')
    expect(mobileArticle).toContain('aria-label="表示言語を選ぶ"')
    expect(mobileArticle).toContain('src="/assets/language.svg"')
    expect(mobileArticle).toContain("/assets/language-menu.js")
    expect(mobileArticle).toContain(
      '<script type="module" src="/assets/mobile-navigation.js"></script>'
    )
    expect(readFileSync(join(output, "index.html"), "utf8")).not.toContain(
      '<script type="module" src="/assets/mobile-navigation.js"></script>'
    )
    expect(
      manifest.referenceCoverage.find(({ area }) => area === "language")
    ).toEqual({
      area: "language",
      planned: 106,
      published: 106,
      missing: [],
    })
    expect(
      manifest.referenceCoverage.find(({ area }) => area === "interop")?.missing
        .length
    ).toBeGreaterThan(0)
    const grammar = readFileSync(
      join(output, "docs/language/grammar/index.html"),
      "utf8"
    )
    const normativeGrammar = readFileSync(
      join(root, "docs/spec/grammar.md"),
      "utf8"
    )
      .split("```ebnf\n")[1]
      .split("```")[0]
      .trimEnd()
    expect(textContent(grammar)).toContain(normativeGrammar)
    const japaneseApi = readFileSync(
      join(output, "ja/docs/library/array/function/get/index.html"),
      "utf8"
    )
    expect(japaneseApi).not.toContain("APIの詳細説明（英語原文）")
    expect(textContent(japaneseApi)).toContain(
      "getは、0から数える位置にある配列の要素をMaybeで返します。"
    )
    expect(textContent(japaneseApi)).toContain("この宣言の読み方")
    expect(textContent(japaneseApi)).toContain("引数は2個です")
    expect(textContent(japaneseApi)).not.toMatch(/準備中|整備中/u)
    expect(japaneseApi).toContain("型パラメーター")
    expect(japaneseApi).toContain('href="/ja/docs/library/array/"')
    for (const route of [
      "/",
      "/docs/",
      "/docs/first-run/",
      "/ja/docs/first-run/",
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
      "/docs/language/effects/pure-expressions/",
      "/docs/language/effects/maybe/",
      "/docs/language/effects/either/",
      "/docs/language/effects/effect-type/",
      "/docs/language/effects/effect-functions-contract-form/",
      "/docs/language/effects/effect-functions-inferred-form/",
      "/docs/language/effects/effectful-for/",
      "/docs/language/effects/environment-requirements/",
      "/docs/language/effects/error-channels/",
      "/docs/language/effects/task/",
      "/docs/language/effects/sequential-and-parallel/",
      "/docs/language/effects/runtime-boundaries/",
      "/docs/language/effects/defects/",
      "/ja/docs/language/effects/pure-expressions/",
      "/ja/docs/language/effects/defects/",
      "/docs/language/effects/cancellation-and-resources/",
      "/ja/docs/language/effects/cancellation-and-resources/",
      "/docs/language/effects/scheduler-fairness/",
      "/ja/docs/language/effects/scheduler-fairness/",
      "/docs/language/effects/fiber-supervision/",
      "/ja/docs/language/effects/fiber-supervision/",
      "/docs/language/effects/signals-and-transactions/",
      "/ja/docs/language/effects/signals-and-transactions/",
      "/docs/language/effects/derived-signals/",
      "/ja/docs/language/effects/derived-signals/",
      "/docs/language/effects/signal-operators/",
      "/ja/docs/language/effects/signal-operators/",
      "/docs/language/effects/subscriptions-and-lifetime/",
      "/ja/docs/language/effects/subscriptions-and-lifetime/",
      "/docs/language/effects/exceptions-and-algebraic-effects/",
      "/ja/docs/language/effects/exceptions-and-algebraic-effects/",
      "/docs/library/",
      "/docs/language/modules/identity/",
      "/ja/docs/language/modules/identity/",
      "/docs/language/modules/packages/",
      "/ja/docs/language/modules/packages/",
      "/docs/language/modules/top-level/",
      "/ja/docs/language/modules/top-level/",
      "/docs/language/modules/visibility/",
      "/ja/docs/language/modules/visibility/",
      "/docs/language/modules/imports/",
      "/ja/docs/language/modules/imports/",
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
    const moduleIdentity = readFileSync(
      join(output, "docs/language/modules/identity/index.html"),
      "utf8"
    )
    expect(textContent(moduleIdentity)).toContain(
      "A type name alone does not identify its declaration"
    )
    expect(textContent(moduleIdentity)).toContain('from "./domain.ssrg"')
    expect(moduleIdentity).toContain('href="/docs/language/modules/packages/"')
    expect(moduleIdentity).not.toContain('href=""')
    for (const slug of [
      "identity",
      "packages",
      "top-level",
      "visibility",
      "imports",
    ]) {
      const translated = readFileSync(
        join(output, `ja/docs/language/modules/${slug}/index.html`),
        "utf8"
      )
      expect(translated).toContain('<html lang="ja">')
      expect(textContent(translated)).not.toMatch(/準備中|整備中|#[0-9]+/u)
      expect(translated).toContain('class="reference-sequence"')
    }
    const visibility = readFileSync(
      join(output, "docs/language/modules/visibility/index.html"),
      "utf8"
    )
    expect(textContent(visibility)).toContain("pub opaque struct Counter")
    expect(textContent(visibility)).toContain("PrivateExport")
    expect(textContent(visibility)).toContain(
      "record type has no field `value`"
    )
    expect(textContent(visibility)).toContain(
      'import { punctuate } from "./greeting"'
    )
    expect(home).toContain('<html lang="en">')
    expect(home).toContain("THE SESERAGI PROGRAMMING LANGUAGE")
    expect(textContent(home)).toContain("pub effect fn main")
    expect(home).toContain('href="https://seseragi.vercel.app/tour/"')
    const documentation = readFileSync(join(output, "docs/index.html"), "utf8")
    expect(textContent(documentation)).toContain("Seseragi Documentation")
    expect(textContent(documentation)).toContain("Language Reference")
    expect(textContent(documentation)).toContain("Standard Library")
    expect(textContent(documentation)).toContain("Functions and operators")
    expect(textContent(documentation)).toContain("Collections")
    expect(textContent(documentation)).toContain(
      "The optional Tour lets you practise by running code"
    )
    expect(textContent(documentation)).not.toContain("Get Started")
    expect(documentation).not.toContain("docs-sidebar")
    expect(documentation).toContain('href="/docs/first-run/"')
    expect(documentation.match(/reference-area-card/g)).toHaveLength(2)
    const languageModel = readFileSync(
      join(output, "docs/language/model/non-features/index.html"),
      "utf8"
    )
    expect(languageModel).toContain(
      "An absent construct is not a missing application capability"
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
    expect(textContent(language)).toContain("does not start the described I/O")
    const japanese = readFileSync(
      join(output, "ja/docs/language/syntax/function-application/index.html"),
      "utf8"
    )
    expect(japanese).toContain('<html lang="ja">')
    expect(textContent(japanese)).toContain(
      "関数名の後に、引数を空白で区切って書きます"
    )
    expect(textContent(japanese)).not.toMatch(/準備中|整備中|#[0-9]+/u)
    expect(textContent(japanese)).not.toMatch(/#[0-9]+/u)
    expect(japanese).toContain("言語リファレンス")
    expect(japanese).toContain('class="breadcrumbs"')
    expect(japanese).toContain('class="reference-sequence"')
    expect(japanese).toContain('href="/ja/docs/language/syntax/method-calls/"')
    const operators = readFileSync(
      join(output, "docs/language/syntax/operator-precedence/index.html"),
      "utf8"
    )
    expect(textContent(operators)).toContain(
      "9. Field and method access with . and indexing with []."
    )
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
      "nickname: Maybe<String> is different: the field is required, even when its stored value is Nothing."
    )
    const typeSystem = readFileSync(
      join(output, "docs/language/types/type-system/index.html"),
      "utf8"
    )
    expect(textContent(typeSystem)).toContain("prevent execution")
    expect(textContent(typeSystem)).toContain("TypesUnderstanding types")
    for (const [route, explanation] of [
      ["types/kinds", "Maybeだけでは、値の型はまだ決まらない"],
      ["traits/do-notation", "Nothingなら残りの行を実行しません"],
      ["effects/effect-type", "Effectを作っただけでは、処理は開始されない"],
      ["effects/environment-requirements", "必要なサービスをEffectの型に書く"],
      ["effects/task", "Task<A>はEffect<{}, Never, A>の別名です"],
      [
        "effects/signals-and-transactions",
        "Signalの現在値は、Effectの中で読み書きする",
      ],
      ["types/type-system", "Intを受け取る関数にStringを渡す"],
      [
        "types/generic-functions",
        "指定したStringと引数のIntが矛盾するためエラーになります",
      ],
      ["types/generic-adts", "失敗値の型Eはこの値だけでは分かりません"],
      ["types/generic-structs", "更新では、フィールドの型を変更できない"],
      ["types/generic-aliases", "型の別名は、値を作るコンストラクターではない"],
      [
        "types/newtypes",
        "UserId 42とOrderId 42は同じ整数を持ちますが、型は別です",
      ],
      ["types/variance", "要素の型が違う配列は、そのまま代入できない"],
      ["types/requirement-merge", "コンソールを使う計算と時計を使う計算"],
      [
        "types/let-polymorphism-and-rank",
        "現在の実装にはこの制約が残っています",
      ],
      ["modules/imports", "非公開の名前は、名前を知っていてもimportできません"],
      [
        "modules/identity",
        "importの綴りや別名を変えても、関数を複製したり、新しいモジュールを作ったりはしません",
      ],
      [
        "modules/re-exports",
        "pub importで別ファイルの公開宣言を再公開すると、利用側が指定する窓口をまとめられます",
      ],
      ["patterns/match", "ガード条件が偽ならその分岐を使いません"],
      [
        "expressions/conditionals",
        "0や空文字列を偽として扱うような変換はありません",
      ],
    ]) {
      const prose = textContent(
        readFileSync(
          join(output, `ja/docs/language/${route}/index.html`),
          "utf8"
        )
      )
      expect(prose).toContain(explanation)
      expect(prose).not.toMatch(
        /型scheme|namespace alias|canonical identity|準備中|整備中|#[0-9]+/u
      )
    }
    for (const route of ["types/variance", "modules/re-exports"]) {
      const prose = textContent(
        readFileSync(join(output, `docs/language/${route}/index.html`), "utf8")
      )
      expect(prose).not.toContain("Current compiler limitation")
    }
    expect(
      manifest.examples.some(({ id }) => id === "types-variance-invalid")
    ).toBe(true)
    const builtInTypes = readFileSync(
      join(output, "docs/language/types/built-in-types/index.html"),
      "utf8"
    )
    expect(textContent(builtInTypes)).toContain("-9007199254740991")
    expect(textContent(builtInTypes)).toContain(
      "Unit represents normal completion without a payload"
    )
    expect(textContent(builtInTypes)).toContain("Never has no values")
    expect(textContent(builtInTypes)).toContain(
      "fromInt 3 returns the Float 3.0"
    )
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
    expect(textContent(recordTypes)).toContain("only { name: String }")
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
    expect(textContent(instance)).toContain("Compare two values for equality")
    for (const locale of ["", "ja/"]) {
      const principle = readFileSync(
        join(
          output,
          `${locale}docs/language/model/immutable-by-default/index.html`
        ),
        "utf8"
      )
      expect(principle).toContain('id="reading-the-example"')
      expect(principle.match(/seseragi-highlight/g)).toHaveLength(2)
      expect(textContent(principle)).toContain("SES-P0001")
      expect(textContent(principle)).toContain("original.score = 11")
      expect(textContent(principle)).toContain(
        "let updated = { ...original, score: 11 }"
      )
      expect(textContent(principle)).toContain("10 -> 11")
    }
    const repeatedManifest = build(repeatedOutput)
    expect(repeatedManifest).toEqual(manifest)
  } finally {
    rmSync(directory, { recursive: true, force: true })
  }
})
