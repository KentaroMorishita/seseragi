# Missing values, failures, and validation: bounded reader verification

This batch starts at `bd5d39ebaf164c32cf562624d971e531309b68c7` on
`docs/697-reader-contract`. It changes fifteen existing page identities, each in
English and Japanese. It does not accept the remaining data/validation library
surface. Compiler/runtime code and canonical reference metadata are unchanged.

## Ownership and reader purpose

The typed page sets live under `apps/site/src/reference/editorial/data-validation`.
`data-validation-model.ssrg` uses the existing semantic Block renderer and
CorrectionCopy paragraph pairs. `data-validation-catalog.ssrg` guards every
callable by module, full identity, value namespace, and function kind. The three
module introductions retain the generated import guidance, API index, and
metadata. Every callable retains its exact declaration and declaration-reading
panel. Parent-owned integration registers the dispatcher, module blocks, examples,
and tests without replacing earlier corrections, including StateT.get.

| Existing identity | Complete local task | Program slug |
| --- | --- | --- |
| `std/maybe` | Display present, missing, and empty nicknames | `maybe-module` |
| `std/maybe::withDefault` | Choose a display value, retaining empty string and zero | `withdefault` |
| `std/maybe::orElse` | Try another source while keeping possible absence | `orelse` |
| `std/either` | Return a checked seat count or its failure reason | `either-module` |
| `std/either::fold` | Turn either outcome into a display string | `fold` |
| `std/either::mapLeft` | Add field context while retaining success | `mapleft` |
| `std/prelude::Monad::flatMap` | Calculate a quote after accepting the seat count | `flatmap` |
| `std/validation` | Show all four combinations of two independent checks | `validation-module` |
| `std/validation::valid` | Return a value already accepted by the caller | `valid` |
| `std/validation::invalid` | Report one input error | `invalid` |
| `std/validation::invalidMany` | Return two existing errors in order | `invalidmany` |
| `std/validation::fromEither` | Convert existing success or one failure | `fromeither` |
| `std/validation::toEither` | Preserve every error at a conversion boundary | `toeither` |
| `std/maybe::traverse` | Return all seat numbers or no result | `maybe-traverse` |
| `std/either::traverse` | Check every count and retain the first failure | `either-traverse` |

The intended reader uses ordinary TypeScript functions, types/interfaces,
branches, arrays and try/catch. Each body explains its local alternatives and
branches, function and call syntax, result type, execution wrapper and consumer.
None requires a Tour, ADT terminology, Rust, Haskell, or prior knowledge of match.
Generic declaration placeholders follow the concrete example.

## Source and comparison contract

`apps/site/scripts/data-validation-readers.ts` registers fifteen independent
Seseragi files and fifteen independent TypeScript files under
`apps/site/examples/{src,comparisons}/api-data-validation`. The displayed source
and Playground seed are the canonical file itself. TS panels have no Seseragi
launch link. Both output panels use the same exact verified stdout.

The TS nickname examples use `string | undefined`, explicit missing checks, and
`??`; they do not misuse truthiness to reject empty strings or zero. The orElse
comparison is a function with eager arguments. The checked-seat examples use a
plain object with an explained `ok` field and ordinary branching. Validation's
module comparison uses two independent if checks and a local errors array; no
imitation applicative library pads the TS version. Leaf error-group examples
explain the TS nonempty tuple syntax rather than silently claiming that TS
cannot express the invariant. Traversal maps all input checks before selecting
an error, preserving native callback evaluation rather than substituting a
short-circuiting validator loop. Examples use strings and small integer counts;
no equivalence of all JS numbers and Seseragi Int values is claimed.

The appeal is explicit result shapes, reusable missing/error operations, and
reusable independent validators with ordered nonempty errors. The prose also
acknowledges when the small direct TS implementation is already straightforward.
It does not claim fewer characters or general language superiority.

## Semantic boundaries

- withDefault and orElse arguments are eager; Seseragi `??` delays its fallback
- fold calls the selected callback body once; mapLeft skips its callback on Right
- Either flatMap skips the next callback on Left and propagates either the original
  failure or the next failure; Maybe skips Nothing; Array concatenates results
- Validation combines independent errors in order and has no Monad instance
- invalidMany requires NonEmptyList, including when an ordinary List happens to
  have elements; both empty and nonempty ordinary List probes are rejected
- fromEither wraps its E as one error, including an array-valued E; toEither keeps
  the entire ordered NonEmptyList of errors
- Maybe/Either traverse still calls later callbacks after Nothing/Left; a later
  runtime defect remains observable. It is not a short-circuiting lookup or a
  promise that callbacks continue after an actual defect
- Empty Array traversal yields Just [] or Right []; successful order and the
  first Left in input order are preserved

