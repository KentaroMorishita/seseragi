# Traits reader revision: verification

This is the local 13-article revision under #601 / #630. No new public work item,
issue comment or completion claim was published for this batch. The local work-item
draft and raw execution evidence are retained outside the checkout.

## Scope and reader

The base commit is `882a6814d7c965da375ba56c1bef084e773820eb`. These existing article
identities, routes and titles are retained in both English and Japanese:

- `traits/model`
- `traits/declarations`
- `traits/instances`
- `traits/constraints`
- `traits/methods-versus-traits`
- `traits/method-calls`
- `traits/deriving`
- `traits/standard-operators`
- `traits/coherence`
- `traits/laws`
- `traits/do-notation`
- `traits/do-block-typing`
- `traits/do-desugaring`

The intended reader knows everyday TypeScript functions, interfaces and object
fields. The first trait example labels tickets and people, with a checked TS
interface implementation for the same input and output. The explanation explicitly
distinguishes TS implementation objects from Seseragi's type-selected declarations
without claiming the TS task requires traits or unnecessary infrastructure.

The do articles start with missing address parts, then distinguish the inner String
or Int from the outer Maybe. They explain `Just`, `Nothing`, `match`, `<-`, `pure`
and the lambda receiving a payload locally. Monad terminology follows the concrete
absence behavior. The desugaring page runs both present and absent cases, and
retains the distinction between skipped continuation work and arguments already
evaluated before a function call.

Detailed rules remain: declaration signatures and type parameters, instance
ownership and overlap, conditional requirements even for empty values, dot-method
lookup versus trait selection, supported deriving traits and field conditions,
operator mappings and ordering, laws beyond typing, do result typing, irrefutable
patterns and evaluation/scope preservation. Known implementation gaps are described
briefly beside the relevant detailed rule, not presented as alternative semantics.

## Canonical examples and repairs

`apps/site/scripts/trait-readers.ts` registers 30 exact-source records:

- 13 complete primary Seseragi programs
- 13 page-specific compile-invalid programs
- one separately runnable law-violating implementation
- one supported Effect/generic-Monad witness
- one additional generic-Monad type-error program
- one complete TypeScript comparison

Each primary source and its rejected counterpart is checked independently outside
the site package. Tests execute the literal repair stated by the article, rather
than merely compiling an unrelated positive program. Rejected examples and the TS
comparison do not have runnable Seseragi Playground links. Complete positive
programs retain exact-source Playground URLs, source hashes and matching highlight
text.

The broken Eq example compiles and prints `False` for self-equality. Its correction
prints `True`. This is deliberately distinct from the rejected function returning
a String where Bool is required. The example checks concrete properties; it does
not claim to prove a law for every possible value.

The advanced witness verifies all of the following in one executable program:

- service-dependent Effect do, with its service explicitly provided
- recoverable String failure in Effect do and a checked recovery path
- one `Monad<M>` function applied to Maybe and fallible Effect
- an explicitly annotated final Effect value satisfying the do result type

The result is `ready`, `(42, 42)`, `42`, `-1`, and `Just (2, 2)` on separate lines.
A plain Int is rejected where `M<A>` is required; a unary Box type without a Monad
instance reports `SES-T0201`. These witnesses do not imply arbitrary Effect type
combinations or unrelated user-defined instances have all been verified. Task's
alias `Effect<{}, Never, A>` is not used to impose a Task-only restriction on do.

## Commands and results

Tests used Bun 1.3.9 and the verified published Linux CLI:

```text
seseragi 0.61.19 (release, commit a8641b5a81a4, target x86_64-unknown-linux-gnu)
```

Compiler/runtime/Cargo/toolchain sources have no diff between the release commit
`a8641b5a81a4063054e4db4061aedaff86c55a7a` and this batch's base. No compiler or
runtime source was edited. The independently retained development binary was used
only to confirm the five discrepancy probes, not mislabeled as a fresh build of
this documentation batch.

```sh
source /workspace/scratch/677b5e8ef737/seseragi-env.sh
export SESERAGI_BIN=/tmp/seseragi-first-run-699.oOVHG8/bin/seseragi
SESERAGI_TRAITS_OUTPUT=target/site-traits-review \
  bun test apps/site/tests/trait-readers.test.ts apps/site/tests/explanations.test.ts
```

