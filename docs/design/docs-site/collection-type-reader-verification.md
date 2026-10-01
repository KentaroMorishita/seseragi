# Collection type and constructor reader verification

## Scope and source ownership

Documentation base: checkpoint 6, `bd5d39ebaf164c32cf562624d971e531309b68c7`.
This batch authors nine existing identities / eighteen complete locale bodies:

- `/docs/library/map/opaque-type/map/`
- `/docs/library/set/opaque-type/set/`
- `/docs/library/non-empty-list/opaque-type/nonemptylist/`
- `/docs/library/iterator/opaque-type/iterator/`
- `/docs/library/collection/opaque-type/{sizeerror,reducestep}/`
- `/docs/library/collection/constructor/{nonpositivesize,next,done}/`

Japanese routes prepend `/ja`. Iterator keeps canonical identity
`std/prelude::Iterator` and module/route owner `std/iterator`. No compiler,
runtime, specification, canonical metadata or existing module/callable body is
changed. Array/List retain their existing language and module owners.

Typed overlays, the exact identity/module/namespace/kind dispatcher and shared
Block helper are in `apps/site/src/reference/editorial/collection-types/`.
Canonical sources are in `apps/site/examples/{src,comparisons}/collection-types/`,
with rejected native fixtures in `examples/invalid/src/collection-types/`.
`apps/site/scripts/collection-type-readers.ts` registers nine identities, six
native/TS pairs and eight rejected examples. Shared catalog/build/check/main
registries and package locks belong to the parent integration task.

## Reader and comparison decisions

Japanese was written first; English preserves its facts, boundaries and tasks.
The audience knows ordinary TS types/interfaces but need not know named data
alternatives or match. Each deep-linked body explains its own required notation
and the difference between a type annotation and constructing a value.

Map and Set use ordinary TypeScript collections and explicit copies only because
the task retains old values. Content equality is tested explicitly rather than
misrepresenting JS `===`. NonEmptyList is compared with a readonly required-first
TS tuple and a checked conversion, not an assertion. Iterator uses a small
retained-position TS countdown model to reproduce `3,3,2,done`; prose separately
explains why a normal JS iterator advances. SizeError uses a returned named-shape
result, not artificial exceptions. ReduceStep shows both an idiomatic loop/break
and a callback returning named decision objects; its extra first TS `30` is
explicitly identified as the ordinary loop's result. JSON versus native display
spelling and small-integer limits are stated, not silently normalized.

NonPositiveSize deliberately constructs a reason with payload 2: construction
is not sign validation. Done 99 is read as a value and subsequent code still
prints. The early-stop contract excludes future pulls/callback calls, not work
already used to build a strict Array. No infinite example is run to completion.

## Executed bounded checks

The official CLI is `seseragi 0.61.19`, release commit `a8641b5a81a4`, target
`x86_64-unknown-linux-gnu`, SHA-256
`987b0834c1ac299d0e4922e33160a324fa1d85dde909867b15660bac6e9d53b7`.
This is the installed published compiler, not a newly built compiler from this
checkout. Bun is 1.3.9; installed TypeScript is 5.8.3. No package installation,
Cargo build, full-site build or WASM regeneration was performed by this worker.
Temporary rendering packages update their own lock through the existing
`renderPageClosure` helper; shared checkout locks are parent-owned.

Commands use `source /workspace/scratch/677b5e8ef737/seseragi-env.sh` and
`SESERAGI_BIN=/tmp/seseragi-first-run-699.oOVHG8/bin/seseragi`.

- `bun test apps/site/tests/collection-type-readers.test.ts`: **10 tests / 1,161
  assertions pass**, 45.05 seconds on the final corrected prose
- Six canonical native programs lint and run in isolation with exact stdout,
  final newlines and empty stderr
- Six TS counterparts strictly typecheck with noUncheckedIndexedAccess using
  the local compiler path, and execute with their own exact documented stdout
- All six exact Playground source seeds compile through committed WASM and run
  through the existing browser-runtime harness; this is not interactive browser QA
- Eight SES-T0101 failures, a TS empty-tuple rejection, and the ordinary JS
  iterator advancement contrast are checked
- Exact dispatcher probes alter owner, identity, namespace and kind; all 57
  generated instance leaves are rejected
- Focused real production rendering covers eighteen authored locale bodies,
  exact declarations, paired prose, native/TS panels, output panels, seeds,
  same-identity locale links, owning-module backlinks and 74 exact-title authored
  links with adjacent purposes. Linked targets and fragments resolve inside the
  bounded fixture; the shared Language Reference landing link is outside it
- `bun test runtime/ts/tests/collection.test.ts runtime/ts/tests/iterator.test.ts
  runtime/ts/tests/map.test.ts runtime/ts/tests/set.test.ts`: 22 tests / 4,111
  assertions pass, covering persistent state, iteration, equality and stopping
