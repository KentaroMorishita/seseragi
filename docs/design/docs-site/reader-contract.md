# Reader contract for the official Docs

Route coverage, bilingual module presence, compilation and browser checks are
technical evidence. None of them proves that a person can understand an article.
The previous core-corpus completion claim is withdrawn for reader acceptance.

## Reader and assumed knowledge

The primary reader develops everyday applications in TypeScript. They use
ordinary `type` / `interface` declarations, functions, parameters, return values,
arrays, objects, `if` and `try` / `catch`. They are evaluating a new language,
not looking for type puzzles. Do not assume Rust, Haskell, functional-programming
terminology, algebraic data types (ADTs), pattern matching or advanced type theory.
Even a TypeScript discriminated union needs a local explanation if a comparison
uses one; knowing `type` does not imply knowing that technique.

| May be familiar | Explain when first needed | Keep after the basic case or in a linked detail section |
| --- | --- | --- |
| A value, a name, ordinary numbers/strings/booleans, arrays and object fields | Seseragi `let`, immutable values, `Int` / `Float` / `Bool` / `String`, literal spelling, a type annotation | Numeric bounds, representation, `Never`, state APIs and persistent-data implementation |
| Function declaration, arguments, returned values and TS type annotations | `fn`, each `:` and `->`, expression bodies, a block's final value, calls separated by spaces | Partial application after an ordinary call; explicit type arguments, rank, kinds and compiler lowering later |
| An object has named fields; a condition chooses a branch | A value with one of several named alternatives; constructing one; `match` branches and handling all alternatives | Generic ADTs, nested patterns, guards and formal exhaustiveness rules |
| An operation may fail; `try` / `catch` handles a thrown exception | A missing value, a success/failure value, and how the caller handles each; introduce `Maybe`, `Either` and then `Effect` by the problem each solves | Trait machinery, Effect requirements, defects, cancellation and resource rules after the relevant simple case |

“Later” is not permission to omit a rule, and a link is not a prerequisite
lesson. Give the reader enough explanation here to read the example. For a
deep-link visitor, briefly restate the needed idea rather than requiring a
previous article. Do not equate TS `const` with immutable object contents, TS
`number` with every Seseragi numeric type, or `Either` / `Effect` with
`try` / `catch`; explain the useful similarity and relevant difference.

## Concept dependencies, not a reading course

