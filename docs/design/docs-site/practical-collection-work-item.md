# Everyday collection operations reader batch

## Reader task

An ordinary TypeScript developer wants to read an optional item, select matching
records, show a page of results, count them, or choose an empty-state message.
Teach the native Array, Map and Set task first, then the corresponding Seseragi
operation, its useful difference and only the syntax needed for that example.
Japanese is authored first, followed by English.

## Exact existing scope

The batch covers twenty existing public functions and forty EN/JA bodies:

- `std/array::{get, head, isEmpty, length, filter, find, take, drop}`
- `std/list::{get, head, isEmpty, length, filter, find, take, drop}`
- `std/map::{filter, isEmpty}`
- `std/set::{filter, isEmpty}`

The exact owner/identity/namespace/kind tuples and routes are recorded in
`apps/site/scripts/practical-collection.ts`. Every selected item is a value
function with process and browser support. Existing Array, List, Map and Set
module entrances already reach these routes and are not counted or rewritten.
The shallow family dispatcher rejects wrong owners, namespaces, kinds and
lookalike identities. No declaration-reading override is required.

Fourteen canonical Seseragi/TypeScript pairs provide the visible task examples.
Three separate boundary pairs cover present zero, false and empty strings.
Seven invalid fixtures and complete repairs establish the actual type errors.
No terminal/settings/stdin article, new module, compiler signature, instance or
runtime behavior is introduced.

## Comparison and representation boundaries

Use idiomatic native TypeScript indexing, filter, find, slice, length and
Map/Set entry/element arrays. Those native operations are already concise and
filter/slice do not mutate their input. Do not lengthen the TS example to
manufacture a brevity advantage. The List comparisons use arrays for the same
task; they do not assert equivalent storage or performance.

- get is zero-based; negative indices are absent, unlike Array.at(-1)
- head and find return Maybe; Just 0, Just False and Just empty text remain
  present values, not absence
- filter keeps every match; find stops at the first match
- take/drop treat negative counts as zero, so the TS boundary example clamps
  with Math.max rather than silently using negative slice semantics
- Map filter receives key first, value second; Set filter receives an element
- Map/Set retained order is insertion order, not a sort
- isEmpty tests element/entry count, not the truthiness of stored data
- Array length/head/isEmpty use direct queries; List length/get traverse the
  linked representation, while List head/isEmpty inspect its first constructor
- List drop can share its remaining tail. Complexity and sharing statements
  come from implementation evidence, not elapsed test timings

Examples use small finite in-memory values. The calculations need no provider;
Console is used only to display their results. JSON encoding makes displayed
observations comparable between languages. The examples use primitive Map keys
and Set elements rather than pretending arbitrary JS object identity equals
Seseragi Eq/Hash behavior.

## Verification and acceptance

Keep format/typechecking, official-CLI process execution, strict native TS
execution on Node and Bun, committed-WASM/current-runtime checking, exact
rejection/repair diagnostics, whole-body rendering and independent language
reading as separate evidence categories. Source changes require rerunning the
exact changed files. TypeScript checking does not execute or polyfill code.

The browser-runtime test runs inside Bun. Real browser-host execution, UI
behavior, deployment and first-time human-reader feedback remain separate.
Whole-site integration, package locks and publication are parent-owned steps;
this bounded authoring task does not change runtime/compiler implementations.
