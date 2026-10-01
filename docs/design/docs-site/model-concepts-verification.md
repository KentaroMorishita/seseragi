# Model concepts and grammar reader verification

Verified 2026-10-01 from local checkpoint
`882a6814d7c965da375ba56c1bef084e773820eb`. This record covers the four conceptual
pages below. The accompanying six-principle pass completes the bounded ten-page
Model/Grammar scope; `what-is-seseragi` and `immutable-by-default` are unchanged.
These are scoped implementation and agent reading checks, not human first-reader,
browser, deployed-site or whole-corpus acceptance.

## Four page identities and their purpose

- `language.model.programs`, `/docs/language/model/programs/`: start with a
  function that constructs a greeting and a compact public main that returns
  output work. Explain initialization, Effect construction and runner execution
  separately. The existing explicit `Unit` / `Console` / `ConsoleError` example
  comes later, with each term explained locally. Libraries, package entry
  selection, service provision, imports and ordinary-failure display rules remain.
- `language.model.non-features`, `/docs/language/model/non-features/`: show a
  normal score calculation, retain the original value and use an if result.
  Preserve all ten alternative categories and the boundaries around Maybe,
  Either, Effect, arbitrary unions/intersections, deriving/macros and explicit
  state/early termination. The page does not invent an invalid example simply
  to fill a template.
- `language.model.design-principles`, `/docs/language/model/design-principles/`:
  retain an overview that helps readers choose a relevant question. It is not
  made into another compulsory course or a fictional executable lesson.
- `language.grammar`, `/docs/language/grammar/`: explain a small EBNF rule,
  distinguish grammar notation from source, then map a real declaration with
  and without an annotation to the production. Retain the complete normative
  grammar, syntax/semantics distinction, ambiguity rules, source information
  and purpose-labelled links.

All four have paired Japanese and English content owned by typed page modules.
Existing important heading IDs and page identities remain. The established
`explainPage (guide.explanation ()) (content ())` contract is preserved for the
three model pages; Grammar retains its existing input-dependent page function.
Programs uses `WithoutExample` so the heavy explicit entry signature is not
moved ahead of the ordinary calculation and its explanation.

## Canonical executable evidence

New examples are registered through `scripts/model-concepts.ts` and the existing
`canonicalExample` machinery. The old explicit-entry source is reused, not
registered a second time.

| Canonical ID | Exact stdout |
| --- | --- |
| `model-reader-programs` | `Hello, Aki!\n` |
| `model-reader-non-features` | `79, 80, pass\n` |
| `model-reader-grammar` | `3, 4\n` |
| `language-program-entry` (existing source) | `Hello, Seseragi!\n` |

The first three files are in `apps/site/examples/src/language/` using their ID
as the basename. The existing entry source is `program-entry.ssrg` there.
All four pass native lint and run as independent `main.ssrg` files in fresh
temporary directories. Their exact Playground URL seeds compile in the
committed WASM compiler and execute through the existing Playground runtime
with matching output (its capture API trims the final newline). Native checks
assert the complete stdout. Hash and highlighted-token text equal file bytes.

Two meaningful boundary examples are checked:

1. `model-reader-entry-rejected.ssrg` under the valid-source directory contains
   a pure `pub fn main -> String`. It is a valid function declaration and lint
   succeeds, but run exits 2 with `invalid entry point: program must export
   pub effect fn main` (the actual message quotes the signature with backticks).
   The page explicitly distinguishes these checks. Replacing it with
   `pub effect fn main = println (greeting "Aki")` prints `Hello, Aki!\n`.
   This source is not added to the aggregate of type-invalid fixtures.
2. `model-reader-grammar.ssrg` under `examples/invalid/src/language/` assigns
   `"three"` to an Int-annotated binding. Its grammatical form is valid, but
   lint rejects it with status 2 and SES-T0101. Replacing the String with `3`
   runs and prints `3\n`.

Neither boundary panel has a runnable Playground link. Programs also displays
the tested `seseragi lint main.ssrg` and `seseragi run main.ssrg` commands; setup
remains owned by the existing First Run page. The prose uses familiar TypeScript
operations as context without inventing a padded comparator or language-wide
superiority claim.

## Normative grammar and identifier terminology

`docs/spec/grammar.md` was not changed. The focused test reads its existing EBNF
fence, applies the same trailing-whitespace removal as `generatorInput`, and
asserts that the rendered complete-grammar terminal contains those exact bytes.
The illustrative boolean rule is clearly labelled as notation, not a replacement
for the normative productions or a program to run.

The uncased-identifier explanation is grounded in three distinct sources:

- `docs/spec/01-syntax.md` §1.1 permits Unicode XID identifiers and explicitly
  demonstrates `let 次の値 = 42`.
- `lexer.rs::scan_identifier` classifies an uppercase initial character as
  `IdentifierUpper` and otherwise assigns a valid identifier to
  `IdentifierLower`. `lexes_unicode_and_apostrophe_identifier_spelling` explicitly
  asserts that `次の値'` is an `IdentifierLower` token.
- The grammar defines `lower-name = LOWER_IDENTIFIER`. Its trailing explanation
  speaks generally about dividing identifiers by initial case, while §1.9's
  blanket lowercase wording does not explicitly describe uncased characters.

This is a normative wording ambiguity, not permission to rewrite the language
around an observed implementation. The article clarifies the category name
consistently with §1.1 and the existing lexer test; neither the productions nor
compiler/spec behavior was changed. It also distinguishes an operator's
precedence from its associativity instead of conflating them.

## Checks and review boundaries

- `tests/model-concepts.test.ts`: **8 passed, 0 failed, 247 assertions** in the
  last focused run (12.22 seconds). It covers the four programs, both boundary
  checks/repairs, WASM/runtime, and eight real EN/JA article renders.
- Tests compare related-link labels against current destination locale titles,
  rather than accepting an approximate or stale label. Code panels, output,
  locale routes, unique heading IDs and retained anchors are checked.
- Scoped TypeScript/Biome, page/example formatting and `git diff --check` pass.
  The runtime was Bun 1.3.9; native checks used the published Linux x64 CLI
  `0.61.19 (release, commit a8641b5a81a4)`.
- `target/site-model-concepts-review` contains the eight production-module
  renders. A deliberately small catalog has no complete sidebar areas, so this
  is not proof of all-site navigation, route closure, responsive CSS, keyboard
  behavior or a browser journey.
- Both locale bodies were read. The author corrected an above/below reference,
  explained trait/result vocabulary locally, restored the module/import boundary
  and simplified the overview's purpose. Rendered Japanese advisory lint was
  run with the pinned yomiyasu guidance. Remaining list-density flags correspond
  to an intentional navigation list, the ten retained alternatives and related
  links; they are not treated as readability acceptance scores.
- Independent reading of all twenty Model/Grammar bodies is tracked separately.
  No new GitHub issue/comment, commit, push, PR, release or deployment was made
  by this scoped pass. Full integration and lock updates belong to the parent
  checkpoint, not this focused proof.
