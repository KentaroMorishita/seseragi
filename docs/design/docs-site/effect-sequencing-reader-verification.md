# Effect sequencing reader verification

Status: authored-source execution, corrected bounded production rendering and
complete paired independent reading are verified. This document distinguishes native
execution, committed-WASM execution, rendered documentation, reading review,
real-browser inspection and publication.

## Reproducible scope

The thirteen exact existing identities are listed in
`apps/site/scripts/effect-sequencing-reader.ts`. Public sources live under
`apps/site/examples/src/effect-sequencing/`, and comparisons under
`apps/site/examples/comparisons/effect-sequencing/`. The example mapping is the
single source of displayed source IDs, Playground source seeds and expected
output. The canonical signatures remain unchanged.

Run the focused checks with a verified release CLI selected by `SESERAGI_BIN`:

```sh
SESERAGI_BIN=/path/to/released/seseragi bun test \
  apps/site/tests/effect-sequencing-reader.test.ts \
  apps/site/tests/effect-sequencing-editorial.test.ts
```

Bun must be available on the execution PATH. `EFFECT_SEQUENCING_RENDER_DIR`
optionally retains all rendered bodies. Renderer verification uses temporary
packages containing the production relative-import closure; it does not change
the site package lock. Site-wide integration is a separate check.

The tested native compiler is official release 0.61.19, commit a8641b5a81a4,
Linux x86_64, binary SHA-256
`987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7`.
The TS host is Bun 1.3.9; strict comparisons use repository TypeScript 5.8.3.
Current-runtime invariant tests import the repository's actual Effect runtime;
they do not substitute for the separate release-CLI checks.

## Executable evidence

The nine complete sources cover approved/rejected work, validated preview
preparation, required optional counts, stage-specific error messages, a selective
fallback, deferred preparation, a deliberately unexecuted unsafe calculation,
ordinary LoopControl values and sequential traversal. Every program is linted,
formatter-checked and executed in an isolated directory. All nine TS counterparts
are strict-checked and executed with exactly the same displayed output.

All nine exact Playground source seeds compile and execute through the committed
WASM compiler and browser execution adapter. This establishes source/compiler/
runtime behavior; it is not a claim that the public Playground website was
clicked in a real browser for this batch.

Four negative fixtures retain real SES-T0101 diagnostics and executable repairs:
passing narrow Either results to a wider formatter, mixing Effect constructors
inside an ordinary-function do, using Break as an Effect, and passing plain text
to fromEither. Explicit effect-function contracts repair the first two without
casts or compiler changes. A missing-value or expected-error explanation must not
assume implicit widening of Either payload types.

Four native runtime probes show that eager division-by-zero fails before the next
print, whereas an unexecuted deferred version succeeds. Executing that deferred
operation through attempt, recover or mapError still causes a runtime defect and
never reaches the following print. Deferral does not make a calculation safe.

Current-runtime assertions separately establish:

- fail is unexecuted when constructed, and a typed failure skips subsequent work
- fromEither uses the already evaluated input across repeated executions
- fromMaybe distinguishes missing from a present zero or empty string
- attempt returns Left/Right, mapError skips success, and recover can itself fail
- succeed's ordinary argument is eager; defer calls its callback each time
- all three typed-error wrappers let defects and actual pending cancellation pass
- Break/Continue are ordinary values; successful stopping does not pull another
  iterator item; empty input calls no callback; failure starts no later callback

The identical strings in the friendly defer example do not by themselves prove
construction timing. The counter assertions and native defect probes provide
that evidence. The TS counterpart uses an ordinary function rather than an eager
Promise. The extra TS division fixture explicitly rejects zero because normal JS
number division would otherwise produce Infinity; it is not treated as the same
numeric contract by default.

## Rendered and reader-facing evidence

The initial guard/render run passed two tests with 1,643 assertions across all 26
locale bodies. It checked exact four-field identity selection and negative
controls; canonical sources, output and signatures; full locale paragraph
pairing; task/TS/Seseragi/reading order; same-identity language links; related-title
round trips; and section-local navigation. Unselected Effect pages remain generic.
Break and Continue no longer display the false generic Effect summary.

Independent English and Japanese readers inspected all thirteen full bodies in
each language, including examples, outputs, generated declarations and related
navigation. This was independent agent reading, not actual novice-human feedback.
Corrections explain Never in the positions used by succeed/fail/attempt, explain
Unit/() in defer's callback, limit mapError wording to its wrapped operation,
state that empty traversal still prints Completed, and align Japanese LoopControl
metadata guidance with the demonstrated public constructors. The English generated reading and
all canonical signatures remain preserved by the reading adapter. The succeed
page now offers a direct defer continuation.

The corrected combined execution, guard and production-render run passes **nine
tests, zero failures, 1,879 assertions**. Exact LoopControl adapter controls
include wrong owner, namespace, kind, neighboring identity and changed-baseline
cases. The site-mode TypeScript check, strict TS comparisons, Biome and prose
extractor tests pass. Both independent readers then reread all thirteen final
bodies in their language, not only changed paragraphs. All five necessary
Japanese findings are resolved, no new required correction was found, and all
fourteen Playground links per locale still carry the exact displayed source.
The English LoopControl body is byte-for-byte unchanged by the Japanese-only
adapter. This remains independent agent reading, not novice-human acceptance.

## Advisory Japanese prose check

The linter is restored from the reader contract's pinned yomiyasu commit
`30ee6041c328ce21d38a7963f667e079a93d7a12`. Its verified Git blob is
`87c87c93b02fea18c39f471da68f1ebe104d40d8`, SHA-256
`e15ca11dd879cd7726961abf0fccbb34dfcd8256f456eb5738b8ae743b2df595`.
The advisory report ran in the foreground on all thirteen complete rendered
Japanese pages, excluding preformatted code. The first pass reported 126
unwanted half-width-space warnings, 62 repeated sentence endings, six negative
parallelism notes and one high-list-ratio note. After one editorial pass, the
spacing warnings are gone; the final report has 65 repeated endings, seven
negative-parallelism notes and one list-ratio note. The added local explanations
account for some of the remaining polite endings. The failure/cancellation and
value-versus-operation distinctions remain necessary; the list note comes from
the generated module API directory. They were not rewritten merely to lower a
score. No lint server or background process was started.

## Scope limits

No runtime/compiler implementation, new routes, temporal APIs or browser-provider
contracts are changed. Whole-site integration, publication, live desktop/mobile
review and actual first-time-reader feedback remain separate. Route presence,
word counts, paired paragraphs and successful compilation do not establish that
a person understands the APIs.

## Retained result fingerprints

These identify the final retained evidence without embedding local execution
paths in the published document:

- `final-focused.log`: `02c78017d77d816b9afc957809fc8d3a1ba629c51950da0381c95667326a6604`
- `selected-bodies.json`: `edda4c46dec93e9e813f73b99176fefdf8c6fbb7def462125101611042bdfcf5`
- `yomiyasu-final.json`: `3fe289561b10ac63ef4501eb7c9ba82baff4be3816d5eea47b6783bdc3612aac`
