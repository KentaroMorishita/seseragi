# Array/List construction and pairing review evidence

Work item: [#714](https://github.com/KentaroMorishita/seseragi/issues/714), under
#601 / #629 / #630. Local baseline: `0ee111373e56f0316a5fc26b9f893d2cef4c7a0b`.
This bounded review covers existing pages, not the absent article roadmap.

## Scope and ownership

The paired English/Japanese routes are `/docs/library/array/`,
`/docs/library/list/`, and each module's `function/empty/`, `singleton/`,
`fromiterable/`, `zip/`, `zipwith/`, and `unzip/`: fourteen page identities.
The Array introduction gains a construction/pairing selector; its earlier
examples and Maybe/Either consumers remain.

Each of the twelve functions has an exact compiler identity selected with its
module, namespace and kind. Page-specific prose is owned by typed `en.ssrg` and
`ja.ssrg` files. `sequence-model.ssrg` provides the small shared explanatory
structure; `sequence-editorial.ts` owns only source identities and expected
output. Compiler reference declarations, descriptions, constraints and runtime
code remain unchanged.

## Meaning checked against current sources

- `docs/spec/10-library-surface.md` §10.5 defines sequence order, strict results,
  shortest-side pairing, input preservation and processing cost.
- The canonical reference JSON supplies `empty<A> arg1: Unit -> Array<A>` /
  `List<A>` and the right-first argument order for `zip` / `zipWith`.
- `runtime/ts/src/array.ts` and `list.ts` implement empty/singleton,
  finite `fromIterable`, pairing and separation. List's runtime constructor is
  `singletonList`; the public Seseragi function remains `std/list::singleton`.
- `runtime/ts/tests/sequence.test.ts` verifies callback argument/order/count,
  shortest-side termination, generic tuple values and custom Iterable order.

Every operation has a complete executable example for each collection type:

- `empty`: Unit `()` calls, annotation-based element type and explicit `<Int>`.
- `singleton`: integer, string, nested empty collection, and the length of a
  singleton empty string. The final result is one, avoiding the ambiguity of
  showing an empty string without quotes.
- `fromIterable`: conversion from the other finite collection, preserved
  `[30, 10, 20]` order, unchanged source and empty input. Endless input is
  described but deliberately not executed.
- `zip`: `(name, score)` versus `(score, name)`, unequal lengths in both
  argument orders and both empty-side cases.
- `zipWith`: subtraction makes left/right reversal observable; extra elements
  are discarded and either empty side produces no callback results.
- `unzip`: complete tuple binding `let (names, scores) = ...`, use of both
  returned collections, and the pair of empty collections from empty input.

List literals, tuple positions, annotations and the output wrapper are explained
locally. These operations return ordinary collections, so no new Maybe/Either
consumer is needed. The preserved Array module retains those earlier examples.

## Verification

The first focused suite passed three tests with 428 assertions. It executed all
12 canonical sources and rendered 32 routes: 14 edited identities plus the two
untouched `length` controls, each in both locales. Checks compare exact compiler
signatures, displayed source and output, source-bearing Playground URLs,
module/API links and unchanged control descriptions. Separate dispatch probes
reject wrong modules, namespaces, kinds and identities. Array's inventory test
now recognizes the six additional exact identities while retaining its earlier
pilot checks.

The existing runtime tests selected by `zipWith preserves|zip/unzip` passed:
three tests and 31 assertions. They cover both collection implementations and
custom Iterable behavior. No runtime test or implementation was changed.

The combined new, Array and Text editorial suites passed eleven tests with
1,290 assertions. This retains the previous Array/Text examples, signatures,
consumer bridges, exact descriptions for untouched controls and dispatch guards.
The final focused rerender after the wording pass passed one test with 366
assertions. Its current 32 HTML files are in `/tmp/sequence714-rendered`.
The source and rendered bodies include the local explanation of `let`. Japanese prose checking is advisory; module indexes are useful
lists rather than narrative paragraphs. A small style pass improved repeated
endings and removed an unnecessary contrast while preserving technical rules.

## Review limits

The focused renderer uses the actual production import closure, compiler
metadata and document renderer. Its navigation fixture contains these twelve
functions and two controls. Links from the existing Array introduction to the
older pilot are valid production routes but are outside this filtered fixture.
It is not evidence for the complete catalog, a browser layout check, or a human
reader's acceptance. Independent text/HTML review is a separate step. Parent
integration owns final locks, full-site validation, commits and publication.

Final prose check: 16 Japanese routes (including two controls), {'sentence_end_repetition': 23, 'excess_list': 2}.
Remaining ending repetitions were read in context; both list-ratio notices
refer to module operation indexes. No technical distinction was removed to
satisfy an advisory style score. Final report: `/tmp/sequence714-prose-report-final.json`.
The independent simulated-reader review read all fourteen edited page identities
in both locales and found no remaining explanation blocker. It requested one
small English correction: nine occurrences of “a Array” across five files
now use “an Array”. The corrected production rerender passed one test with
366 assertions; the five current English HTML bodies were also checked for
the corrected wording. The reviewer is rereading that correction separately.
This is text/HTML evidence, not human or browser acceptance.