Semantic sources: `docs/spec/09-standard-library.md` §§9.2/9.7;
`docs/spec/10-library-surface.md` §§10.4–10.6; canonical
`examples/spec/artifacts/stdlib-schema-1/reference/module.json`; and
`runtime/ts/src/{sum,validation,traversable,list}.ts`. Planning probes under
`/tmp/seseragi-data-validation-plan` established feasibility only. Their old
results are not counted as final article verification.

The upstream description of `std/prelude::Monad::flatMap` still incorrectly
states only collection transformation/flattening. This batch corrects that exact
site identity in both locales. It deliberately does not globally replace the
shared description key or edit compiler metadata. That upstream debt remains.

## Verification record

The local check is `apps/site/tests/data-validation-readers.test.ts`. It uses the
official Linux CLI 0.61.19, release commit `a8641b5a81a4`, Bun 1.3.9, and installed
TypeScript 5.8.3. No dependency installation, Cargo build, WASM regeneration, or
whole-site build is part of this author-owned check.

```sh
source ../seseragi-env.sh
SESERAGI_BIN=/tmp/seseragi-first-run-699.oOVHG8/bin/seseragi \
DATA_VALIDATION_READER_RENDER_DIR=/tmp/seseragi-data-validation-authored/rendered \
  bun test apps/site/tests/data-validation-readers.test.ts
```

The tests execute the fifteen native source files with lint/run, strict-typecheck
and execute all fifteen TS sources, compare exact outputs, compile the exact
fifteen Playground seeds through committed WASM, and run them in the existing
browser-execution runtime harness. That harness execution is not browser/UI
verification. Additional source probes check runtime-defect exit 70,
SES-T0101/SES-T0201 rejection, skipped branches, lazy fallback and collection
flatMap. Runtime instrumentation independently checks callback order and
conversion/accumulation payloads.

Dispatch probes cover all twelve callable identities and four malformed-key
controls per identity, plus the unchanged StateT.get overlay. Inventory assertions
retain 63 modules / 1,812 leaves. Real production renderer closures cover every
selected locale plus narrow generated controls and linked destinations. The
render test checks every localized authored paragraph and scalar field, both
source panels, exact native source links, output text, exact declarations, actual
H1 titles, purpose-labelled link destinations, and same-identity locale links.
Its title guard uses complete authored bodies, not only related-link fragments.

Initial test runs caught assertion-authoring mistakes: the expected Show spelling
of a string, an incorrect StateT canonical identity, a nonexistent sourceUrl test
field, comma-separated match branches in the probe, and an over-specific anchor
regex. These were test fixes; they did not require compiler/runtime or article
behavior changes. Their failed logs are retained as failed attempts.

Final counts, extracted-prose lint, and independent review are recorded below. Browser desktop/mobile behavior and genuine first-time human reading
remain unverified; no technical gate substitutes for those categories.

## Remaining scope

Maybe.sequence and Either.sequence/mapRight/bimap/swap remain unreviewed. Owning
type/constructor and instance leaves are not accepted by this batch. Transformer
modules and placeholders are deferred, with the prior StateT.get correction
preserved. The parent owns one later integrated site-scoped checkpoint after
independent content review. No public issue/comment, push, PR, or deployment was
performed.

## Final author-owned result (2026-10-01 UTC)

The complete scoped suite passed **6 tests / 1,524 assertions** in 75.78 seconds.
The final source inventory contains 79 authored/test/source files; the reviewed
artifact inventory identifies exactly 30 selected HTML bodies. Additional linked
and generated controls make 42 HTML files in the bounded artifact directory.
`/tmp/seseragi-data-validation-authored/test-pass3.log` is the passing run;
`test-initial.log` and `test-pass2.log` retain the unsuccessful assertion attempts.
The typed examples and page files pass CLI format checks, the TS files pass
Biome, and `git diff --check` passes. No parent-wide integration is claimed here.

Japanese was drafted first using the pinned yomiyasu `tech` guidance
(`30ee6041c328ce21d38a7963f667e079a93d7a12`); English preserves its inputs,
conditions and limits. A foreground advisory pass ran on prose extracted from
all fifteen complete Japanese article bodies. Source/output code blocks were
excluded and inline syntax was marked. There were 63 repeated-sentence-ending
warnings and three negative-parallelism informational findings, with scores
68–90. The negative cases retain actual distinctions: present zero versus a
fallback, Int versus number range, and one error versus NonEmptyList of errors.
Repeated polite active sentence endings were retained where they name the actor,
value and result directly; prose was not distorted to increase a score. Raw
unmarked canonical identities initially produced false colon warnings; correcting
the extraction to mark complete identities removed those false positives, without
changing article text. This was one advisory editorial pass, not reader acceptance.
No lint or browser process remains from this work.

Artifacts are in `/tmp/seseragi-data-validation-authored/`:

