# Map operation explanation evidence

Scope is fifteen existing identities: the std/map module and empty, singleton,
fromEntries, containsKey, insert, upsert, remove, keys, values, entries, size,
mapValues, mapKeysWith and mergeWith. Both locales are authored in typed modules.
The existing get correction is retained; filter and isEmpty are controls.
See [local work item](map-editorial-work-item.md).

## Behavior and comparison evidence

Fourteen complete Seseragi programs and their displayed TypeScript counterparts
execute independently with a fifteen-second bound per process. One test passes
84 assertions for successful execution, exact equal output and canonical source
links. TypeScript counterparts copy before mutation when preserving the source
is part of the task, use presence checks for empty values, and explicitly combine
colliding values instead of silently overwriting them. Each file typechecks.

- Duplicate input keys use the final value but keep the first position.
- Existing-key insert/upsert retains position; new keys append. Removal followed
  by reinsertion appends the key, and a missing removal keeps the same mapping.
- A present empty string takes upsert's Just branch; Nothing is a missing key.
- keys/values/entries return insertion-order Arrays; values may repeat.
- mapValues preserves keys and source values. mapKeysWith resolves three keys
  as A/B then A/B/C, retaining the first output-key position.
- mergeWith receives right before left, but its resolver receives leftValue
  before rightValue. Swapping the Maps changes the concatenated values and order.

Normative source: docs/spec/10-library-surface.md §10.5. Implementation sources:
runtime/ts/src/map.ts and the current canonical standard reference artifact.
The existing runtime Map suite passes ten tests / 4,058 assertions, covering
callback counts/order, equality/hash collisions and preserved versions. These
are supplemental runtime checks, not a browser or reader acceptance claim.

Exact-identity dispatch and the wrapper around the runtime suite pass two tests
with nineteen assertions. The dispatcher test accepts fourteen new identities
and rejects wrong namespace/kind/module/symbol combinations. It does not replace
or broaden the separate correction dispatcher.

Production rendering passes 416 assertions across 36 locale pages: thirty
edited bodies, the preserved get page and untouched filter/isEmpty controls in
each locale. It checks exact declarations, canonical Seseragi and TypeScript
panels, matching output, locale field parity and all fifteen authored selector
links against destination titles. Fresh HTML: `/tmp/map15-rendered`.

Total author checks are four tests / 547 assertions, plus the nested existing
runtime suite noted above. TypeScript, Biome (helper/test), Seseragi formatting
and whitespace checks pass. The TS comparison files retain three advisory
string-concatenation style suggestions for readability; there are no TS errors.
The Japanese advisory on fifteen edited bodies, before the final optional
type-argument wording clarification, records {'sentence_end_repetition': 65, 'negative_parallelism': 3, 'excess_list': 1}. Technical contrasts
and the useful operation-selector list are retained. Report:
`/tmp/map15-prose-edited.json`.

The fixture uses real production import closures and only the filtered Map
module; it does not verify all cross-module or type navigation. Independent whole-body reading has covered all thirty bodies. It requested
shared syntax/copy corrections: name Int or String values accurately, explain
empty-call type arguments without assuming Int values, and distinguish arrows
between parameters from the final result-type arrow. A targeted reread also
clarified that explicit type arguments are optional when a surrounding annotation
supplies the empty Map types. Both locales are corrected;
programs and output are unchanged. The final production render passes 416 assertions. The independent targeted
reread cleared all corrected EN/JA statements, including all twenty-eight leaf
instances of the optional type-argument explanation. All thirty edited bodies
have no remaining reader blocker; paired panels and seeded links are consistent.
Final edited-artifact digest:
b08e638194e5f66ba2d8efa00bd25321ebdfbe2d6d1516e91546befcc5273a4f.
This simulated-reader evidence does not establish browser or real-human acceptance. Parent integration
owns aggregate imports, final locks and full-site checks. No public tracker
comment, commit, push, PR, browser verification or deployment is implied here.

## Checkpoint 5 exact module-title repair

Full-title preflight found 28 leaf links labelled Map while their module's H1
is std/map. The shared label now uses std/map in both locales; destinations
and adjacent purpose text are unchanged. The focused test now checks every
authored leaf body as well as the module selector against real rendered titles:
58 title links across thirty bodies. The refreshed render passes 444 assertions.
TypeScript, Biome and whitespace checks pass. Independent label-only rereading
verified all 28 exact replacements and retained the prior semantic review.
Current selected-artifact digest:
9414135193d2bef12a80a9a207d3001394d9edff50fc0a86cc4d58b027219367.
This replaces the earlier digest above; browser/human limits remain.
