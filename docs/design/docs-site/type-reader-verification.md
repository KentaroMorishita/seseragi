# Type names and reuse reader pass (#712)

Local implementation and scoped evidence for
[#712](https://github.com/KentaroMorishita/seseragi/issues/712), following the
[reader contract](reader-contract.md). Existing TypeScript knowledge is an
entry point; generic type parameters, tuples and named alternatives are
explained where used rather than assumed. This is not actual-reader sign-off.

## Scope and retained reference

Ten existing identities under `/docs/language/types/`, with matching Japanese
pages, cover type checking, optional fields, aliases, distinct IDs, explicit
conversion and reuse through type parameters. The six type pages already
revised in #701/#705 are untouched. No model, trait or other data page is edited
by this batch.

The complete first examples have a public main and concrete output. Their
walkthroughs explain input/result relationships rather than leading with kind,
rank, inference algorithms or trait theory. Deeper original rules and examples
remain, with an explicitly labelled transition and their original walkthroughs.
Each new introduction has a purpose-labelled link to a relevant existing page.

Source comparison against `0ee11137` confirms all twenty original locale copy
bodies and the lexical content of all ten detailed page functions are retained,
with two bounded corrections: numeric conversion text now identifies
`std/float.fromInt` instead of an unspecified `toFloat`, and the English optional-
field rule explicitly distinguishes the outer absence in `Maybe<Maybe<String>>`
from a present field storing `Nothing`, matching the Japanese rule. The specification's
wording is not edited. The route `type-identity-and-coercion` retains its existing
ID, `language.types.identity-coercion`.

Nine old declaration-only examples retain their exact source bytes but no longer
advertise an independently runnable Playground link. The new complete examples
use the canonical source loader and seeded Playground links. Existing rule,
identity/representation, checking/inference, diagnostic and related-rule section
IDs remain available.

## Native examples and rejected changes

Exact source paths, example identities, outputs and expected diagnostics are
recorded in [type-readers.ts](../../../apps/site/scripts/type-readers.ts).
Each source is copied to its own temporary entry file before execution/checking.

| Route | Output lines | Rejected change |
| --- | --- | --- |
| `type-system/` | `1200` | String price passed to Int parameter: `SES-T0101` |
| `optional-record-fields/` | `Aki`, `A` | Optional nickname passed where required: `SES-T0101` |
| `generic-aliases/` | `42`, `ready` | String payload in Named<Int>: `SES-T0101` |
| `newtypes/` | `user: 42` | OrderId passed to UserId parameter: `SES-T0101` |
| `type-identity-and-coercion/` | `21.0` | Int passed directly to Float parameter: `SES-T0101` |
| `generic-functions/` | `21, 21`, `ready, ready` | Int argument after explicit String choice: `SES-T0101` |
| `polymorphism/` | `1`, `Aki` | Int and String assigned to the same A within one call: `SES-T0101` |
| `type-parameter-scope/` | `42`, `ready, ready` | A referenced after its function declaration ends: `SES-N0001`, followed by a mismatch diagnostic |
| `generic-structs/` | `43`, `42!` | Box<Int> spread update replaces payload with String: `SES-T0101` |
| `generic-adts/` | `ok: 42`, `error: missing` | String success value in Outcome<String, Int>: `SES-T0101` |

All twenty fixtures are source-level examples; there are no runtime-error
fixtures in this batch. Error explanations give a correction and preserve
intent: rewrapping an order number as a UserId merely to silence the checker
would not repair a semantic ID mix-up.

A fresh separate probe reproduces the existing #683 limitation: an unannotated
`let identity = \value -> value` reports `SES-T0101` for an unresolved lambda
parameter. The polymorphism page keeps this caveat in its detailed section and
uses a named function with explicit type parameters in the complete example.
No compiler or runtime changes were made.

## Fair TypeScript comparisons

Three complete strict-TypeScript examples use the same inputs and outputs:

- `named-value`: ordinary generic record alias and accessor, output `42` then
  `ready`; a String supplied to Named<number> reports `TS2322`
- `duplicate-value`: ordinary generic function returning a two-position tuple,
  output `21, 21` then `ready, ready`; explicit string with an integer argument
  reports `TS2345`
- `distinct-id`: ordinary object types with literal `kind` fields distinguish
  UserId and OrderId, output `user: 42`; swapping the IDs reports `TS2345`

The text acknowledges that TypeScript supports aliases and generics well. It
explains literal tags without requiring advanced branded types, and distinguishes
TypeScript number from Seseragi's Int/Float instead of presenting a false
inability to perform the same arithmetic.

## Verification

```sh
SESERAGI_TYPES_OUTPUT=target/site-type-review \
  bun test apps/site/tests/type-readers.test.ts
bun test apps/site/tests/explanations.test.ts \
  apps/site/tests/reference-titles.test.ts apps/site/tests/coverage.test.ts
bun apps/site/scripts/check-content-map.ts
```

Current author results (2026-10-01):

- Three focused tests passed, 634 assertions: identity/local-copy, native and
  strict-TS execution, twenty production locale renders, exact canonical code
  panels/output, local section IDs, locale links and standalone-link metadata
- Existing explanation/title/coverage checks: twelve tests passed, 13,492 assertions
- Content map: 351 unique routes, no new normative leaves
- Scoped Biome, canonical Seseragi format checks, the site's supported TypeScript
  flags for the new test/registry, and whitespace checks passed
- An additional ad hoc `--strict` check of the imported test/build closure
  reports the existing `reference.ts:291` module-index narrowing error. The site
  check's configured flags do not enable `--strict`; unrelated source was left
  untouched. The three TypeScript comparison programs do pass with `--strict`

An independent agent read all twenty whole locale bodies and reported no
required fix or core-body blocker. Its optional English clarification to the
nested optional-field rule was applied: `Nothing` means no field, while
`Just Nothing` means a present field storing `Nothing`. A separate native probe
prints exactly those two results. The targeted paired production rerender passed and its new English wording
and unchanged Japanese counterpart were checked. The parent confirmed the
reviewer’s final paired reread passed with no body-level blocker. Shared ledger
and issue-comment integration remain owned by the reviewer. This remains bounded agent text/HTML review, not human-reader or
browser acceptance.

Focused rendering passed. The focused fixture uses production `renderDocument`, only the
relative page import closure, no reference modules and an empty navigation-area
catalog. It verifies article content and local structure, not complete production
navigation, desktop/mobile layout, browser interaction or actual-reader acceptance.
No browser access restriction was bypassed.