- `source-manifest.json`: exact authored source/test SHA-256 records
- `reviewed-body-manifest.json`: exact selected route/body SHA-256 records
- `rendered/`: fresh real-renderer bodies, handed to the independent reviewer
- `ja-prose/`, `yomiyasu-pass1.json`: extracted complete JA prose and advisory output
- `test-pass3.log`: complete passing focused run

The entire thirty-body set was handed to the independent agent reader after the
passing render, before any acceptance claim. Its result is recorded
separately below. A knowledgeable agent review is not genuine first-time-user feedback.
Desktop/mobile browser review remains unverified, as do visual overflow,
language-switch interaction, and clicked related-link/previous-next round trips.
The static href/title/locale checks do not substitute for those interactions.

## Independent-reader correction and final artifact

The independent reader read all thirty complete bodies and found one local
explanation gap on the Validation module: its two-argument declarations used an
arrow between parameters as well as an arrow before the return type. The generic
setup sentence did not distinguish them. Both locales now identify name and seats
as the two parameters, explain that the first arrow separates them, identify the
final arrow as introducing the return type, and identify the expression after =
as the returned value. The other fourteen examples were audited for the same
issue; their shown local function declarations each have one parameter.

The corrected complete thirty-body renderer test passed **1 test / 1,235
assertions** in 37.52 seconds. `rendered-revision2/` is the final content artifact.
The exact native/TS sources were unchanged; `final-program-records.json` recaptures
all fifteen source hashes and matching native/TS outputs (43 lines total).
`revision2-diff.json` verifies that only the two Validation module bodies changed;
the other twenty-eight bodies are byte-identical to those already read. The
independent reader was sent both corrected bodies for follow-up review.

A second and final foreground advisory JA pass retained the same 63 ending
warnings and three technical negative-comparison findings. The final extracted
prose is in `ja-prose-revision2/` and `yomiyasu-pass2.json`. No third stylistic pass
is planned. Scores remain 68–90 and are not acceptance criteria.

Final evidence manifests:

- `source-manifest-revision2.json`: SHA-256
  `3fc5bd918a1afc39678240b944817ab43b143b2a258ffb3c3b47fc6450f9d768`
- `reviewed-body-manifest-revision2.json`: SHA-256
  `4188e5a2b8f4419c70bf68fede19cf67605e2cefea3f28390de57277fcf4ca69`

The following records are per identity; “code” includes native lint/run, strict
TS execution parity, and exact committed-WASM seed execution. “Rendered” includes
both locale bodies and the exact copy/source/declaration/title/locale assertions.
UI and actual first-time-human feedback remain unverified for every row.

| Identity | Code | Rendered EN/JA | Independent agent reader |
| --- | --- | --- | --- |
| `std/maybe` | Pass | Pass | Agent complete-body review cleared |
| `std/maybe::withDefault` | Pass | Pass | Agent complete-body review cleared |
| `std/maybe::orElse` | Pass | Pass | Agent complete-body review cleared |
| `std/either` | Pass | Pass | Agent complete-body review cleared |
| `std/either::fold` | Pass | Pass | Agent complete-body review cleared |
| `std/either::mapLeft` | Pass | Pass | Agent complete-body review cleared |
| `std/prelude::Monad::flatMap` | Pass | Pass | Agent complete-body review cleared |
| `std/validation` | Pass | Pass | Corrected module reread cleared |
| `std/validation::valid` | Pass | Pass | Agent complete-body review cleared |
| `std/validation::invalid` | Pass | Pass | Agent complete-body review cleared |
| `std/validation::invalidMany` | Pass | Pass | Agent complete-body review cleared |
| `std/validation::fromEither` | Pass | Pass | Agent complete-body review cleared |
| `std/validation::toEither` | Pass | Pass | Agent complete-body review cleared |
| `std/maybe::traverse` | Pass | Pass | Agent complete-body review cleared |
| `std/either::traverse` | Pass | Pass | Agent complete-body review cleared |


The independent agent reader subsequently reread both complete corrected module
bodies and cleared all thirty bodies. Its separate source/artifact checks found
zero errors: 30 native panels and exact seed links, 30 TS panels, 60 output panels,
54 authored links matching destination H1 titles, 60 same-identity locale links,
24 leaf declaration/owner checks, and 300 local-fragment occurrences. The
independent byte comparison confirmed the same 28 unchanged / 2 corrected split.

Independent records:

- `/tmp/data-validation-independent-review-final.json`
- `/tmp/data-validation-independent-reread-diff.json`
- `/tmp/data-validation-independent-reviewed30-final.sha256`, aggregate SHA-256
  `a36e2029167625c4c45fdde14239480664c37ab7f319234e72d8941a09185b08`

That manifest sorts artifact-relative paths and uses lines of the form
`<HTML-byte SHA256>  <path>\n`; its aggregate is the hash of those manifest bytes.
This is independent agent reading and static verification, not browser interaction
or genuine first-time-human acceptance.
