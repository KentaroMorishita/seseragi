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
