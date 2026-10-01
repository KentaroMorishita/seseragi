# Data, alternatives and matching reader pass (#705)

Status: local implementation and scoped technical evidence. This is not a full
site/browser acceptance or actual first-time-reader sign-off. The batch follows
[#705](https://github.com/KentaroMorishita/seseragi/issues/705) and the
[reader contract](reader-contract.md). The #702 text/limited-HTML feedback is
agent review, not a substitute for an actual reader or browser session.

## Scope and explanations

The ten existing English routes below retain their IDs and Japanese mirrors.
Ordinary objects lead to records, named structs and compatibility rules. An
ordinary conditional leads to a non-generic Delivery with Pending/Shipped
alternatives, value construction and match. Field extraction leads to patterns
that can always succeed. Type constructors and closed service requirements are
later details, each explained locally rather than required for basic objects.
This is a dependency map, not a new mandatory lesson sequence.

All twenty original locale `copy` bodies and all ten detailed page bodies were
compared with `90cd575c`: their deep-rule prose, IDs and section composition were retained. The later
independent review corrected two inherited code captions: the nominal example
is not recursive and the binding example is not nested. Those caption changes
are intentional accuracy fixes, in addition to function naming/formatting. New explanations,
complete examples, outputs and rejected cases appear before the detailed rules.
Old declaration-only examples remain available and are identified as fragments.
Their eight standalone Playground links are disabled. New complete examples
keep source-seeded Playground links.

## Canonical results

Routes are below `/docs/language/`. Source paths and exact IDs are recorded in
[data-choices.ts](../../../apps/site/scripts/data-choices.ts), reused by the
site's existing canonical-example pipeline and the focused test.

| Route | Exact output lines | Rejected case / observed code |
| --- | --- | --- |
| `data/records/` | `10`, `42`, `Aki` | Duplicate explicit field / `SES-T0101` |
| `data/structs/` | `Aki`, `Mio`, `1` | Missing required field / `SES-T0101` |
| `data/algebraic-data-types/` | `not shipped`, `tracking: JP42` | Int payload where String is required / `SES-T0101` |
| `types/closed-records/` | `ready` | Explicit entry contract omits required Console / `SES-E0001` |
| `types/nominal-and-structural-types/` | `Aki`, `Mio`, `Notebook` | Product passed where Customer is required / `SES-T0101` |
| `types/type-constructors/` | `42`, `ready` | String field in Box<Int> / `SES-T0101` |
| `expressions/conditionals/` | `500`, `0` | Int used as Bool condition / `SES-T0101` |
| `patterns/binding-rules/` | `Aki: 42` | Duplicate name in one pattern / `SES-N0002` |
| `patterns/irrefutable-patterns/` | `Notebook: 2`, `0`, `10` | Array-head pattern in let / `SES-T0101` |
| `patterns/match/` | `not shipped`, `tracking: JP42` | Missing Pending branch / `SES-T0301` |

The two Delivery pages use one canonical runnable source. The TypeScript
counterpart uses an ordinary union of objects with a `kind` field and `switch`.
The article explains the field and union locally; they are not assumed prior
knowledge. Both programs print the same two messages. With strict checking and
an explicit result type, TypeScript also rejects a wrong payload (`TS2322`) and
a removed pending branch (`TS2366`); the text does not claim this checking is
exclusive to Seseragi.

The records and structs entry wrappers now use one interpolated `println`
instead of `do` / `_ <-`, preserving output. The other initial examples likewise
avoid requiring effect-sequencing knowledge. #703's record example explanation
was coordinated to match the same updated source, not copied into another
example system.

Closed records are explicitly a service-requirement detail. The tested program
uses `with Console`; omitting it from the explicit signature demonstrates the
observed missing-requirement diagnostic. An unused generic alias is not counted
as evidence that a host can execute its requirement. No compiler or runtime code
was changed to fit the prose.

## Bounded independent reading result

An independent agent read the twenty rendered English/Japanese article bodies
without filling explanation gaps from the source or specification. The first
pass found no core explanation blocker and identified four small corrections:
remove inaccurate “recursive” and “nested” example captions, and make the
TypeScript-panel and rejected-match references direction-neutral. Those fixes
were applied; all twenty pages were rendered again and the affected text was
reread. The bounded reread found no remaining core-body explanation blocker.
This is agent text/HTML review, not actual first-time-reader acceptance or
browser/layout approval. The shared core reader ledger remains separately owned.

## Checks and remaining boundaries

Run the bounded checks with:

```sh
bun test apps/site/tests/data-choices.test.ts
bun test apps/site/tests/explanations.test.ts \
  apps/site/tests/reference-titles.test.ts apps/site/tests/coverage.test.ts
bun apps/site/scripts/check-content-map.ts
```

2026-10-01 local results:

- Three data tests passed with 504 assertions: stable IDs/bilingual introductions;
  independently executed complete programs and rejected cases; strict TypeScript
  comparison; twenty production-rendered locale pages; exact first code bytes,
  expected output, retained detailed section IDs, locale alternates and the
  complete-program/fragment Playground-link distinction
- Existing explanation/title/coverage tests: ten passed, 13,487 assertions
- Content map: 351 unique routes; no normative reference leaves added
- Focused TypeScript, Biome, Seseragi formatting and `git diff --check` passed

The focused renderer compiles only each article's import closure in a temporary
package and uses a catalog without navigation areas. Saved output at
`target/site-data-review` is explicitly labelled with this limitation. It does
not verify full sidebar/previous-next behavior, all production links, CSS
geometry, desktop/mobile readability, browser interaction or console errors.
The helper's per-subprocess budget is 180 seconds for this larger twenty-page
closure; existing smaller callers retain their 90-second default.

Japanese was authored first using the pinned yomiyasu technical-writing
principles, then paired with English. Its advisory linter is not installed here
and was not run. Agent text/HTML reading feedback must be recorded separately
from technical checks and actual-reader acceptance. No core ledger checkboxes
are changed by this report.

Semantic sources: [record and type identity](../../spec/02-types.md),
[data, expressions and patterns](../../spec/03-data-and-expressions.md), and
[effects](../../spec/05-effects.md). These are authoring provenance; the articles
explain the required ideas locally.
