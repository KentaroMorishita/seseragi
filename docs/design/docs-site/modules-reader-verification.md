# Module reader batch verification (#711)

## Authored scope

The eleven existing `/docs/language/modules/` identities now have paired
Japanese/English explanations: identity, packages, top-level, visibility,
imports, specifier-resolution, re-exports, namespaces-and-resolution,
dependency-graphs, initialization and entry-points. The primary reader uses
ordinary TypeScript functions, objects, types and import/export. Necessary
Seseragi syntax and terms are introduced beside the actual files, before the
more detailed rules.

The import article uses ordinary two-file TypeScript and Seseragi greeting
programs with the same argument and exact output. It does not claim a
Seseragi-only advantage for a task both languages already express concisely.
The later articles distinguish declaration visibility from package reachability,
local aliases from declaration identity, pure initialization calls from stored
Effects, and an imported main from the manifest-selected entry.

The old deep rules remain, including root/package identity, Unicode/case/path
normalization, direct dependencies and exports, opaque representations,
namespace and instance visibility, operator association, re-exports and
cycles, initialization-before-use, entry requirements/failures, and foreign
loader versus developer-tool execution restrictions. The known top-level
expression discrepancy tracked by #679 is described as an implementation gap,
not published as a currently verified rejection. No compiler, runtime, loader
or security behavior changed.

## Exact executable sources

`apps/site/scripts/module-examples.ts` registers 93 canonical source records
from `apps/site/examples/projects/modules/`: 28 complete native projects
(16 valid and 12 rejected cases) plus a two-file TypeScript counterpart.
Project descriptors name every relative file, entry directory, expected
stdout or diagnostic, and complete repair. TOML records use exact plain text,
not the Seseragi syntax highlighter. All source records suppress single-file
Playground links because these articles explain complete filesystem projects.

The tests copy entire project directories to temporary roots, including both
packages in dependency cases. They update locks only in those copies and
assert that canonical source bytes remain unchanged. Every rejected project
has its own diagnostic test and an executed valid repair. Additional literal
prose repairs were checked independently: removing the two unused imports in
the type-only cycle prints `Aki`, and removing only `punctuate` from the private
re-export prints `Hello, Aki!` with the helper file unchanged.

Important verified observations:

- Named, aliased, namespace and re-exported greeting calls retain their output.
- `.ssrg` and extensionless imports share one named type; copied declarations
  instead produce SES-T0101 even when both printed type names are `UserId`.
- Pure `double 21` initialization supplies 42, while the dependency's stored
  Effect and public main produce no output.
- Private imports/re-exports report `PrivateExport`; opaque field access
  reports SES-T0101; duplicate value declarations report SES-N0002.
- Missing package exports report SES-N0104; value/type-only cycles and implicit
  directory-index lookup report SES-K0001; early reads report SES-N0201.
- A main succeeding with Int is rejected; the selected entry must succeed
  with Unit. Imported operator association yields `10 - (3 - 2) = 9`.
- The cycle repair explicitly changes the impossible original equations to
  calculations from a shared base; it does not pretend to preserve their result.

## Grouped source panels

The existing `explainPage` helper gained `ArticleExamples(Array<String>)`.
It moves complete panels in explicit order and removes only those selected
panels from their old positions. Missing IDs, duplicate requests and duplicate
matching panels produce visible build errors without partial selection.
Existing `ArticleExample` and `WithoutExample` HTML stays byte-identical.
The structural test recognizes formatter-introduced whitespace without
relaxing literal-ID membership or uniqueness checks.

This uses the existing semantic Block renderer and canonical ExampleSource
shape; there is no second renderer, prose format or Playground project loader.

## Validation

Environment: Linux x86_64, Bun 1.3.9, published CLI 0.61.19 (release commit
`a8641b5a81a4`). Its compiler/runtime sources match the checkout; the working
base is local checkpoint `0ee111373e56f0316a5fc26b9f893d2cef4c7a0b`.

The final scoped command ran:

- `module-projects.test.ts`
- `modules-reader.test.ts`
- `explanation-groups.test.ts`
- `explanations.test.ts`

Result: **41 tests passed, 15,313 assertions**, zero failures. It includes
22 rendered module pages and 28 small grouped-selection regression renders,
canonical bytes and exact filesystem labels, complete primary-file order,
paired paragraph arrays, locale identity, purpose-labelled exact-title links,
strict TypeScript comparison execution and legacy rendering equivalence.

All 44 module page/locale/guide files, the module block helper and explanation
helper passed formatting checks. All eleven page entry modules passed
`lint --deny-warnings`. Biome and the scoped new-file strict TypeScript check
passed. An extra strict experiment including the preexisting
`explanations.test.ts` reaches an existing `reference.ts:291` undefined-index
error; that unrelated file was not changed. The repository's configured full
TSC command is non-strict and belongs to the integration gate.

Fresh HTML is under `target/site-modules-review`; the foreground advisory
Japanese report is `target/site-modules-yomiyasu.json`. The pinned yomiyasu
technical guidance was used for Japanese-first editing. Two advisory passes
found sentence-ending repetition and necessary negative comparisons; these
were reviewed without removing restrictions to optimize a score. No other
finding category remained. Code blocks were excluded from extracted prose.

Independent agent reading covered all 22 complete bodies and found no
remaining explanation blocker. The optional positional wording in the import
comparison was changed to `この例も` / `This version also` and rerendered.
This is agent review, not a first-time human reader's acceptance.

Raw command logs and the two literal-repair results are retained locally under
`/workspace/shared/seseragi-modules-authoring/`. Generated artifacts are not
committed. Whole-site integration for the concurrent batches is recorded
separately. Browser/layout checks were not performed in this scoped batch;
no full `check:site`, publication, deployment or issue closure is claimed.
