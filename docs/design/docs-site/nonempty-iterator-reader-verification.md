# NonEmptyList and Iterator reader verification

## Scope and ownership

Base documentation checkpoint: `afe906eee87dd0b588730a578554851f0e2404ac`.
This batch authors eight existing identities in Japanese and English:

- `std/non-empty-list` module introduction
- `std/non-empty-list::{cons,fromList,reduce1,singleton}`
- `std/iterator` module introduction
- `std/iterator::{next,unfold}`

The sixteen edited locale bodies remain typed Seseragi modules with the
existing semantic Block renderer. The exact identity dispatcher and shared
article helper are `nonempty-iterator-catalog.ssrg` and
`nonempty-iterator-model.ssrg`. Canonical examples and comparisons live under
`apps/site/examples/{src,comparisons}/api-nonempty-iterator/`; descriptors are
owned by `apps/site/scripts/nonempty-iterator-readers.ts`.

The existing NonEmptyList head, tail and toList overlays are preserved. The
NonEmptyList and Iterator opaque-type pages are also unchanged controls.
Shared build/check registries, aggregate example imports, package locks and
complete-site integration belong to the parent task. No compiler, runtime,
specification or canonical reference metadata change is included.

## Reader and semantic choices

NonEmptyList starts with a familiar score collection, a known first element,
or an empty input. The pages locally explain List's backtick literal, spaced
function calls, type annotations, Just/Nothing, match, reduction callbacks and
the output wrapper. The TypeScript examples use a readonly required-first-element
tuple, length checks and ordinary Array operations. The text explains that
readonly restricts this reference at the type level; it is not runtime freezing.
Zero remains a valid first element. The reduction example uses subtraction to
make the left-to-right order observable and checks a singleton separately.

Iterator starts with a countdown. Every relevant page explains the current
element, next state or returned continuation where used. Re-reading the original
Iterator yields the same result, while advancing requires the returned rest.
The TypeScript counterpart uses an ordinary generator and states its different
advancement behavior explicitly. The module comparison creates a fresh generator
for each full reading. The next comparison deliberately displays `3, 3, 2` for
Seseragi and `3, 2, 1` for JavaScript; the prose explains why. The unfold example
also reads only one value from an endless sequence and warns against collecting
it to completion. Redundant generator return-type annotations were removed so
the comparison does not introduce unnecessary advanced TypeScript notation.

Normative and implementation traceability:

- `docs/spec/10-library-surface.md` §10.5, `std/non-empty-list`: construction,
  optional conversion, head/tail types and reduction order
- The same specification's §10.6 Iterator contract: persistent reading,
  deferred first step, one step call per next, no caching guarantee, pure
  steps and potentially endless traversal
- `runtime/ts/src/list.ts`: NonEmptyList construction/conversion and reduce1
- `runtime/ts/src/iterator.ts`: state closures, next and collection behavior
- Compiler-generated reference metadata supplies declarations unchanged

## Executed checks

The final focused command ran
`apps/site/tests/nonempty-iterator-readers.test.ts`: **six tests passed, zero
failed, 749 assertions**, in 35.28 seconds. Evidence is retained at
`target/nonempty-iterator-reader-review/final-release-test.log`.

The published CLI is version 0.61.19, release commit `a8641b5a81a4`, Linux
x86_64. Its SHA-256 was checked before use:
`987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7`.
This is the verified published compiler, not a binary claimed to have been
built from the current documentation HEAD. A retained local wrapper delegates
to it and adds `--profile release` only to run commands. Lock updates occur
only in isolated temporary verification packages. Bun is 1.3.9; TypeScript
and Biome are the installed repository versions.

The checks cover:

- All eight canonical Seseragi programs lint and execute with exact stdout,
  including final newlines, normal and boundary inputs
- All eight TypeScript counterparts execute with their own exact documented
  stdout and pass strict TypeScript checking
- Every exact Playground seed compiles through the committed WASM adapter
  and executes through the existing runtime harness with matching output