Final combined result: **21 tests, 14,448 assertions, zero failures**. This includes
26 rendered locale bodies, 78 exact purpose-labelled destination titles, paired
paragraph counts, preserved page identity, matching locale alternates, exact main
and rejected source panels, native outputs, rejected diagnostics, literal repairs,
and strict checking/execution of the TS comparison.

Other checks passed:

- Biome for the new helper, test and TS comparison: 3 files
- configured non-strict TypeScript options for the new helper and test imports
- `seseragi format --check` on 53 owned article/helper modules and all 29 Seseragi fixtures
- `git diff --check`

The source inventory contains 85 files (article modules, fixtures, helper and test).
The sorted path-to-SHA256 JSON inventory has SHA256
`d71b607b2674a8080da18954921db02db74a9979b25b6c6d1b32516bd61f472d`.

## Observed specification/implementation discrepancies

Each probe was independently run with the published CLI and the retained current
source development binary, with the same status and diagnostic outcome. Exact
source and stderr are saved in the external evidence directory. These findings do
not authorize compiler edits or public issue changes.

| Probe | Observed result | Reader treatment |
| --- | --- | --- |
| Local `Label.label` qualification | `SES-N0001`, although chapter 4.5 specifies trait-qualified disambiguation | Preserve the rule, state the tested limitation, use an unambiguous ordinary call in the runnable example |
| Duplicate instance hidden by a target-type alias | lint accepts it; direct duplicate reports `SES-T0202` | Preserve alias expansion/coherence requirement and warn against depending on acceptance |
| Standard Eq instance with String result | lint accepts it; custom Label signature mismatch is rejected | Preserve matching-signature requirement and mention the standard-trait checking gap |
| Direct `add` without result annotation | `SES-T0201`; explicit `Distance` result annotation executes | Explain the annotation in the example rather than claim inference worked |
| Unannotated final `effects.succeed` in fallible do | `SES-T0101` constructor mismatch; explicitly typed final Effect executes | Preserve normative Never failure widening and describe the tested boundary |

Read-only issue searches found related historical scopes: #198 concerns imported
trait aliases/canonical instance identity, whereas this repro uses a local target
type alias; #264 concerns formatter declaration boundaries. They are not assumed to
be open owners of these new reproductions. Other broad trait/prelude issues do not
establish ownership of the exact failures. No new issue or comment was posted.

An initial compact trait declaration exposed formatter joining of the following
declaration. The executable fixtures now use explicit separated multiline
declarations; formatting is idempotent and final execution is checked after
formatting. Two initial repair assertions also caught direct console printing of a
Maybe value as `[object Object]`; the published snippets explicitly call `show`
when textual constructor display is intended. No runtime behavior was changed.

## Rendered reading and Japanese prose

Final scoped HTML is in `target/site-traits-review`: 13 routes in each locale.
The fixture uses the production article renderer and real page modules, but an
empty navigation-area list. It is not full-site navigation or browser evidence.

The independent agent reader read all 26 bodies and found no first-example or
explanation blocker. Their two requested refinements were applied: ordering
comparisons specifically call `compare` once, and deeper English paragraphs use
concrete implementation terminology instead of unexplained “evidence”. The final
changed bodies were returned for bounded rereading. Reader-review acceptance is
recorded separately in the shared ledger; this is not a genuine first-time human
reader or mobile/desktop browser sign-off.

The pinned yomiyasu tech guidance was used, followed by two rendered-prose advisory
passes. The final 13-page Japanese report has 49 sentence-ending repetition
warnings and 5 technical negative contrasts. Those contrasts explain type-selected
instances, payload versus wrapper, type constructors, source diagnostics and the
absence of implicit conversion. They were retained for accuracy rather than
rewritten to optimize a lint score. No metaphor, filler, excessive formatting or
other advisory category remained. Python ran in the foreground.

Raw logs, gap probes, local work-item draft, issue-search summaries, source and HTML
hash inventories are under `/workspace/shared/seseragi-traits-authoring/` outside
Git. Parent-owned aggregate imports, locks, full generation and final integration
are pending the common checkpoint. Browser execution remains excluded by the
existing access decision. No full `check:site` pass is claimed.
