# Nine exact API summary corrections — local verification

This batch corrects nine reader-facing API pages under the existing #601/#630
site work. It does not change compiler declarations, the canonical reference
JSON, runtime behavior, or the signatures and constraints rendered from that
JSON. It is not acceptance of the surrounding modules or the whole API catalog.
No new public issue/comment, browser session, deployment, or remote write was
performed for this batch.

## Exact scope and source boundaries

| Identity | Kind | Actual operation taught |
| --- | --- | --- |
| `std/map::get` | function | Key lookup with present, absent, and present-empty-string results |
| `std/web/storage::get` | effect-function | Area/key storage read with separate absence and typed access failure |
| `std/transformer/state::get` | function | Read carried state without changing it; retain the chosen outer computation |
| `std/queue::take` | effect-function | One FIFO take; open-empty waiting and closed-buffer draining |
| `std/non-empty-list::head` | function | Direct first element from a guaranteed nonempty input |
| `std/non-empty-list::tail` | function | Ordinary, possibly empty List after the first element |
| `std/non-empty-list::toList` | function | Preserve all elements while forgetting the nonempty type guarantee |
| `std/set::toArray` | function | Array conversion in specified insertion order |
| `std/set::toList` | function | List conversion in specified insertion order |

The nine `page/en/ja.ssrg` modules live at corresponding module/operation paths
under `apps/site/src/reference/editorial`. `correction-model.ssrg` renders their
paired copy. `correction-catalog.ssrg` requires exact identity, module, value
namespace, and item kind. The parent integrated it before the previous
function-only dispatcher; the existing collection dispatcher remains guarded.

`apps/site/scripts/api-corrections.ts` registers 18 canonical source records:
nine Seseragi and nine TypeScript. The files are under
`apps/site/examples/src/api-corrections` and
`apps/site/examples/comparisons/api-corrections`. The additional native
`queue-cleanup.ssrg` supports verification and is not an extra article panel.
Eight primary programs and the cleanup fixture are process-compatible.
`storage-get.ssrg` is an integration module with no `main` and no seeded
Playground link. It is excluded from the process example aggregate.

The exact false upstream descriptions remain metadata debt in
`examples/spec/artifacts/stdlib-schema-1/reference/module.json`; the site's
identity overlay corrects both page summaries and declaration descriptions.
Untouched controls continue to display their original descriptions. Changes to
the compiler-owned metadata need their own authorized work and are not hidden
inside this editorial correction.

## Executable verification

Base commit: `6bd210c7903b9c98b1de9f924daa750c191640dc`.
Tests use Bun 1.3.9 and the official Linux CLI 0.61.19, release commit
`a8641b5a81a4`, selected through `SESERAGI_BIN`. This batch has no compiler or
runtime edits. The release compiler/runtime sources matched the current source
at the preceding integration checkpoint.

From the repository root, after sourcing the established environment:

```sh
source ../seseragi-env.sh
export SESERAGI_BIN="$(cat /tmp/seseragi-first-run-699-path)/bin/seseragi"
API_CORRECTIONS_RENDER_DIR=/tmp/seseragi-nine-api-rendered \
  bun test apps/site/tests/api-corrections.test.ts
```

Result: **7 tests, 625 assertions, 0 failures**. The suite covers:

- Eight independent native programs with exact stdout, empty stderr, source
  hashes, and seeded-link/source equality; plus scoped queue cleanup output
- All nine TypeScript bridges under strict TypeScript checking, seven complete
  programs with exact output, and mock calls to the two integration consumers
- Web-target compilation of the Storage module with a temporary caller; this
  builds an artifact without executing a browser
- Real browser-storage adapter and provider decoder over an isolated mock host:
  area/key routing, Local/Session, present text, empty text, absent key, and
  SecurityError conversion. Host writes/removals/clears throw if attempted
- The existing bounded `runtime/ts/probes/effect-concurrency.ts` runtime probe,
  including already-waiting taker cancellation, FIFO and closed draining
- Nine wrong-argument mutations. Map, StateT, NonEmptyList head/tail report
  `SES-T0101`; Queue, Storage, NonEmptyList toList, and Set conversions report
  `SES-T0201` through the enclosing Functor/Show call. These are the observed
  diagnostics, not invented direct argument-error messages. The unchanged
  published programs are the executable repairs
- 51 identity-dispatch probes, including wrong module/kind/namespace and
  same-name other modules; the old Array chunksOf overlay still applies only
  to its original function kind
- A focused production render of 42 pages: the 18 corrected bodies, six module
  overviews and six untouched API controls in both locales. Exact declarations,
  canonical panels, outputs, all local EN/JA array paragraphs, and absence of
  opposite-locale paragraphs are asserted

Each spawned native/probe/typecheck command has a 30-second limit; the existing
focused-render helper retains its 90-second limit. This focused suite does not
replace full-site generation, navigation/link checking, or browser verification.

The scoped cleanup fixture confirms cleanup before a later offer/take. Its
readiness handshake is not proof that the child reached a blocked take. The
separate runtime probe establishes that waiting-cancellation boundary. No
sleep-based timing assertion is used.

Storage's terminal panel is explicitly labeled as mock-host cases, not visible
output from the source. Its function returns text for an application to use.
The TypeScript catch intentionally handles a broader class of thrown values
than Seseragi's typed `recover`, and the page states that difference. `Never`
is explained as a handled typed failure channel, not impossible runtime failure.
No real browser storage was accessed. These proofs are not browser acceptance.

## Reader review and final local checks

The first render incorrectly paired paragraph arrays because the new helper
passed `arrays.zip` arguments in the wrong order. Its earlier content acceptance
was withdrawn. The Japanese and English source copy was present; rendered
walkthrough/rule paragraphs were swapped. The helper now uses the established
`arrays.zip japanese english` order. Tests require every expected paragraph and
reject every opposite-locale paragraph for all 18 bodies. Paired array lengths
are checked as well, so a zip cannot silently omit a paragraph.

An independent reader fully reread the corrected 18 bodies. No blocker remains.
The final pass also checked:

- The head comparison describes scalar output rather than array output
- Every NonEmptyList page explains its own `singleton`/`cons` construction
- Storage defines the service locally and links the exact existing environment
  requirements title with a concrete purpose
- All 16 native seeded links equal their displayed source; both Storage pages
  have no seeded launch

All nine Japanese rendered bodies were also self-read in full. Advisory prose
lint ran over those nine bodies using the established pinned yomiyasu helper:
27 sentence-ending repetitions and nine technical negative contrasts remain.
They do not change the verified contracts and were not mechanically rewritten.

Final scoped static checks passed: Biome over 11 files, the configured-style
TypeScript check of the helper/test closure, `seseragi format --check` over 39
owned source files, and `git diff --check`. The hidden queue-cleanup fixture
needed a second formatter pass to stabilize one closing-brace indentation; its
native test was rerun after that whitespace-only change.

Successful current rendered bodies are retained at
`/tmp/seseragi-nine-api-rendered`. A route-shaped, 18-article copy and full local
logs/inventories are under `/workspace/shared/seseragi-nine-api-authoring`.
The reader ledger records the independent artifact hash separately. Shared
registrations/aggregate imports are parent-owned; final locks and broad
integration belong to the next common checkpoint.