- Runtime assertions verify no eager unfold step, repeated pulls, returned
  continuations, repeated ended results, left-reduction callback order and
  no callback for a singleton. A bounded maximum-Int check confirms a
  repeated constant state stays valid without arithmetic overflow
- Complete invalid snippets reject Array-as-List, possibly empty reduction
  and an effectful iterator step with SES-T0101; the equivalent empty TS tuple
  is rejected with TS2322. The successful examples supply the corrected forms
- Six function identities and two module introductions are selected exactly;
  incorrect owner, namespace, item kind, identity and module-name probes do
  not receive these overlays. Earlier NonEmptyList corrections survive
- All sixteen edited EN/JA bodies retain exact canonical declarations,
  their entire paired paragraph/scalar copy, exactly one native and one TS
  source panel, exact localized terminal titles and outputs, and the actual
  rendered Playground URL's exact source bytes
- All thirty authored page-name links match rendered destination H1 titles.
  Deliberately corrupted labels fail the checker. Function checks cover the
  whole body; module checks end at the established generated-API boundary
- Ten localized control bodies retain the prior sources/descriptions; their
  presence is not counted as new editorial coverage

Biome checked ten owned TypeScript files, helper/test TypeScript checking
passed, all 34 owned Seseragi files passed canonical formatting, and the
explicit whitespace check passed for all 45 owned files, including untracked files. See `static-check.log`. One initial formatting
issue in unfold's match indentation was corrected before the final run.

## Retained rendering and review

Fresh release-profile rendering uses the real production reference and document
module import closure with the two selected compiler modules. It emits 26 locale
pages: sixteen edited bodies and ten controls. The artifact root is
`target/nonempty-iterator-reader-review/html/`. A copied edited-only subset,
`edited-html/`, is used for advisory prose checks; `en-reading.txt` and
`ja-reading.txt` contain the full selected article text including code/output.

The sixteen-file selected hash is
`a83763908cc4ecac9bc331532eae4077b4acdb5b484fdb6837b518e1c7075a33`.
`manifest.json` lists the exact paths. Hashing sorts artifact-relative paths,
then hashes each path, NUL, file bytes and NUL. This hash excludes controls.

Rendered Japanese prose was extracted using the existing `check-prose.py`,
excluding preformatted code and marking inline code, then checked in the
foreground with the existing pinned yomiyasu linter. All eight edited Japanese
bodies are in `yomiyasu.json`. Its advisory findings were 22 repeated-ending
warnings, three negative-parallelism warnings and one list warning. The negative
phrases distinguish original versus rest, length versus zero-valued input, and
calculation versus output. They express required technical distinctions. The
list warning concerns the generated public API inventory. These are retained;
the findings are not treated as an acceptance score.

Independent whole-body EN/JA reading found an important boundary in the first
unfold draft: incrementing an Int is not genuinely endless because it eventually
overflows. The example now uses repeatValue, returning Just (value, value), and
the matching TypeScript generator also repeats its input. The displayed first
10 is unchanged. Both explanations and the canonical panels were refreshed; the
final suite above includes that correction. The independent reviewer then read
both complete updated unfold bodies and their native/TS panels, confirming that
repeatValue now honestly demonstrates an endless sequence. No shared prose
changed in this repair. The reviewer completed all sixteen edited bodies and
separately checked thirty exact title links, sixteen native source seeds/panels,
sixteen TS panels, thirty-two outputs and eighty local fragments. No controls
were accepted. This is an independent agent reading, not actual first-time-reader
acceptance or browser interaction evidence.

## Boundaries

Selected-module rendering does not establish complete cross-module route/link
closure or full-site integration. WASM/runtime-harness execution is not a
browser session. Localhost browser access remains blocked; no browser workaround
was used, and desktop/mobile appearance, overflow and interaction are unverified.
No actual first-time-reader acceptance, publication, push, PR or deployment is
claimed. Complete-site generation and aggregate gates remain parent-owned.