Use these dependencies to choose examples and order explanations **within** an
article. They do not impose a site-wide Next sequence. Concrete page owners and
the bounded pilot are in [site architecture](site-architecture.md#12-entrance-and-pilot-page-map).

| Concept to explain | Enough context to establish first | Boundary of the first explanation |
| --- | --- | --- |
| Values and names | A familiar calculation or named object field | Name a value with `let`; read the result; show that updating creates a new value |
| Basic type annotations | A value and its name | Read `name: Type`; distinguish a checked annotation from converting a value |
| Function declaration | Named values and the basic types used in this example | Identify the function name, each argument, return type and body; explain `fn` and `->` locally |
| Function call | A fully defined small function | Supply all arguments and explain the result before introducing a function waiting for more arguments |
| Blocks and ordinary branching | A call and a returned value | Explain local names, the final value and a Bool condition; explain `if` before using it to motivate other branch forms |
| Data alternatives | Simple values and fields | Show one everyday value that can have different named shapes; define the alternatives before saying “ADT” |
| `match` | The alternatives in this example, and choosing a branch | Construct an input, read the matching branch and explain why every possible alternative needs handling |
| Missing values and failure | The reader's `try` / `catch` experience; locally explain alternatives and `match` if used | Separate “no value” from “failed”; show what the caller receives and how it responds, before a general Effect signature |

ADT, `match` and failure are the next candidate scope after the values/functions
pilot is reviewed in #702, not additional prerequisites for that pilot. A
complete executable wrapper may need `main`, an import or output operation.
Explain what those lines do without turning them into an Effect/type-system
lesson; distinguish the calculation from the wrapper. Do not label unexplained
code “boilerplate” and expect the reader to ignore it.

## Showing the appeal with TypeScript comparisons

Begin with a recognizable task and use idiomatic TypeScript for that task.
Both versions must have the same input, result and relevant constraints. Do not
pad TypeScript with needless classes, types, mutation or exception handling, or
select different failure behavior to make Seseragi look better. A TS solution
can already be concise; say what, if anything, Seseragi changes in that case.

Explain the observable benefit: which data shapes are visible, how the flow can
be read, or which mistake the compiler catches. Fewer characters alone is not
an argument. The first entrance comparison must be readable without ADT,
`match`, traits or Effect knowledge. More demanding comparisons belong after
those ideas have been introduced, not in an unexplained hero example.

#698 owns selection and verification. Use the existing canonical/site-owned
example machinery: typecheck and execute both versions, compare outputs and
stated failure behavior, and keep displayed and Playground source tied to the
verified source. State numeric, mutation, runtime or error differences that
limit the comparison. Do not imply whole-language superiority, unsupported
features or production readiness from a single example.

## What the reader must be able to do

An article must answer one named question without sending its reader away to
discover what that question means. The Docs must explain the language's purpose,
appeal and basic concepts without a Tour visit. An article
is not a Tour lesson, but being a reference does not excuse missing explanations.

1. Explain the purpose in ordinary language before stating formal rules.
2. Introduce each necessary term where first used. Identify any genuine
   prerequisites, and summarize the part needed here; a prerequisite link alone
   is not an explanation.
3. Define every name in the first example. Start with the smallest example that
   demonstrates this concept, not an unrelated complete Tour lesson.
4. Explain how to read that example, what its important lines do, and why its
   output or type follows. Execute runnable examples and check actual output.
5. Distinguish the simple case from advanced rules. Explain type, evaluation,
   failure, resource and cost rules where they apply, with their consequences.
6. Describe a plausible mistake, its diagnostic or result, and how to correct it.
   Define additional names in rejected snippets or explain their relevance.
7. Avoid compiler-internal vocabulary unless the reader needs it. When needed,
   explain it; do not swap English jargon for equally unexplained Japanese jargon.
8. Review Japanese as an explanation written for a reader, not as a literal
   translation. Page title, breadcrumb, navigation labels and links must agree.
9. A related link states the question it helps answer. It is optional after the
   local explanation, not a mandatory unexplained jump.
10. Previous/next stays within the same navigation section. Show the current
    section and its overview. Category changes require a deliberate choice.

## Authoring boundary

Keep `page.ssrg`, `en.ssrg` and `ja.ssrg` as typed Seseragi. The existing semantic
Block model remains the sole article renderer. `reader-article.ssrg` shares a
reader-facing explanation pattern with arrays of page-owned paragraphs; it is
not a second document format. More specialised topics may use their own typed
sections rather than forcing irrelevant headings into this pattern.

Preserve normative traceability in the internal content map. Neither a public
specification-chapter label nor a link to a specification replaces explanation.
Do not silently change a specification or compiler behavior to make an example
appear to work. Record discrepancies and use genuinely executable examples.

## Japanese-first editing with yomiyasu

Use the `tech` guidance from [yomiyasu](https://github.com/nanaism/yomiyasu),
pinned for this review to `30ee6041c328ce21d38a7963f667e079a93d7a12`.
Write the Japanese explanation first. Then revise English from that explanation,
preserving the same example, conditions, restrictions and result. The normative
specification and executable sources remain the authority for language behavior.

Identify the subject, action and object of each explanation. Replace vague
references and abstract conclusions with the actual value, operation or error.
Explain a required term locally before using it. Keep a real restriction or
negative example when it is needed to explain correct usage; do not remove it
merely because a style checker detects a negative sentence. Retain readable
paragraphs and use lists for genuinely parallel items such as alternatives.

Review the entire rendered page, including existing rules and mistakes, not only
the introductory explanation. Review generated module and API pages too: their
shared templates and descriptions are reader-facing prose, not exemptions.
Keep source keywords, code, signatures and diagnostic identifiers unchanged.

Run the bundled yomiyasu lint on prose extracted from rendered Japanese pages.
Exclude code blocks and mark inline code so prose rules do not alter syntax.
Its report is advisory, not an acceptance score. Make at most two editorial
passes for its findings; record intentional technical exceptions instead of
rewriting valid facts to obtain a clean score. Run Python in the foreground and
verify that no lint or browser process remains after the checks.

## Reader acceptance remains separate

Review both rendered locales against the everyday TypeScript reader described
above, not an unspecified “basic programming” audience. Answer the ten questions
above for the page, not for the overall language. Read the text in sequence,
follow a necessary link and return, and check mobile/desktop layouts.

No minimum word count, heading count or automatic jargon detector establishes
reader acceptance. Automated checks protect examples, paragraph pairing, exact
titles, navigation boundaries and rendering. They cannot replace reading.

Keep completion evidence separate:

- **Code and meaning:** exact source, command/tool version, output or diagnostic,
  semantic source and relevant TS comparison results. A typecheck alone does not
  establish the claimed behavior.
- **Display and navigation:** both rendered locales, desktop/mobile code and
  prose, headings, same-identity locale links, related-link round trips and
  section-local previous/next. A blocked build/browser check remains blocked.
- **Reader review:** can the reader explain the task, important lines, result,
  benefit and next destination without coaching? Record where they needed help.
  Label author/agent self-review separately from an actual first-time reader's
  feedback, with the reviewer's relevant prior knowledge.

No one category stands in for the others. The design-only #697 change selects
the contract and scope; it does not accept rendered pages. #698/#699 verify
examples and first-run steps, #700/#701 apply them, and #702 records first-time
reading feedback and decides whether to expand the scope.

`reader-review.md` inventories all 106 core routes. Its eight existing checks
record earlier bounded self-review, not acceptance under this revised contract
or first-time-reader sign-off. Keep that history and record new evidence per
changed route; never bulk-check or erase the outstanding work. Non-core semantic
explanations remain separate from generated API signature coverage. Route
presence, translation completeness and word/heading counts cannot close them.
