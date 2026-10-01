# Set operations and selection: local work item

Parent scope: #601 / #629 / #630. Local draft only, not a published issue.
Baseline: afe906eee87dd0b588730a578554851f0e2404ac.

Improve the existing std/set module and twelve generic function pages: empty,
singleton, fromIterable, contains, insert, remove, size, map, union, intersection,
difference and isSubsetOf. These are thirteen existing identities and twenty-six
English/Japanese bodies. Preserve the earlier toArray/toList corrections and
retain filter/isEmpty as unchanged controls. No absent routes are added.

The ordinary TypeScript reader should understand membership, uniqueness,
insertion order, unchanged source values, explicit directional calls and the
possibility that mapping collapses several inputs into one result. Explain Unit,
optional type arguments, Array/List spelling and the executable wrapper locally.
Use fair TS counterparts with the same small-integer inputs and observable output.

Acceptance: exact module/namespace/kind dispatch; canonical bilingual programs,
TS counterparts and outputs; ordering/empty/deduplication boundaries; existing
runtime callback and equality contracts; all authored page-title links checked
against actual H1s; focused production rendering and independent bilingual body
review. Compiler/runtime/metadata, shared registries and final locks remain under
the existing ownership constraints. No public write, push or deployment.
