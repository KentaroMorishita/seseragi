# Data operations and expressions verification (#713)

Verified 2026-10-01 for [#713](https://github.com/KentaroMorishita/seseragi/issues/713),
starting from local checkpoint `0ee111373e56f0316a5fc26b9f893d2cef4c7a0b`.
This record covers six existing article identities and their Japanese mirrors.
It is scoped implementation and agent reading evidence, not browser, deployed
site, whole-corpus, or first-time human-reader acceptance.

## Scope and reader-facing changes

The catalog contains seven Data and five Expressions articles. Records, structs,
ADTs, blocks, conditionals and evaluation belong to completed earlier batches.
This pass changes only the remaining four Data and two Expressions articles:

| Route below `/docs/language/` | Reader task | Exact primary output (`\n` separates lines) |
| --- | --- | --- |
| data/tuples-arrays-and-lists/ | Extract a tuple, handle indexed absence, and prepend a list element | `answer\n42\nJust 20\nNothing\n20\n` followed by the List display `` `[Ren, Aki, Mio]\n `` |
| expressions/lambdas/ | Pass a score transformation capturing a bonus, then partially apply a two-input lambda | `[12, 22, 32]\n8\n` |
| expressions/ranges-and-comprehensions/ | Select even squares, compare endpoints, and inspect nested-generator order | `[4, 16, 36, 64]\n[4, 16, 36]\n[(1, 1), (1, 2), (2, 1), (2, 2)]\n` |
| data/newtypes/ | Construct, extract, and reconstruct a UserId | `42\n43\n` |
| data/impls-and-methods/ | Define an immutable progress update and a receiver-only reader | `2\n3\n` |
| data/operator-overloads/ | Extract two Scores, add their integers, and return a Score | `42\n` |

Every page uses the existing `ReaderCopy` / `readerPage` and `explainPage`
components. Prose remains in page-owned typed EN/JA modules. The new source
metadata helper is not a page-content format or another renderer. The additive
multi-file explanation change in #711 requires no special handling here;
these complete single-file examples use `WithoutExample` so the introductory
question remains separate from the example in the normal reader body.

Five earlier primary examples contained declarations without an executable
entry. The new programs can all run independently. The collections example
removes the unexplained `do` / `_ <-` wrapper and now prints `selected`, which
its explanation previously computed but did not display. Existing canonical
files are not overwritten; the page now references its new verified source.

The newtype page owns concrete wrapping and extraction. The separate #712
`types/newtypes` page owns distinct-ID identity and the alias comparison.
The impl page starts with a non-generic `Progress`, linking generic impls as a
later detail. No Array/List API articles or completed object/pattern pages were
edited. The Score overload example explains constructor, pattern, `self`,
`impl`, return type and standard `Add` correspondence locally before discussing
trait details. It does not depend on a money/domain-validation framework.

## Canonical sources, errors, and repairs

`apps/site/scripts/data-operations.ts` supplies six valid and six invalid
examples through the existing `canonicalExample` helper. Sources are
`apps/site/examples/src/language/data-operations-<key>.ssrg`; rejected files use
the same basename in `apps/site/examples/invalid/src/language/`.

| Key | Rejected source | Observed code | Verified correction and result |
| --- | --- | --- | --- |
| collections | String `"20"` in `Array<Int>` | SES-T0101 | Use `20`; prints `[10, 20, 30]` |
| lambdas | String body under `Int -> Int` | SES-T0101 | Use `value + 1`; prints `3` |
| comprehensions | Int guard in a typed function | SES-T0101 | Use `value % 2 == 0`; prints `[4, 16]` |
| newtypes | `raw 42` where UserId is required | SES-T0101 | Use `raw (UserId 42)`; prints `42` |
| methods | First method parameter named `progress` | SES-T0503 | Rename declaration and use to `self`; prints `2` |
| operators | Int returned where Score is declared | SES-T0101 | Wrap with `Score (...)`; prints `42` |

All valid programs are written as `main.ssrg` in fresh temporary directories and
pass native `lint` and `run` with exact stdout including the final newline.
All invalid programs fail lint with status 2 and the recorded code. Every
stated repair is run and checked. Rejected panels intentionally have no runnable
Playground link.

Each valid Playground URL decodes to the exact canonical source bytes. The
committed WASM compiler accepts those decoded programs, and the existing
Playground execution runtime produces matching output. Its capture API trims
the final newline; the native test checks the complete stdout. No browser is
used for this source/compiler/runtime boundary check. SHA-256 metadata and
highlight-token text are checked against the source files as well.

## Preserved rules and specification checks

The previous detailed rule, typing, evaluation, invalid-case and diagnostic
paragraphs were retained or reworded without removing their meaning, after the
worked example. Old example-specific references were aligned to the new source.
The rule sources are `docs/spec/03-data-and-expressions.md` §§3.6, 3.8, 3.9,
3.11–3.13, with type/visibility context in the linked existing articles.

- Tuple arity, homogeneous/empty collection typing, index placement and Maybe
  results, distinct Array/List types and explicit conversions remain explained.
  The later cost paragraph distinguishes repeated Array-tail copying from
  shared List tails and explains the `head` / `tail` names.
- Lambda context/parameter annotations, one-at-a-time arguments, outer lexical
  names, delayed body evaluation, tuple distinction and foreign-callback
  conversion rules remain present.
- Range endpoints, step/empty-descending behavior, Array/List result spelling,
  Iterable requirements, Bool guards, pattern filtering, nested order and
  source-expression evaluation count remain present. Pure comprehensions do
  not accept an Effect source.
- Newtype identity, explicit construction/extraction, once-only input evaluation,
  opaque visibility, representation freedom and lack of inherited operations or
  field/update syntax remain present. The article does not claim validation or
  ID uniqueness from wrapping an integer.
- Impl ownership, static receiver resolution, required `self`, generic details,
  duplicate names, `pub`, immutable returns, receiver-only invocation and the
  distinction from trait `instance` declarations remain present.
- Operator return types, standard-trait correspondence, static selection,
  left-to-right operand evaluation, generic constraints, fixed precedence,
  supported operator set, `!=` from `==`, `Ord`, and duplicate implementations
  remain present.

## Current compiler gap: top-level comprehension guards

[The exact probe comment](https://github.com/KentaroMorishita/seseragi/issues/713#issuecomment-5932468384)
records these minimal sources and all native lint/build/run results. Open
`comprehension`, `guard`, and `Bool top-level` issue searches found no matching
owner. #683 concerns lambda generalization, and #709 concerns declaration
names/annotations; neither was represented as owning this separate gap.

```seseragi
let squares: Array<Int> = [value * value | value <- 1 ..= 4, value]
pub effect fn main = println (show squares)
```

This invalid Int guard is accepted by both tested CLIs: lint/build/run all exit
0, and stdout is `[1, 4, 9, 16]\n`. Replacing the final `value` condition with
`"yes"` is also accepted with the same result. In contrast:

```seseragi
fn squares -> Array<Int> = [value * value | value <- 1 ..= 4, value]
pub effect fn main = println (show (squares ()))
```

The function form rejects in all three commands with exit 2 and SES-T0101,
“Match guard not bool”, expected Bool / actual Int. Its documented repair to
`value % 2 == 0` succeeds in all three commands and prints `[4, 16]\n`.
The public article retains the normative Bool rule and a short versioned caveat;
it does not teach the accepted invalid form as an alternative.

Both probe binaries were checked with Bun 1.3.9 on Linux x64:

- Published CLI: `0.61.19 (release, commit a8641b5a81a4)`.
- Integration-rebuilt development CLI: `0.61.19 (development, commit 90cd575c1dc4,
  dirty)`. The saved `--version` output determines this provenance; it is not
  the earlier debug executable. Compiler/runtime/Cargo contents are unchanged
  from `ab42da841d76`. The initial issue-comment binary label was corrected in
  place; the sources and results did not change.

No compiler, runtime, specification, or security behavior was changed for this
work. No separate compiler issue was created for this finding.

## Final checks and bounded reading review

- `apps/site/tests/data-operations.test.ts`: **10 pass, 0 fail, 515 assertions**
  using the published Linux CLI and Bun 1.3.9. The final canonical-source run
  took 14.60 seconds. It includes all native examples/errors/repairs, WASM
  execution, and twelve actual EN/JA page renders.
- Scoped TypeScript, Biome, changed-page and fixture Seseragi formatting, and
  `git diff --check` passed. Registration and eventual lock/aggregate updates
  are coordinated with the integration owner.
- `target/site-data-operations-review` contains the twelve rendered pages.
  Source panels, output, titles, route identities, locale links, heading IDs,
  exact-source Playground seeds and escaped HTML are checked. This tiny catalog
  intentionally has no sidebar areas and cannot establish complete navigation,
  route closure, CSS/mobile appearance, keyboard behavior or deployed behavior.
- The author read all six Japanese and six English rendered bodies, corrected
  stale example names, explained templates and trait terminology locally, and
  simplified a repetitive collection-cost paragraph. The pinned yomiyasu
  guidance informed two bounded advisory passes on rendered Japanese prose,
  excluding code/output. Residual warnings are mainly repeated polite endings
  and necessary syntax/type distinctions, not acceptance scores.
- Independent rendered-body review is tracked separately in the reader-review
  ledger. No first-time human reader, browser, full-site build or publication
  was performed in this scoped task. The issue acceptance remains open.
