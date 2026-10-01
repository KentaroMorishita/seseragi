# Absence, failure and Effect reader pass (#710)

Status: local implementation and scoped technical evidence for
[#710](https://github.com/KentaroMorishita/seseragi/issues/710), using the
[reader contract](reader-contract.md). This is not full-site/browser acceptance
or actual first-time-reader sign-off.

## Scope and ordering

The ten existing English identities below and their Japanese mirrors are kept.
Missing values and failure reasons come before the description/execution of work.
Every article locally explains the alternatives, match, do/bind or callback
notation used in its complete first example. TypeScript try/catch is a starting
point, with propagation and numeric differences stated explicitly.

Task, concurrency, cancellation/resources and Signal pages were not changed.
Their existing references in deeper rules remain available; they are not
prerequisites for reading these initial examples.

All twenty original locale copy bodies and all ten detailed page bodies were
compared with `90cd575c` and retained, including rule/diagnostic section IDs.
Original example walkthroughs are kept under a separate declaration-only heading.
An inherited introduction typo is corrected in both locales and the retained
walkthrough: the inferred output requirement is `{ console: Console }`, not
`Console` alone. A typed function witness verifies compatibility with the exact
normalized signature used in the explanation.

## Canonical execution and diagnostic evidence

Routes are below `/docs/language/effects/`. Exact files, IDs, expected output
and failure stages are recorded in
[failure-readers.ts](../../../apps/site/scripts/failure-readers.ts), used by the
existing canonical-example loader and the focused tests.

| Route | Output lines | Rejected or terminating example |
| --- | --- | --- |
| `pure-expressions/` | `60` | Plain Int returned from effect fn: `SES-T0101` |
| `maybe/` | `Aki`, `anonymous`, `empty:` | Int fallback for Maybe<String>: `SES-T0101` |
| `either/` | `ok: 2`, `error: must be positive` | String in the Int success alternative: `SES-T0101` |
| `effect-type/` | `before`, `run`, `run` | Effect value assigned to Int: `SES-T0101` |
| `effect-functions-contract-form/` | `ready` | Missing Console requirement: `SES-E0001` |
| `effect-functions-inferred-form/` | `ready` | with clause without ->: `SES-P0001` |
| `runtime-boundaries/` | `only main` | Nonpublic main passes lint but execution preparation exits 2 |
| `error-channels/` | `Aki`, `fallback: missing` | Unaligned String and Int failure types: `SES-E0001` |
| `environment-requirements/` | `Hello, Aki` | Empty environment cannot satisfy settings: `SES-T0101` |
| `defects/` | `3`, `cannot divide by zero` | Deferred integer division by zero passes lint, exits 70 with runtime defect and empty stdout despite recover |

Ten complete sources are executed after copying each to an independent temporary
file. Eight compile-invalid sources are kept separate from the two lint-valid
runtime-error programs in `apps/site/examples/runtime-errors`. No runtime failure
is falsely counted as a compile-invalid fixture.

The complete Effect example saves a plan before main without printing, then
executes it twice. The boundary example leaves one plan unused and prints only
the operation returned by main. These are observable execution-order claims,
not a claim that all pure argument evaluation is deferred.

The TypeScript comparison uses ordinary throw/new Error and caller try/catch for
the same integer inputs 2 and 0. Both versions print identical messages, but the
text distinguishes exception propagation from returning Left. Strict TypeScript
rejects a string input (`TS2345`). A separate numeric probe demonstrates why the
defect analogy must not equate language behavior: TypeScript prints Infinity for
7 / 0 and 3.5 for 7 / 2 without entering catch; the shown Seseragi Int operation
has an integer quotient and zero-division defect.

The environment example uses an explicitly typed ordinary selector function,
then service/provide. Its source makes the required record and selected Settings
visible rather than relying on an underconstrained anonymous selector.
No compiler or runtime changes were made to obtain these results.

## Bounded independent reading result

An independent agent read all twenty rendered locale bodies without filling
explanation gaps from source/specification. It reported no required correction
or core-body explanation blocker. The Japanese explicit-contract comparison
with try/catch was optionally simplified for readability; the targeted English
and Japanese page render passed again and the refreshed Japanese text was
checked. This is text/limited-HTML agent review, not browser or actual-reader
acceptance. The reviewer owns the shared ledger and issue-comment integration.

## Verification and limits

```sh
SESERAGI_FAILURE_OUTPUT=target/site-failure-review \
  bun test apps/site/tests/failure-readers.test.ts
bun test apps/site/tests/explanations.test.ts \
  apps/site/tests/reference-titles.test.ts apps/site/tests/coverage.test.ts
bun apps/site/scripts/check-content-map.ts
```

2026-10-01 results:

- Three failure-reader tests passed, 541 assertions: identities, paired local
  explanations, ten native results, eight compiler diagnostics, two runtime
  failures, strict TS comparison/numeric behavior and twenty locale renders
- The rendered test verifies exact first-example source bytes and output,
  preserved detailed section IDs, same-identity locale links, enabled complete
  example links and disabled standalone links on ten legacy declaration fragments
- Existing explanation/title/coverage tests: twelve passed, 13,492 assertions
- Content map: 351 unique routes, with no added normative leaves
- Focused TypeScript, Biome and `git diff --check` passed; source was formatted
  through the current CLI

The focused renderer uses copied article import closures with `areas: []`.
`target/site-failure-review/README.md` labels this limitation. It does not verify
full navigation, production sidebar/previous-next behavior, desktop/mobile CSS
geometry, overflow, browser interaction or console errors. No browser or server
was started. The twenty-page render uses the bounded helper's explicit
180-second subprocess budget.

Japanese was authored first and English paired to its meaning and conditions,
using the pinned yomiyasu technical-writing guidance. Its advisory linter is
not installed in this workspace and was not run. Independent text/HTML reading
feedback and actual-reader acceptance remain distinct from these technical
checks. This report changes no shared core ledger checkboxes.

Semantic sources: [failure and effects](../../spec/05-effects.md),
[types](../../spec/02-types.md), and compiler-owned std/effect metadata. These are
provenance for authoring; the articles explain needed concepts locally.
