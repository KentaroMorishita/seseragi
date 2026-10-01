# Core reference reader-review ledger

Scope: 106 mapped core-language routes, each reviewed in English and Japanese.
This is **reader acceptance**, not route, translation-module or compilation coverage.
All entries were reopened after the corpus-wide reader feedback. Checked entries
record a bounded self-review, not user sign-off or corpus-wide acceptance.

Use [reader-contract.md](reader-contract.md) for the review questions. Record a
review result and evidence per route before checking it. A rewrite alone is not
acceptance. The initial rewrite covers method calls, pipelines, records, structs,
collections and three trait pages; these eight have now received a separate rendered
reading review at 320/390/1280px in both locales. Chromium and WebKit verified
their example source, output, paragraph pairing and topic-boundary navigation.
The Japanese records and trait explanations were also inspected as rendered
text and screenshots. `check:site` passed; its 424 article/locale/viewport cases
protect rendering only and do not accept the other 98 pages. Existing model/type/Effect rewrite history is also not a blanket
acceptance for those pages.

## #697 reader-contract revision (2026-10-01)

The revised contract targets everyday TypeScript developers using ordinary
`type` / `interface`, functions, arrays, objects and `try` / `catch`; it does not
assume ADT, `match`, Rust/Haskell or advanced type theory. The design was checked
against main `ab42da841d76d322584fac096c249d586ae539ec` and the actual page owners,
not just the illustrative sitemap. See the
[entrance and pilot map](site-architecture.md#12-entrance-and-pilot-page-map).

The eight checked routes below retain their **historical bounded self-review**
status. They are not eight acceptances under the revised TS-reader contract and
are not actual first-time-reader sign-off. #697 changes design documents only:
no page is newly accepted, no checkbox changes, and the other 98 core routes
remain unaccepted. The earlier successful checks described below are historical
results, not a claim that a current build/browser check ran in this revision.

The #701 pilot revisits these six existing routes (all below `/docs/language/`):

- `model/immutable-by-default/`
- `types/built-in-types/`
- `types/annotations-and-inference/`
- `syntax/function-application/`
- `expressions/blocks-and-local-declarations/`
- `types/function-types-and-currying/`

Record each changed route's source revision, change, code/output/diagnostic
checks, rendered EN/JA desktop/mobile/navigation review, and unresolved reader
questions alongside its ledger entry. Distinguish not run or blocked from a
passing check. #699/#700 also record first-run and entrance evidence here under
their own routes, without adding them to the 106 core-reference total.

#702 records who reviewed and their prior knowledge, whether the review was
self-review/simulated or an actual first-time reader, and what the reader could
explain or execute without help: purpose/appeal, how to try it, example input,
important lines, result and where to look next. Note the exact stopping point or
misunderstanding, link the responsible page/issue, and repeat the same question
after a fix. Actual-reader confirmation still missing must remain explicit.
Only then decide the next data-alternatives, matching and failure scope; do not
bulk-accept the corpus from a rewrite, output count or self-review.

Design-only verification: `check-content-map.ts` passed with 351 unique routes;
coverage and reference-title tests passed (6 tests). Source checks confirmed the
six pilot identities, unchanged 106 ledger entries/eight checks, unchanged
normative map obligations and valid relative document links. `git diff --check`
passed. No site build, rendered/browser review or first-time-reader acceptance
was performed for this Markdown-only revision.

## Page-by-page review

### Corpus-wide explanation pass (2026-10-01)

This pass covers every currently implemented page, not only the original eight:
105 conceptual language articles now own bilingual question/example/result
explanations; the grammar appendix explains its notation locally. Original
rule and diagnostic sections remain available after those explanations.
Japanese prose, code captions and related-page names were edited together.
The 63 library modules have localized purposes and usage guidance; all 1,812
symbol/instance pages have Japanese primary descriptions and signature
walkthroughs. Canonical descriptions and signatures remain the source of truth.

Structural completeness is tested separately from reader acceptance. Native
execution checks cover 31 concrete result claims in addition to the reader
examples. Unknown upstream API descriptions fail translation coverage rather
than producing an untranslated primary description. Per-route acceptance below
is deliberately not bulk-checked by this authoring pass: the full walkthrough
and API-specific examples still require page-by-page reader review.

Explanation text is owned by each page's `en.ssrg` and `ja.ssrg`. Its summary
is reused by the article copy; `guide.ssrg` only selects the optional example
and pairs locale copies. Absence is `WithoutExample`, not an empty-string identity.

### Japanese-first editorial pass with yomiyasu (2026-10-01)

The next editorial pass uses yomiyasu's `tech` guidance. All 106 existing
language pages were edited in Japanese and English, including the original rule,
evaluation and mistake sections, not just their new introductions. Six paired
entrance pages were also revised. Explanations now identify the relevant value,
operation and result instead of relying on compiler-internal vocabulary.

Technical review corrected the exception for inferred `effect fn` signatures,
the difference between `bracket` and scope-registered release, and the conditions
for automatic subscription cleanup. The normative specification, runnable
examples and API declarations were preserved.

All 376 distinct canonical API descriptions now have explicit Japanese/English
reader-copy pairs. Original descriptions remain unchanged in the compiler
metadata; 49 pairs received substantive editorial corrections. Instance pages
explain the operations provided by each trait and any conditional/structural
requirements. This shared copy reaches all 1,812 generated API/instance pages,
but is not 1,812 individually authored examples or reader acceptances. The 63
existing module introductions were read and retained where accurate.

`check-prose.py` extracts rendered Japanese paragraphs and list items, excluding
code blocks and marking inline code. Its yomiyasu report is advisory. Repeated
endings, lists of genuine alternatives and negative constraints are reviewed in
context; a warning-free score is not a completion condition. Lint runs in the
foreground and does not start a browser or persistent Python process.

The rendered reading pass also found a duplicated walkthrough on seven design
principle pages. Each now explains its canonical example once; the rules,
constraints and verified output remain. Constructor reading now treats `A`, not
`Just: A`, as the argument type in `Just: A -> Maybe<A>`.

Verification: `bun run check:site` passed, including deterministic generation of
3,974 routes, executable examples, canonical metadata, 12 Bun tests and 424
language article/locale/viewport checks. Additional instance-purpose coverage
passed in the focused explanation tests. Chromium and WebKit also passed 72
paired route/locale/viewport combinations at 390/1280px, including drawer
close/Escape and same-identity locale switching. Japanese desktop and mobile
screenshots and the resource/immutability explanations were read as rendered.

The final foreground yomiyasu report covers all 1,987 Japanese routes. Its
remaining findings include repeated sentence endings, real negative constraints
and lists of genuine alternatives. The sole colon-format warning is the
canonical `std/prelude:::` operator identity, not prose punctuation; retain it.
These advisory findings do not override the specification or count as reader
acceptance. The review browser, its Python daemon and the local Bun server were
stopped and their processes/listening ports checked after use.

Page-by-page acceptance remains separate below. Neither this editorial pass nor
the generated route count bulk-checks the remaining entries. API-specific
examples, all outstanding reader walkthroughs and the 245 unimplemented non-core
articles remain work to do.

- [ ] `/docs/language/model/what-is-seseragi/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/design-principles/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/expression-oriented/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/immutable-by-default/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/no-hidden-danger/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/backend-independent-semantics/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/diagnosable-behavior/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/visible-costs/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/readable-density/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/programs/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/model/non-features/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/source-text/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/literals/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/character-string-escapes/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/layout-and-line-continuation/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/function-application/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [x] `/docs/language/syntax/method-calls/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [x] `/docs/language/syntax/pipelines-and-low-precedence-application/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/operator-precedence/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/custom-operators/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/reserved-words-and-names/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/syntax/optional-record-fields/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/type-system/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/built-in-types/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/type-constructors/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/annotations-and-inference/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/polymorphism/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/nominal-and-structural-types/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/optional-record-fields/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/closed-records/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/requirement-merge/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/function-types-and-currying/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/type-identity-and-coercion/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/recursive-declarations/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/kinds/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/type-parameter-scope/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/generic-functions/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/let-polymorphism-and-rank/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/generic-adts/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/generic-structs/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/generic-impls-and-methods/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/generic-aliases/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/newtypes/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/variance/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/types/erasure-and-runtime-representation/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/expressions/evaluation/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/expressions/blocks-and-local-declarations/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/patterns/binding-rules/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/patterns/irrefutable-patterns/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/expressions/conditionals/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/data/algebraic-data-types/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [x] `/docs/language/data/structs/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/data/newtypes/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [x] `/docs/language/data/records/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [x] `/docs/language/data/tuples-arrays-and-lists/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/expressions/ranges-and-comprehensions/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/patterns/match/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/expressions/lambdas/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/data/impls-and-methods/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/data/operator-overloads/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [x] `/docs/language/traits/model/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [x] `/docs/language/traits/declarations/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [x] `/docs/language/traits/instances/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/constraints/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/method-calls/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/coherence/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/standard-operators/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/laws/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/deriving/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/methods-versus-traits/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/do-notation/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/do-desugaring/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/traits/do-block-typing/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/pure-expressions/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/maybe/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/either/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/effect-type/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/effect-functions-contract-form/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/effect-functions-inferred-form/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/effectful-for/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/environment-requirements/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/error-channels/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/task/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/sequential-and-parallel/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/runtime-boundaries/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/defects/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/cancellation-and-resources/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/scheduler-fairness/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/fiber-supervision/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/signals-and-transactions/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/derived-signals/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/signal-operators/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/subscriptions-and-lifetime/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/effects/exceptions-and-algebraic-effects/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/identity/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/packages/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/top-level/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/visibility/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/imports/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/specifier-resolution/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/re-exports/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/namespaces-and-resolution/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/dependency-graphs/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/initialization/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/modules/entry-points/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation
- [ ] `/docs/language/grammar/` — English explanation / Japanese explanation / prerequisites / example walkthrough / navigation

## #702 entrance and values/functions calibration (2026-10-01)

### Reviewer and limits

An independent review agent read the supplied production-component HTML as text,
after the page authors' own implementation checks. The assumed reader uses
ordinary TypeScript `type` / `interface`, functions, arrays, objects and
`try` / `catch`. No Rust, Haskell, ADT, `match` or type-theory knowledge was
assumed. Explanatory gaps were judged from the presented page bodies rather than
filled from the compiler, specification or source implementation.

This is a **simulated reader review**, not an actual first-time person's test.
The agent has prior language-documentation context and cannot reproduce an
unprimed reader. This pass did not execute the examples or installation commands.
It assessed whether the text supplies their purpose, notation, steps and expected
result. Local browser previews were blocked, so the review used parsed HTML,
not screenshots, browser interaction or visual inspection.

The reviewed working tree was based on
`90cd575c1dc42566bb622b8e9baf794f5c71cf3b`, with the uncommitted #699/#700/#701
changes. Artifacts were reread after the three findings below were fixed:

- `target/site-entrance-review/`: the four entrance pages in Japanese;
  the renderer uses a deliberately small navigation fixture. Its README records
  why missing groups and limited previous/next links cannot establish production
  navigation behavior. The duplicate synthetic introduction/self-next link was
  removed from that fixture; it was not reported as a production-site defect.
- `target/site-pilot-review/`: the six Japanese pilot bodies; targeted English
  parity rereads for immutability, built-in types and blocks.
- `target/site-first-run-review/`: Japanese and English first-run bodies.

The final 15-file review snapshot has SHA-256
`6d902162a819e043da22ccaf2f81e0580caf3580fa87b7cd1b6237355d90798f`.
This digest covers those 11 Japanese and four English HTML files, sorted by
repository-relative path, hashing each path, a NUL, its bytes and a NUL. It
identifies a local review snapshot, not a published revision or a full-site build.

### Reading scope and questions

The Japanese reading covered `/ja/`, `/ja/docs/`, `/ja/docs/language/`,
`/ja/docs/language/model/what-is-seseragi/` and `/ja/docs/first-run/`, followed by
these six existing core identities under `/ja/docs/language/`:

1. `model/immutable-by-default/`
2. `types/built-in-types/`
3. `types/annotations-and-inference/`
4. `syntax/function-application/`
5. `types/function-types-and-currying/`
6. `expressions/blocks-and-local-declarations/`

The same questions were used when rereading corrected text: What useful task is
being performed? What are the inputs? What does each unfamiliar token in the
main example mean? What result should appear? What small change could the reader
try? Where does the text send a reader who needs the next detail? TypeScript
comparisons were checked for equivalent tasks and explicit limitations.

### Bounded findings by page

- **Home and introduction:** the shipping policy gives a concrete purpose and
  the expected `3700, 5000` output. Ordinary calls are explained before partially
  applied calls. The text explains what `standardTotal` still waits for, and
  acknowledges the TypeScript arrow-function equivalent. The numeric domain,
  invalid string input and swapped-parameter limitations are explicit.
- **Docs and Language indexes:** their own introduction, first-run and
  value/function paths supply useful starting points. Tour is optional practice;
  it is not required to understand the documentation's core explanation.
- **First run:** OS selection, a standalone `main.ssrg`, silent successful
  `lint`, expected `run` output and PATH recovery are stated. Linux execution
  verification and unexecuted macOS/Windows instructions are clearly separated.
  The reviewer could identify the next command and success condition from the
  text, but did not independently execute them. Execution evidence belongs to
  [first-run-verification.md](first-run-verification.md).
- **Immutable by default:** original/updated values and the `10 -> 11` output
  make the benefit concrete. Record terminology was corrected as recorded below.
- **Built-in types:** familiar product values introduce `Int`, `Float`, `Bool`
  and `String` before numeric limits, `Char`, `Unit` and `Never`. The corrected
  conversion guidance names its import and explains its result locally.
- **Annotations and inference:** the declaration, each arrow, the call and
  inferred result are explained. The current compiler's annotation-enforcement
  limitation is separated from the language rule.
- **Function application:** full application comes before partial application;
  the text explains parentheses, the tuple mismatch and how to repair it.
- **Function types and currying:** the remaining function is described before
  the terminology is introduced. The ordinary TypeScript equivalent is fair.
- **Blocks and local declarations:** quantity, subtotal and final value form a
  traceable calculation. The scope error includes an actionable repair. The
  unnecessary TypeScript contrast was removed as recorded below.

No additional core comprehension blocker was identified in these bodies. Some
later rule sections are dense, but they follow locally explained examples with
observable results. This judgment is bounded to the reviewed text and does not
establish that a real reader felt motivated or completed a program unaided.

### Findings returned to authors and reread

1. **#701, Japanese immutability:** the sentence previously called `name` and
   `score` records. The final rendered text says the braces form the record and
   those names are its fields: 「波括弧で囲んだ全体がレコードで、nameとscoreはその中の名前付きの項目です。」
   English already made the distinction correctly. The repair instructions now
   also tell readers to display `updated.score` when showing the updated value.
2. **#701, blocks in both languages:** the sentence about not needing an outer
   variable could suggest that TypeScript requires one. It was removed. The
   final comparison is simply TypeScript's `return subtotal + 5` versus
   Seseragi's final `subtotal + 5` expression.
3. **#699, first-run fixture in both languages:** the supplied review HTML's
   “Open in Playground” link pointed only to the Playground homepage. The author
   traced this to the focused rendering fixture: the production canonical URL
   was already source-seeded. The fixture was corrected to represent it. The
   final review HTML carries a `source` query whose decoded value is exactly
   `pub effect fn main = println "Hello, Seseragi!"` plus a newline. The reviewer
   verified the href and decoded source; opening/running it in a browser remains
   unverified in this pass. This is a review-fixture correction, not a claimed
   production-link regression or production-source fix.

During the authors' repair verification, the built-in-types conversion guidance
was also changed from `toFloat` to `fromInt` imported from `std/float`. The final
English and Japanese HTML explain where the import goes, what it imports and
that `fromInt 3` produces `Float` `3.0`. This reviewer reread that guidance but
did not claim the author's execution check as an independent execution.

### Author verification carried forward

The following are the implementing agents' recorded checks, inspected by this
reviewer but not independently rerun here. Keep them separate from the reader
judgments above:

- **#699:** [first-run-verification.md](first-run-verification.md) records the
  published Linux x64 CLI and pinned Bun installation in a fresh home, successful
  lint/run and expected output, recovery diagnostics and the platform limits.
  The focused production-component render covers the two first-run pages and
  Docs entry links; it is not a full-site render.
- **#700:** the final focused entrance render covers four pages in both locales
  with 79 assertions, using the canonical comparison source. A separate focused
  explanation/title/coverage run passed 10 tests and 13,487 assertions. The
  artifact README and review report document its synthetic navigation boundary.
- **#701:** after the review corrections,
  `SESERAGI_PILOT_OUTPUT=target/site-pilot-review bun test apps/site/tests/values-functions.test.ts`
  passed four tests and 340 assertions. It renders 12 page/locale combinations,
  checks canonical panel bytes and identities, runs six positive examples, six
  rejected examples, six repairs, three deeper call/conversion examples and a
  declarations-only rejection. Its empty-area catalog does not verify actual
  sidebar/previous-next behavior or related-link round trips. The separate
  conceptual-article explanation test passed one test and 1,786 assertions.

The six #701 valid outputs are, respectively: immutability `10 -> 11`, built-in
types `Notebook: 3, 2.5, True`, annotations `22`, application `3`, currying
`3, 3` and blocks `65`. The corresponding rejected examples report
`SES-P0001`, `SES-T0101`, `SES-T0101`, `SES-T0101`, `SES-T0101` and `SES-N0001`.
These execution records establish the illustrated behavior; they do not show
that a reader understood or successfully reproduced it unaided.

### Decision and remaining acceptance

The revised pattern is suitable for the next scoped authoring pass: a familiar
task, complete small program, local reading of unfamiliar notation, observable
result, practical benefit, then further rules and a concrete mistake/repair.
Use page-specific explanations rather than identical generic sections. Preserve
the explicit TypeScript comparison limits; explain data alternatives and
`match` as new concepts, and connect later failure handling to ordinary
`try` / `catch` before introducing its new notation.

This calibration supports continuing the already authorized site-wide work. It
does not accept the next data/choice pages, the whole 106-page corpus, generated
API pages or the non-core backlog. Those need their own scoped evidence.

Still unresolved: actual-reader understanding and unaided completion; full
production catalog/previous-next navigation; external destinations; responsive
layout, code overflow, browser controls and console errors; complete English
reader review beyond the targeted parity checks; and independent macOS/Windows
execution. Local fragment targets in the supplied Japanese review HTML were
checked, which does not replace full production-link or browser verification.

All 106 core ledger rows and eight historical checkmarks above are unchanged.
No actual-reader acceptance box is checked and #702 remains open.

## #703 Examples and Releases: independent text/HTML review (2026-10-01)

The same independent review agent read all four current production-component
HTML bodies: `/examples/`, `/ja/examples/`, `/releases/` and `/ja/releases/`.
The assumed reader knows everyday TypeScript variables, functions, objects and
ordinary error handling. The review used the presented prose, code panels,
results and link destinations; it did not fill explanatory gaps from compiler
source or the language specification. This remains a simulated first-use review,
not an unprimed human test or a browser/layout review.

The final output is under `target/site-examples-releases-review/`. The four-file
snapshot SHA-256 is
`6e2b74516f9afdee839cc8ec1b435edb34a5139673bf4537590157053ff1f074`.
It hashes the files in sorted order, using each path relative to that directory,
a NUL, its bytes and a NUL. These standalone-page renders use an empty reference
catalog; they do not validate global reference navigation or a full-site build.

Findings:

- Examples establishes its audience, a one-file local run path and a direct
  first-run destination. Each task gives inputs, useful operations and expected
  output. Suggested experiments have specific consequences: subtotal `4999`
  gives `5499`, score `99` changes only the second line, and changing all three
  pipeline inputs to `4` gives three `9` lines.
- The final record panel uses the updated single-template-string wrapper and
  explains interpolation and `\n`. The stale source link was removed by the
  author, leaving the exact-source Playground link. The pipeline panel retains
  its explained `do`/`_ <-` wrapper and source link.
- Releases identifies a dated, pinned `0.61.19` snapshot rather than promising
  live tracking. It distinguishes published and development builds, lists
  OS/CPU-specific archives and checksums, routes installation to first-run,
  describes the VSIX choice and makes lockfile recovery conditional on the
  stated diagnostic. Linux execution and unexecuted macOS/Windows procedures
  are separated. English and Japanese preserve those limits.
- No blocking task, prerequisite, notation, output or installation-selection
  ambiguity was found in this bounded reading. One optional Japanese sentence
  was made more direct: incompatible type arguments had been incorrectly
  accepted. The reviewer reread the corrected final HTML.

[examples-releases-verification.md](examples-releases-verification.md) owns the
author's separate source/execution/release-fact evidence. It records three
canonical examples, their suggested edits, CLI and Playground compiler/runtime
results and the official release assets. The latest reported focused render and
execution test passed three tests and 71 assertions. Those are author checks,
not independently rerun execution or external-release verification by this
reader reviewer.

Remaining: actual-reader understanding and unaided experimentation; real
Playground/browser interaction; desktop/mobile geometry, overflow and screenshots;
full-site integration/navigation; and macOS/Windows execution. No checkbox is
changed and #703 remains open. These four pages do not complete #629's non-core
backlog or establish acceptance of the rest of the site.

## #704 Array pilot: independent text/HTML review (2026-10-01)

The independent review read the Japanese module body
`/ja/docs/library/array/` and these eleven API bodies below
`/ja/docs/library/array/function/`:

- `chunksof/`
- `dropwhile/`
- `findindex/`
- `groupby/`
- `init/`
- `last/`
- `reduceright/`
- `sort/`
- `sortby/`
- `takewhile/`
- `windows/`

English parity and final correction reads covered the same module plus
`chunksof/`, `groupby/`, `init/`, `last/`, `reduceright/`, `sortby/` and `windows/`
under `/docs/library/array/function/`. The audience knows normal TypeScript
array operations; `Maybe`, `Either`, `match` and trait terminology were treated
as unfamiliar unless explained in the page. No tuple notation appeared in these
examples. The reviewer used only rendered text/code to judge comprehension,
without borrowing missing explanations from implementation or specification.

Artifacts: `/tmp/array704-rendered/`. The final reviewed 20-file subset (12
Japanese, eight English) has SHA-256
`b169aec6ca17ff53d40b0261247fecac97c780c0f48d8f34b425fb9cf0a0daea`.
The digest uses the same relative-path/NUL/bytes/NUL method as #703 above. The
fixture contains only the selected Array entries; the additional untouched
`Array.length` control is implementation-test evidence, not a newly accepted
reader page. The fixture cannot prove the full API list, 63-module navigation or
global previous/next behavior.

Useful results of the reading:

- The module selects operations by intended result. `chunksOf` and `windows`
  show leftovers, overlap, empty input, oversized widths and invalid sizes.
- The final `1` in `[1, 2, 4, 1]` makes `takeWhile`/`dropWhile` stopping behavior
  observable. Their comparison with `filter` explains a real choice.
- `findIndex` already showed complete `match` handling, a zero-based result and
  the missing/empty cases. `init` distinguishes `Just []` from `Nothing`, while
  `last` distinguishes `Just 0` from absence.
- `groupBy` explains key order and group order from its actual output.
  `reduceRight` traces the subtraction steps, so direction and element-first
  argument order affect an observable result.
- `sort`/`sortBy` show the unchanged original. Equal scores make stable order
  visible; the Ord requirement is explained as the ability to compare order.

Findings returned and confirmed in the fresh rendered HTML:

1. `chunksOf`, `windows`, `last`, `init` and `groupBy` explained their wrapped
   results but asked the reader to use `match` without complete extraction
   syntax. The module now supplies complete Maybe and Either consumer programs
   at `#consume-maybe` and `#consume-either`, with output and an explanation of
   the inspected expression, braces, selected case, bound name and branch result.
   All five affected pages in both locales have purpose-labelled direct links
   to the correct existing anchors. Their immediate local explanations remain.
2. `sortBy` referred to `<Int>` although its example uses an annotated empty
   record array. Both locales now explain the actual `nobody` annotation.
3. The English single-type-parameter sentence used incorrect plural grammar.
   The final signature-reading prose now uses the singular form where needed.
4. The writer also clarified that `reduceRight` requires an explicit initial
   value, including for empty input. The reviewer read the final wording in
   both languages.

No blocking explanation issue remained after this text/HTML reread. It supports
using these editorial patterns in further scoped work, not accepting unrelated
API pages. [array-editorial-verification.md](array-editorial-verification.md)
records the author's distinct technical evidence: thirteen standalone canonical
sources, the final eight-test/13,854-assertion Array/explanation run, the
16-test/317-assertion runtime suite, canonical metadata/identity guards and the
untouched control. The reviewer did not independently rerun those tests.

Still unverified: an actual first-time TS reader's understanding and ability to
use the examples; full English page-by-page comprehension beyond targeted parity;
real browser/module-to-API navigation; desktop/mobile code layout and controls;
and full-site generation/integration. No core ledger row or prior checkmark is
changed, #704 remains open, and this 12-page pilot does not complete the
remaining library/API or non-core backlog.

## #705 Data and choices: independent text/HTML review (2026-10-01)

The independent review covered all ten article bodies in Japanese and English,
using ordinary TypeScript `type` / `interface`, objects, functions and conditionals
as the starting knowledge. ADTs, constructors, `match`, structural/nominal typing
and irrefutable patterns were not assumed known. Presented text and code supplied
the explanations; the reviewer did not fill gaps from source or specification.
The reviewed identities, all under `/docs/language/` with `/ja/` mirrors, are:

- `data/records/`
- `data/structs/`
- `types/nominal-and-structural-types/`
- `expressions/conditionals/`
- `data/algebraic-data-types/`
- `patterns/match/`
- `patterns/binding-rules/`
- `patterns/irrefutable-patterns/`
- `types/type-constructors/`
- `types/closed-records/`

This was a simulated first-use text/HTML review with prior documentation-review
context. It was not an unprimed human test, an execution check or visual/browser
inspection. All Japanese bodies and all English prose were read. Rendered code
and output panels were compared across the ten locale pairs and matched exactly.
Local fragment targets existed in all twenty files. These checks do not establish
global navigation or external-link behavior.

Final artifacts: `target/site-data-review/`, generated from the working tree
based on `90cd575c1dc42566bb622b8e9baf794f5c71cf3b`. The twenty-file snapshot
SHA-256 is `c57def24c3c061728324efb608f11bb7de89aeff0c8bb6e2e46b7214ca959b08`.
It uses sorted paths relative to the artifact directory, then path/NUL/bytes/NUL
as in the #703 record. The fixture has no navigation areas. Its README explicitly
limits the evidence to article bodies rather than the full production catalog.

Reading findings:

- Records, structs and type compatibility connect familiar object fields to
  immutable copies and separately named data. The examples show original and
  updated values, then make the record/struct conversion visible.
- The shipping conditional explains `Bool`, both returned amounts and the
  corresponding TypeScript conditional operator before detailed rules.
- Delivery introduces Pending/Shipped alternatives, value construction and
  extracting a tracking string without first requiring a success/failure model.
  The TypeScript `kind`/union/`switch` counterpart is explained locally and does
  not claim that exhaustive checking is exclusive to Seseragi.
- Match explains each side of the branch arrow and why the bound tracking name
  is available. Its omitted-Pending example gives a concrete correction.
- Binding starts from familiar object destructuring. Irrefutable patterns use
  a fixed pair versus a possibly empty array, including why a visibly nonempty
  literal still cannot make an array-head `let` pattern valid.
- Type constructors distinguish a type argument from a value using `Box<Int>`
  and `Box<String>`. Closed records are positioned as optional entry-service
  detail and explain `Console`, `Unit` and `fails` in the actual example; they
  are not presented as exact-shape objects or a prerequisite for basic records.
- Complete first examples and retained declaration fragments are clearly
  distinguished. The latter are available for deeper reference without
  promising standalone execution.

The first pass found no core-example comprehension blocker and returned four
precise wording fixes. All were confirmed in the fresh rendered HTML:

1. The nominal/structural example contains no recursion. Its captions now say
   「名前で区別するデータ型と、フィールドで区別するレコード」 and
   “Nominal data and structural records,” removing the inaccurate qualifier.
2. The binding example directly unpacks `Player`. Its captions now say
   「structのフィールドから値を取り出す」 and “Unpack struct fields,” without
   claiming nesting.
3. The ADT paragraph referred to a TypeScript panel “below” after that panel had
   already appeared. Both locales now refer to “this TypeScript example.”
4. The Japanese match paragraph similarly referred to a rejected example below
   its already-rendered panel. It now describes the example without a direction.

No remaining core-body explanation blocker was identified after the reread.
[data-choices-verification.md](data-choices-verification.md) separately records
the author's checks: three tests/504 assertions for programs, rejected cases,
the strict TypeScript comparison and twenty production-component renders;
ten existing explanation/title/coverage tests with 13,487 assertions; the
351-route content map; and scoped format/type/whitespace checks. The reviewer
read that evidence after judging the prose and did not independently rerun its
executions. No compiler or runtime change is accepted through this review.

Still unresolved: actual-reader understanding and unaided use, browser code
readability and interaction, desktop/mobile geometry, full production
sidebar/previous-next/link behavior, console errors and full-site integration.
This batch informs the next scoped work but does not accept all core, generated
API or non-core routes. The 106 rows and eight historical checkmarks remain
unchanged; no actual-reader checkbox is checked and #705 remains open.

## #708 Text and Unicode units: independent text/HTML review (2026-10-01)

The independent reviewer read all Japanese and English prose for the fifteen
changed identities: `/docs/library/text/`, plus the following routes below
`/docs/library/text/function/`, each with its same-identity `/ja/` counterpart:

- `concat/`, `join/`, `split/`
- `contains/`, `startswith/`, `endswith/`
- `replace/`, `replaceall/`, `trim/`, `lines/`
- `lengthbytes/`, `lengthscalars/`, `scalarat/`, `slicescalars/`

The assumed reader uses ordinary TypeScript string/array operations. Unicode
scalars, graphemes and the Maybe/Either result-handling syntax were treated as
new until locally explained. The reviewer used rendered prose/code/output rather
than source or specification to fill explanation gaps. This is an independent
agent's simulated reader review with prior documentation context, not an actual
first-time person's test or a browser session.

Final artifacts are under `/tmp/text708-rendered/`. The thirty reviewed files,
excluding the untouched `isEmpty` control, have snapshot SHA-256
`7cb6273241ad53cf9513eb278ec3766b5f850214c4d866b047ab8ee84582574d`.
It hashes sorted filenames relative to that directory, with path/NUL/bytes/NUL
as above. All fifteen locale pairs had identical rendered code/output panels.
Local fragments and referenced module anchors existed in both locales. The
selected-metadata fixture does not establish the complete API catalog,
63-module navigation, production previous/next behavior or full-site generation.

Reading findings:

- The module explains the same string as five scalars, ten UTF-8 bytes and four
  graphemes, then shows TypeScript's six UTF-16 units and APIs for the other
  counts. The comparison attributes differences to selected units, without
  presenting TypeScript as incapable or promising cross-version segmentation
  equivalence for every string.
- `scalarAt` and `sliceScalars` have complete Maybe/Either consumer programs.
  Their prose explains the selected case, contained name and branch result.
  Zero-based positions, excluded end position, absence, valid empty ranges and
  invalid unclamped ranges can be traced through the output.
- `concat`/`join` distinguish a String result from an array and explain the
  display brackets. `split` and `lines` use JSON output so `[]` and `[""]` are
  visibly different, with explanations of empty fields and trailing newlines.
- Literal search, case sensitivity, first/all replacement, non-overlapping
  matches, empty search text and literal `$&` are shown concretely. The `trim`
  example distinguishes Unicode White_Space from the retained U+FEFF case.
- No substantive task, result-use or boundary-explanation blocker was found.
  This does not show that an actual reader reproduced the examples unaided.

Two wording changes were returned and reread in the final English/Japanese HTML:

1. `trim` referred to its U+FEFF case as the final input, although an empty-string
   case followed. It now identifies “the input containing U+FEFF” directly.
2. `concat` compared its result to an ambiguous “Array.concat”. It now explicitly
   names `concat` in Seseragi's `std/array` module, avoiding confusion with a
   familiar TypeScript method.

[text-editorial-verification.md](text-editorial-verification.md) separately owns
the author's implementation evidence: fifteen executed Seseragi programs and
one checked/executed TypeScript comparison, four text tests with 467 assertions,
a subsequent focused rendering assertion pass, selected Unicode runtime tests
and scoped format/whitespace checks. The reviewer did not rerun those executions.
The untouched `isEmpty` render is a regression control, not new reader acceptance.

Still unresolved: actual-reader understanding and unaided use; browser
interaction, code readability, desktop/mobile geometry and screenshots; complete
site navigation and generation/integration. No core row, historical checkmark
or issue acceptance box changes through this review. #708 remains open, and this
fifteen-page batch does not accept the other Text, Unicode or library/API pages.

## #710 Absence, failure and Effect: independent text/HTML review (2026-10-01)

All ten Japanese and English article bodies were read, including retained deeper
rules, under the everyday-TypeScript contract. Familiar `try` / `catch` was a
starting point; FP, Task, Effect and trait theory were not assumed known. The
reviewer used presented text and code rather than implementation/specification
knowledge to bridge missing explanations. The exact routes below
`/docs/language/effects/`, with the same `/ja/` identities, are:

- `pure-expressions/`
- `maybe/`
- `either/`
- `effect-type/`
- `effect-functions-contract-form/`
- `effect-functions-inferred-form/`
- `runtime-boundaries/`
- `error-channels/`
- `environment-requirements/`
- `defects/`

The independent agent review is simulated first-use reading with prior
documentation context. It is not an actual novice's test, execution verification
or browser/layout acceptance. All ten rendered code/output panel sets matched
across locales, and local fragments resolved in all twenty files.

Final artifacts: `target/site-failure-review/`, with SHA-256
`f63e7e72067acbc1c23b6a1db74cfee9635e29711a4e29c77052dfa05b3cc0cc`
for its twenty HTML files, using the sorted relative-path/NUL/bytes/NUL method
above. The output uses production article components with an empty-area catalog.
Its README records why this does not establish full navigation or visual layout.

Useful findings:

- `Maybe` distinguishes missing data from a present empty string. Its `??`
  example explains what is returned and when a fallback is evaluated.
- `Either` begins with positive-number validation and a familiar TypeScript
  try/catch counterpart. The text explicitly separates thrown control flow from
  returning `Left`, limits the compared inputs and acknowledges TypeScript
  success/failure result unions.
- The stored Effect example prints `before`, `run`, `run`, making deferred
  execution and repeated execution visible. The boundary example prints only
  `only main`, separating unused constructed work from work returned by main.
- Explicit and inferred forms explain the three parts of the actual Effect
  type. The inferred-function witness uses the requirement record
  `{ console: Console }`, rather than treating `Console` alone as that record.
- Recovery explains the lambda argument, successful-value binding and the
  replacement Effect. The Settings example shows both the selector and provided
  environment, so the source of the displayed label is traceable.
- Defects uses checked division and a separate deferred zero-division example
  that `recover` does not catch. TypeScript's `Infinity`/`3.5` behavior is stated
  instead of equating it with Seseragi integer arithmetic. Compiler errors and
  runtime termination remain distinct.

No blocking explanation issue or required rewrite was found. One optional
Japanese sentence in the explicit-contract page was simplified to say that
TypeScript try/catch itself does not declare this failure-type contract, removing
the harder-to-read “throws” wording. The independent reviewer confirmed the
simplified sentence in the refreshed production-component HTML.

[failure-reader-verification.md](failure-reader-verification.md) owns the
author's separate technical evidence: three tests/541 assertions, ten native
positive programs, eight compile-invalid cases, two lint-valid runtime failures,
strict TypeScript comparison/numeric probes and twenty locale renders. The
broader explanation/title/coverage suite passed twelve tests/13,492 assertions,
with 351 content-map routes. These were inspected reports, not independently
rerun executions by the reading reviewer. The final optional wording change
received its own targeted EN/JA render check.

Remaining acceptance includes actual-reader understanding and unaided use,
browser code readability and interactions, desktop/mobile geometry, full
sidebar/previous-next/link behavior, console errors and full-site integration.
Task, concurrency, resource/cancellation and Signal articles were not accepted by
this review. The 106 core rows and eight historical checks remain unchanged,
no actual-reader box is checked, and #710 stays open.

## #707 Syntax and evaluation: independent text/HTML review (2026-10-01)

The independent reviewer read all eleven Japanese and English article bodies,
including detailed rules, current-compiler caveats and rejection repairs. The
audience knows everyday TypeScript values, functions and objects; ADTs, traits
and advanced type theory were not used to fill gaps. The reading relied on the
presented HTML text/code rather than source or specification knowledge.

Ten routes are under `/docs/language/syntax/`, with the same `/ja/` mirrors:

- `source-text/`
- `literals/`
- `character-string-escapes/`
- `layout-and-line-continuation/`
- `reserved-words-and-names/`
- `optional-record-fields/`
- `method-calls/`
- `pipelines-and-low-precedence-application/`
- `operator-precedence/`
- `custom-operators/`

The eleventh is `/docs/language/expressions/evaluation/` and its Japanese mirror.
`syntax/function-application/` was not rerewritten or newly accepted here.

Final artifacts: `target/site-syntax-review/`. Its twenty-two HTML files have
SHA-256 `0c569010479c3d1063006768e3d2af9f4e6f790261a460e1a52e7932eecd9a7b`,
using the sorted relative-path/NUL/bytes/NUL method above. All eleven paired
code/output sets matched, and local fragments resolved in all twenty-two files.
The focused render uses real production components and canonical sources with
an empty-area catalog, not a complete site-navigation or browser fixture.

Reading findings:

- Source and literals start with named, familiar values and observable output.
  The escape program makes written backslashes, quotes, Unicode escapes and
  actual printed characters distinguishable, including the five-line result.
- Layout now really contains a multiline pipeline, a block and a parenthesized
  multiline call. Their `7, 7` result matches the explained calculation. The
  pipeline page also connects its notation to ordinary one-argument calls.
- Optional fields introduce Maybe through a familiar fallback task without
  demanding match knowledge first. Methods explain the receiver, `impl`,
  `self`, arguments and unchanged original through `10` and `15` outputs.
- Custom operators begin with a String-only operation rather than requiring
  generic/trait knowledge. Grouping and argument evaluation remain separate.
- Evaluation keeps its main calculation focused on evaluated arguments and a
  `23` result. Value/storage identity is explicitly labelled as a different,
  later limitation rather than being presented as an evaluation diagnostic.
- The pages preserve normative spelling rules while identifying specific
  0.61.19 conformance/diagnostic gaps. Acceptance of an invalid declaration or
  delimiter is not described as permission to write it.

Two reading findings were returned to the author and reread in both locales:

1. The precedence reference compressed levels into bands such as `7..4` and
   `-1..-3`, preventing the reader from determining each operator's exact level.
   The final list spells out all thirteen levels from 9 through -3, the operators
   sharing each level and their grouping. It also explains that `infixr 4` shares
   level 4 with `+`, `-` and `:`, and that conflicting same-level grouping needs
   parentheses. The reviewer could now answer that lookup from the page itself.
2. “Values/functions start lowercase” appeared inconsistent with the valid
   Japanese name `次の値` demonstrated by the source-text page. The final naming
   introduction and rules distinguish cased initial letters from permitted
   uncased characters, explicitly including Japanese value/function names.

No remaining body explanation blocker was identified after the final reread.
This is an independent agent's simulated reading result with prior context,
not actual-human comprehension or a claim of successful unaided programming.

[syntax-reader-verification.md](syntax-reader-verification.md) separately owns
the author's source and runtime evidence: fifteen tests/1,042 assertions,
eleven standalone native examples, eleven compiler rejections and repairs,
WASM/compiler-runtime output parity and twenty-two production-component renders.
It records release/development compiler provenance, precedence and Unicode
classification checks, and existing conformance tracking. The reader reviewer
inspected this report after the prose review and did not independently rerun
those tests or use implementation details to supply missing reader explanations.

Still unresolved: actual-reader understanding and unaided use, full-site
generation/integration, full navigation and route closure, real browser behavior,
desktop/mobile layout, keyboard access and deployed-site behavior. The 106 core
rows and eight historical checkmarks remain unchanged. No actual-reader or issue
acceptance box is checked, and #707 remains open.

## #713 Data operations: independent text/HTML review (2026-10-01)

All six Japanese article bodies were read first, followed by all six English
bodies, including their examples, outputs, detailed rules and rejection repairs.
The assumed reader uses ordinary TypeScript functions, objects, arrays and
try/catch; familiarity with FP, ADTs, traits or advanced type theory was not used
to supply missing explanations. The reviewer read presented HTML text and code
before consulting the author's technical evidence.

The exact routes below `/docs/language/`, with matching `/ja/` routes, are:

- `data/tuples-arrays-and-lists/`
- `expressions/lambdas/`
- `expressions/ranges-and-comprehensions/`
- `data/newtypes/`
- `data/impls-and-methods/`
- `data/operator-overloads/`

Artifacts: `target/site-data-operations-review/`. The twelve HTML files have
SHA-256 `9b0b8b887901fdbaeda878ab65e9dcc9ed31f45f92a09d1fb1bdb845287b97c0`,
using the sorted relative-path/NUL/bytes/NUL method above. All six paired
code/output panel sets matched, and local fragments resolved in all twelve
files. The production-component fixture has an empty navigation-area catalog,
so this reading does not validate complete navigation or visual presentation.

No explanation blocker or required correction was found:

- Collections shows both `Just` and `Nothing`, then immediately uses `match`
  to obtain an integer. Tuple extraction, zero-based indices, List's leading
  backtick and prepend syntax are explained beside the actual output.
- Lambdas defines `map`, the backslash/arrow form, capture of the outer bonus
  and partial application. The reader can trace both `[12, 22, 32]` and `8`.
- Comprehensions explains `|`, `<-`, conditions, inclusive/exclusive endpoints
  and nested iteration order. The versioned top-level-let Bool-check caveat
  preserves the language rule instead of teaching a compiler gap as syntax.
- Newtype construction and extraction are explicit, and the example does not
  promise input validation or unique IDs. The separate Types page owns the
  comparison between ID identities and aliases.
- Methods explains `self`, the new immutable result and why the receiver-only
  `count` call needs no `()`. Operator overloads introduces its standard-trait
  correspondence locally and distinguishes it from declaring a new symbol.

[data-operations-verification.md](data-operations-verification.md) owns the
author's separate execution evidence: ten tests/515 assertions, six native
valid programs, rejected variants and repairs, WASM/runtime output checks,
exact-source Playground URLs and twelve production-component renders. It also
records the comprehension compiler gap and exact binary provenance. These
checks were not independently rerun by the reading reviewer.

This is an independent agent's simulated reading with prior documentation
context. Actual first-time-reader comprehension and unaided use, browser code
readability and interactions, desktop/mobile layout, complete navigation and
full-site integration remain separate. No acceptance checkbox changes, and
#713 remains open. This six-identity review does not accept other Data,
Expressions or library articles.

## #714 Array/List construction: independent text/HTML review (2026-10-01)

The reviewer read the fourteen edited identities in Japanese and English,
including full article bodies, code, output and exact declarations. These are
`/docs/library/array/` and `/docs/library/list/`, plus each module's six routes
under `function/`: `empty/`, `singleton/`, `fromiterable/`, `zip/`, `zipwith/`
and `unzip/`. Japanese mirrors have the same identities under `/ja/`.

The audience assumption remains everyday TypeScript, without FP, List, tuple
pattern or Unit knowledge being supplied from the implementation. The review
used rendered text only. The unchanged Japanese `length` controls were also
read to distinguish them from the rewritten articles; their terse reference
content is not newly accepted by this pass.

Final artifacts: `/tmp/sequence714-rendered/`. The twenty-eight edited HTML
files, excluding the four untouched control renders, have SHA-256
`6ffeb152338bc1d50cf2e520d882bc139ed78ef2fb97a1c698755660d21f2d36`
using sorted relative-path/NUL/bytes/NUL hashing. All fourteen locale code/output
panel pairs matched, and their local fragments resolved. The focused catalog
contains the twelve selected functions and two controls; it cannot establish
the full Array/List inventory or global navigation.

The main reader questions can be answered from the pages:

- `empty` defines `()` as the required Unit value argument and distinguishes
  an uncalled function from its empty result. An annotation and an explicit
  type argument both show how an empty collection obtains its element type.
- List's leading backtick is introduced locally. `singleton` shows an empty
  collection nested as one element and counts a wrapped empty string as one.
- `fromIterable` explains ordered conversion and full consumption of finite
  input, including unchanged source and empty output. Endless input is clearly
  stated not to finish; the review does not claim to have executed it.
- `zip` makes the right-first call order visible with names and scores.
  `zipWith` uses subtraction to expose the separate callback order. Both
  describe truncation and both empty sides without implying a mismatch error.
- `unzip` actually names and uses both returned collections. It explains why
  values discarded by a prior `zip` cannot be recovered.

No explanation blocker was found. A small English correction changed nine
instances of “a Array” to “an Array” in the Array empty/fromIterable/singleton/
unzip pages and List fromIterable page. The reviewer reread all five corrected
English bodies in the refreshed production-component HTML.

[sequence-editorial-verification.md](sequence-editorial-verification.md) owns
the author's technical evidence: twelve executed canonical programs, the
initial three tests/428 assertions, selected runtime tests, eleven combined
editorial tests/1,290 assertions and the final focused rerender with 366
assertions. The independent reviewer did not rerun those executions. Original
Array examples and Maybe/Either consumers were read in place and remain part
of the module's explanation.

This is bounded agent text/HTML reading, not actual-human or browser acceptance.
Visual geometry, interactions, full route closure, complete navigation and
full-site integration remain unverified here. The two control identities and
remaining Array/List APIs are outside this editorial acceptance scope. #714
stays open; no issue or reader-acceptance checkbox changes.

## #712 Type names and reuse: independent text/HTML review (2026-10-01)

All ten Japanese bodies were read in the order below, followed by all ten
English bodies, including retained detailed sections. The assumed reader knows
ordinary TypeScript `type`/`interface`, functions, objects and try/catch, but
does not need prior generics, ADT, match or type-theory expertise to read the
complete first examples. Implementation and specification knowledge were not
used to fill explanatory gaps.

The exact routes under `/docs/language/types/`, with `/ja/` mirrors, are:

- `type-system/`
- `optional-record-fields/`
- `generic-aliases/`
- `newtypes/`
- `type-identity-and-coercion/`
- `generic-functions/`
- `polymorphism/`
- `type-parameter-scope/`
- `generic-structs/`
- `generic-adts/`

Final artifacts: `target/site-type-review/`. Its twenty HTML files have SHA-256
`acfaca4bed132ded112be9c37de3fdb41744fa8c5376fb16a65f7147b4032982`,
using sorted relative-path/NUL/bytes/NUL hashing. All ten code/output panel sets
matched across locales, and local fragments resolved. The README documents the
production-component fixture's empty navigation areas and absent full catalog;
this is article-content evidence only.

The examples support concrete distinctions without requiring theory first:

- A labelled number/string shows aliases reusing a shape. The distinct-ID
  example then explains what newtype adds, while stating that construction
  does not establish ID existence, positivity or other validation.
- Repeated `A` means the same type within one call. Separate calls and separately
  declared parameters choose types independently. Tuple creation/extraction
  and the distinction between a function name and a returned value are local.
- Explicit Int-to-Float conversion changes the value passed to the calculation;
  merely changing an annotation does not. The comparison does not suggest
  TypeScript cannot perform the same division.
- Complete ordinary TypeScript aliases, generic functions and tagged object
  types perform the same tasks. The text acknowledges their usefulness instead
  of relying on advanced branded types or an unfair limitation.
- Generic structs distinguish a same-type spread update from constructing a
  new Box at another type. Generic ADTs explain named alternatives, `match`,
  missing type-argument information and Failure as an ordinary value, then
  leave recursive Tree details in the later reference section.

No required explanation correction was identified. One optional English
clarification was applied and reread in both locales: the optional-record-field
rule now names `id?: Maybe<String>` and its `Maybe<Maybe<String>>` read result.
It explicitly distinguishes absent `Nothing` from present `Just Nothing`,
matching the Japanese paragraph and replacing the ambiguous “an optional
access” wording. Some deeper English reference sections retain specialist
terms; they are not prerequisites for the complete first examples.

[type-reader-verification.md](type-reader-verification.md) separately owns the
author's ten native outputs, ten rejected programs, three strict-TypeScript
same-task comparisons, three focused tests/634 assertions and twenty locale
renders. It records preservation of the detailed rules and section IDs, the
existing lambda-inference limitation, the configured static checks and an
unrelated additional strict-check finding. The nested-absence clarification
received its own native probe and paired rerender. These are inspected author
results, not new executions by this reviewer.

This remains a simulated independent-agent reading with prior context, not a
test with an actual novice. Browser interactions and code readability,
desktop/mobile layout, complete sidebar/previous-next behavior, deployed output
and full-site integration are outside these artifacts. The 106 core rows and
eight historical checks remain unchanged across these three additions. No
actual-reader or issue acceptance box is checked, and #712 remains open.

## #711 Modules and complete projects: independent text/HTML review (2026-10-01)

The independent reviewer read all eleven Japanese bodies and then all eleven
English bodies, including complete file panels, commands, outputs, rejected
projects, repairs and deeper rules. The reader contract assumes ordinary
TypeScript import/export, functions, objects and types; package-resolution
theory, compiler architecture, FP and Effect theory were not used to bridge
explanatory gaps. The reading used presented HTML before consulting the
author's implementation evidence.

The exact routes under `/docs/language/modules/`, with `/ja/` mirrors, are:

- `identity/`
- `packages/`
- `top-level/`
- `visibility/`
- `imports/`
- `specifier-resolution/`
- `re-exports/`
- `namespaces-and-resolution/`
- `dependency-graphs/`
- `initialization/`
- `entry-points/`

Final artifacts: `target/site-modules-review/`. The twenty-two HTML files have
SHA-256 `9e4b22b449cd13d70f1284f889803ab3f25ff02101273f5af517d2f301040610`,
using sorted relative-path/NUL/bytes/NUL hashing. All eleven code/output panel
sets matched across locales, including the configuration and source files,
and local fragments resolved. This is a focused production-component render,
not a complete catalog or real-browser fixture.

No blocking explanation issue was found:

- The greeting project shows its configuration and both source files. The
  instructions identify the project root and commands, the relative-import
  base, and why pasting only main into a single-file Playground is insufficient.
  The equivalent TypeScript files use the same input and output without a
  false claim that the familiar import/export task needs Seseragi.
- The package example explicitly creates sibling app and library directories
  and runs `lock update app` and `run app` from their parent. It separates
  manifest exports from declaration-level `pub`, and explains that the language
  version range is not the CLI product version.
- Separate advanced projects have file labels and their own configuration.
  The imports mistake returns explicitly to the first greeting project.
  Visibility captions distinguish the private-helper and opaque examples;
  re-export failures identify which original files remain in use.
- Same-file aliases preserve declaration identity, while copying a newtype
  declaration creates another type. The displayed error with two `UserId`
  names is explained through their different defining files.
- Initialization calculates `double 21`, yet does not run the saved Effect or
  the dependency's main. Function availability is distinguished from whether
  the value read by that function has initialized. The repair moves the value
  preparation, not merely the function declaration.
- Dependency cycles include type-only imports. The repaired value cycle is
  honestly described as a redesign from a shared base, rather than preserving
  the original unsatisfiable equations. The type-only repair explains that
  its unused imports can simply be removed.
- Entry points explain the selected main, Unit argument and success result,
  required services and failure display. The later foreign-loader section
  defines Task locally and keeps runtime loading separate from development
  tools inspecting source.

The author had already corrected import-diagnostic placement, the return to
the greeting example, visibility captions, Task wording and bounded path-rule
clarifications before this reading; those current forms were read, not counted
as new independent findings. The reviewer requested one optional positional
copy correction in imports: the paragraph after the TypeScript panels now
says “この例も” / “This version also” instead of pointing to a following example.
Both refreshed paragraphs were reread, with no remaining body blocker.

[modules-reader-verification.md](modules-reader-verification.md) separately
owns the author's execution and source evidence: 28 complete native projects
(16 valid and 12 rejected), executed repairs, a strict two-file TypeScript
comparison, 93 canonical source records and the final 41 tests/15,313 assertions.
The report records grouped-panel regression checks, source preservation,
formatting, lint and scoped static checks. These are inspected author results,
not independent executions by the reading reviewer.

This is simulated independent-agent reading with prior documentation context.
It does not establish actual first-time-reader understanding or unaided use.
Browser code readability and interactions, desktop/mobile geometry, keyboard
behavior, complete navigation and route closure, deployment and whole-site
integration remain outside this text/HTML evidence. Local browser access was
blocked and separate preview authorization was still pending at the checkpoint;
neither is recorded as a browser pass. Across #711–#714, the 106 core rows and
eight historical checks are unchanged, and no issue or actual-reader checkbox
is checked. #711 remains open.

## Remaining 41 core identities: independent text/HTML review (2026-10-01)

The four batches below received independent whole-body Japanese reading followed
by whole-body English reading, including first programs, outputs, mistakes,
repairs and retained reference details. The assumed reader uses everyday
TypeScript functions, objects, arrays, `type`/`interface`, import/export and
try/catch. FP, Rust/Haskell, advanced type theory and compiler knowledge were
not used to fill gaps in the presented text. The reviewer consulted the author
verification reports only after reading the rendered bodies.

This is an agent simulating a first-use reading, with prior documentation
context. It is not a test of an actual novice or successful unaided programming.
All 82 locale bodies use focused production-component HTML. Code/output panel
sets matched for all 41 locale pairs, and local fragments resolved. Empty or
deliberately small catalogs exclude complete navigation, route closure and
browser/layout acceptance. No new public work item or comment was posted for
these batches while the separate authorization request remained pending.

### Seven remaining type articles

Exact routes under `/docs/language/types/`, with matching `/ja/` routes:

- `generic-impls-and-methods/`
- `let-polymorphism-and-rank/`
- `recursive-declarations/`
- `variance/`
- `requirement-merge/`
- `kinds/`
- `erasure-and-runtime-representation/`

Final artifacts: `target/site-type-limits-review/`, fourteen HTML files,
SHA-256 `6fe3372702a29d3d3b47b3e5b0b5c789971cef9016fdc83e4f8f96ab32f7541b`
using the sorted relative-path/NUL/bytes/NUL method above.

The first examples make the advanced rules concrete. Box's receiver determines
`A`, while `replace<B>` constructs a new Box at the argument's type. The let/rank
page distinguishes naming an already generic function from receiving a callback
whose `A` is chosen by the enclosing call. The #683 caveat explains that the
unconstrained identity lambda is a current inference limitation, separately from
the specified generalization rule; this reviewer assessed the explanation,
not the compiler truth from implementation knowledge.

Variance explicitly distinguishes type compatibility from immutability and
acknowledges TypeScript's accepted array view without claiming it removes fields.
Kinds defines the `_` slot and separates choosing `Maybe` from choosing its
payload. Erasure avoids promising a memory layout or zero allocation. Recursion
limits the constant-stack statement to direct self tail calls in ordinary pure
`fn`, warns that mutual recursion lacks that guarantee, and distinguishes the
older example that does not normalize negative input.

No blocking explanation issue was found. One optional local definition was
added and reread in both locales: the requirement-merge example now explains
`Never` as no recoverable/ordinary typed failure, beside its first use in the
Effect signature. The final paired render includes that clarification.

[type-limits-verification.md](type-limits-verification.md) owns the author's
separate evidence: three tests/449 assertions, seven native outputs and seven
rejected programs, two strict TypeScript comparisons and mutations, a separate
array-view witness and fourteen locale renders. The parent reran the same
focused suite after adding the Never definition. These are reported technical
checks, not executions by the independent reading reviewer.

### Ten Model and Grammar articles

Nine routes are under `/docs/language/model/`: `programs/`, `non-features/`,
`design-principles/`, `expression-oriented/`, `no-hidden-danger/`,
`backend-independent-semantics/`, `diagnosable-behavior/`, `visible-costs/` and
`readable-density/`. The tenth is `/docs/language/grammar/`. Each has the same
identity under `/ja/`. What-is-Seseragi and immutable-by-default were not
rewritten by this batch.

Final artifact groups and hashes, using sorted relative-path/NUL/bytes/NUL:

- `target/site-model-concepts-review/`: programs, non-features,
  design-principles and grammar; eight HTML files;
  `1acac8e91c8453a81c2b697e0b4d3c32a7aa068cc847b05938c9ef19e5714d93`
- `target/site-model-reader-review/`: the other six principles; twelve files;
  `b4c47d98ca7228c73b688b1effec89d8e7d62bcc6130b0f2072ce8a98b22dc31`

Programs separates ordinary calculation, Effect construction and runner
execution. It correctly explains why a pure main can pass lint yet fail entry
validation during run. Absence introduces and consumes Maybe locally, retains
the complete-match rule and bounds the current top-level-let checking caveat.
The numeric comparison holds negative inputs constant while distinguishing
Int and Float, without inventing another production backend. The cost page
separates element count, stored results and callback work from measured time.
Readable-density contains a real multiline pipeline and keeps intermediate
names as a useful alternative. The overview helps choose a question rather
than imposing a course. Grammar first teaches EBNF notation, then maps a real
declaration to it and separates syntax from name/type validity.

No first-example blocker was found. One optional Japanese cost clarification
was applied and reread: inlining expands a function body at its call site, and
prohibited omission/reordering concerns necessary observable work. The text no
longer appears to prohibit the inlining it has just allowed.

[model-concepts-verification.md](model-concepts-verification.md) records eight
tests/247 assertions and [model-principles-verification.md](model-principles-verification.md)
records eleven tests/477 assertions. They separately own native/WASM output,
actual rejected cases and repairs, the exact unchanged normative grammar panel,
source/link checks and the recorded exhaustive-match discrepancy. The reader
did not rerun those executions or use their source details to supply explanations.

### Thirteen trait articles

Exact routes under `/docs/language/traits/`, with `/ja/` mirrors:

- `model/`
- `declarations/`
- `instances/`
- `constraints/`
- `methods-versus-traits/`
- `method-calls/`
- `deriving/`
- `standard-operators/`
- `coherence/`
- `laws/`
- `do-notation/`
- `do-block-typing/`
- `do-desugaring/`

Final artifacts: `target/site-traits-review/`, twenty-six HTML files,
SHA-256 `addc56eabb40ae3319aa3b03ab85abeaa74e37a64a0340897e6297f9799488ee`
using sorted relative-path/NUL/bytes/NUL hashing.

Ticket and Person connect the trait declaration, type-specific instance and
call through the same label operation. The TypeScript interface comparison
does the same job without padding the familiar solution. Dot methods remain
distinct from trait calls, and the empty-array case explains why the element
implementation is still required. Laws demonstrates a deliberately wrong but
well-typed Eq and distinguishes its failed property from a type error.

The three do pages start from missing address parts, consume Maybe results,
explain `pure` and `flatMap`, and distinguish payload types from the outer
computation type. Desugaring also explains that skipping a continuation cannot
undo evaluation of arguments already supplied to a function. The relevant
compiler caveats preserve normative rules rather than teaching accepted invalid
forms as alternatives.

No first-example blocker or substantive locale mismatch was found. Two optional
clarifications were applied: only ordering comparisons are described as calling
`compare` once, and deeper English text uses concrete implementation language
instead of undefined “evidence” terminology. The author also made current-module
instance candidates explicit, introduced ADT locally and added a standard-Eq
signature-check caveat. All changed EN/JA paragraphs were reread in the final
HTML, with no remaining explanation blocker.

[traits-reader-verification.md](traits-reader-verification.md) owns the author's
21 tests/14,448 assertions, thirteen primary programs, rejected modifications
and literal repairs, the separately runnable law violation, strict TypeScript
comparison, advanced do witnesses and five current compiler discrepancies.
These inspected reports remain separate from the reader's text/HTML assessment.

### Eleven remaining Effect, lifecycle and Signal articles

Exact routes under `/docs/language/effects/`, with `/ja/` mirrors:

- `effectful-for/`
- `task/`
- `sequential-and-parallel/`
- `cancellation-and-resources/`
- `scheduler-fairness/`
- `fiber-supervision/`
- `signals-and-transactions/`
- `derived-signals/`
- `signal-operators/`
- `subscriptions-and-lifetime/`
- `exceptions-and-algebraic-effects/`

Final artifacts: `/tmp/lifecycle-effects-rendered/`, twenty-two HTML files,
SHA-256 `bd52006fe4481f6809a8c8320c84daf1af6663b2c7767305c928b64f089fd492`
using sorted relative-path/NUL/bytes/NUL hashing.

The Task counter's `[0, 1, 2]` explains delayed construction and repeated,
uncached execution. Parallel results are explicitly input-ordered without
claiming that their output proves scheduling speed or strict alternation. The
resource trace shows reverse release before recovery, and Fiber uses a ready
handshake to ensure the child registered cleanup before scope exit. The example
does not rely on an arbitrary sleep or timing guess.

Signals shows `[2, 12]` as stable notifications, distinguishes a captured integer
from a changing Signal, and explains `<$>`/`<*>` without implying a Monad
instance. Subscription history `[1, 2, 2]` followed by a current value of `3`
shows that release stops observation without freezing the source. The expected
failure example keeps recovery distinct from defects and cancellation, and the
unsupported try/catch case reports unresolved names rather than promising a
different parser diagnostic.

One local explanation gap was corrected. The resource, Fiber and subscription
first examples used `effects.scoped $ do` without explaining `$`; resources also
used `|> recover`, and subscriptions used `:=` and `*source`. The final paired
paragraphs now show the equivalent parenthesized call, explain the pipeline's
argument and identify Signal update/read as procedures executed in do. All six
paragraphs were reread. Japanese mistake paragraphs also now refer neutrally to
the already displayed rejected code; the eleven corrected references were read.
No remaining explanation blocker was found.

[lifecycle-reader-verification.md](lifecycle-reader-verification.md) owns the
author's eleven native examples and eleven rejected variants, existing resource
and concurrency probes, staged Signal rollback test and five tests/604
assertions. The final prose-only rerender passed one test/326 assertions. No
executable example changed for the reading fixes. The reader did not independently
execute the programs or perform a browser interaction.

### Exact core-route coverage and remaining acceptance

The union of the bounded review route sets was compared with the 106 existing
core rows above: **106 distinct identities, zero missing identities and zero
duplicate batch ownership**. The sorted route list, joined with newline and a
final newline, has SHA-256
`f9cee8f9ecb79a97b1e8170bbfebd4748e779b1a3bf67ed99720b7c2b05b2930`.
The route counts are 1 entrance identity, 6 values/functions pilot, 10 data and
choices, 10 first Effect pages, 11 syntax/evaluation, 10 first Types pages,
6 data operations, 11 modules, then these 7 Types, 10 Model/Grammar, 13 Traits
and 11 Effects. Library/API, First Run, homepage, Examples and Releases reviews
are outside that core total.

The earlier #702 pass used targeted English checks. To close that depth gap,
the reviewer now read the complete English what-is-Seseragi body and all six
English pilot bodies, finding no additional explanation blocker. These seven
HTML files have combined SHA-256
`873e95dc2dc6fe4e448f9360e0f78a46f0124457d0cfcd6b71d3ecedbe6c969b`,
using sorted repository-relative path/NUL/bytes/NUL hashing. Their earlier
Japanese bodies had already been read in full. The historical record above is
preserved; this completion supplements it rather than relabeling the original
review's depth.

Thus **106/106 existing core identities have received independent-agent
whole-body text/HTML reading in both locales under the revised TypeScript-reader
contract**, across the recorded artifact revisions. This is coverage of that
review activity, not 106 actual-reader or site acceptances. It does not establish
that all pages were regenerated together from one integrated checkpoint.

Actual novice comprehension and unaided use remain untested. Browser access was
blocked and preview authorization remained pending; desktop/mobile layout,
keyboard behavior, code readability, live interactions, complete navigation and
deployed behavior are not passes. Full-site integration has its own technical
gates. The 106 historical rows and eight checked boxes remain byte-for-byte
unchanged. No actual-reader or issue acceptance checkbox is newly checked, and
no pending external posting is represented as completed.

## Checkpoint 4: library landing, List and nine API corrections (2026-10-01)

An independent agent read the complete current English and Japanese article
bodies for **22 identities, 44 locale bodies**: the library landing, twelve List
identities and nine corrected API identities. The assumed reader uses everyday
TypeScript functions, arrays, objects, type/interface declarations and ordinary
try/catch, with no assumed Seseragi, Rust, Haskell, pattern matching or functional
programming theory. Explanations were assessed from rendered prose and code,
without filling gaps from compiler/runtime sources or specifications.

This is a text/HTML review of focused production-component renders, extracted
with an HTML parser. It is not a browser, visual, actual-human or unaided-use
acceptance. The count describes this checkpoint's review activity: the List
overview was reviewed before and is read again here, so the count must not be
added to earlier totals as twenty-two newly unique library identities. None of
these library routes changes the earlier 106-core route coverage or its eight
historical checked boxes.

All hashes below use SHA-256 over sorted artifact-relative paths, each followed
by NUL, file bytes and NUL. No public comment or issue acceptance was posted.

### Library landing: one identity, two bodies

Exact routes: `/docs/library/` and `/ja/docs/library/`. Artifacts are
`target/site-library-landing-review/{docs/library,ja/docs/library}/index.html`.
Final paired hash:
`6e5777004552a8f1edc1e431b6d0285d0c49da01ec07a914879112f6c260ae40`.

Both complete bodies were read, including all 63 purpose/target entries. The
six task choices provide a useful starting point: collection operations, text
and JSON, numbers and bytes, state and coordination, files and processes, and
web interfaces and networking. Maybe, checked results, deferred Effect work and
environment-supplied services are introduced where they affect that choice.
The page does not require a Tour or a sequential course before an API lookup.

The filesystem and networking paragraphs distinguish target availability from
supplied capabilities. Browser support does not imply access to arbitrary OS
files; pure HTML/SVG values are distinguished from live browser operations.
There is no executable example on this landing and no new run/output promise.

Two small copy corrections were requested and reread in the final HTML:
English List now says "immutable" rather than the potentially ambiguous
"persistent"; Japanese Storage now includes "読み書きの失敗", covering read
failures as well as writes. No remaining explanation blocker was found.

The reviewer checked that article content contains 63 unique module links,
once each; purpose-labelled reader-choice links preserve locale and target the
named sections/articles, and local fragments resolve. This checks the landing's
selection surface, not the bodies of all 63 linked module pages.

[library-landing-verification.md](library-landing-verification.md) owns the
author's separate one-test/772-assertion result, canonical target/purpose
associations and preservation of unrelated landing-directory markup. Its fixture
has the real 63-module directory metadata but empty symbol lists and no complete
core navigation. Full route closure, actual clicks, responsive layout and
keyboard behavior remain separate integration/browser work.

### List: twelve identities, twenty-four bodies

Exact routes, each also under `/ja/`:

- `/docs/library/list/`
- `/docs/library/list/function/chunksof/`
- `/docs/library/list/function/windows/`
- `/docs/library/list/function/dropwhile/`
- `/docs/library/list/function/takewhile/`
- `/docs/library/list/function/findindex/`
- `/docs/library/list/function/init/`
- `/docs/library/list/function/last/`
- `/docs/library/list/function/reduceright/`
- `/docs/library/list/function/sort/`
- `/docs/library/list/function/sortby/`
- `/docs/library/list/function/groupby/`

Artifacts: `/tmp/list-next12-rendered/`. Final hash of these edited 24 HTML files,
excluding controls:
`f227584e4425f84a930cb09bc2d2277980f5d36d33deda367b8bca27c42f834e`.

Every complete body was read in both languages. The module explains the leading
List backtick, its first/rest access pattern and the different cost of accessing
later positions. The retained zip example makes argument order, tuple order,
shorter-input truncation and empty inputs concrete. Complete Maybe and Either
consumers appear on the relevant API pages as well as the module; readers do
not have to infer how to use the wrapped results from a printed constructor.

The operation-specific distinctions are clear: takeWhile/dropWhile stop at the
first failed condition; findIndex returns the first zero-based position; init
distinguishes a successful empty remainder from absent input; last keeps zero
distinct from absence. chunksOf retains a short last batch, while windows keeps
only complete widths, and both reject nonpositive sizes even for empty input.
reduceRight walks the subtraction example and contrasts its element-first
callback with TypeScript's accumulator-first callback. sort/sortBy explain
unchanged input, numeric order and stable ties; groupBy consumes present and
missing groups while preserving first-seen keys and input order within groups.
The TypeScript comparisons provide workable alternatives without inventing
language limitations.

Two bounded copy corrections were applied and reread: groupBy's odd/even
remainder description now refers to the positive integers actually shown, and
println is described as displaying a value and adding a newline, matching the
examples that pass Int directly. The final module also includes the shared
"immutable" List summary. No remaining explanation blocker was found. The
reviewer checked paired code/output panels and included local anchor targets.

[list-editorial-verification.md](list-editorial-verification.md) owns the
author's thirteen native canonical programs, three behavior/TypeScript tests
with 76 assertions, final production rendering with 340 assertions and the
14-assertion identity-guard check: five tests/430 assertions in total. The
30-page render also includes empty, zip
and length in both locales as unchanged controls. Those six control bodies are
not accepted by this review. The catalog is filtered to one module; complete
module inventories, global navigation, actual execution and browser behavior
are not established by the reader's text review.

### Nine corrected APIs: nine identities, eighteen bodies

Exact routes, each also under `/ja/`:

- `/docs/library/map/function/get/`
- `/docs/library/transformer/state/function/get/`
- `/docs/library/queue/effect-function/take/`
- `/docs/library/web/storage/effect-function/get/`
- `/docs/library/non-empty-list/function/head/`
- `/docs/library/non-empty-list/function/tail/`
- `/docs/library/non-empty-list/function/tolist/`
- `/docs/library/set/function/toarray/`
- `/docs/library/set/function/tolist/`

The corrected flat render is `/tmp/seseragi-nine-api-rendered/`; the same
eighteen article files are retained with route-shaped paths in
`/workspace/shared/seseragi-nine-api-authoring/reader-html/`. The reviewer checked
that those eighteen files match the corresponding flat-render files. The hash
using the route-shaped artifact-relative paths is:
`9a9f015cc199acee1a29c739eda955ae418b04ab98ec9469ca6adfe1c2df6d21`.

The earlier render was **not accepted**: reversed arguments to the new bilingual
paragraph helper's `arrays.zip` call swapped complete walkthrough and rule
paragraphs between languages. Japanese pages displayed English paragraphs and
English pages displayed Japanese paragraphs. The author repaired the pairing
and added assertions requiring each expected paragraph while rejecting its
opposite-locale counterpart. After regeneration, the reviewer read **all eighteen
complete bodies again**, rather than clearing the batch from spot checks.

The fresh bodies make the corrected meanings usable. Map.get reads a key and
distinguishes an empty stored string from an absent key, with a fair TypeScript
undefined/has comparison. StateT.get uses an explicit initial state, explains
Unit, both tuple positions and the Maybe wrapper, and distinguishes removing
the StateT layer from executing a possible outer Effect. Its TypeScript example
uses an ordinary state-taking function rather than requiring another library.

Queue.take demonstrates FIFO draining after close and explains that only an
open empty queue waits. It locally explains the lambda, `<$>`, `|>`, recovery,
error constructors, mapError and the remaining failure type. Its TypeScript
counterpart is explicitly a consumer of an application-supplied asynchronous
queue, not a claimed queue implementation or an Array.shift equivalent.

NonEmptyList explains how singleton/cons establish a nonempty input, why head
returns a value directly, why a singleton's tail is an ordinary empty List, and
why toList retains the elements while dropping the type's nonempty guarantee.
The TypeScript nonempty tuple comparison preserves that guarantee fairly. The
head-only comparison caption was corrected to describe the displayed scalar
integers rather than array/List output, and all three singleton descriptions
now identify the NonEmptyList they construct. These changes were reread.

Set conversions preserve insertion order and distinguish reinserting an
existing member from removing it before reinsertion. Empty results and the
original unchanged Set are shown. Duplicate handling belongs to Set
construction, and the TypeScript comparison explicitly copies before mutating
the comparison Set. List and Array representation differences are explained.

Storage remains a browser integration function. The local Storage-service
definition and purpose-labelled environment-requirements link were added and
reread. The code constructs an Effect; the calling application must supply the
service, execute the work and use the returned text. Present empty text,
absence and StorageError are distinct. The outcome panel is explicitly labelled
as mock-host cases, not browser output or the standalone module's output; its
test boundary does not establish browser persistence or actual storage access.
Both locales have no seeded Playground launch for this integration module.

No explanation blocker remained after the complete reread. The reviewer checked
that all nine paired code/output/declaration panel sets match, local fragments
resolve, sixteen seeded runnable links exactly match their displayed first
source, and the two Storage bodies link the correct localized environment
requirements article. These are text/source/link checks, not click or execution
evidence. The six module overviews and six untouched controls in both locales
that accompany the 42-file fixture are outside this nine-API acceptance scope.

[api-summary-corrections-verification.md](api-summary-corrections-verification.md)
owns the author's separate seven tests/625 assertions, eight standalone
Seseragi outputs and seven TypeScript program outputs. Storage compiles with a
temporary web caller and is exercised through provider/mock tests; the
TypeScript queue consumer uses a mock queue, while the native sample uses real
FIFO/closed-drain operations. The independent reading reviewer did not rerun
these executions. Canonical signatures, runtime behavior and compiler-owned
metadata are not changed or accepted wholesale by this reader-facing review.

### Checkpoint boundary

The final scoped text/HTML review has no remaining explanation blocker. Full
site integration has its own technical gate. Browser access remained blocked
and preview authorization was pending during this reading: desktop/mobile
layout, overflow, keyboard interaction, live browser APIs and actual novice
comprehension are unverified. The existing 106 rows and eight historical checks
are preserved unchanged; no actual-reader or public issue checkbox is newly
checked. This record does not declare the remaining library/API backlog complete.

## Checkpoint 5: Map, Unicode/graphemes and Regex (2026-10-01)

An independent agent read **40 existing Library identities, 80 complete locale
bodies**: fifteen Map identities, fourteen Unicode/grapheme identities and eleven
Regex identities, each in English and Japanese. The reader assumption remains
everyday TypeScript functions, arrays, objects, type/interface declarations and
ordinary try/catch, without assumed Seseragi, pattern matching or functional
programming theory. The agent read production-rendered HTML prose and code with
an HTML parser; implementation/specification knowledge was not used to fill
explanatory gaps. Author technical reports were inspected separately after the
body reading.

These are text/HTML review results, not actual-human or browser acceptance.
The scoped counts exclude the generated control pages described below. They
do not change the earlier 106-core coverage statement or historical checkboxes.
Each hash uses SHA-256 over sorted artifact-relative path, NUL, file bytes and
NUL. There were no public writes or issue-acceptance changes.

### Map: fifteen identities, thirty bodies

The exact routes are `/docs/library/map/` and its
`function/{empty,singleton,fromentries,containskey,insert,upsert,remove,keys,values,entries,size,mapvalues,mapkeyswith,mergewith}/`
pages, each with the corresponding `/ja/` route. Artifacts are retained in
`/tmp/map15-rendered/`. The final hash of the thirty edited HTML files is
`b08e638194e5f66ba2d8efa00bd25321ebdfbe2d6d1516e91546befcc5273a4f`.

The module explains when to choose a key/value table, how repeated values differ
from repeated keys, and how to keep successive immutable versions. The example
shows replacement retaining position, insertion appending a key, and the original
remaining unchanged. Each new leaf provides complete Seseragi and TypeScript
programs with the same displayed outputs. The TypeScript programs copy before
mutation where preserving the original is part of the task, rather than treating
mutation as a limitation that prevents the task.

The boundary explanations are concrete. fromEntries keeps the last duplicate
value at the first key position. containsKey and upsert preserve a present zero
or empty string; upsert locally consumes Just/Nothing and explains why its
callback returns a stored value rather than requesting deletion. A missing
remove needs no error branch. keys, values and entries distinguish complete
Arrays from JavaScript iterators, and size counts keys rather than insertions
or distinct values.

mapKeysWith shows three colliding values combined as A/B then A/B/C, with an
unrelated key between them that does not split the group. mergeWith contrasts
its right-Map-first call order with the callback's left-value-first order.
Swapping the Maps visibly changes both concatenation and ordering. The
TypeScript loops implement those same choices explicitly.

The reviewer requested shared-copy corrections: string-valued upsert and
collision examples must not be described as Int-valued; empty calls must not
always be labelled `empty<String, Int>`; and the function syntax explanation
must distinguish arrows between parameters from the final result arrow.
The first generalization of the empty-call sentence also sounded as though
explicit type arguments were mandatory, contradicting the empty page's
surrounding annotation. The final paragraph now permits either `<...>` type
arguments or a surrounding annotation, while retaining the required Unit
argument. The final EN/JA shared paragraphs were reread, including all
twenty-eight leaf instances of the last correction. No remaining explanation
blocker was found.

The reviewer checked that all fifteen paired source/output/declaration panel
sets match, all thirty seeded links exactly match their displayed first source,
and local fragments resolve. Programs, outputs and declaration panels remained
unchanged through the prose corrections.

[map-editorial-verification.md](map-editorial-verification.md) owns the author's
separate four tests/519 assertions, fourteen native programs and TypeScript
counterparts, and the nested existing runtime Map suite with ten tests/4,058
assertions. The final production render passed 416 assertions. Its 36-file
fixture includes the existing get correction and untouched filter/isEmpty pages
in both locales as controls; those six bodies are not newly accepted here.
The filtered Map catalog does not establish complete module/type navigation.

### Unicode and graphemes: fourteen identities, twenty-eight bodies

Exact routes, each also under `/ja/`:

- `/docs/library/text/grapheme/`
- `/docs/library/text/grapheme/function/{length,clusters,at,slice,byteboundaries}/`
- `/docs/library/text/unicode/`
- `/docs/library/text/unicode/function/{normalize,isnormalized,fullcasefold,version}/`
- `/docs/library/text/function/{casefold,tolower,toupper}/`

Artifacts are retained in `target/site-unicode-reader-review/`. The final hash
of these twenty-eight edited HTML files is
`bc4d1bf6f03b7f97b162ac45615be1e95e8ed6349c8b2838c5b01151ffcfdd23`.

The same label makes the units understandable: A, an emoji with skin-tone
modifier and e with a combining accent produce three graphemes, five scalars,
twelve UTF-8 bytes and seven TypeScript UTF-16 units. The TypeScript comparison
uses Intl.Segmenter rather than implying JavaScript cannot count visible
clusters. The text separates pinned Seseragi Unicode rules from host versions,
without extrapolating equality on these examples to all strings or versions.

The individual operations explain preserving the original spelling, direct
String/Array results, Maybe for an absent position and Either for invalid ranges.
slice includes valid empty ranges, rejects reversed/negative/out-of-range bounds
and shows a concrete repair. byteBoundaries explains the final end offset and
why empty input returns `[0]`; it prevents confusing byte positions with cluster
positions accepted by slice.

Normalization, case folding and display casing have separate tasks. NFC/NFD
show composed/decomposed spellings; NFKC/NFKD explain lost compatibility details.
Straße/STRASSE demonstrate folding without falsely equating it with lowercasing.
Greek sigma, dotted I and expanding uppercase results explain why casing can
change length and may not be reversible. The text does not promise collation,
confusable detection, username validation or arbitrary host equivalence.

One concrete correction was required: version originally described `()` as a
no-argument invocation although its signature takes Unit. The final complete
EN/JA version bodies were read and now identify the no-additional-information
Unit value. The local scalar/Char definition on grapheme.at was strengthened,
and the shared wrapper now introduces let and the graphemes import. Version's
shorter wrapper follows its actual non-template-string program. Both local
definitions and all twenty-four leaf wrapper instances were reread. No remaining
explanation blocker was found.

The reviewer checked all fourteen paired source/output/declaration panel sets,
thirty-two source-seeded links across the module and API examples, and local
fragments. Panels remained unchanged by the prose fixes.

[unicode-reader-verification.md](unicode-reader-verification.md) owns the author's
eighteen tests/906 assertions: twelve native and committed-WASM/runtime examples,
two strict TypeScript counterparts, the invalid-range repair, version/source
checks and exact dispatcher/render assertions. The final focused catalog has
three real modules with selected symbols. Its unchanged Text module, concat and
Unicode.isMark control bodies in both locales are excluded from this review.
WASM/runtime harness execution is not a browser interaction or visual check.

### Regex: eleven identities, twenty-two bodies

The exact routes are `/docs/library/regex/` and its
`function/{compile,compilewith,defaultoptions,escape,ismatch,find,findall,split,replaceall,replaceallwith}/`
pages, each with the corresponding `/ja/` route. Artifacts are retained in
`/tmp/seseragi-regex-rendered/`. The final hash of the twenty-two edited HTML
files is
`f01233dc185c8999d1ac1b9e96f4a13015ca07508425cb350fa410c31fa034a9`.

The first task finds an order reference and uses its matched text. compile
rejection, successful compilation and a valid pattern with no match are separate
outcomes, with local Left/Right and Just/Nothing consumers. Pattern syntax,
string-level backslash escaping, option fields, Unit, record access and the
findAll comprehension are explained beside use. The unsupported look-ahead
example defines what that notation requests before contrasting its acceptance
by JavaScript.

find distinguishes unmatched captures from missing named groups, while
replaceAllWith explains both optional layers around an indexed capture. The
examples contrast byte `3..9` with TypeScript UTF-16 `2..8`, and recommend using
the returned matched text instead of mixing indexing units. Empty matches,
leftmost start, earlier alternatives, greedy repetition, non-overlap and scalar
progress are observable in the displayed outputs.

The TypeScript comparisons preserve meaningful differences. Unicode word
characters differ despite the same shorthand; split shows captured separators
and edge-empty differences; literal replacement uses a TypeScript callback and
separately demonstrates replacement-string expansion. The escape comparison
uses the simpler String.includes for literal containment and explicitly says
it does not generate a Regex fragment. Regex callbacks return ordinary strings,
and I/O remains in the output entry.

The only requested copy correction was in compileWith: its initial wording
implied full case folding could produce uppercase STRASSE. The final EN/JA
paragraph says simple folding does not expand ß into two scalars ss, while the
separate full-folding text operation maps Straße to strasse. Both refreshed
paragraphs were reread; the Regex comparison against STRASSE is unchanged.
No remaining explanation blocker was found in the twenty-two complete bodies.

The reviewer checked eleven paired source/output/declaration panel sets,
twenty-two exact seeded links and local fragments. All panels remained unchanged
through the final wording fix. The two RegexSpan control pages in the 24-file
fixture are not accepted by this review.

[regex-reader-verification.md](regex-reader-verification.md) owns the author's
separate six tests/987 assertions and the final targeted production rerender
with 763 assertions. Its eleven native, eleven committed-WASM/runtime and eleven
TypeScript examples, direct matching-contract probes, real type rejections,
repairs and dispatcher checks remain technical evidence from the author. The
independent reader did not execute these programs or use those source details
to supply missing prose explanations.

### Checkpoint boundary

All forty scoped identities have complete paired text/HTML reading and verified
rereads of the final requested changes. Fourteen extra locale control files
across the three fixtures are excluded. The focused catalogs do not establish
full route closure, complete cross-module inventories, actual link navigation,
desktop/mobile appearance, overflow, keyboard behavior or live browser execution.
Actual novice comprehension remains untested, browser access remained blocked,
and preview authorization was pending. Parent integration owns the complete-site
technical gate. The prior ledger bytes, 106 checkbox rows and eight historical
checks remain unchanged. No acceptance checkbox or public tracker status is
advanced by this record, and the whole library/API backlog is not declared done.

### Checkpoint 5 preflight: corrected page-title links

Integration preflight found page-name links whose labels did not exactly match
the destination heading. The independent agent reread the freshly rendered
labels and their adjacent purpose text in both locales after the authors'
repairs. This is a bounded link/prose reread; the complete body readings above
remain the evidence for the other content.

All twenty-eight Map leaf bodies changed only the module anchor from `Map` to
`std/map`, matching the rendered module H1 in both locales. The adjacent purpose
still says “choose by the task you need” / “必要な処理から操作を選べます”.
The same repair in the earlier List batch changes twenty-two leaf anchors from
`List` to `std/list`; its purpose remains “choose another operation by its
result” / “必要な結果から操作を選べます”. Exact before/after HTML comparison
verified that these labels were the only changes in those files; the two module
bodies and the six control files in each fixture were unchanged.

The Unicode module's eight links per locale now use the exact destination
titles: normalize, isNormalized, fullCaseFold, caseFold, toLower, toUpper,
version and std/text/grapheme. Their purposes appear immediately after the
anchors, separated by an English or Japanese colon. The reviewer read all
sixteen resulting entries and checked their labels against the rendered target
H1s. They still distinguish normalization, comparison, display casing, data
version and grapheme selection/counting. The std/text#text-units section link is
unchanged. Only the two Unicode module bodies changed; all other files in that
fixture, all link destinations and the displayed source/output/declaration
panels were unchanged.

The following hashes supersede the respective earlier artifact hashes in this
ledger. They use the same sorted artifact-relative path, NUL, file bytes, NUL
convention and exclude the same controls:

- Map, thirty selected files in `/tmp/map15-rendered/`:
  `9414135193d2bef12a80a9a207d3001394d9edff50fc0a86cc4d58b027219367`
- Unicode/graphemes, twenty-eight selected files in
  `target/site-unicode-reader-review/`:
  `1cf967a4023d7142ffdf722ab51c76c6d529be93423b15e5f2554dd5e5ca0bfd`
- Earlier List batch, twenty-four selected files in `/tmp/list-next12-rendered/`:
  `a800fdab37b8577a2b470a11caf61d340f99bae67121f81a9359871b57f3f047`

The authors separately reported successful refreshed production-render checks:
Map 444 assertions, List 362 assertions and the complete Unicode focused suite
of eighteen tests/941 assertions. The Map and Unicode checks cover 58 and 56
authored page-name links respectively against actual destination titles. These
technical checks supplement the independent text/HTML reread; they do not
establish actual browser navigation or novice comprehension. The extended
integration title audit and full-site gate were still pending when this bounded
record was added. No additional identities, control-body acceptances, historical
checkbox changes or public tracker updates are claimed.

### Extended title repairs and completion of earlier reading gaps

The extended integration audit found the same title-label issue in the earlier
Array and Text batches. The independent agent read all corrected EN/JA selector
entries and leaf-footer purposes from their fresh production-component HTML.
The repair keeps the exact destination title inside the anchor and places the
task description beside it. No additional explanation blocker was found.

For Array, the bounded reread covered the module and the eleven original pilot
functions listed in #704 above. The overview contains seventeen choices: the
eleven pilot operations plus empty, singleton, fromIterable, zip, zipWith and
unzip. All thirty-four localized selector labels match the rendered destination
H1s. Their purposes still distinguish keeping/skipping a prefix, locating a
position, sorting/grouping, batching/windows and constructing/pairing arrays.
The twenty-two original pilot footers now link `std/array`, followed by
“compare Array operations and choose by the result you need” or
“Arrayの操作を比べ、必要な結果から選べます。”

The new fixture is `/tmp/array-title-repair-rendered/`. It contains 38 locale
files so that all seventeen selector destinations and the length control are
available. Only the module and eleven original pilot identities received this
bounded reread; the six construction APIs and length controls are not newly
accepted by it. The leaf comparison used the earlier #704 HTML, while the
overview comparison used the later #714 HTML containing the construction
choices. Displayed source/output/declaration panels were unchanged.

For Text, the reread covered the module plus concat, join, split, contains,
startsWith, endsWith, replace, replaceAll, trim, lines, lengthScalars,
lengthBytes, scalarAt and sliceScalars in both locales, at the routes recorded
in #708. All twenty-eight selector labels and twenty-eight footer labels match
the rendered destination headings. Purposes still distinguish literal matching,
first/all replacement, end trimming, line splitting and scalar/byte units. The
leaf footer now names `std/text` before its existing action text. All URLs and
source/output/declaration panels are unchanged; both isEmpty control files are
unchanged and excluded. Fresh files are in `target/site-text-title-review/`.

The original #704 record deliberately had only targeted English coverage. The
reviewer now read the **complete English bodies** for the four missing routes:

- `/docs/library/array/function/dropwhile/`
- `/docs/library/array/function/takewhile/`
- `/docs/library/array/function/findindex/`
- `/docs/library/array/function/sort/`

These readings included every explanatory paragraph, code/output panel and
declaration. The prefix examples make the unvisited final `1` visible;
findIndex explains the complete Just/Nothing consumer and the TypeScript `-1`
difference; sort demonstrates numeric order and an unchanged input. One small
accuracy correction was requested: “show numbers then prints” now reads
“The second println call prints”. The refreshed English paragraph was reread,
and an exact HTML comparison confirmed that this phrase was the only change
in the 38-file refresh. No body blocker remains. Together with the earlier
readings, these four additions close the original Array pilot's English
whole-body gap; they do not rewrite its historical partial-review record.

The reviewer also read the three previously unrecorded complete English
entrance bodies: `/`, `/docs/` and `/docs/language/`. The artifact root is
`/workspace/shared/seseragi-integration-checkpoint4-20261001/full-site-attempt1/`;
their sources were unchanged for this checkpoint. The shipping task gives
bounded inputs, output and a local explanation of partial application. Docs
offers concrete next actions and keeps the Tour optional. Language Reference
starts with a short values/functions path before its broader concept index.
No explanation blocker was found. All 85 internal body links have existing
targets and fragments in that retained output; this is a filesystem check,
not a browser click-through. The earlier Japanese readings remain recorded
under #702. The seven core/pilot English gaps were already closed at checkpoint 3,
and first-run had already been read in both locales.

Final selected-file hashes, using the same relative-path/NUL/bytes/NUL method:

- Array module and eleven pilot APIs, twenty-four current locale files:
  `a3ac36f25f1d4e7a0d2dbde6e64be1706306563c75e074a38da2e41076198623`
- The original #704 twenty-file subset within that fresh Array fixture:
  `a0c02113db30a03ced22f3806c20875d3329b6e6fc3906544fc47bb1105346cf`
- The four newly completed English Array bodies:
  `e7b0d0994d97a64e5853d880865b1e77f1d90a2dba1d1cb04161fd8b4929a36c`
- Text module and fourteen APIs, thirty current locale files:
  `2e7657f9c368af0fac72b5d2e1b21c084855da8c94942709c790f93bafc628bd`
- Three complete English entrance files, relative to the retained checkpoint-4
  full-site root:
  `11826d0cfc6ce692e03b6e518932937a8f827d16aec5c6f4c810796aee34a447`

The Array and Text hashes supersede their earlier artifact hashes, without
changing what those historical readings covered. The authors separately
reported 362 assertions for the final Array render and four tests/600
assertions for Text. The reviewer also read the integrator's
`all-overlay-title-mismatches.json`: 100 authored identities, 200 locale bodies,
283 checked references and no mismatches. That is evidence for title consistency
across that inventory, not 100 body acceptances inferred from a link audit.

No known partial-locale gap remains within this reviewer's recorded authored
scopes after these additions. Unreviewed API bodies, generated controls and the
rest of the site remain outside those scopes. The complete-site integration
gate was still in progress; browser appearance, interaction and actual novice
comprehension remain unverified. All prior ledger bytes, the 106 original
checkbox rows and eight historical checked boxes are preserved. No public
tracker status or acceptance checkbox is changed.

## Checkpoint 6: Set, Char/Text and NonEmptyList/Iterator reader review (2026-10-01)

Baseline: `afe906eee87dd0b588730a578554851f0e2404ac`, with the intentional local
checkpoint-6 changes. This record covers **34 existing identities and all 68
complete English/Japanese bodies**: Set 13/26, Char/Text 13/26, and
NonEmptyList/Iterator 8/16. Japanese routes add `/ja` to the routes below.

An independent agent with ordinary TypeScript/programming knowledge read the
rendered bodies in sequence, including every explanatory paragraph, native and
TypeScript program, output, exact declaration and related-link purpose. This is
an agent-simulated reader review, not feedback from an actual first-time human
reader. The reviewer did not use prior Seseragi knowledge or source material to
supply missing explanations. Source/specification inspection was used after
reading to verify a discovered numeric-boundary issue and to check the actual
published panels and source-seeded links.

### Set: thirteen identities, twenty-six bodies

- `/docs/library/set/`
- `/docs/library/set/function/{empty,singleton,fromiterable,contains,insert,remove,size,map,union,intersection,difference,issubsetof}/`

The membership tasks, first-occurrence uniqueness, insertion order and unchanged
originals are understandable locally. The examples explain empty typing and
Unit, explicit right-first combination calls, directional difference/subset
checks, and mapping several members to the same result. TypeScript uses its
ordinary Set, array/filter/every operations and copies before mutation only
where retaining the earlier value is part of the task. The comparison does not
claim a general equality guarantee for arbitrary element types.

The module initially promised a TypeScript counterpart on every operation
page, although filter/isEmpty remain controls. Both locales now identify the
insert/remove pages specifically. The refreshed module correction was read in
context. The reviewer also noted shared helper prose about syntax or JSON that
some programs did not use. The final helper cleanup keeps List syntax only on
fromIterable, parameter/result syntax with map's parity function, member/JSON
formatting only where used, and relevant empty-Set/Unit, TS-spread and copy notes.
All 72 changed paragraphs across the twenty-four leaf bodies were reread in the
fresh artifact. Both module files are byte-identical to the already reviewed
module correction; other leaf text is unchanged. No remaining bounded reader
correction was found.

Final artifact: `/tmp/set13-reader-polished/`. Independent checks cover 56
exact-title authored anchors with adjacent purposes, 26 canonical native panels
and exact Playground seeds, 24 TypeScript panels, and 162 local fragments.
Code/output/declaration panels were read in the complete bodies and preserved
through the helper cleanup. The eight localized toArray/toList/filter/isEmpty
control bodies are excluded from new reader coverage.

The author's separate evidence is
[set-editorial-verification.md](set-editorial-verification.md): four tests / 741
assertions, twelve native/TypeScript pairs, the existing runtime contract suite,
focused production rendering and static checks. These are author execution
results, not programs independently rerun by the reader.

### Char/Text: thirteen identities, twenty-six bodies

- `/docs/library/char/`
- `/docs/library/char/function/{codepoint,fromcodepoint,tostring}/`
- `/docs/library/text/function/{trimstart,trimend,words}/`
- `/docs/library/text/unicode/function/{generalcategory,isalphabetic,isdecimaldigit,ismark,iswhitespace,simplecasefold}/`

Char/scalar/String distinctions are explained before they matter. fromCodePoint
introduces Maybe, Just, Nothing and match locally, including the branch result
and unusable-number cases. Ordinary padding and sentence splitting precede the
NEL/FEFF boundaries. Property pages explain one-scalar inputs, the actual
property tested, and the limits of inferring a whole-string rule from one Char.
The folding page distinguishes comparison forms from display lowercasing and
keeps the unchanged original available.

TypeScript comparisons preserve concise built-ins or property regexes and
explain the selected inputs and conditions. General-category and simple-fold
comparisons explicitly disclose their narrower boolean questions rather than
claiming to return the same category/transformed value. Regex notation, host
Unicode-data limitations, whitespace differences and display spelling are
explained where used. No reader-blocking correction was requested after the
complete paired reading.

Final artifact: `target/site-char-text-review/`. Independent checks cover 44
exact-title authored anchors with adjacent purposes, 28 canonical native panels
and exact Playground seeds, 26 TypeScript panels, 54 output panels and 184 local
fragments. The repeated module examples account for the panel counts. The twelve
localized Text/Unicode/grapheme module and concat/fullCaseFold/isEmpty control
bodies are excluded from new reader coverage.

The author's separate evidence is
[char-text-reader-verification.md](char-text-reader-verification.md): eighteen
tests / 1,067 assertions, twelve native and twelve committed-WASM/runtime
programs, twelve checked/executed TypeScript counterparts, six invalid/repair
pairs, production rendering and static checks. WASM/runtime-harness execution
is not an actual browser session, and these are not reader-run executions.

### NonEmptyList/Iterator: eight identities, sixteen bodies

- `/docs/library/non-empty-list/`
- `/docs/library/non-empty-list/function/{cons,fromlist,reduce1,singleton}/`
- `/docs/library/iterator/`
- `/docs/library/iterator/function/{next,unfold}/`

The score examples explain why preserving a known first element is useful,
zero versus absence, constructing versus checking a collection, and left-to-right
reduction with the first element as the starting result. TypeScript's nonempty
readonly tuple is explained without pretending ordinary TS cannot express the
same requirement. The Iterator examples introduce its current element and
returned continuation, Just/Nothing and tuple parts locally. Re-reading the
original versus reading rest is observable, and the TypeScript generator's
advancing state and differing next outputs are disclosed rather than hidden.

The first unfold draft called an incrementing Int sequence endless. The reviewer
found that its `value + 1` eventually exceeds the safe-integer range and defects,
as specified by `01-syntax.md` and `09-standard-library.md`; one bounded next call
had not exposed that boundary. The author replaced it with `repeatValue`, which
returns `Just (value, value)`, and a matching TS generator retaining its state.
The displayed first result remains 10. Both corrected complete EN/JA unfold
bodies, native/TS panels and unchanged outputs were reread. The repeated-state
explanation now supports the claim without an eventual arithmetic overflow.
No other bounded reader correction remains.

Final artifact: `target/nonempty-iterator-reader-review/html/`. Independent
checks cover 30 exact-title authored anchors with adjacent purposes, 16 canonical
native panels and exact Playground seeds, 16 TypeScript panels, 32 output panels
and 80 local fragments. The ten localized head/tail/toList and two opaque-type
control bodies are excluded from new reader coverage.

The author's separate evidence is
[nonempty-iterator-reader-verification.md](nonempty-iterator-reader-verification.md):
six tests / 749 assertions for the corrected source, including native/TS/WASM
examples, release-profile production rendering and a bounded maximum-Int
constant-state regression. Those executions and static checks remain separate
from the independent reading judgment.

### Exact reviewed artifacts and checkpoint boundary

The following **edited-only** hashes use an explicit manifest convention,
different from earlier path/NUL/bytes/NUL aggregates in this ledger. Sort the
artifact-relative HTML paths lexicographically. For each file, write its complete
HTML-byte lowercase SHA-256, two ASCII spaces, relative path and LF to a UTF-8
manifest. The aggregate below is SHA-256 of those complete manifest bytes.

- Set, 26 files: `/tmp/set13-independent-reviewed26.sha256`;
  `4017dd61381d3678cadb040365f887ecf76f007f302fa3f0e0091bb3f8bcabd6`
- Char/Text, 26 files: `/tmp/char-text-independent-reviewed26.sha256`;
  `db12e585b1ff98550e1adb0423d6cfabbae78c7135ecfd526fa66e427c3b2523`
- NonEmptyList/Iterator, 16 files:
  `/tmp/nonempty-iterator-independent-reviewed16.sha256`;
  `32abfa85763d41819b99fa42597a17e9b867d45e5bb82c693e21de146e43b25f`

Independent structural records are
`/tmp/set13-independent-review-final.json`,
`/tmp/char-text-independent-review-structure.json`, and
`/tmp/nonempty-iterator-independent-review-final.json`. The checks verify source,
titles and fragments in retained HTML; they are not browser click-throughs or
proof of a person's comprehension.

Thirty extra locale control files across the three focused fixtures are excluded.
Selected-module production rendering does not establish full-site generation,
complete cross-module inventories, route/navigation closure or interaction.
Full-site integration and retention comparison remain parent/integrator-owned
and were unverified when this record was appended. Browser access remains
blocked; desktop/mobile layout, clipping/overflow, keyboard behavior and live
browser execution have not been reviewed here. Actual novice comprehension and
human first-time-reader acceptance remain unverified.

This is a bounded explanation/readability/appeal judgment, separate from native
execution, rendering tests, full-site validation and actual human feedback. It
does not accept the remaining API/library backlog or declare the whole site
done. All prior ledger bytes/history, 106 original checkbox rows and eight
historical checks are preserved. No acceptance checkbox, public tracker status,
publication, commit, push or deployment is advanced by this record.

## Checkpoint 7: collection types and data/validation reader review (2026-10-01)

Baseline: `bd5d39ebaf164c32cf562624d971e531309b68c7`, with the intentional local
checkpoint-7 changes. This record covers **24 existing identities and 48 complete
English/Japanese bodies**: nine collection type/constructor identities with
eighteen bodies, and fifteen data/validation identities with thirty bodies.
Japanese routes add `/ja` to the routes below. These counts do not include
supporting destinations, prior callable/module explanations or instance pages.

### Reviewer attribution and evidence boundary

The collection author's separate independent reader,
`/root/finish_collection_type_plan/review_collection_type_bodies`, read all
eighteen collection bodies, then reread all ten bodies affected by its requested
copy corrections and all six bodies affected by final source formatting. Its
complete report is retained at
`target/collection-type-reader-review/reader-review.md` and
`/tmp/collection-types-reader-review.md`.

The coordinating independent reader, `/root/review_collection_text_docs`, read
all thirty data/validation bodies and reread the two corrected Validation module
bodies. It also initially read fourteen overlapping collection bodies (Map,
Set, NonEmptyList, Iterator, SizeError, NonPositiveSize and ReduceStep, each in
both locales), before duplicate reading was stopped. That overlapping partial
read is **not** the authority for the complete eighteen-body collection review
or its later repair/formatting rereads; those belong to the named separate
reader. The coordinating reader read and consolidated that final report.

Both are agent-simulated reviews against the ordinary TypeScript reader contract,
not actual first-time human feedback. They read full rendered prose, native and
TypeScript programs, output panels, declarations and navigation context. No
prior Seseragi knowledge was used to fill a missing local explanation. Source
and metadata checks verify disputed claims, ownership, displayed bytes and
links; they do not substitute for body reading. Neither reader independently
executed the displayed programs or conducted a browser session.

### Collection types and constructors: nine identities, eighteen bodies

- `std/map::Map`: `/docs/library/map/opaque-type/map/`
- `std/set::Set`: `/docs/library/set/opaque-type/set/`
- `std/non-empty-list::NonEmptyList`:
  `/docs/library/non-empty-list/opaque-type/nonemptylist/`
- `std/prelude::Iterator`: `/docs/library/iterator/opaque-type/iterator/`
- `std/collection::SizeError`: `/docs/library/collection/opaque-type/sizeerror/`
- `std/collection::NonPositiveSize`:
  `/docs/library/collection/constructor/nonpositivesize/`
- `std/collection::ReduceStep`:
  `/docs/library/collection/opaque-type/reducestep/`
- `std/collection::Next`: `/docs/library/collection/constructor/next/`
- `std/collection::Done`: `/docs/library/collection/constructor/done/`

The Map price table and Set tags begin with familiar lookup/deduplication tasks,
then explain explicit types, construction versus annotation, unchanged earlier
values and content equality separately from insertion order. Missing price and
zero remain distinct. The TS comparisons use ordinary Map/Set, copy when
retaining originals is required, and compare contents explicitly; neither
claims TS const freezes contents or equates all number values with Int.

NonEmptyList explains a required first score and a possibly empty tail. Its
Maybe consumer, backtick List syntax, construction/checking alternatives and
ordinary-List type errors are local. The TS readonly tuple has its required
first element, rest syntax and limits explained without an unchecked assertion.
Iterator explains a retained position versus its returned continuation, lazy
construction, repeated pulls without consumption and the absence of a caching
guarantee. Its canonical prelude identity and std/iterator page/operation owner
are explicitly distinguished. The TS example honestly contrasts advancing
generators with a small persistent-position model for the same 3,3,2,done output.

SizeError and NonPositiveSize distinguish the reason type, public constructor,
Either success/failure alternatives and the operations doing validation.
`NonPositiveSize 2` is deliberately valid construction: its Int payload is not
magically restricted to nonpositive values. Empty input does not skip invalid
size checking. ReduceStep, Next and Done explain a running result and the
consumer's continue/stop decision. Constructing Done does not terminate main;
reduceUntil interprets it, stops further pulls/callbacks and cannot undo earlier
array construction. The TS comparison begins with an ordinary concise loop
before showing callback-shaped decisions; its extra first output is explained.

The separate reader requested and verified three paired corrections:

1. Map/Set now distinguish the Unit type from its `()` value instead of calling
   `()` a value named Unit.
2. ReduceStep's exact declaration is `ReduceStep<_>`, so its shared constructor
   `arg1` explanation was removed from that type page. The placeholder now
   explains the carried-result type; constructor argument prose remains on the
   actual Next/Done pages.
3. SizeError/NonPositiveSize now explain fn, named parameters and their order,
   the first versus final arrows, the expression after =, Array type arguments
   and grouped inner calls locally, rather than relying on an earlier lesson.

All ten corrected complete bodies were reread. A subsequent canonical formatter
wrapped the Map and SizeError example sources. Exactly six bodies changed:
Map, SizeError and NonPositiveSize in both locales. The separate reader reread
all six complete bodies and independently verified the other twelve HTML files
were byte-identical, all eighteen prose/heading sequences were unchanged, and
all native panels matched their decoded Playground seeds. The six affected
panels also matched the final canonical source bytes. No behavior or output
changed, and no bounded reading blocker remains.

Final artifact: `/tmp/collection-types-render/`, also retained under
`target/collection-type-reader-review/rendered/`. Only the eighteen routes in
`scope.json` are new reading scope. The reader resolved 140 local article and
navigation target occurrences with no missing target or authored-label/H1
mismatch, including 74 authored exact-title links with adjacent purposes. It
read selected optional follow-ons for usefulness without adding those controls
to acceptance. Generic Language Reference landing links were outside the
reduced fixture. `format-reread-check.json` records the final byte/seed checks.

The author's separate evidence is
[collection-type-reader-verification.md](collection-type-reader-verification.md):
ten tests / 1,161 assertions for the focused batch; six native/strict-TS/WASM
source sets, eight rejection fixtures, exact guarded dispatch, rendering and
static checks. Separate runtime and regression suites are recorded there.
These execution results do not belong to the independent reader review.

### Data and validation: fifteen identities, thirty bodies

- `/docs/library/maybe/`
- `/docs/library/maybe/function/{withdefault,orelse,traverse}/`
- `/docs/library/either/`
- `/docs/library/either/function/{fold,mapleft,traverse}/`
- `std/prelude::Monad::flatMap`: `/docs/library/prelude/function/flatmap/`
- `/docs/library/validation/`
- `/docs/library/validation/function/{valid,invalid,invalidmany,fromeither,toeither}/`

The coordinating reader read all thirty complete initial bodies sequentially.
Nicknames supply the concrete present/missing/empty-string distinction; zero
remains present. withDefault and orElse explain their result shapes and eager
argument evaluation, and distinguish a delayed fallback or a branch performing
the next lookup. The TS ??/undefined and eager helper comparisons disclose that
difference rather than hiding it.

The seat examples explain Either's returned Left/Right data, local match
consumers and why try/catch alone does not inspect it. fold's selected callback,
mapLeft's unchanged success and flatMap's dependent successful step have their
argument order and skipped bodies explained. The generic Monad signature comes
after the concrete Either example. The exact site identity has a broad
flatMap explanation for Maybe/Either/collections instead of claiming only
collection flattening; unchanged upstream metadata remains a separate debt.
The TS examples retain ordinary branching and early returns.

Validation starts with two independent field checks, all four success/failure
combinations and ordered error display. valid/invalid create result values;
they do not perform the application's checks or throw. invalidMany requires a
NonEmptyList rather than an ordinary List that happens to contain elements.
fromEither retains its E as one error, including an array-valued E; toEither
retains the entire ordered NonEmptyList and explains the changed failure type.
The direct TS two-if/errors-array form and the nonempty tuple comparisons are
fairly described, including where the simple TS form is already sufficient.

Both traversal articles explain source order, empty success and the fact that a
Nothing/Left does not stop later callback evaluations. Their TS versions map
all checks first, so an early-return loop is not falsely presented as having
the same evaluation contract. Later runtime defects are not claimed to be
avoided. The Either traversal keeps the first failure, while Validation is the
alternative for retaining every independent error.

One paired module correction was requested. The initial Validation setup said
that the type after -> was the return type, but bookingLabel and validateBooking
also have an arrow between their two parameters. The final text identifies name
and seats in order, the separating arrow, the final result arrow and the returned
expression after =. Both corrected **complete module bodies** were reread, with
all programs, outputs, comparison and remaining rules in context. Independent
byte comparison confirms the other twenty-eight reviewed files are unchanged.
No remaining bounded reading blocker was found.

Final artifact: `/tmp/seseragi-data-validation-authored/rendered-revision2/`.
Independent checks pass for 54 authored exact-title links with adjacent purposes,
30 canonical native panels and exact Playground seeds, 30 TS panels, 60 output
panels, 60 same-identity locale links, 24 exact callable declarations/owners and
300 local-fragment occurrences. Module H1s match their owners. The twelve extra
fixture files are linked destinations or unchanged controls, not new coverage.
Evidence is `/tmp/data-validation-independent-review-final.json` and
`/tmp/data-validation-independent-reread-diff.json`.

The author's separate evidence is
[data-validation-reader-verification.md](data-validation-reader-verification.md):
six tests / 1,524 assertions for the full focused suite, including fifteen
native, strict-checked TS and committed-WASM/runtime-harness programs; the final
prose-only refresh passed one rendering test / 1,235 assertions. Canonical
programs and their outputs did not change for that correction. The reader did
not rerun these executions. WASM/runtime-harness checks are not browser UI
verification, and prose-linter scores are not reader acceptance.

### Exact final manifests and scope limits

The two edited-only reviewer checksum manifests use the same convention as
checkpoint 6: sort render-root-relative HTML paths; write the lowercase SHA-256
of each complete file's bytes, two ASCII spaces, its relative path and LF to a
UTF-8 manifest. The hashes here are SHA-256 of the complete manifest bytes:

- Collection types, eighteen files:
  `/tmp/collection-types-reader-reviewed.sha256`, retained as
  `target/collection-type-reader-review/reader-reviewed.sha256`;
  `218510d1541e42ec4e762de3096d66122d46cf01a63bf72081812109d82488b3`
- Data/validation, thirty files:
  `/tmp/data-validation-independent-reviewed30-final.sha256`;
  `a36e2029167625c4c45fdde14239480664c37ab7f319234e72d8941a09185b08`

The collection reader also independently recomputed the author's alternative
rendered hash, using sorted localized route + NUL + HTML bytes + NUL:
`d7ca17453212ae3215dcd63d1e74a8d8409fae4d9428146df843c262a1cd2c66`.
Its author-provided typed-source hash for 29 overlay/helper/dispatcher files,
using sorted repository-relative path + NUL + bytes + NUL, is
`6746ef7053e59e09ce6d0bb61335257f159529ca14da5ead11fa8d6861a822fd`.
These final collection hashes supersede the pre-formatting values in transient
review messages; different hash encodings must not be conflated.

The collection fixture's other 46 localized target/control files, the
data/validation fixture's twelve supporting files, earlier callable/module
readings and all 57 generated collection instance identities are excluded from
new acceptance. Their presence, guarded-dispatch tests or link checks cannot
inflate the 24-identity / 48-body count.

All conclusions are bounded agent explanation/readability/appeal judgments.
Complete-site integration, retained full-site matching and route/navigation
closure remain parent/integrator-owned and unverified by this record. Browser
access remains blocked; no alternate browser route was used. Desktop/mobile
layout, clipping/overflow, keyboard behavior, live execution and real locale
switching remain unverified. Actual first-time human comprehension is still
unverified. No whole-site completion or backlog clearance follows from these
batches, and no public tracker status is changed.

All prior ledger bytes/history, the 106 original checkbox rows and eight
historical checked boxes are preserved. This appendix changes no acceptance
checkbox and authorizes no publication, commit, push or deployment.

## Checkpoint 8: numeric and result-foundation reader review (2026-10-01)

This record adds bounded independent agent reading of **27 existing identities /
54 complete localized bodies**: thirteen numeric identities / twenty-six EN/JA
bodies and fourteen result-foundation identities / twenty-eight EN/JA bodies.
The baseline is checkpoint 7, `15bd8858becffa32d19f55f66d630593ac90c603`, with
intentional uncommitted documentation work. All fifty-four initial bodies were
read by `/root/review_collection_text_docs`; no nested reviewer's reading is
substituted for that coverage. The same reviewer fully reread every corrected
body: four numeric bodies and ten result-foundation bodies. Those rereads do not
increase the unique-body count.

The reviewer used the ordinary TypeScript baseline in
[reader-contract.md](reader-contract.md): familiar types/interfaces, functions,
arrays, objects, conditions and exception handling, with no assumed ADT, match,
FP, Rust or Haskell background. The full HTML main content was read in sequence,
including native and TypeScript programs, separate outputs, all rules and
mistakes, canonical declarations and generated declaration guidance, breadcrumbs,
related choices and section navigation. Necessary explanation was assessed from
the article itself rather than filled in from specification or implementation
knowledge. Source/metadata checks after reading established exact ownership and
display consistency, not comprehension by a human.

### Numeric scope, reading result and corrections

The exact thirteen identities, each with its `/ja` route, are:

- `std/int`: `/docs/library/int/`
- `std/int::{parse,format,checkedAdd,checkedSubtract,checkedMultiply,checkedDivide,checkedRemainder}`:
  `/docs/library/int/function/{parse,format,checkedadd,checkedsubtract,checkedmultiply,checkeddivide,checkedremainder}/`
- `std/float`: `/docs/library/float/`
- `std/float::{parse,format,fromInt,isFinite}`:
  `/docs/library/float/function/{parse,format,fromint,isfinite}/`

The Int module starts with a settings count and an add-five calculation. The
reader can follow whole-string parsing, returned failure alternatives, local
match branches and the distinction between invalid input and an excessive
sum. The parse page explains optional signs, extra leading zeroes, whitespace,
range and negative-zero normalization. Its TS counterpart checks the complete
matched string and safe-integer range; permissive prefix parsing is not presented
as equivalent validation. Detailed diagnostic equivalence is explicitly not
claimed.

The arithmetic leaves state their own operand order, with subtraction amount
first and division/remainder divisor first. Wrapper parameter order and the
library order are distinguished. Just/Nothing and Left/Right are introduced
locally as returned alternatives, with no implication that try/catch consumes
them. Signed quotient truncation, signed remainder, zero-divisor handling,
integer bounds, separate positivity requirements and the difference from
clamping are clear. The TypeScript examples retain direct arithmetic under the
same already-safe-integer precondition; division uses locally explained BigInt
conversion to preserve the quotient at the range boundary. Int format honestly
acknowledges that TS String is already concise.

Float starts with splitting a count between packages and explicitly converting
Int to Float. The pages distinguish type annotation from conversion, exact
conversion from later rounded arithmetic, complete parsing from finite-value
requirements, and canonical stored text from fixed-place or localized display.
NaN, both infinities, signed zero, overflow and underflow have local explanations
and observable consequences. Float format and fromInt retain their real TS
spelling differences rather than adding a padded formatter solely for parity.
The fromInt function explains both arguments, the final result arrow and the
returned expression after `=`. Number.isFinite is a direct, fairly described TS
counterpart, and finiteness is not mistaken for positivity, integrality, the Int
range or exact decimal arithmetic.

The reviewer requested two bounded corrections. Japanese Float format and
isFinite called `()` a value named Unit; they now identify it as a value of the
Unit type. The isFinite paragraph also identifies the infinity functions as
returning those values. The TypeScript checkedDivide counterpart contained a
redundant zero-normalizing branch after BigInt division, where negative zero
cannot arise; it now interpolates the result directly. The checkedRemainder
counterpart retains its actual JavaScript remainder normalization and explanation.
Both corrected Japanese Float bodies and both checkedDivide bodies were fully
reread. Independent byte comparison confirms the other twenty-two bodies were
unchanged. No remaining bounded reader blocker was found.

The Float module openly describes the current HalfUp implementation defect:
incorrect rounding immediately below 0.5, changes to some large integral values,
and erroneous out-of-range conversion results. It distinguishes those faulty
results from the intended midpoint rule. **toInt and roundIntegral remain gated
and are not accepted by this record.** Their links explicitly identify the
limitation; no reviewed program depends on HalfUp. This is not a runtime fix or
acceptance of those two APIs.

Final numeric artifact:
`target/numeric-reader-review/rendered/`. Its thirty-six HTML bodies contain the
twenty-six edited bodies plus ten supporting/control bodies; the latter are not
new coverage. Exact scope is in `apps/site/scripts/numeric-readers.ts` and
`target/numeric-reader-review/body-manifest.json`. Independent final checks found
zero errors: 74 authored exact-title links with adjacent purposes, 26 native
panels and exact decoded Playground source seeds, 26 TS panels, 52 output panels,
52 same-identity locale links, 22 exact callable declarations/owners, 260 local
fragment occurrences and 222 available in-main library target occurrences. The
last two are occurrence counts, not additional accepted pages. Generic links
outside the reduced fixture are not full-site closure.

Independent proofs and full per-identity report:
`/tmp/numeric-independent-review-final.json`,
`/tmp/numeric-independent-reread-diff.json`, and
`/tmp/numeric-independent-reader-review.md`. The pre-repair snapshot is
`/tmp/numeric-initial-reader-snapshot`.

The author's separate execution evidence is
[numeric-reader-verification.md](numeric-reader-verification.md): six tests /
1,486 assertions for the frozen full suite and two tests / 1,244 assertions for
the corrected native/strict-TS and production-render scope. The full run covers
thirteen native programs, thirteen TS counterparts, exact committed-WASM seeds,
bounded runtime rules and dispatcher controls. The correction rerun establishes
the final counterpart and panel bytes. These overlapping runs are not summed,
and the reviewer did not independently execute the programs. Untouched numeric
operations, std/math and existing built-in-type pages remain outside the batch.

### Result-foundation scope, reading result and corrections

The exact fourteen identities, each with its `/ja` route, are:

- `std/prelude::{Maybe,Either}`:
  `/docs/library/prelude/type/{maybe,either}/`
- `std/prelude::{Just,Nothing,Right,Left}`:
  `/docs/library/prelude/constructor/{just,nothing,right,left}/`
- `std/validation::Validation`:
  `/docs/library/validation/opaque-type/validation/`
- `std/validation::{Valid,Invalid}`:
  `/docs/library/validation/constructor/{valid,invalid}/`
- `std/maybe::sequence`: `/docs/library/maybe/function/sequence/`
- `std/either::{mapRight,bimap,sequence,swap}`:
  `/docs/library/either/function/{mapright,bimap,sequence,swap}/`

Maybe, Just and Nothing begin with supplied or absent nicknames, retaining empty
strings and zero as real values. Type positions, annotations, payload extraction,
no-import Prelude ownership and complete match alternatives are local. Just does
not validate contents or delay its argument. Nothing has no argument or reason,
and the invalid Nothing () call is explained. The TypeScript versions use ordinary
undefined checks without padding, and the benefit is shared present/missing
operations rather than a claim of stronger application validation.

Either and its constructors use accepted seat counts and rejection reasons.
Local if/match consumers explain which payload is available. Right 0 demonstrates
that constructing a successful shape does not validate positivity; Left returns
data without printing, throwing or stopping the caller's later steps. Both TS
object alternatives and their ok field are explained locally. The possible
unoccupied side still has a type. The Either type article states that each API
owns the meaning of its two sides, including uses other than success/failure.

Validation and its constructors display an accepted name or existing ordered
form errors. They explicitly separate constructing results from performing the
checks. Invalid requires NonEmptyList rather than an ordinary List that happens
to contain values; singleton/cons, the List backtick and ordered Array/text display
are explained. The TS nonempty tuple condition is described locally and retained
in both comparisons. Type placeholders, imported ownership, error-versus-success
type positions and public constructors for the opaque type are accounted for.
Advanced independent-check composition is mentioned only after the complete
construction/consumption example and has an optional purpose-labelled destination.

The two sequence articles distinguish already-obtained results from deferred
lookups. They show ordered all-success results, missing/failed positions and empty
success; Either preserves only the first error. Returning that failure is not
claimed to prevent evaluation that already built the input Array. TS examines the
same already-created values. mapRight and bimap explain selected callback bodies,
argument order, unchanged branches, eager argument expressions and the retained
intermediate Either. The TS examples use direct conditionals; the intermediate
result difference is disclosed instead of treating more TS infrastructure as
necessary.

Swap uses a consumer with a reversed existing branch contract. Both the full
article and its entire generated declaration-reading paragraph correctly state
that original Right becomes new Left, original Left becomes new Right, and payload
meaning is unchanged. A moved reason remains the original rejection reason;
swapping does not validate, retry or recover the booking. The TS adapter preserves
the same information using a locally explained kind field. The text explicitly
chooses ordinary branching when no reversed contract needs adapting.

Four bounded correction categories were resolved:

- Remove the irrelevant show explanation from Maybe, Nothing, Validation, Valid
  and Invalid in both locales, whose displayed programs do not use show
- Identify the std/validation ownership of the valid, invalid and invalidMany
  helper functions instead of using an unintroduced validation alias
- Align Validation's Japanese generated opaque-type advice with its public
  Valid/Invalid constructors and match, instead of directing readers only to
  functions; the English advice already allowed constructors and operations
- Remove Nothing's Japanese temporal wording that suggested a bound absence
  would later change into a string; describe the absent value and alternative
  type instead

All ten changed complete EN/JA bodies were reread. Independent byte comparison
confirms the other eighteen bodies unchanged, including both full swap bodies.
No remaining bounded reader blocker was found. The declaration-reading overrides
are selected by exact identity, owner module, namespace and kind: std/either::swap
as a value/function, and std/validation::Validation as a type/opaque-type. They
preserve canonical signatures and generic type-parameter guidance. Inspection of
that selection after reading is a bounded implementation check, not acceptance
of unrelated generated declarations.

Final result-foundation artifact:
`/tmp/seseragi-result-foundations-authored/rendered-revision2/`.
The initial tree remains at `/tmp/seseragi-result-foundations-authored/rendered/`;
the reader retained its own selected-file snapshot at
`/tmp/result-foundation-initial-reader-snapshot`. The fixture has seventy HTML
files, of which forty-two are older linked destinations or controls, excluded
from new acceptance. All twenty-three supporting generated instance identities
remain unreviewed. Transformer pages and the preserved StateT.get control are not
new reader coverage.

Independent final checks have zero errors: 76 authored exact-title links with
adjacent purposes, 28 native panels and exact source-seeded Playground links,
28 TS panels, 56 output panels, 56 same-identity locale links, 28 canonical
declarations/owners/namespaces/kinds, 280 local-fragment occurrences and 216
available in-main library target occurrences. Proofs and complete per-identity
report are `/tmp/result-foundation-independent-review-final.json`,
`/tmp/result-foundation-independent-reread-diff.json`, and
`/tmp/result-foundation-independent-reader-review.md`.

The author's separate evidence is
[result-foundation-reader-verification.md](result-foundation-reader-verification.md).
Its successful initial frozen suite passed seven tests / 1,600 assertions. The
final repaired suite passed eight tests / 1,652 assertions, and six affected prior
tests / 1,555 assertions also passed. These are separate overlapping execution
runs, not a combined score. They cover fourteen native/strict-TS pairs, fourteen
committed-WASM seeds, fifteen native boundary/rejection probes, callback counts,
exact scoped-reading/dispatch guards and real rendered bodies. Earlier lock-update
timeout attempts remain failed attempts, not accepted proof. Native and TS
programs/output were unchanged by the reader corrections. The reviewer did not
independently execute the programs.

### Final manifests, integration boundary and remaining acceptance

The two edited-only reviewer manifests use the checkpoint 6/7 checksum-line
convention: sort artifact-relative selected HTML paths lexicographically; write
lowercase SHA-256 of the complete HTML bytes, two ASCII spaces, the relative path
and LF to a UTF-8 manifest. The aggregate hashes below are SHA-256 of those
manifest bytes:

- Numeric, twenty-six files:
  `/tmp/numeric-independent-reviewed26-final.sha256`;
  `621ea5a32193c1e46aad856732a8273c95db41401837d40f7a4e7ec774358923`
- Result foundations, twenty-eight files:
  `/tmp/result-foundation-independent-reviewed28-final.sha256`;
  `30dc290299152b0b58b5d34a409143aa9c692cff57d27af048fd9b5df9ed1c6d`

The result author's different JSON body manifest, `body-manifest-final.json`,
hashes to `96cd106eb95877fe23269466cd6a61cf48bacfc79e4775ac34e0e338cf425854`.
That encoding is not interchangeable with the reviewer's checksum manifest.
Supporting/control files are excluded from both reviewer manifests.

The parent separately refactored the shared editorial dispatcher into shallow,
lazy fallback helpers to avoid the deep match-chain parsing cost, preserving
exact selection order and fallbacks. That integration change is **not** counted
as this reviewer's semantic or full-site acceptance. The parent/integrator owns
its tests, full-site output matching, route/navigation closure and publication.
A source-seeded link check or committed-WASM runtime harness is not browser UI
verification. Advisory Japanese prose lint is also separate from reader judgment.

The conclusion is a bounded agent assessment of understanding, concrete usefulness
and fair comparison for the fifty-four bodies. Actual first-time human
comprehension remains unverified. Browser access remained blocked; no bypass or
alternate browser route was used. Desktop/mobile layout, clipping, overflow,
keyboard use, actual locale switching and clicked navigation round trips remain
unverified. Whole-site build/acceptance and live publication verification remain
outside this record. Neither the HalfUp defect nor the gated rounding APIs,
control/instance/transformer pages or the broader backlog are closed by this
appendix.

All earlier ledger bytes and history, the 106 original checkbox rows and eight
historical checked boxes are preserved. This appendix changes no checkbox. No
public comment, tracker update, commit, push or deployment was performed by this
reviewer; the separately authorized publication has another owner.
