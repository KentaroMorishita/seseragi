# Remaining type rules: concrete reader pass

Local scoped implementation for the seven remaining type pages under #601/#630,
following the [reader contract](reader-contract.md). The next work-item remains
a local draft while public issue/comment permission is pending. This pass follows
#712's everyday-TypeScript examples without assuming knowledge of kind, rank,
variance or runtime type representations.

## Scope and retained rules

The seven route identities under `/docs/language/types/`, their Japanese mirrors,
existing page IDs and original rule/identity/checking/diagnostic anchors remain.
Previously completed type pages are not edited. Each first example is a complete
program with observed output, a rejected modification and a repair. A local
purpose-labelled link provides the next relevant existing page.

Original detailed page bodies and their example walkthroughs remain after an
explicit transition. Source comparison against `882a6814` preserves twelve of
fourteen original locale copy bodies exactly. The two recursion copies qualify
an inherited sentence: the synchronous constant-stack self-tail-call guarantee
is for ordinary pure `fn`. This matches specification §14.8 and the lowering's
`!is_effect` guard; Effect continuation rules are not rewritten or conflated.
Two inherited Japanese captions were translated rather than leaving English
"local mutual recursion group" / "requirement-polymorphic Effect alias".

Six old declaration-only sources newly have standalone Playground links
suppressed; requirement-merge already had suppression. Their source bytes remain
unchanged. Complete examples use the canonical source loader and seeded links.

## Executable evidence

[type-limits.ts](../../../apps/site/scripts/type-limits.ts) records exact sources,
outputs and diagnostics. The test copies every complete program to an independent
temporary entry file and runs it. Negative programs are linted independently.

| Route | Output lines | Rejected modification |
| --- | --- | --- |
| `erasure-and-runtime-representation/` | `43`, `ready!` | A type parameter used as a returned value: `SES-N0001` |
| `generic-impls-and-methods/` | `43`, `ready!` | Box<Int>.get received as String: `SES-T0101` |
| `kinds/` | `Just 42`, `Nothing`, `[1, 2]` | Int supplied where a one-argument constructor is required: `SES-T0604`, with a follow-on annotation mismatch |
| `let-polymorphism-and-rank/` | `42`, `ready`, `Just 42` | An arbitrary callback parameter A assumed to be Int inside the generic body: `SES-T0101` |
| `recursive-declarations/` | `True`, `False`, `True` | Reference to a later local function outside rec: `SES-N0001` |
| `requirement-merge/` | `Hello!` | Different field types at the same requirement name: `SES-E0001` |
| `variance/` | `Aki`, `[Aki]` | Array element type changed by assignment alone: `SES-T0101` |

The recursion example converts negative input to its magnitude before entering
the small mutual-recursion demonstration. Its text explicitly warns against
using this implementation for large integers and distinguishes the retained
older declaration example, which does not normalize negative input.

The requirements example uses an explicit `effect fn` contract and a checked
function-type witness:

- `Configured<R, A> = Effect<R & { settings: Settings }, Never, A>`
- `configured: Unit -> Configured<{ theme: Theme }, String>`
- The required environment therefore contains both `theme: Theme` and
  `settings: Settings`; failure is `Never`, success is `String`
- `provide` supplies those values before the computation runs; the process
  entry retains the Console requirement for printing

This demonstrates a type requirement merge, not a runtime record merge. An
earlier ordinary-fn do probe demanded one common Effect constructor and rejected
the narrower individual service operations. The final explicit effect-function
contract is the verified form; no compiler behavior was changed to obtain it.

## TypeScript comparisons and implementation gaps

Two complete comparisons are strict-typechecked and executed:

- `generic-methods`: ordinary `class Box<A>`, a readonly payload and method-local
  `replace<B>` produce the same `43` / `ready!` output. Receiving number.get() as
  string reports `TS2322`
- `record-array-view`: explicit map projection produces the same `Aki` / `[Aki]`
  output; a numeric name reports `TS2322`. A separate checked/run witness also
  verifies that TypeScript accepts `const view: Named[] = users`, preserves the
  element reference and leaves its id intact. The prose states this difference
  instead of claiming the TS assignment deletes fields or is rejected

[Issue #683](https://github.com/KentaroMorishita/seseragi/issues/683) remains open.
A fresh exact probe of the unannotated let-bound identity lambda reports
`SES-T0101` for its unresolved parameter. The page retains the implementation
caveat and uses an explicitly generic named function. This is not presented as
a language prohibition or a compiler fix.

[Issue #681](https://github.com/KentaroMorishita/seseragi/issues/681) is closed,
and the collection-invariance rejection remains reproducible here. No obsolete
"current compiler limitation" is restored to the variance page.

## Scoped verification and review boundary

```sh
SESERAGI_TYPE_LIMITS_OUTPUT=target/site-type-limits-review \
  bun test apps/site/tests/type-limits.test.ts
bun apps/site/scripts/check-content-map.ts
```

2026-10-01 author checks:

- Three focused tests passed, 449 assertions: identities/local explanations,
  seven native outputs, seven genuine rejection fixtures, two strict TS
  comparisons and mutations, the TS array-view difference, fourteen locale
  renders, exact canonical source/output, retained anchors, locale links and
  complete/declaration link metadata
- Scoped Biome and the site's configured TypeScript flags passed for the new
  registry/test/comparisons; Seseragi sources were canonically formatted
- Current source labels for all internal page-title links in these seven pages
  match the actual destination title in both languages
- Content map remains 351 unique routes
- The corpus explanation/title/coverage run had eleven passes and one failure
  in the concurrently edited `model/no-hidden-danger` explanation-wrapper
  contract. That other writer's source and the shared test were not changed here;
  final integration must reconcile and rerun the corpus gate
- Final targeted recursion EN/JA production rendering passed after the pure-fn
  guarantee clarification; independent whole-body review is pending

The focused renderer uses the production `renderDocument` and the relative page
import closure, with an empty navigation-area catalog and no generated reference
modules. It verifies the article output, not complete production navigation,
visual desktop/mobile layout, browser interaction or actual-reader acceptance.
No whole-site build or browser-restriction workaround was attempted by this batch.