- `bun test apps/site/tests/api-corrections.test.ts
  apps/site/tests/nonempty-iterator-readers.test.ts`: 13 tests / 1,392 assertions
  pass. Their retired opaque-type controls are replaced with existing
  Hash<NonEmptyList<A>> and Iterable<Iterator<A>, A> instance controls, preserving
  fixture module/item/body counts and asserting real signature/reading plus
  absence of editorial. This is not reader acceptance of the instance pages

The initial new render test failed because its bounded-fixture exclusion covered
`/docs/language/` but missed `/ja/docs/language/`; both generated landing links are
now explicitly outside that fixture. An initial dispatcher test also had a
same-line match-branch separator error in its generated test harness, corrected
without changing production semantics. Neither attempt is counted as passing.

## Independent review and remaining gates

An independent agent read all eighteen rendered articles in sequence, including
programs, outputs, declarations and related destinations, without using source to
supply missing explanations. Its precise scope and final artifact record follow
below. This is simulated reader feedback, not actual first-time human acceptance.

The bounded render is not the complete-site integration gate. Browser appearance,
interaction, desktop/mobile layout and actual novice comprehension remain outside
these execution assertions. All 57 generated instance leaves remain outside the
new reader scope. Existing checkpoint history is preserved rather than redefined.


## Final retained evidence and review corrections

Evidence root: `target/collection-type-reader-review/`. `rendered/` contains the
bounded 64-route fixture; `rendered/scope.json` identifies only the eighteen new
bodies. `focused-test.log`, `runtime-test.log`, `regression-test.log`,
`example-evidence.json` (initial proof), `final-example-manifest.json`,
`source-manifest.json`, `reader-review.md`,
`reader-reviewed.sha256` and `yomiyasu-final.json` retain the separate evidence.

The reviewer found three precise local explanation issues and requested paired
corrections: ReduceStep's shared constructor wording mentioned an arg1 absent
from its type declaration; Map/Set called Unit a value instead of distinguishing
the type from (); SizeError/NonPositiveSize assumed earlier function-notation
knowledge. All were corrected. The reviewer then reread all ten affected complete
locale bodies; the other eight retained their initial complete reading. No
remaining blocker was found within this agent-simulated scope. The reviewer read,
but did not independently execute, the displayed programs. Its fresh link check
resolved 140 local article/navigation occurrences with no missing target or
authored-title mismatch; generic Language Reference landing links were outside
the reduced fixture. Supporting pages were not added to new reader acceptance.

Final hashes:

- Eighteen HTML bodies, sorted route + NUL + bytes + NUL:
  `d7ca17453212ae3215dcd63d1e74a8d8409fae4d9428146df843c262a1cd2c66`
- Twenty-nine typed overlay/helper/dispatcher files, sorted repository-relative
  path + NUL + bytes + NUL:
  `6746ef7053e59e09ce6d0bb61335257f159529ca14da5ead11fa8d6861a822fd`
- Reviewer's eighteen-body SHA-256 manifest bytes:
  `218510d1541e42ec4e762de3096d66122d46cf01a63bf72081812109d82488b3`

The final advisory Japanese lint inspected nine complete rendered Japanese
bodies, excluding preformatted code, using the existing local yomiyasu lint artifact, SHA-256
`047b38a29c22ec46ef7e22d65194a8beee088b44ca7ed712ec787f6e5c96216a`.
No fresh upstream download was performed.
It reports 52 sentence-ending repetition warnings and three negative-parallelism
notes. The negative wording preserves relevant distinctions: lazy pulls versus
an already-built collection, small Int/number comparisons rather than type-wide
equivalence, and construction rather than annotation doing Set deduplication.
The polite explanatory endings are retained where artificial variation would
obscure the subject and action. The report is advisory, not a pass/acceptance
score. No further rewriting was performed solely to reduce its score.

Final scoped TypeScript/Biome checks and Seseragi format checks pass. No lint
server or browser process was started for these checks. Browser layout,
interaction, complete-site integration and actual first-time human acceptance
remain separate. No public tracker, repository push, PR or deployment was made.


A final source-format audit found two canonical native programs, Map and
SizeError, needing line wrapping. The canonical formatter changed only those
programs; all tests were rerun successfully with their new source hashes and
Playground seeds. Exactly six rendered bodies changed (Map, SizeError and
NonPositiveSize in both locales); the other twelve remained byte-identical.
No prose, behavior or output changed. The final example manifest supersedes
initial example source hashes for these two files. All fourteen native positive
and rejected fixtures and all twenty-nine typed modules now pass canonical
format checking.

The independent reviewer reread all six formatter-affected complete bodies and
found no blocker. It independently verified that the other twelve HTML bodies
are byte-identical to the previous review, all eighteen paragraph/heading
sequences are unchanged, all eighteen native panels equal their decoded
Playground seeds, and the six affected panels equal the final canonical source
bytes. `format-reread-check.json` retains these checks. The final rendered hash
and reviewer manifest hash above supersede earlier formatter-preceding hashes.
